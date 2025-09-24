use crate::connection::{Connection, LogSettings};
use crate::error::Error;
use crate::odbc::{Odbc, OdbcArgumentValue, OdbcArguments, OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow, OdbcTypeInfo};
use crate::transaction::Transaction;
use either::Either;
use flume::SendError;
use futures_core::future::BoxFuture;
use futures_util::future;
use odbc_api::handles::StatementImpl;
use odbc_api::{Cursor, CursorRow, IntoParameter, Nullable, Preallocated, ResultSetMetadata};
use sqlx_rt::spawn_blocking;
use std::sync::{Arc, Mutex};

mod executor;

/// A connection to an ODBC-accessible database.
///
/// ODBC uses a blocking C API, so we offload blocking calls to the runtime's blocking
/// thread-pool via `spawn_blocking` and synchronize access with a mutex.
#[derive(Debug)]
pub struct OdbcConnection {
    pub(crate) inner: Arc<Mutex<odbc_api::Connection<'static>>>,
    pub(crate) log_settings: LogSettings,
}

impl OdbcConnection {
    pub(crate) async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let conn = spawn_blocking({
            let options = options.clone();
            move || establish_connection(&options)
        })
        .await
        .map_err(|_| Error::WorkerCrashed)??;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            log_settings: LogSettings::default(),
        })
    }

    /// Returns the name of the actual Database Management System (DBMS) this
    /// connection is talking to as reported by the ODBC driver.
    pub async fn dbms_name(&mut self) -> Result<String, Error> {
        let inner = self.inner.clone();
        spawn_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.database_management_system_name()
                .map_err(|e| Error::Protocol(format!("Failed to get DBMS name: {}", e)))
        })
        .await
        .map_err(|_| Error::WorkerCrashed)?
    }

    pub(crate) async fn ping_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        spawn_blocking(move || {
            let conn = inner.lock().unwrap();
            let res = conn.execute("SELECT 1", (), None);
            match res {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Protocol(format!("Ping failed: {}", e))),
            }
        })
        .await
        .map_err(|_| Error::WorkerCrashed)?
    }

    pub(crate) async fn begin_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        spawn_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.set_autocommit(false)
                .map_err(|e| Error::Protocol(format!("Failed to begin transaction: {}", e)))
        })
        .await
        .map_err(|_| Error::WorkerCrashed)?
    }

    pub(crate) async fn commit_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        spawn_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.commit()
                .and_then(|_| conn.set_autocommit(true))
                .map_err(|e| Error::Protocol(format!("Failed to commit transaction: {}", e)))
        })
        .await
        .map_err(|_| Error::WorkerCrashed)?
    }

    pub(crate) async fn rollback_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        spawn_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.rollback()
                .and_then(|_| conn.set_autocommit(true))
                .map_err(|e| Error::Protocol(format!("Failed to rollback transaction: {}", e)))
        })
        .await
        .map_err(|_| Error::WorkerCrashed)?
    }

    pub(crate) async fn execute_stream(
        &mut self,
        sql: &str,
        args: Option<OdbcArguments>,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        let inner = self.inner.clone();
        let sql = sql.to_string();
        spawn_blocking(move || {
            let mut guard = inner.lock().unwrap();
            if let Err(e) = execute_sql(&mut guard, &sql, args, &tx) {
                let _ = send_stream_result(&tx, Err(e));
            }
        })
        .await
        .map_err(|_| Error::WorkerCrashed)?;
        Ok(rx)
    }

    pub(crate) async fn prepare(
        &mut self,
        sql: &str,
    ) -> Result<(u64, Vec<OdbcColumn>, usize), Error> {
        let inner = self.inner.clone();
        let sql = sql.to_string();
        spawn_blocking(move || do_prepare(&mut inner.lock().unwrap(), sql.into()))
            .await
            .map_err(|_| Error::WorkerCrashed)?
    }
}

impl Connection for OdbcConnection {
    type Database = Odbc;

    type Options = OdbcConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            // Drop connection by moving Arc and letting it fall out of scope.
            drop(self);
            Ok(())
        })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.ping_blocking())
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(future::ok(()))
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        false
    }
}

// --- Blocking helpers ---

fn establish_connection(options: &OdbcConnectOptions) -> Result<odbc_api::Connection<'static>, Error> {
    let env = odbc_api::environment().map_err(|e| Error::Configuration(e.to_string().into()))?;
    let conn = env
        .connect_with_connection_string(options.connection_string(), Default::default())
        .map_err(|e| Error::Configuration(e.to_string().into()))?;
    Ok(conn)
}

type ExecuteResult = Result<Either<OdbcQueryResult, OdbcRow>, Error>;
type ExecuteSender = flume::Sender<ExecuteResult>;

fn send_stream_result(
    tx: &ExecuteSender,
    result: ExecuteResult,
) -> Result<(), SendError<ExecuteResult>> {
    tx.send(result)
}

fn execute_sql(
    conn: &mut odbc_api::Connection<'static>,
    sql: &str,
    args: Option<OdbcArguments>,
    tx: &ExecuteSender,
) -> Result<(), Error> {
    let params = prepare_parameters(args);
    let stmt = &mut conn.preallocate().map_err(Error::from)?;

    if let Some(mut cursor) = stmt.execute(sql, &params[..])? {
        handle_cursor(&mut cursor, tx);
        return Ok(());
    }

    let affected = extract_rows_affected(stmt);
    let _ = send_done(tx, affected);
    Ok(())
}

fn extract_rows_affected(stmt: &mut Preallocated<StatementImpl<'_>>) -> u64 {
    let count_opt = match stmt.row_count() {
        Ok(count_opt) => count_opt,
        Err(e) => {
            log::warn!("Failed to get ODBC row count: {}", e);
            return 0;
        }
    };

    let count = match count_opt {
        Some(count) => count,
        None => {
            log::debug!("ODBC row count is not available");
            return 0;
        }
    };

    let affected = match u64::try_from(count) {
        Ok(count) => count,
        Err(e) => {
            log::warn!("Failed to convert ODBC row count to u64: {}", e);
            return 0;
        }
    };
    affected
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
    send_stream_result(tx, Ok(Either::Left(OdbcQueryResult { rows_affected })))
}

fn send_error(tx: &ExecuteSender, error: Error) {
    if let Err(e) = send_stream_result(tx, Err(error)) {
        log::error!("Failed to send error from ODBC blocking task: {}", e);
    }
}

fn send_row(tx: &ExecuteSender, row: OdbcRow) -> Result<(), SendError<ExecuteResult>> {
    send_stream_result(tx, Ok(Either::Right(row)))
}

fn collect_columns<C>(cursor: &mut C) -> Vec<OdbcColumn>
where
    C: ResultSetMetadata,
{
    let count = cursor.num_result_cols().unwrap_or(0);
    (1..=count).map(|i| create_column(cursor, i as u16)).collect()
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

fn stream_rows<C>(cursor: &mut C, columns: &[OdbcColumn], tx: &ExecuteSender) -> Result<bool, Error>
where
    C: Cursor,
{
    let mut receiver_open = true;
    let mut row_count = 0;

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
        row_count += 1;
    }

    let _ = row_count; // kept for potential logging
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
        DataType::Float { .. } | DataType::Double => extract_float::<f64>(row, col_idx, &type_info)?,

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

        DataType::Unknown | DataType::Other { .. } => match extract_text(row, col_idx, &type_info) {
            Ok(v) => v,
            Err(_) => extract_binary(row, col_idx, &type_info)?,
        },
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

    let (is_null, blob) = if !is_some { (true, None) } else { (false, Some(buf)) };

    Ok(crate::odbc::OdbcValue {
        type_info: type_info.clone(),
        is_null,
        text: None,
        blob,
        int: None,
        float: None,
    })
}

fn do_prepare(
    conn: &mut odbc_api::Connection<'static>,
    sql: Box<str>,
) -> Result<(u64, Vec<OdbcColumn>, usize), Error> {
    let mut prepared = conn.prepare(&sql)?;
    let columns = collect_columns(&mut prepared);
    let params = usize::from(prepared.num_params().unwrap_or(0));
    Ok((0, columns, params))
}
