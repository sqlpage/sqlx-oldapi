use super::decode_column_name;
use crate::error::Error;
use crate::odbc::{
    connection::MaybePrepared, OdbcArgumentValue, OdbcArguments, OdbcColumn, OdbcQueryResult,
    OdbcRow, OdbcTypeInfo,
};
use either::Either;
use flume::{SendError, Sender};
use odbc_api::handles::{AsStatementRef, Statement};
use odbc_api::{Cursor, CursorRow, IntoParameter, Nullable, ResultSetMetadata};

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
            if let Some(mut cursor) = prepared.execute(&params[..])? {
                handle_cursor(&mut cursor, tx);
            }
            extract_rows_affected(&mut *prepared)
        }
        MaybePrepared::NotPrepared(sql) => {
            let mut preallocated = conn.preallocate().map_err(Error::from)?;
            if let Some(mut cursor) = preallocated.execute(&sql, &params[..])? {
                handle_cursor(&mut cursor, tx);
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

fn handle_cursor<C>(cursor: &mut C, tx: &ExecuteSender)
where
    C: Cursor + ResultSetMetadata,
{
    let columns = collect_columns(cursor);

    match stream_rows(cursor, &columns, tx) {
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

fn collect_columns<C>(cursor: &mut C) -> Vec<OdbcColumn>
where
    C: ResultSetMetadata,
{
    let count = cursor.num_result_cols().unwrap_or(0);
    (1..=count)
        .map(|i| create_column(cursor, i as u16))
        .collect()
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

fn stream_rows<C>(cursor: &mut C, columns: &[OdbcColumn], tx: &ExecuteSender) -> Result<bool, Error>
where
    C: Cursor,
{
    let mut receiver_open = true;

    while let Some(mut row) = cursor.next_row()? {
        let values = collect_row_values(&mut row, columns)?;
        let row_data = OdbcRow {
            columns: columns.to_vec(),
            values: values.into_iter().map(|(_, value)| value).collect(),
        };

        if send_row(tx, row_data).is_err() {
            receiver_open = false;
            break;
        }
    }
    Ok(receiver_open)
}

fn collect_row_values(
    row: &mut CursorRow<'_>,
    columns: &[OdbcColumn],
) -> Result<Vec<(OdbcTypeInfo, crate::odbc::OdbcValue)>, Error> {
    columns
        .iter()
        .enumerate()
        .map(|(i, column)| collect_column_value(row, i, column))
        .collect()
}

fn collect_column_value(
    row: &mut CursorRow<'_>,
    index: usize,
    column: &OdbcColumn,
) -> Result<(OdbcTypeInfo, crate::odbc::OdbcValue), Error> {
    use odbc_api::DataType;

    let col_idx = (index + 1) as u16;
    let type_info = column.type_info.clone();
    let data_type = type_info.data_type();

    let value = match data_type {
        DataType::TinyInt
        | DataType::SmallInt
        | DataType::Integer
        | DataType::BigInt
        | DataType::Bit => extract_int(row, col_idx, &type_info)?,

        DataType::Real => extract_float::<f32>(row, col_idx, &type_info)?,
        DataType::Float { .. } | DataType::Double => {
            extract_float::<f64>(row, col_idx, &type_info)?
        }

        DataType::Char { .. }
        | DataType::Varchar { .. }
        | DataType::LongVarchar { .. }
        | DataType::WChar { .. }
        | DataType::WVarchar { .. }
        | DataType::WLongVarchar { .. }
        | DataType::Date
        | DataType::Time { .. }
        | DataType::Timestamp { .. }
        | DataType::Decimal { .. }
        | DataType::Numeric { .. } => extract_text(row, col_idx, &type_info)?,

        DataType::Binary { .. } | DataType::Varbinary { .. } | DataType::LongVarbinary { .. } => {
            extract_binary(row, col_idx, &type_info)?
        }

        DataType::Unknown | DataType::Other { .. } => {
            match extract_text(row, col_idx, &type_info) {
                Ok(v) => v,
                Err(_) => extract_binary(row, col_idx, &type_info)?,
            }
        }
    };

    Ok((type_info, value))
}

fn extract_int(
    row: &mut CursorRow<'_>,
    col_idx: u16,
    type_info: &OdbcTypeInfo,
) -> Result<crate::odbc::OdbcValue, Error> {
    let mut nullable = Nullable::<i64>::null();
    row.get_data(col_idx, &mut nullable)?;

    let (is_null, int) = match nullable.into_opt() {
        None => (true, None),
        Some(v) => (false, Some(v)),
    };

    Ok(crate::odbc::OdbcValue {
        type_info: type_info.clone(),
        is_null,
        text: None,
        blob: None,
        int,
        float: None,
    })
}

fn extract_float<T>(
    row: &mut CursorRow<'_>,
    col_idx: u16,
    type_info: &OdbcTypeInfo,
) -> Result<crate::odbc::OdbcValue, Error>
where
    T: Into<f64> + Default,
    odbc_api::Nullable<T>: odbc_api::parameter::CElement + odbc_api::handles::CDataMut,
{
    let mut nullable = Nullable::<T>::null();
    row.get_data(col_idx, &mut nullable)?;

    let (is_null, float) = match nullable.into_opt() {
        None => (true, None),
        Some(v) => (false, Some(v.into())),
    };

    Ok(crate::odbc::OdbcValue {
        type_info: type_info.clone(),
        is_null,
        text: None,
        blob: None,
        int: None,
        float,
    })
}

fn extract_text(
    row: &mut CursorRow<'_>,
    col_idx: u16,
    type_info: &OdbcTypeInfo,
) -> Result<crate::odbc::OdbcValue, Error> {
    let mut buf = Vec::new();
    let is_some = row.get_text(col_idx, &mut buf)?;

    let (is_null, text) = if !is_some {
        (true, None)
    } else {
        match String::from_utf8(buf) {
            Ok(s) => (false, Some(s)),
            Err(e) => return Err(Error::Decode(e.into())),
        }
    };

    Ok(crate::odbc::OdbcValue {
        type_info: type_info.clone(),
        is_null,
        text,
        blob: None,
        int: None,
        float: None,
    })
}

fn extract_binary(
    row: &mut CursorRow<'_>,
    col_idx: u16,
    type_info: &OdbcTypeInfo,
) -> Result<crate::odbc::OdbcValue, Error> {
    let mut buf = Vec::new();
    let is_some = row.get_binary(col_idx, &mut buf)?;

    let (is_null, blob) = if !is_some {
        (true, None)
    } else {
        (false, Some(buf))
    };

    Ok(crate::odbc::OdbcValue {
        type_info: type_info.clone(),
        is_null,
        text: None,
        blob,
        int: None,
        float: None,
    })
}
