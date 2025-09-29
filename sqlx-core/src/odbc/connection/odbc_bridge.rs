use super::decode_column_name;
use crate::error::Error;
use crate::odbc::{
    connection::MaybePrepared, ColumnData, OdbcArgumentValue, OdbcArguments, OdbcColumn,
    OdbcQueryResult, OdbcRow, OdbcTypeInfo, OdbcValue,
};
use either::Either;
use flume::{SendError, Sender};
use odbc_api::buffers::{AnySlice, BufferDesc, ColumnarAnyBuffer};
use odbc_api::handles::{AsStatementRef, Nullability, Statement};
use odbc_api::DataType;
use odbc_api::{Cursor, IntoParameter, ResultSetMetadata};
use std::cmp::min;
use std::sync::Arc;

// Bulk fetch implementation using columnar buffers instead of row-by-row fetching
// This provides significant performance improvements by fetching rows in batches
// and avoiding the slow `next_row()` method from odbc-api
const BATCH_SIZE: usize = 128;
const DEFAULT_TEXT_LEN: usize = 512;
const DEFAULT_BINARY_LEN: usize = 1024;
const DEFAULT_NUMERIC_TEXT_LEN: usize = 128;
const MIN_TEXT_LEN: usize = 1024;
const MAX_TEXT_LEN: usize = 4096;

struct ColumnBinding {
    column: OdbcColumn,
    buffer_desc: BufferDesc,
}

fn build_bindings<C>(cursor: &mut C) -> Result<Vec<ColumnBinding>, Error>
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
        let buffer_desc = map_buffer_desc(cursor, index as u16, &column.type_info, nullable)?;
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
) -> Result<(), Error> {
    let params = prepare_parameters(args);

    let affected = match maybe_prepared {
        MaybePrepared::Prepared(prepared) => {
            let mut prepared = prepared.lock().expect("prepared statement lock");
            if let Some(cursor) = prepared.execute(&params[..])? {
                handle_cursor(cursor, tx);
            }
            extract_rows_affected(&mut *prepared)
        }
        MaybePrepared::NotPrepared(sql) => {
            let mut preallocated = conn.preallocate().map_err(Error::from)?;
            if let Some(cursor) = preallocated.execute(&sql, &params[..])? {
                handle_cursor(cursor, tx);
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

fn handle_cursor<C>(mut cursor: C, tx: &ExecuteSender)
where
    C: Cursor + ResultSetMetadata,
{
    let bindings = match build_bindings(&mut cursor) {
        Ok(b) => b,
        Err(e) => {
            send_error(tx, e);
            return;
        }
    };

    match stream_rows(cursor, bindings, tx) {
        Ok(true) => {
            let _ = send_done(tx, 0);
        }
        Ok(false) => {}
        Err(e) => {
            send_error(tx, e);
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
) -> Result<BufferDesc, Error>
where
    C: ResultSetMetadata,
{
    let data_type = type_info.data_type();
    let buffer_desc = match data_type {
        DataType::TinyInt | DataType::SmallInt | DataType::Integer | DataType::BigInt => {
            BufferDesc::I64 { nullable }
        }
        DataType::Real => BufferDesc::F32 { nullable },
        DataType::Float { .. } | DataType::Double => BufferDesc::F64 { nullable },
        DataType::Bit => BufferDesc::Bit { nullable },
        DataType::Date => BufferDesc::Date { nullable },
        DataType::Time { .. } => BufferDesc::Time { nullable },
        DataType::Timestamp { .. } => BufferDesc::Timestamp { nullable },
        DataType::Binary { length }
        | DataType::Varbinary { length }
        | DataType::LongVarbinary { length } => BufferDesc::Binary {
            length: if let Some(length) = length {
                length.get().clamp(MIN_TEXT_LEN, MAX_TEXT_LEN)
            } else {
                MAX_TEXT_LEN
            },
        },
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
            max_str_len: if let Some(length) = length {
                length.get().clamp(MIN_TEXT_LEN, MAX_TEXT_LEN)
            } else {
                MAX_TEXT_LEN
            },
        },
        DataType::Unknown => BufferDesc::Text {
            max_str_len: MAX_TEXT_LEN,
        },
        DataType::Decimal { .. } | DataType::Numeric { .. } => BufferDesc::Text {
            max_str_len: min(DEFAULT_NUMERIC_TEXT_LEN, MAX_TEXT_LEN),
        },
    };

    Ok(buffer_desc)
}

fn stream_rows<C>(
    cursor: C,
    bindings: Vec<ColumnBinding>,
    tx: &ExecuteSender,
) -> Result<bool, Error>
where
    C: Cursor + ResultSetMetadata,
{
    let buffer_descriptions: Vec<_> = bindings.iter().map(|b| b.buffer_desc).collect();
    let buffer = ColumnarAnyBuffer::from_descs(BATCH_SIZE, buffer_descriptions);
    let mut row_set_cursor = cursor.bind_buffer(buffer)?;

    let mut receiver_open = true;

    while let Some(batch) = row_set_cursor.fetch()? {
        let columns: Vec<_> = bindings.iter().map(|b| b.column.clone()).collect();

        // Create ColumnData instances that can be shared across rows
        let column_data_vec: Vec<_> = bindings
            .iter()
            .enumerate()
            .map(|(col_index, binding)| {
                create_column_data(batch.column(col_index), &binding.column.type_info)
            })
            .collect();

        for row_index in 0..batch.num_rows() {
            let row_values: Vec<_> = column_data_vec
                .iter()
                .map(|column_data| OdbcValue::new(Arc::clone(column_data), row_index))
                .collect();

            let row = OdbcRow {
                columns: columns.clone(),
                values: row_values,
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

fn create_column_data(slice: AnySlice<'_>, type_info: &OdbcTypeInfo) -> Arc<ColumnData> {
    use crate::odbc::value::convert_any_slice_to_value_vec;

    Arc::new(ColumnData {
        values: convert_any_slice_to_value_vec(slice),
        type_info: type_info.clone(),
    })
}
