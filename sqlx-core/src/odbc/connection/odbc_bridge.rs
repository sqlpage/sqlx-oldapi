use super::decode_column_name;
use crate::error::Error;
use crate::odbc::{
    connection::MaybePrepared, ColumnData, OdbcArgumentValue, OdbcArguments, OdbcBatch,
    OdbcBufferSettings, OdbcColumn, OdbcQueryResult, OdbcRow, OdbcTypeInfo,
};
use either::Either;
use flume::{SendError, Sender};
use odbc_api::buffers::{AnySlice, BufferDesc, ColumnarAnyBuffer};
use odbc_api::handles::{AsStatementRef, Nullability, Statement};
use odbc_api::{Cursor, IntoParameter, ResultSetMetadata};
use std::sync::Arc;

// Bulk fetch implementation using columnar buffers instead of row-by-row fetching
// This provides significant performance improvements by fetching rows in batches
// and avoiding the slow `next_row()` method from odbc-api

struct ColumnBinding {
    column: OdbcColumn,
    buffer_desc: BufferDesc,
}

fn build_bindings<C>(
    cursor: &mut C,
    buffer_settings: &OdbcBufferSettings,
) -> Result<Vec<ColumnBinding>, Error>
where
    C: ResultSetMetadata,
{
    let column_count = cursor.num_result_cols().unwrap_or(0);
    let mut bindings = Vec::with_capacity(column_count as usize);
    for index in 1..=column_count {
        let column = create_column(cursor, index as u16);
        let nullable = cursor
            .col_nullability(index as u16)
            .unwrap_or(Nullability::Unknown)
            .could_be_nullable();
        let buffer_desc = map_buffer_desc(
            cursor,
            index as u16,
            &column.type_info,
            nullable,
            buffer_settings,
        )?;
        bindings.push(ColumnBinding {
            column,
            buffer_desc,
        });
    }
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
    match arg {
        OdbcArgumentValue::Int(i) => Box::new(i.into_parameter()),
        OdbcArgumentValue::Float(f) => Box::new(f.into_parameter()),
        OdbcArgumentValue::Text(s) => Box::new(s.into_parameter()),
        OdbcArgumentValue::Bytes(b) => Box::new(b.into_parameter()),
        OdbcArgumentValue::Null => Box::new(Option::<String>::None.into_parameter()),
    }
}

fn handle_cursor<C>(mut cursor: C, tx: &ExecuteSender, buffer_settings: OdbcBufferSettings)
where
    C: Cursor + ResultSetMetadata,
{
    match buffer_settings {
        OdbcBufferSettings::Buffered { .. } => {
            let bindings = match build_bindings(&mut cursor, &buffer_settings) {
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
        OdbcBufferSettings::Unbuffered => {
            match stream_rows(cursor, Vec::new(), tx, buffer_settings) {
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

fn map_buffer_desc<C>(
    _cursor: &mut C,
    _column_index: u16,
    type_info: &OdbcTypeInfo,
    nullable: bool,
    buffer_settings: &OdbcBufferSettings,
) -> Result<BufferDesc, Error>
where
    C: ResultSetMetadata,
{
    use odbc_api::DataType;

    let data_type = type_info.data_type();

    // Helper function to determine buffer length with fallback
    let max_column_size = match buffer_settings {
        OdbcBufferSettings::Buffered {
            max_column_size, ..
        } => *max_column_size,
        OdbcBufferSettings::Unbuffered => 4096, // Default value for unbuffered mode
    };

    let buffer_length = |length: Option<std::num::NonZeroUsize>| {
        if let Some(length) = length {
            if length.get() < 255 {
                length.get()
            } else {
                max_column_size
            }
        } else {
            max_column_size
        }
    };

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
        DataType::Binary { length }
        | DataType::Varbinary { length }
        | DataType::LongVarbinary { length } => BufferDesc::Binary {
            length: buffer_length(length),
        },
        // Text types
        DataType::Char { length }
        | DataType::WChar { length }
        | DataType::Varchar { length }
        | DataType::WVarchar { length }
        | DataType::LongVarchar { length }
        | DataType::WLongVarchar { length }
        | DataType::Other {
            column_size: length,
            ..
        } => BufferDesc::Text {
            max_str_len: buffer_length(length),
        },
        // Fallback cases
        DataType::Unknown => BufferDesc::Text {
            max_str_len: max_column_size,
        },
        DataType::Decimal { .. } | DataType::Numeric { .. } => BufferDesc::Text {
            max_str_len: max_column_size,
        },
    };

    Ok(buffer_desc)
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
    match buffer_settings {
        OdbcBufferSettings::Buffered { batch_size, .. } => {
            stream_rows_buffered(cursor, bindings, tx, batch_size)
        }
        OdbcBufferSettings::Unbuffered => {
            // For unbuffered mode, we don't need bindings since we build columns dynamically
            stream_rows_unbuffered(cursor, tx)
        }
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
        // Create ColumnData instances that can be shared across rows
        let column_data: Vec<_> = bindings
            .iter()
            .enumerate()
            .map(|(col_index, binding)| {
                create_column_data(batch.column(col_index), &binding.column)
            })
            .collect();

        let odbc_batch = Arc::new(OdbcBatch {
            columns: Arc::clone(&col_arc),
            column_data,
        });

        for row_index in 0..batch.num_rows() {
            let row = OdbcRow {
                row_index,
                batch: Arc::clone(&odbc_batch),
            };

            if send_row(tx, row).is_err() {
                receiver_open = false;
                break;
            }
        }

        if !receiver_open {
            break;
        }
    }

    Ok(receiver_open)
}

fn stream_rows_unbuffered<C>(mut cursor: C, tx: &ExecuteSender) -> Result<bool, Error>
where
    C: Cursor + ResultSetMetadata,
{
    let mut receiver_open = true;

    // For unbuffered mode, we need to build column information for each row
    let column_count = cursor.num_result_cols().unwrap_or(0);
    let mut columns = Vec::with_capacity(column_count as usize);

    for index in 1..=column_count {
        columns.push(create_column(&mut cursor, index as u16));
    }

    let col_arc: Arc<[OdbcColumn]> = Arc::from(columns);

    while let Some(mut cursor_row) = cursor.next_row()? {
        // Create a single-row batch for this row
        let column_data: Vec<_> = (0..column_count)
            .map(|col_index| {
                let column = &col_arc[col_index as usize];
                // Convert CursorRow data to ColumnData format
                // Column indices are 1-based in odbc-api
                create_column_data_from_cursor_row(&mut cursor_row, (col_index + 1) as u16, column)
            })
            .collect();

        let odbc_batch = Arc::new(OdbcBatch {
            columns: Arc::clone(&col_arc),
            column_data,
        });

        let row = OdbcRow {
            row_index: 0, // Single row in this batch
            batch: Arc::clone(&odbc_batch),
        };

        if send_row(tx, row).is_err() {
            receiver_open = false;
            break;
        }
    }

    Ok(receiver_open)
}

fn create_column_data(slice: AnySlice<'_>, column: &OdbcColumn) -> Arc<ColumnData> {
    use crate::odbc::value::convert_any_slice_to_value_vec;

    let (values, nulls) = convert_any_slice_to_value_vec(slice);
    Arc::new(ColumnData {
        values,
        type_info: column.type_info.clone(),
        nulls,
    })
}

fn create_column_data_from_cursor_row(
    cursor_row: &mut odbc_api::CursorRow<'_>,
    column_index: u16,
    column: &OdbcColumn,
) -> Arc<ColumnData> {
    use crate::odbc::value::OdbcValueVec;
    use odbc_api::DataType;

    let data_type = column.type_info.data_type();

    // Create single-element vectors for this row
    let (values, nulls) = match data_type {
        DataType::TinyInt => {
            let mut value = 0i8;
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::TinyInt(vec![value]), vec![is_null])
        }
        DataType::SmallInt => {
            let mut value = 0i16;
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::SmallInt(vec![value]), vec![is_null])
        }
        DataType::Integer => {
            let mut value = 0i32;
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::Integer(vec![value]), vec![is_null])
        }
        DataType::BigInt => {
            let mut value = 0i64;
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::BigInt(vec![value]), vec![is_null])
        }
        DataType::Real => {
            let mut value = 0.0f32;
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::Real(vec![value]), vec![is_null])
        }
        DataType::Float { .. } | DataType::Double => {
            let mut value = 0.0f64;
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::Double(vec![value]), vec![is_null])
        }
        DataType::Bit => {
            let mut value = odbc_api::Bit(0);
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::Bit(vec![value]), vec![is_null])
        }
        DataType::Date => {
            let mut value = odbc_api::sys::Date::default();
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::Date(vec![value]), vec![is_null])
        }
        DataType::Time { .. } => {
            let mut value = odbc_api::sys::Time::default();
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::Time(vec![value]), vec![is_null])
        }
        DataType::Timestamp { .. } => {
            let mut value = odbc_api::sys::Timestamp::default();
            let is_null = cursor_row.get_data(column_index, &mut value).is_err();
            (OdbcValueVec::Timestamp(vec![value]), vec![is_null])
        }
        DataType::Char { .. }
        | DataType::Varchar { .. }
        | DataType::LongVarchar { .. }
        | DataType::WChar { .. }
        | DataType::WVarchar { .. }
        | DataType::WLongVarchar { .. }
        | DataType::Binary { .. }
        | DataType::Varbinary { .. }
        | DataType::LongVarbinary { .. }
        | DataType::Other { .. }
        | DataType::Unknown
        | DataType::Decimal { .. }
        | DataType::Numeric { .. } => {
            // For text and binary data, use get_text
            let mut buf = Vec::new();
            match cursor_row.get_text(column_index, &mut buf) {
                Ok(true) => {
                    // Successfully got text, convert to string
                    let text = String::from_utf8_lossy(&buf).to_string();
                    (OdbcValueVec::Text(vec![Some(text)]), vec![false])
                }
                Ok(false) => {
                    // NULL value
                    (OdbcValueVec::Text(vec![None]), vec![true])
                }
                Err(_) => {
                    // Error, treat as NULL
                    (OdbcValueVec::Text(vec![None]), vec![true])
                }
            }
        }
    };

    Arc::new(ColumnData {
        values,
        type_info: column.type_info.clone(),
        nulls,
    })
}
