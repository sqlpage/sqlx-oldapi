use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::thread;

use flume::{SendError, TrySendError};
use futures_channel::oneshot;

use crate::error::Error;
use crate::odbc::{
    OdbcArgumentValue, OdbcArguments, OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow,
    OdbcTypeInfo,
};
#[allow(unused_imports)]
use crate::row::Row as SqlxRow;
use either::Either;
#[allow(unused_imports)]
use odbc_api::handles::Statement as OdbcStatementTrait;
use odbc_api::handles::StatementImpl;
use odbc_api::{Cursor, CursorRow, IntoParameter, Nullable, Preallocated, ResultSetMetadata};

// Type aliases for commonly used types
type OdbcConnection = odbc_api::Connection<'static>;
type TransactionResult = Result<(), Error>;
type TransactionSender = oneshot::Sender<TransactionResult>;
type ExecuteResult = Result<Either<OdbcQueryResult, OdbcRow>, Error>;
type ExecuteSender = flume::Sender<ExecuteResult>;
type PrepareResult = Result<(u64, Vec<OdbcColumn>, usize), Error>;
type PrepareSender = oneshot::Sender<PrepareResult>;

#[derive(Debug)]
pub(crate) struct ConnectionWorker {
    command_tx: flume::Sender<Command>,
    join_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Debug)]
enum Command {
    Ping {
        tx: oneshot::Sender<()>,
    },
    Shutdown {
        tx: oneshot::Sender<()>,
    },
    Begin {
        tx: TransactionSender,
    },
    Commit {
        tx: TransactionSender,
    },
    Rollback {
        tx: TransactionSender,
    },
    Execute {
        sql: Box<str>,
        args: Option<OdbcArguments>,
        tx: ExecuteSender,
    },
    Prepare {
        sql: Box<str>,
        tx: PrepareSender,
    },
    GetDbmsName {
        tx: oneshot::Sender<Result<String, Error>>,
    },
}

impl Drop for ConnectionWorker {
    fn drop(&mut self) {
        self.shutdown_sync();
    }
}

impl ConnectionWorker {
    pub async fn establish(options: OdbcConnectOptions) -> Result<Self, Error> {
        let (command_tx, command_rx) = flume::bounded(64);
        let (conn_tx, conn_rx) = oneshot::channel();
        let thread = thread::Builder::new()
            .name("sqlx-odbc-conn".into())
            .spawn(move || worker_thread_main(options, command_rx, conn_tx))?;

        conn_rx.await.map_err(|_| Error::WorkerCrashed)??;
        Ok(ConnectionWorker {
            command_tx,
            join_handle: Some(thread),
        })
    }

    pub(crate) async fn ping(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        send_command_and_await(&self.command_tx, Command::Ping { tx }, rx).await
    }

    pub(crate) async fn shutdown(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        send_command_and_await(&self.command_tx, Command::Shutdown { tx }, rx).await
    }

    pub(crate) fn shutdown_sync(&mut self) {
        // Send shutdown command without waiting for response
        // Use try_send to avoid any potential blocking in Drop

        if let Some(join_handle) = self.join_handle.take() {
            let (mut tx, _rx) = oneshot::channel();
            while let Err(TrySendError::Full(Command::Shutdown { tx: t })) =
                self.command_tx.try_send(Command::Shutdown { tx })
            {
                tx = t;
                log::warn!("odbc worker thread queue is full, retrying...");
                thread::sleep(std::time::Duration::from_millis(10));
            }
            if let Err(e) = join_handle.join() {
                let err = e.downcast_ref::<std::io::Error>();
                log::error!(
                    "failed to join worker thread while shutting down: {:?}",
                    err
                );
            }
        }
    }

    pub(crate) async fn begin(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        send_transaction_command(&self.command_tx, Command::Begin { tx }, rx).await
    }

    pub(crate) async fn commit(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        send_transaction_command(&self.command_tx, Command::Commit { tx }, rx).await
    }

    pub(crate) async fn rollback(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        send_transaction_command(&self.command_tx, Command::Rollback { tx }, rx).await
    }

    pub(crate) async fn execute_stream(
        &mut self,
        sql: &str,
        args: Option<OdbcArguments>,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        self.command_tx
            .send_async(Command::Execute {
                sql: sql.into(),
                args,
                tx,
            })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        Ok(rx)
    }

    pub(crate) async fn prepare(
        &mut self,
        sql: &str,
    ) -> Result<(u64, Vec<OdbcColumn>, usize), Error> {
        let (tx, rx) = oneshot::channel();
        send_command_and_await(
            &self.command_tx,
            Command::Prepare {
                sql: sql.into(),
                tx,
            },
            rx,
        )
        .await?
    }

    pub(crate) async fn get_dbms_name(&mut self) -> Result<String, Error> {
        let (tx, rx) = oneshot::channel();
        send_command_and_await(&self.command_tx, Command::GetDbmsName { tx }, rx).await?
    }
}

// Worker thread implementation
fn worker_thread_main(
    options: OdbcConnectOptions,
    command_rx: flume::Receiver<Command>,
    conn_tx: oneshot::Sender<Result<(), Error>>,
) {
    // Establish connection
    let conn = match establish_connection(&options) {
        Ok(conn) => {
            log::debug!("ODBC connection established successfully");
            let _ = conn_tx.send(Ok(()));
            conn
        }
        Err(e) => {
            let _ = conn_tx.send(Err(e));
            return;
        }
    };

    let mut stmt_manager = StatementManager::new(&conn);

    // Process commands
    while let Ok(cmd) = command_rx.recv() {
        log::trace!("Processing command: {:?}", cmd);
        match process_command(cmd, &conn, &mut stmt_manager) {
            Ok(CommandControlFlow::Continue) => {}
            Ok(CommandControlFlow::Shutdown(shutdown_tx)) => {
                log::debug!("Shutting down ODBC worker thread");
                drop(stmt_manager);
                drop(conn);
                send_oneshot(shutdown_tx, (), "shutdown");
                break;
            }
            Err(()) => {
                log::error!("ODBC worker error while processing command");
            }
        }
    }
    // Channel disconnected or shutdown command received, worker thread exits
}

fn establish_connection(options: &OdbcConnectOptions) -> Result<OdbcConnection, Error> {
    // Get or create the shared ODBC environment
    // This ensures thread-safe initialization and prevents concurrent environment creation issues
    let env = odbc_api::environment().map_err(|e| Error::Configuration(e.to_string().into()))?;

    let conn = env
        .connect_with_connection_string(options.connection_string(), Default::default())
        .map_err(|e| Error::Configuration(e.to_string().into()))?;

    Ok(conn)
}

/// Statement manager to handle preallocated statements
struct StatementManager<'conn> {
    conn: &'conn OdbcConnection,
    // Reusable statement for direct execution
    direct_stmt: Option<Preallocated<StatementImpl<'conn>>>,
    // Cache of prepared statements by SQL hash
    prepared_cache: HashMap<u64, odbc_api::Prepared<StatementImpl<'conn>>>,
}

impl<'conn> StatementManager<'conn> {
    fn new(conn: &'conn OdbcConnection) -> Self {
        log::debug!("Creating new statement manager");
        Self {
            conn,
            direct_stmt: None,
            prepared_cache: HashMap::new(),
        }
    }

    fn get_or_create_direct_stmt(
        &mut self,
    ) -> Result<&mut Preallocated<StatementImpl<'conn>>, Error> {
        if self.direct_stmt.is_none() {
            log::debug!("Preallocating ODBC direct statement");
            self.direct_stmt = Some(self.conn.preallocate().map_err(Error::from)?);
        }
        Ok(self.direct_stmt.as_mut().unwrap())
    }

    fn get_or_create_prepared(
        &mut self,
        sql: &str,
    ) -> Result<&mut odbc_api::Prepared<StatementImpl<'conn>>, Error> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        let sql_hash = hasher.finish();

        match self.prepared_cache.entry(sql_hash) {
            Entry::Vacant(e) => {
                log::trace!("Preparing statement for SQL: {}", sql);
                let prepared = self.conn.prepare(sql)?;
                Ok(e.insert(prepared))
            }
            Entry::Occupied(e) => {
                log::trace!("Using prepared statement for SQL: {}", sql);
                Ok(e.into_mut())
            }
        }
    }
}
// Helper function to send results through oneshot channels with consistent error handling
fn send_oneshot<T>(tx: oneshot::Sender<T>, result: T, operation: &str) {
    if tx.send(result).is_err() {
        log::warn!("Failed to send {} result: receiver dropped", operation);
    }
}

fn send_stream_result(
    tx: &ExecuteSender,
    result: ExecuteResult,
) -> Result<(), SendError<ExecuteResult>> {
    tx.send(result)
}

async fn send_command_and_await<T>(
    command_tx: &flume::Sender<Command>,
    cmd: Command,
    rx: oneshot::Receiver<T>,
) -> Result<T, Error> {
    command_tx
        .send_async(cmd)
        .await
        .map_err(|_| Error::WorkerCrashed)?;
    rx.await.map_err(|_| Error::WorkerCrashed)
}

async fn send_transaction_command(
    command_tx: &flume::Sender<Command>,
    cmd: Command,
    rx: oneshot::Receiver<TransactionResult>,
) -> Result<(), Error> {
    send_command_and_await(command_tx, cmd, rx).await??;
    Ok(())
}

// Utility functions for transaction operations
fn execute_transaction_operation<F>(
    conn: &OdbcConnection,
    operation: F,
    operation_name: &str,
) -> TransactionResult
where
    F: FnOnce(&OdbcConnection) -> Result<(), odbc_api::Error>,
{
    log::trace!("{} odbc transaction", operation_name);
    operation(conn)
        .map_err(|e| Error::Protocol(format!("Failed to {} transaction: {}", operation_name, e)))
}

#[derive(Debug)]
enum CommandControlFlow {
    Shutdown(oneshot::Sender<()>),
    Continue,
}

type CommandResult = Result<CommandControlFlow, ()>;

// Returns a shutdown tx if the command is a shutdown command
fn process_command<'conn>(
    cmd: Command,
    conn: &'conn OdbcConnection,
    stmt_manager: &mut StatementManager<'conn>,
) -> CommandResult {
    match cmd {
        Command::Ping { tx } => handle_ping(conn, tx),
        Command::Begin { tx } => handle_begin(conn, tx),
        Command::Commit { tx } => handle_commit(conn, tx),
        Command::Rollback { tx } => handle_rollback(conn, tx),
        Command::Shutdown { tx } => Ok(CommandControlFlow::Shutdown(tx)),
        Command::Execute { sql, args, tx } => handle_execute(stmt_manager, sql, args, tx),
        Command::Prepare { sql, tx } => handle_prepare(stmt_manager, sql, tx),
        Command::GetDbmsName { tx } => handle_get_dbms_name(conn, tx),
    }
}

// Command handlers
fn handle_ping(conn: &OdbcConnection, tx: oneshot::Sender<()>) -> CommandResult {
    match conn.execute("SELECT 1", (), None) {
        Ok(_) => send_oneshot(tx, (), "ping"),
        Err(e) => log::error!("Ping failed: {}", e),
    }
    Ok(CommandControlFlow::Continue)
}

fn handle_begin(conn: &OdbcConnection, tx: TransactionSender) -> CommandResult {
    let result = execute_transaction_operation(conn, |c| c.set_autocommit(false), "begin");
    send_oneshot(tx, result, "begin transaction");
    Ok(CommandControlFlow::Continue)
}

fn handle_commit(conn: &OdbcConnection, tx: TransactionSender) -> CommandResult {
    let result = execute_transaction_operation(
        conn,
        |c| c.commit().and_then(|_| c.set_autocommit(true)),
        "commit",
    );
    send_oneshot(tx, result, "commit transaction");
    Ok(CommandControlFlow::Continue)
}

fn handle_rollback(conn: &OdbcConnection, tx: TransactionSender) -> CommandResult {
    let result = execute_transaction_operation(
        conn,
        |c| c.rollback().and_then(|_| c.set_autocommit(true)),
        "rollback",
    );
    send_oneshot(tx, result, "rollback transaction");
    Ok(CommandControlFlow::Continue)
}
fn handle_prepare<'conn>(
    stmt_manager: &mut StatementManager<'conn>,
    sql: Box<str>,
    tx: PrepareSender,
) -> CommandResult {
    let result = do_prepare(stmt_manager, sql);
    send_oneshot(tx, result, "prepare");
    Ok(CommandControlFlow::Continue)
}

fn do_prepare<'conn>(stmt_manager: &mut StatementManager<'conn>, sql: Box<str>) -> PrepareResult {
    log::trace!("Preparing statement: {}", sql);
    // Use the statement manager to get or create the prepared statement
    let prepared = stmt_manager.get_or_create_prepared(&sql)?;
    let columns = collect_columns(prepared);
    let params = usize::from(prepared.num_params().unwrap_or(0));
    log::debug!(
        "Prepared statement with {} columns and {} parameters",
        columns.len(),
        params
    );
    Ok((0, columns, params))
}

fn handle_get_dbms_name(
    conn: &OdbcConnection,
    tx: oneshot::Sender<Result<String, Error>>,
) -> CommandResult {
    log::debug!("Getting DBMS name");
    let result = conn
        .database_management_system_name()
        .map_err(|e| Error::Protocol(format!("Failed to get DBMS name: {}", e)));
    send_oneshot(tx, result, "DBMS name");
    Ok(CommandControlFlow::Continue)
}

fn handle_execute<'conn>(
    stmt_manager: &mut StatementManager<'conn>,
    sql: Box<str>,
    args: Option<OdbcArguments>,
    tx: ExecuteSender,
) -> CommandResult {
    log::debug!(
        "Executing SQL: {}",
        sql.chars().take(100).collect::<String>()
    );

    let result = execute_sql(stmt_manager, &sql, args, &tx);
    with_result_send_error(result, &tx, |_| {});
    Ok(CommandControlFlow::Continue)
}

// SQL execution functions
fn execute_sql<'conn>(
    stmt_manager: &mut StatementManager<'conn>,
    sql: &str,
    args: Option<OdbcArguments>,
    tx: &ExecuteSender,
) -> Result<(), Error> {
    let params = prepare_parameters(args);
    let stmt = stmt_manager.get_or_create_direct_stmt()?;
    log::trace!("Starting execution of SQL: {}", sql);
    let cursor_result = stmt.execute(sql, &params[..]);
    log::trace!("Received cursor result for SQL: {}", sql);
    send_exec_result(cursor_result, tx)?;
    Ok(())
}

// Unified execution logic for both direct and prepared statements
fn send_exec_result<C>(
    execution_result: Result<Option<C>, odbc_api::Error>,
    tx: &ExecuteSender,
) -> Result<(), Error>
where
    C: Cursor + ResultSetMetadata,
{
    match execution_result {
        Ok(Some(mut cursor)) => {
            handle_cursor(&mut cursor, tx);
            Ok(())
        }
        Ok(None) => {
            let _ = send_done(tx, 0);
            Ok(())
        }
        Err(e) => Err(Error::from(e)),
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
    log::trace!("Processing ODBC result set with {} columns", columns.len());

    match stream_rows(cursor, &columns, tx) {
        Ok(true) => {
            log::trace!("Successfully streamed all rows");
            let _ = send_done(tx, 0);
        }
        Ok(false) => {
            log::trace!("Row streaming stopped early (receiver closed)");
        }
        Err(e) => {
            send_error(tx, e);
        }
    }
}

// Unified result sending functions
fn send_done(tx: &ExecuteSender, rows_affected: u64) -> Result<(), SendError<ExecuteResult>> {
    send_stream_result(tx, Ok(Either::Left(OdbcQueryResult { rows_affected })))
}

fn with_result_send_error<T>(
    result: Result<T, Error>,
    tx: &ExecuteSender,
    handler: impl FnOnce(T),
) {
    match result {
        Ok(result) => handler(result),
        Err(error) => send_error(tx, error),
    }
}

fn send_error(tx: &ExecuteSender, error: Error) {
    if let Err(e) = send_stream_result(tx, Err(error)) {
        log::error!("Failed to send error from ODBC worker thread: {}", e);
    }
}

fn send_row(tx: &ExecuteSender, row: OdbcRow) -> Result<(), SendError<ExecuteResult>> {
    send_stream_result(tx, Ok(Either::Right(row)))
}

// Metadata and row processing
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
            log::debug!("Receiver closed after {} rows", row_count);
            receiver_open = false;
            break;
        }
        row_count += 1;
    }

    if receiver_open {
        log::debug!("Streamed {} rows successfully", row_count);
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

    // Extract value based on data type
    let value = match data_type {
        // Integer types
        DataType::TinyInt
        | DataType::SmallInt
        | DataType::Integer
        | DataType::BigInt
        | DataType::Bit => extract_int(row, col_idx, &type_info)?,

        // Floating point types
        DataType::Real => extract_float::<f32>(row, col_idx, &type_info)?,
        DataType::Float { .. } | DataType::Double => {
            extract_float::<f64>(row, col_idx, &type_info)?
        }

        // String types
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

        // Binary types
        DataType::Binary { .. } | DataType::Varbinary { .. } | DataType::LongVarbinary { .. } => {
            extract_binary(row, col_idx, &type_info)?
        }

        // Unknown types - try text first, fall back to binary
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
        Some(v) => (false, Some(v.into())),
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
