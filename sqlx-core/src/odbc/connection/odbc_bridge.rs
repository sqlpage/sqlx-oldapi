use crate::error::Error;
use crate::odbc::{
    connection::MaybePrepared, OdbcArgumentValue, OdbcArguments, OdbcColumn, OdbcQueryResult,
    OdbcRow, OdbcTypeInfo, OdbcValue,
};
use either::Either;
use flume::{SendError, Sender};
use odbc_api::buffers::{AnySlice, BufferDesc, ColumnarAnyBuffer};
use odbc_api::handles::{AsStatementRef, Nullability, Statement};
use odbc_api::DataType;
use odbc_api::{Cursor, IntoParameter, ResultSetMetadata};
use std::cmp::min;

// Bulk fetch implementation using columnar buffers instead of row-by-row fetching
// This provides significant performance improvements by fetching rows in batches
// and avoiding the slow `next_row()` method from odbc-api
const BATCH_SIZE: usize = 128;
const DEFAULT_TEXT_LEN: usize = 512;
const DEFAULT_BINARY_LEN: usize = 1024;
const DEFAULT_NUMERIC_TEXT_LEN: usize = 128;
const MAX_TEXT_LEN: usize = 1024 * 1024;
const MAX_BINARY_LEN: usize = 1024 * 1024;

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

fn decode_column_name(name_bytes: Vec<u8>, index: u16) -> String {
    String::from_utf8(name_bytes).unwrap_or_else(|_| format!("col{}", index - 1))
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
        DataType::Binary { .. } | DataType::Varbinary { .. } | DataType::LongVarbinary { .. } => {
            BufferDesc::Binary {
                length: DEFAULT_BINARY_LEN,
            }
        }
        DataType::Char { .. }
        | DataType::WChar { .. }
        | DataType::Varchar { .. }
        | DataType::WVarchar { .. }
        | DataType::LongVarchar { .. }
        | DataType::WLongVarchar { .. }
        | DataType::Other { .. }
        | DataType::Unknown => BufferDesc::Text {
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

        for row_index in 0..batch.num_rows() {
            let row_values: Vec<_> = bindings
                .iter()
                .enumerate()
                .map(|(col_index, binding)| {
                    let type_info = binding.column.type_info.clone();
                    let value =
                        extract_value_from_buffer(batch.column(col_index), row_index, &type_info);
                    (type_info, value)
                })
                .collect();

            let row = OdbcRow {
                columns: columns.clone(),
                values: row_values.into_iter().map(|(_, value)| value).collect(),
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

fn extract_value_from_buffer(
    slice: AnySlice<'_>,
    row_index: usize,
    type_info: &OdbcTypeInfo,
) -> OdbcValue {
    match slice {
        AnySlice::I8(s) => OdbcValue {
            type_info: type_info.clone(),
            is_null: false,
            text: None,
            blob: None,
            int: Some(s[row_index] as i64),
            float: None,
        },
        AnySlice::I16(s) => OdbcValue {
            type_info: type_info.clone(),
            is_null: false,
            text: None,
            blob: None,
            int: Some(s[row_index] as i64),
            float: None,
        },
        AnySlice::I32(s) => OdbcValue {
            type_info: type_info.clone(),
            is_null: false,
            text: None,
            blob: None,
            int: Some(s[row_index] as i64),
            float: None,
        },
        AnySlice::I64(s) => OdbcValue {
            type_info: type_info.clone(),
            is_null: false,
            text: None,
            blob: None,
            int: Some(s[row_index]),
            float: None,
        },
        AnySlice::F32(s) => OdbcValue {
            type_info: type_info.clone(),
            is_null: false,
            text: None,
            blob: None,
            int: None,
            float: Some(s[row_index] as f64),
        },
        AnySlice::F64(s) => OdbcValue {
            type_info: type_info.clone(),
            is_null: false,
            text: None,
            blob: None,
            int: None,
            float: Some(s[row_index]),
        },
        AnySlice::Bit(s) => OdbcValue {
            type_info: type_info.clone(),
            is_null: false,
            text: None,
            blob: None,
            int: Some(s[row_index].0 as i64),
            float: None,
        },
        AnySlice::Text(s) => {
            let text = s
                .get(row_index)
                .map(|bytes| String::from_utf8_lossy(bytes).to_string());
            OdbcValue {
                type_info: type_info.clone(),
                is_null: text.is_none(),
                text,
                blob: None,
                int: None,
                float: None,
            }
        }
        AnySlice::Binary(s) => {
            let blob = s.get(row_index).map(|bytes| bytes.to_vec());
            OdbcValue {
                type_info: type_info.clone(),
                is_null: blob.is_none(),
                text: None,
                blob,
                int: None,
                float: None,
            }
        }
        AnySlice::NullableI8(s) => {
            let (is_null, int) = if let Some(&val) = s.get(row_index) {
                (false, Some(val as i64))
            } else {
                (true, None)
            };
            OdbcValue {
                type_info: type_info.clone(),
                is_null,
                text: None,
                blob: None,
                int,
                float: None,
            }
        }
        AnySlice::NullableI16(s) => {
            let (is_null, int) = if let Some(&val) = s.get(row_index) {
                (false, Some(val as i64))
            } else {
                (true, None)
            };
            OdbcValue {
                type_info: type_info.clone(),
                is_null,
                text: None,
                blob: None,
                int,
                float: None,
            }
        }
        AnySlice::NullableI32(s) => {
            let (is_null, int) = if let Some(&val) = s.get(row_index) {
                (false, Some(val as i64))
            } else {
                (true, None)
            };
            OdbcValue {
                type_info: type_info.clone(),
                is_null,
                text: None,
                blob: None,
                int,
                float: None,
            }
        }
        AnySlice::NullableI64(s) => {
            let (is_null, int) = if let Some(&val) = s.get(row_index) {
                (false, Some(val))
            } else {
                (true, None)
            };
            OdbcValue {
                type_info: type_info.clone(),
                is_null,
                text: None,
                blob: None,
                int,
                float: None,
            }
        }
        AnySlice::NullableF32(s) => {
            let (is_null, float) = if let Some(&val) = s.get(row_index) {
                (false, Some(val as f64))
            } else {
                (true, None)
            };
            OdbcValue {
                type_info: type_info.clone(),
                is_null,
                text: None,
                blob: None,
                int: None,
                float,
            }
        }
        AnySlice::NullableF64(s) => {
            let (is_null, float) = if let Some(&val) = s.get(row_index) {
                (false, Some(val))
            } else {
                (true, None)
            };
            OdbcValue {
                type_info: type_info.clone(),
                is_null,
                text: None,
                blob: None,
                int: None,
                float,
            }
        }
        AnySlice::NullableBit(s) => {
            let (is_null, int) = if let Some(&val) = s.get(row_index) {
                (false, Some(val.0 as i64))
            } else {
                (true, None)
            };
            OdbcValue {
                type_info: type_info.clone(),
                is_null,
                text: None,
                blob: None,
                int,
                float: None,
            }
        }
        AnySlice::Date(s) => {
            let text = s.get(row_index).map(|date| format!("{:?}", date));
            OdbcValue {
                type_info: type_info.clone(),
                is_null: text.is_none(),
                text,
                blob: None,
                int: None,
                float: None,
            }
        }
        AnySlice::Time(s) => {
            let text = s.get(row_index).map(|time| format!("{:?}", time));
            OdbcValue {
                type_info: type_info.clone(),
                is_null: text.is_none(),
                text,
                blob: None,
                int: None,
                float: None,
            }
        }
        AnySlice::Timestamp(s) => {
            let text = s.get(row_index).map(|ts| format!("{:?}", ts));
            OdbcValue {
                type_info: type_info.clone(),
                is_null: text.is_none(),
                text,
                blob: None,
                int: None,
                float: None,
            }
        }
        _ => OdbcValue {
            type_info: type_info.clone(),
            is_null: true,
            text: None,
            blob: None,
            int: None,
            float: None,
        },
    }
}
