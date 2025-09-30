use super::decode_column_name;
use crate::error::Error;
use crate::odbc::OdbcValueVec;
use crate::odbc::{
    connection::MaybePrepared, ColumnData, OdbcArgumentValue, OdbcArguments, OdbcBatch,
    OdbcBufferSettings, OdbcColumn, OdbcQueryResult, OdbcRow, OdbcTypeInfo,
};
use either::Either;
use flume::{SendError, Sender};
use odbc_api::buffers::{AnySlice, BufferDesc, ColumnarAnyBuffer};
use odbc_api::handles::{AsStatementRef, CDataMut, Nullability, Statement};
use odbc_api::parameter::CElement;
use odbc_api::{Cursor, IntoParameter, Nullable, ResultSetMetadata};
use std::sync::Arc;

// Bulk fetch implementation using columnar buffers instead of row-by-row fetching
// This provides significant performance improvements by fetching rows in batches
// and avoiding the slow `next_row()` method from odbc-api
#[derive(Debug)]
struct ColumnBinding {
    column: OdbcColumn,
    buffer_desc: BufferDesc,
}

fn build_bindings<C: Cursor>(
    cursor: &mut C,
    max_column_size: usize,
) -> Result<Vec<ColumnBinding>, Error> {
    let column_count = cursor.num_result_cols().unwrap_or(0);
    let mut bindings = Vec::with_capacity(column_count as usize);
    for index in 1..=column_count {
        let column = create_column(cursor, index as u16);
        let nullable = cursor
            .col_nullability(index as u16)
            .unwrap_or(Nullability::Unknown)
            .could_be_nullable();
        let buffer_desc = map_buffer_desc(&column.type_info, nullable, max_column_size)?;
        bindings.push(ColumnBinding {
            column,
            buffer_desc,
        });
    }
    dbg!(&bindings);
    log::trace!(
        "built {} ODBC batch column bindings: {:?}",
        bindings.len(),
        bindings
    );
    Ok(bindings)
}

pub type ExecuteResult = Result<Either<OdbcQueryResult, OdbcRow>, Error>;
pub type ExecuteSender = Sender<ExecuteResult>;

pub fn establish_connection(
    options: &crate::odbc::OdbcConnectOptions,
) -> Result<odbc_api::Connection<'static>, Error> {
    let env = odbc_api::environment().map_err(|e| Error::Configuration(e.to_string().into()))?;
    let conn = env
        .connect_with_connection_string(options.connection_string(), Default::default())
        .map_err(|e| Error::Configuration(e.to_string().into()))?;
    Ok(conn)
}

pub fn execute_sql(
    conn: &mut odbc_api::Connection<'static>,
    maybe_prepared: MaybePrepared,
    args: Option<OdbcArguments>,
    tx: &ExecuteSender,
    buffer_settings: OdbcBufferSettings,
) -> Result<(), Error> {
    log::trace!("Executing {:?} with params {:?}", maybe_prepared, args);
    let params = prepare_parameters(args);

    let affected = match maybe_prepared {
        MaybePrepared::Prepared(prepared) => {
            let mut prepared = prepared.lock().expect("prepared statement lock");
            if let Some(cursor) = prepared.execute(&params[..])? {
                handle_cursor(cursor, tx, buffer_settings);
            }
            extract_rows_affected(&mut *prepared)
        }
        MaybePrepared::NotPrepared(sql) => {
            let mut preallocated = conn.preallocate().map_err(Error::from)?;
            if let Some(cursor) = preallocated.execute(&sql, &params[..])? {
                handle_cursor(cursor, tx, buffer_settings);
            }
            extract_rows_affected(&mut preallocated)
        }
    };

    let _ = send_done(tx, affected);
    Ok(())
}

fn extract_rows_affected<S: AsStatementRef>(stmt: &mut S) -> u64 {
    let mut stmt_ref = stmt.as_stmt_ref();
    let count = match stmt_ref.row_count().into_result(&stmt_ref) {
        Ok(count) => count,
        Err(e) => {
            log::warn!("Failed to get row count: {}", e);
            return 0;
        }
    };

    match u64::try_from(count) {
        Ok(count) => count,
        Err(e) => {
            log::warn!("Failed to get row count: {}", e);
            0
        }
    }
}

fn prepare_parameters(
    args: Option<OdbcArguments>,
) -> Vec<Box<dyn odbc_api::parameter::InputParameter>> {
    let args = args.map(|a| a.values).unwrap_or_default();
    args.into_iter().map(to_param).collect()
}

fn to_param(arg: OdbcArgumentValue) -> Box<dyn odbc_api::parameter::InputParameter + 'static> {
    use odbc_api::parameter::WithDataType;
    use odbc_api::DataType;

    match arg {
        OdbcArgumentValue::Int(i) => Box::new(i.into_parameter()),
        OdbcArgumentValue::Float(f) => Box::new(f.into_parameter()),
        OdbcArgumentValue::Text(s) => Box::new(s.into_parameter()),
        OdbcArgumentValue::Bytes(b) => Box::new(b.into_parameter()),
        OdbcArgumentValue::Date(d) => Box::new(
            WithDataType {
                value: d,
                data_type: DataType::Date,
            }
            .into_parameter(),
        ),
        OdbcArgumentValue::Time(t) => Box::new(
            WithDataType {
                value: t,
                data_type: DataType::Time { precision: 0 },
            }
            .into_parameter(),
        ),
        OdbcArgumentValue::Timestamp(ts) => Box::new(
            WithDataType {
                value: ts,
                data_type: DataType::Timestamp { precision: 6 },
            }
            .into_parameter(),
        ),
        OdbcArgumentValue::Null => Box::new(Option::<String>::None.into_parameter()),
    }
}

fn handle_cursor<C: Cursor + ResultSetMetadata>(
    mut cursor: C,
    tx: &ExecuteSender,
    buffer_settings: OdbcBufferSettings,
) {
    match buffer_settings.max_column_size {
        Some(max_column_size) => {
            // Buffered mode - use batch fetching with columnar buffers
            let bindings = match build_bindings(&mut cursor, max_column_size) {
                Ok(b) => b,
                Err(e) => {
                    send_error(tx, e);
                    return;
                }
            };

            match stream_rows(cursor, bindings, tx, buffer_settings) {
                Ok(true) => {
                    let _ = send_done(tx, 0);
                }
                Ok(false) => {}
                Err(e) => {
                    send_error(tx, e);
                }
            }
        }
        None => {
            // Unbuffered mode - use batched row-by-row fetching
            match stream_rows_unbuffered(cursor, tx, buffer_settings.batch_size) {
                Ok(true) => {
                    let _ = send_done(tx, 0);
                }
                Ok(false) => {}
                Err(e) => {
                    send_error(tx, e);
                }
            }
        }
    }
}

fn send_done(tx: &ExecuteSender, rows_affected: u64) -> Result<(), SendError<ExecuteResult>> {
    tx.send(Ok(Either::Left(OdbcQueryResult { rows_affected })))
}

fn send_error(tx: &ExecuteSender, error: Error) {
    let _ = tx.send(Err(error));
}

fn send_row(tx: &ExecuteSender, row: OdbcRow) -> Result<(), SendError<ExecuteResult>> {
    tx.send(Ok(Either::Right(row)))
}

fn create_column<C>(cursor: &mut C, index: u16) -> OdbcColumn
where
    C: ResultSetMetadata,
{
    let mut cd = odbc_api::ColumnDescription::default();
    let _ = cursor.describe_col(index, &mut cd);

    OdbcColumn {
        name: decode_column_name(cd.name, index),
        type_info: OdbcTypeInfo::new(cd.data_type),
        ordinal: usize::from(index.checked_sub(1).unwrap()),
    }
}

fn map_buffer_desc(
    type_info: &OdbcTypeInfo,
    nullable: bool,
    max_column_size: usize,
) -> Result<BufferDesc, Error> {
    use odbc_api::DataType;

    // Some drivers report datatype lengths that are smaller than the actual data,
    // so we cannot use it to build the BufferDesc.
    let data_type = type_info.data_type();
    let max_str_len = max_column_size;

    let buffer_desc = match data_type {
        // Integer types - all map to I64
        DataType::TinyInt | DataType::SmallInt | DataType::Integer | DataType::BigInt => {
            BufferDesc::I64 { nullable }
        }
        // Floating point types
        DataType::Real => BufferDesc::F32 { nullable },
        DataType::Float { .. } | DataType::Double => BufferDesc::F64 { nullable },
        // Bit type
        DataType::Bit => BufferDesc::Bit { nullable },
        // Date/Time types
        DataType::Date => BufferDesc::Date { nullable },
        DataType::Time { .. } => BufferDesc::Time { nullable },
        DataType::Timestamp { .. } => BufferDesc::Timestamp { nullable },
        // Binary types
        DataType::Binary { .. } | DataType::Varbinary { .. } | DataType::LongVarbinary { .. } => {
            BufferDesc::Binary {
                length: max_column_size,
            }
        }
        // Text types
        DataType::Char { .. }
        | DataType::WChar { .. }
        | DataType::Varchar { .. }
        | DataType::WVarchar { .. }
        | DataType::LongVarchar { .. }
        | DataType::WLongVarchar { .. }
        | DataType::Other { .. } => BufferDesc::Text { max_str_len },
        // Fallback cases
        DataType::Unknown => BufferDesc::Text { max_str_len },
        DataType::Decimal { .. } | DataType::Numeric { .. } => BufferDesc::Text { max_str_len },
    };

    Ok(buffer_desc)
}

fn create_column_data(slice: AnySlice<'_>, column: &OdbcColumn) -> Arc<ColumnData> {
    let (values, nulls) = crate::odbc::value::convert_any_slice_to_value_vec(slice);
    Arc::new(ColumnData {
        values,
        type_info: column.type_info.clone(),
        nulls,
    })
}

fn build_columns_from_cursor<C>(cursor: &mut C) -> Vec<OdbcColumn>
where
    C: ResultSetMetadata,
{
    let column_count = cursor.num_result_cols().expect("no column count found");
    let column_count = u16::try_from(column_count).expect("invalid column count");
    let mut columns = Vec::with_capacity(usize::from(column_count));
    for index in 1..=column_count {
        columns.push(create_column(cursor, index));
    }
    columns
}

fn build_column_data_from_values(
    columns: &[OdbcColumn],
    value_vecs: Vec<OdbcValueVec>,
    nulls_vecs: Vec<Vec<bool>>,
) -> Vec<Arc<ColumnData>> {
    value_vecs
        .into_iter()
        .zip(nulls_vecs)
        .zip(columns.iter())
        .map(|((values, nulls), column)| {
            Arc::new(ColumnData {
                values,
                type_info: column.type_info.clone(),
                nulls,
            })
        })
        .collect()
}

fn send_rows_for_batch(
    tx: &ExecuteSender,
    col_arc: &Arc<[OdbcColumn]>,
    column_data: Vec<Arc<ColumnData>>,
    num_rows: usize,
) -> bool {
    let odbc_batch = Arc::new(OdbcBatch {
        columns: Arc::clone(col_arc),
        column_data,
    });

    let mut receiver_open = true;
    for row_index in 0..num_rows {
        let row = OdbcRow {
            row_index,
            batch: Arc::clone(&odbc_batch),
        };
        if send_row(tx, row).is_err() {
            receiver_open = false;
            break;
        }
    }
    receiver_open
}

fn stream_rows<C>(
    cursor: C,
    bindings: Vec<ColumnBinding>,
    tx: &ExecuteSender,
    buffer_settings: OdbcBufferSettings,
) -> Result<bool, Error>
where
    C: Cursor + ResultSetMetadata,
{
    if buffer_settings.max_column_size.is_some() {
        // Buffered mode
        stream_rows_buffered(cursor, bindings, tx, buffer_settings.batch_size)
    } else {
        // Unbuffered mode - we shouldn't reach here, but handle it just in case
        stream_rows_unbuffered(cursor, tx, buffer_settings.batch_size)
    }
}

fn stream_rows_buffered<C>(
    cursor: C,
    bindings: Vec<ColumnBinding>,
    tx: &ExecuteSender,
    batch_size: usize,
) -> Result<bool, Error>
where
    C: Cursor + ResultSetMetadata,
{
    let buffer_descriptions: Vec<_> = bindings.iter().map(|b| b.buffer_desc).collect();
    let buffer = ColumnarAnyBuffer::from_descs(batch_size, buffer_descriptions);
    let mut row_set_cursor = cursor.bind_buffer(buffer)?;

    let mut receiver_open = true;

    let columns: Vec<OdbcColumn> = bindings.iter().map(|b| b.column.clone()).collect();
    let col_arc: Arc<[OdbcColumn]> = Arc::from(columns);

    while let Some(batch) = row_set_cursor.fetch()? {
        let column_data: Vec<_> = bindings
            .iter()
            .enumerate()
            .map(|(col_index, binding)| {
                create_column_data(batch.column(col_index), &binding.column)
            })
            .collect();

        if !send_rows_for_batch(tx, &col_arc, column_data, batch.num_rows()) {
            receiver_open = false;
            break;
        }
    }

    Ok(receiver_open)
}

fn stream_rows_unbuffered<C>(
    mut cursor: C,
    tx: &ExecuteSender,
    batch_size: usize,
) -> Result<bool, Error>
where
    C: Cursor + ResultSetMetadata,
{
    use odbc_api::DataType;

    let mut receiver_open = true;

    let columns = build_columns_from_cursor(&mut cursor);
    let column_count = columns.len();

    let col_arc: Arc<[OdbcColumn]> = Arc::from(columns.clone());

    fn init_value_vec(dt: DataType, capacity: usize) -> OdbcValueVec {
        match dt {
            DataType::TinyInt => OdbcValueVec::TinyInt(Vec::with_capacity(capacity)),
            DataType::SmallInt => OdbcValueVec::SmallInt(Vec::with_capacity(capacity)),
            DataType::Integer => OdbcValueVec::BigInt(Vec::with_capacity(capacity)), // the SQLite driver reports "INTEGER" even though it supports 64-bit integers
            DataType::BigInt => OdbcValueVec::BigInt(Vec::with_capacity(capacity)),
            DataType::Real => OdbcValueVec::Real(Vec::with_capacity(capacity)),
            DataType::Float { .. } | DataType::Double => {
                OdbcValueVec::Double(Vec::with_capacity(capacity))
            }
            DataType::Bit => OdbcValueVec::Bit(Vec::with_capacity(capacity)),
            DataType::Date => OdbcValueVec::Date(Vec::with_capacity(capacity)),
            DataType::Time { .. } => OdbcValueVec::Time(Vec::with_capacity(capacity)),
            DataType::Timestamp { .. } => OdbcValueVec::Timestamp(Vec::with_capacity(capacity)),
            DataType::Binary { .. }
            | DataType::Varbinary { .. }
            | DataType::LongVarbinary { .. } => OdbcValueVec::Binary(Vec::with_capacity(capacity)),
            _ => OdbcValueVec::Text(Vec::with_capacity(capacity)),
        }
    }

    fn push_get_data<T: Default + Copy + CElement + CDataMut>(
        cursor_row: &mut odbc_api::CursorRow<'_>,
        col_index: u16,
        vec: &mut Vec<T>,
        nulls: &mut Vec<bool>,
    ) -> Result<(), odbc_api::Error>
    where
        Nullable<T>: CElement + CDataMut,
    {
        let mut tmp = Nullable::null();
        cursor_row.get_data(col_index, &mut tmp)?;
        let option = tmp.into_opt();
        nulls.push(option.is_none());
        vec.push(option.unwrap_or_default());
        Ok(())
    }

    fn push_binary(
        cursor_row: &mut odbc_api::CursorRow<'_>,
        col_index: u16,
        vec: &mut Vec<Vec<u8>>,
        nulls: &mut Vec<bool>,
    ) -> Result<(), odbc_api::Error> {
        let mut buf = Vec::new();
        nulls.push(cursor_row.get_binary(col_index, &mut buf).is_err());
        vec.push(buf);
        Ok(())
    }

    fn push_text(
        cursor_row: &mut odbc_api::CursorRow<'_>,
        col_index: u16,
        vec: &mut Vec<String>,
        nulls: &mut Vec<bool>,
    ) -> Result<(), odbc_api::Error> {
        let mut buf = Vec::<u16>::new();
        let txt = cursor_row.get_wide_text(col_index, &mut buf);
        vec.push(String::from_utf16_lossy(&buf).to_string());
        nulls.push(!txt.unwrap_or(false));
        Ok(())
    }

    fn push_bit(
        cursor_row: &mut odbc_api::CursorRow<'_>,
        col_index: u16,
        vec: &mut Vec<bool>,
        nulls: &mut Vec<bool>,
    ) -> Result<(), odbc_api::Error> {
        let mut bit_val = odbc_api::Bit(0);
        let result = cursor_row.get_data(col_index, &mut bit_val);
        vec.push(bit_val.as_bool());
        nulls.push(result.is_err());
        Ok(())
    }

    fn push_from_cursor_row(
        cursor_row: &mut odbc_api::CursorRow<'_>,
        col_index: u16,
        values: &mut OdbcValueVec,
        nulls: &mut Vec<bool>,
    ) -> Result<(), odbc_api::Error> {
        match values {
            OdbcValueVec::TinyInt(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::SmallInt(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Integer(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::BigInt(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Real(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Double(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Bit(v) => push_bit(cursor_row, col_index, v, nulls),
            OdbcValueVec::Date(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Time(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Timestamp(v) => push_get_data(cursor_row, col_index, v, nulls),
            OdbcValueVec::Binary(v) => push_binary(cursor_row, col_index, v, nulls),
            OdbcValueVec::Text(v) => push_text(cursor_row, col_index, v, nulls),
        }
    }

    loop {
        // Initialize per-column containers for this batch
        let mut value_vecs: Vec<OdbcValueVec> = columns
            .iter()
            .map(|c| init_value_vec(c.type_info.data_type(), batch_size))
            .collect();
        let mut nulls_vecs: Vec<Vec<bool>> = (0..column_count)
            .map(|_| Vec::with_capacity(batch_size))
            .collect();

        let mut num_rows = 0;
        while let Some(mut cursor_row) = cursor.next_row()? {
            for col in 0..column_count {
                let col_idx = (col as u16) + 1;
                push_from_cursor_row(
                    &mut cursor_row,
                    col_idx,
                    &mut value_vecs[col],
                    &mut nulls_vecs[col],
                )?;
            }
            num_rows += 1;
            if num_rows == batch_size {
                break;
            }
        }

        let column_data = build_column_data_from_values(&columns, value_vecs, nulls_vecs);

        if !send_rows_for_batch(tx, &col_arc, column_data, num_rows) {
            receiver_open = false;
            break;
        }

        if !receiver_open || num_rows < batch_size {
            break;
        }
    }

    Ok(receiver_open)
}
