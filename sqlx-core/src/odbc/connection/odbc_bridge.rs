use super::describe_column;
use crate::error::Error;
use crate::logger::QueryLogger;
use crate::odbc::OdbcValueVec;
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
        let column = describe_column(cursor, index as u16)?;
        let nullable = cursor
            .col_nullability(index as u16)
            .unwrap_or(Nullability::Unknown)
            .could_be_nullable();
        let buffer_desc = map_buffer_desc(&column.type_info, nullable, max_column_size);
        bindings.push(ColumnBinding {
            column,
            buffer_desc,
        });
    }
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
    logger: &mut QueryLogger<'_>,
) -> Result<(), Error> {
    let params = prepare_parameters(args);

    let (affected, receiver_open) = match maybe_prepared {
        MaybePrepared::Prepared(prepared) => {
            let mut prepared = prepared.lock().map_err(|_| {
                Error::Protocol("ODBC execute: failed to lock prepared statement".into())
            })?;
            let receiver_open = if let Some(cursor) = prepared.execute(&params[..])? {
                handle_result_sets(cursor, tx, buffer_settings, logger)?
            } else {
                true
            };
            (extract_rows_affected(&mut *prepared), receiver_open)
        }
        MaybePrepared::NotPrepared(sql) => {
            let mut preallocated = conn.preallocate().map_err(Error::from)?;
            let receiver_open = if let Some(cursor) = preallocated.execute(&sql, &params[..])? {
                handle_result_sets(cursor, tx, buffer_settings, logger)?
            } else {
                true
            };
            (extract_rows_affected(&mut preallocated), receiver_open)
        }
    };

    if receiver_open && send_done(tx, affected).is_ok() {
        logger.increase_rows_affected(affected);
    }
    Ok(())
}

fn extract_rows_affected<S: AsStatementRef>(stmt: &mut S) -> u64 {
    let mut stmt_ref = stmt.as_stmt_ref();
    let count = match stmt_ref.row_count().into_result(&stmt_ref) {
        Ok(count) => count,
        Err(e) => {
            log::debug!("No row count available: {}", e);
            return 0;
        }
    };

    match u64::try_from(count) {
        Ok(count) => count,
        Err(e) => {
            log::warn!("Invalid row count: {}", e);
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
        OdbcArgumentValue::UInt(u) => Box::new(
            WithDataType {
                value: u,
                data_type: DataType::BigInt,
            }
            .into_parameter(),
        ),
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
        OdbcArgumentValue::Null(type_info) => Box::new(
            WithDataType {
                value: Option::<String>::None.into_parameter(),
                data_type: type_info.data_type(),
            }
            .into_parameter(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arguments::Arguments;
    use odbc_api::handles::HasDataType;

    #[test]
    fn typed_none_parameter_preserves_non_string_data_type() {
        let mut args = OdbcArguments::default();

        args.add(Option::<i32>::None);

        let params = prepare_parameters(Some(args));
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].data_type(), odbc_api::DataType::Integer);
    }
}

fn handle_result_sets<C: Cursor + ResultSetMetadata>(
    mut cursor: C,
    tx: &ExecuteSender,
    buffer_settings: OdbcBufferSettings,
    logger: &mut QueryLogger<'_>,
) -> Result<bool, Error> {
    loop {
        let (receiver_open, finished_cursor) = handle_cursor(cursor, tx, buffer_settings, logger)?;

        if !receiver_open {
            return Ok(false);
        }

        match finished_cursor.more_results()? {
            Some(next_cursor) => cursor = next_cursor,
            None => return Ok(true),
        }
    }
}

fn handle_cursor<C: Cursor + ResultSetMetadata>(
    mut cursor: C,
    tx: &ExecuteSender,
    buffer_settings: OdbcBufferSettings,
    logger: &mut QueryLogger<'_>,
) -> Result<(bool, C), Error> {
    if cursor.num_result_cols()? == 0 {
        let rows_affected = extract_rows_affected(&mut cursor);
        let receiver_open = send_done(tx, rows_affected).is_ok();
        if receiver_open {
            logger.increase_rows_affected(rows_affected);
        }
        return Ok((receiver_open, cursor));
    }

    match buffer_settings.max_column_size {
        Some(max_column_size) => {
            // Buffered mode - use batch fetching with columnar buffers
            let bindings = build_bindings(&mut cursor, max_column_size)?;

            let (receiver_open, cursor) =
                stream_rows_buffered(cursor, bindings, tx, buffer_settings.batch_size, logger)?;
            if receiver_open {
                let _ = send_done(tx, 0);
            }
            Ok((receiver_open, cursor))
        }
        None => {
            // Unbuffered mode - use batched row-by-row fetching
            let (receiver_open, cursor) =
                stream_rows_unbuffered(cursor, tx, buffer_settings.batch_size, logger)?;
            if receiver_open {
                let _ = send_done(tx, 0);
            }
            Ok((receiver_open, cursor))
        }
    }
}

fn send_done(tx: &ExecuteSender, rows_affected: u64) -> Result<(), SendError<ExecuteResult>> {
    tx.send(Ok(Either::Left(OdbcQueryResult { rows_affected })))
}

fn send_row(tx: &ExecuteSender, row: OdbcRow) -> Result<(), SendError<ExecuteResult>> {
    tx.send(Ok(Either::Right(row)))
}

fn map_buffer_desc(type_info: &OdbcTypeInfo, nullable: bool, max_column_size: usize) -> BufferDesc {
    use odbc_api::DataType;

    // Some drivers report datatype lengths that are smaller than the actual data,
    // so we cannot use it to build the BufferDesc.
    let data_type = type_info.data_type();
    let max_str_len = max_column_size;

    match data_type {
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
                max_bytes: max_column_size,
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
    }
}

fn create_column_data(slice: AnySlice<'_>, column: &OdbcColumn) -> Result<Arc<ColumnData>, Error> {
    let (values, nulls) = crate::odbc::value::convert_any_slice_to_value_vec(slice)?;
    Ok(Arc::new(ColumnData {
        values,
        type_info: column.type_info.clone(),
        nulls,
    }))
}

fn build_columns_from_cursor<C>(cursor: &mut C) -> Result<Vec<OdbcColumn>, Error>
where
    C: ResultSetMetadata,
{
    let column_count = cursor.num_result_cols()?;
    let column_count = u16::try_from(column_count)
        .map_err(|_| Error::Protocol(format!("ODBC column count {column_count} exceeds u16")))?;
    let mut columns = Vec::with_capacity(usize::from(column_count));
    for index in 1..=column_count {
        columns.push(describe_column(cursor, index)?);
    }
    Ok(columns)
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
    logger: &mut QueryLogger<'_>,
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
        logger.increment_rows_returned();
    }
    receiver_open
}

fn stream_rows_buffered<C>(
    cursor: C,
    bindings: Vec<ColumnBinding>,
    tx: &ExecuteSender,
    batch_size: usize,
    logger: &mut QueryLogger<'_>,
) -> Result<(bool, C), Error>
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
            .collect::<Result<_, _>>()?;

        if !send_rows_for_batch(tx, &col_arc, column_data, batch.num_rows(), logger) {
            receiver_open = false;
            break;
        }
    }

    let (cursor, _) = row_set_cursor.unbind()?;

    Ok((receiver_open, cursor))
}

fn stream_rows_unbuffered<C>(
    mut cursor: C,
    tx: &ExecuteSender,
    batch_size: usize,
    logger: &mut QueryLogger<'_>,
) -> Result<(bool, C), Error>
where
    C: Cursor + ResultSetMetadata,
{
    let mut receiver_open = true;

    let columns = build_columns_from_cursor(&mut cursor)?;
    let column_count = columns.len();

    let col_arc: Arc<[OdbcColumn]> = Arc::from(columns.clone());

    loop {
        // Initialize per-column containers for this batch
        let mut value_vecs: Vec<OdbcValueVec> = columns
            .iter()
            .map(|c| OdbcValueVec::with_capacity_for_type(c.type_info.data_type(), batch_size))
            .collect();
        let mut nulls_vecs: Vec<Vec<bool>> = (0..column_count)
            .map(|_| Vec::with_capacity(batch_size))
            .collect();

        let mut num_rows = 0;
        while let Some(mut cursor_row) = cursor.next_row()? {
            for col in 0..column_count {
                let col_idx = (col as u16) + 1;
                value_vecs[col].push_from_cursor_row(
                    &mut cursor_row,
                    col_idx,
                    &mut nulls_vecs[col],
                )?;
            }
            num_rows += 1;
            if num_rows == batch_size {
                break;
            }
        }

        let column_data = build_column_data_from_values(&columns, value_vecs, nulls_vecs);

        if !send_rows_for_batch(tx, &col_arc, column_data, num_rows, logger) {
            receiver_open = false;
            break;
        }

        if !receiver_open || num_rows < batch_size {
            break;
        }
    }

    Ok((receiver_open, cursor))
}
