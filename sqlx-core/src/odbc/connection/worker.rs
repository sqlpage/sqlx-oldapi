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
use odbc_api::{Cursor, CursorRow, IntoParameter, ResultSetMetadata};

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
            let _ = conn_tx.send(Ok(()));
            conn
        }
        Err(e) => {
            let _ = conn_tx.send(Err(e));
            return;
        }
    };
    // Process commands
    while let Ok(cmd) = command_rx.recv() {
        if let Some(shutdown_tx) = process_command(cmd, &conn) {
            drop(conn);
            let _ = shutdown_tx.send(());
            break;
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

// Utility functions for channel operations
fn send_result<T: std::fmt::Debug>(tx: oneshot::Sender<T>, result: T) {
    let _ = tx.send(result);
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
    operation(conn)
        .map_err(|e| Error::Protocol(format!("Failed to {} transaction: {}", operation_name, e)))
}

// Returns a shutdown tx if the command is a shutdown command
fn process_command(cmd: Command, conn: &OdbcConnection) -> Option<oneshot::Sender<()>> {
    match cmd {
        Command::Ping { tx } => handle_ping(conn, tx),
        Command::Begin { tx } => handle_begin(conn, tx),
        Command::Commit { tx } => handle_commit(conn, tx),
        Command::Rollback { tx } => handle_rollback(conn, tx),
        Command::Shutdown { tx } => return Some(tx),
        Command::Execute { sql, args, tx } => handle_execute(conn, sql, args, tx),
        Command::Prepare { sql, tx } => handle_prepare(conn, sql, tx),
        Command::GetDbmsName { tx } => handle_get_dbms_name(conn, tx),
    }
    None
}

// Command handlers
fn handle_ping(conn: &OdbcConnection, tx: oneshot::Sender<()>) {
    let _ = conn.execute("SELECT 1", (), None);
    send_result(tx, ());
}

fn handle_begin(conn: &OdbcConnection, tx: TransactionSender) {
    let result = execute_transaction_operation(conn, |c| c.set_autocommit(false), "begin");
    send_result(tx, result);
}

fn handle_commit(conn: &OdbcConnection, tx: TransactionSender) {
    let result = execute_transaction_operation(
        conn,
        |c| c.commit().and_then(|_| c.set_autocommit(true)),
        "commit",
    );
    send_result(tx, result);
}

fn handle_rollback(conn: &OdbcConnection, tx: TransactionSender) {
    let result = execute_transaction_operation(
        conn,
        |c| c.rollback().and_then(|_| c.set_autocommit(true)),
        "rollback",
    );
    send_result(tx, result);
}

fn handle_execute(
    conn: &OdbcConnection,
    sql: Box<str>,
    args: Option<OdbcArguments>,
    tx: ExecuteSender,
) {
    execute_sql(conn, &sql, args, &tx);
}

fn handle_prepare(conn: &OdbcConnection, sql: Box<str>, tx: PrepareSender) {
    let result = match conn.prepare(&sql) {
        Ok(mut prepared) => {
            let columns = collect_columns(&mut prepared);
            let params = prepared.num_params().unwrap_or(0) as usize;
            Ok((0, columns, params))
        }
        Err(e) => Err(Error::from(e)),
    };

    send_result(tx, result);
}

fn handle_get_dbms_name(conn: &OdbcConnection, tx: oneshot::Sender<Result<String, Error>>) {
    let result = conn
        .database_management_system_name()
        .map_err(|e| Error::Protocol(format!("Failed to get DBMS name: {}", e)));
    send_result(tx, result);
}

// Helper functions
fn execute_simple(conn: &OdbcConnection, sql: &str) -> Result<(), Error> {
    match conn.execute(sql, (), None) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Configuration(e.to_string().into())),
    }
}

// SQL execution functions
fn execute_sql(conn: &OdbcConnection, sql: &str, args: Option<OdbcArguments>, tx: &ExecuteSender) {
    let params = prepare_parameters(args);

    if params.is_empty() {
        dispatch_execute(conn, sql, (), tx);
    } else {
        dispatch_execute(conn, sql, &params[..], tx);
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

// Dispatch functions
fn dispatch_execute<P>(conn: &OdbcConnection, sql: &str, params: P, tx: &ExecuteSender)
where
    P: odbc_api::ParameterCollectionRef,
{
    match conn.execute(sql, params, None) {
        Ok(Some(mut cursor)) => handle_cursor(&mut cursor, tx),
        Ok(None) => send_empty_result(tx).unwrap_or_default(),
        Err(e) => send_error(tx, Error::from(e)).unwrap_or_default(),
    }
}

fn handle_cursor<C>(cursor: &mut C, tx: &ExecuteSender)
where
    C: Cursor + ResultSetMetadata,
{
    let columns = collect_columns(cursor);

    match stream_rows(cursor, &columns, tx) {
        Ok(true) => send_empty_result(tx).unwrap_or_default(),
        Ok(false) => {}
        Err(e) => send_error(tx, e).unwrap_or_default(),
    }
}

fn send_empty_result(tx: &ExecuteSender) -> Result<(), SendError<ExecuteResult>> {
    send_stream_result(tx, Ok(Either::Left(OdbcQueryResult { rows_affected: 0 })))
}

fn send_error(tx: &ExecuteSender, error: Error) -> Result<(), SendError<ExecuteResult>> {
    send_stream_result(tx, Err(error))
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
        ordinal: (index - 1) as usize,
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
    while let Some(mut row) = cursor.next_row()? {
        let values = collect_row_values(&mut row, columns)?;
        let row_data = OdbcRow {
            columns: columns.to_vec(),
            values,
        };

        if tx.send(Ok(Either::Right(row_data))).is_err() {
            receiver_open = false;
            break;
        }
    }
    Ok(receiver_open)
}

fn collect_row_values(
    row: &mut CursorRow<'_>,
    columns: &[OdbcColumn],
) -> Result<Vec<(OdbcTypeInfo, Option<Vec<u8>>)>, Error> {
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
) -> Result<(OdbcTypeInfo, Option<Vec<u8>>), Error> {
    let col_idx = (index + 1) as u16;

    // Try text first
    match try_get_text(row, col_idx) {
        Ok(value) => Ok((column.type_info.clone(), value)),
        Err(_) => {
            // Fall back to binary
            match try_get_binary(row, col_idx) {
                Ok(value) => Ok((column.type_info.clone(), value)),
                Err(e) => Err(Error::from(e)),
            }
        }
    }
}

fn try_get_text(row: &mut CursorRow<'_>, col_idx: u16) -> Result<Option<Vec<u8>>, odbc_api::Error> {
    let mut buf = Vec::new();
    match row.get_text(col_idx, &mut buf)? {
        true => Ok(Some(buf)),
        false => Ok(None),
    }
}

fn try_get_binary(
    row: &mut CursorRow<'_>,
    col_idx: u16,
) -> Result<Option<Vec<u8>>, odbc_api::Error> {
    let mut buf = Vec::new();
    match row.get_binary(col_idx, &mut buf)? {
        true => Ok(Some(buf)),
        false => Ok(None),
    }
}
