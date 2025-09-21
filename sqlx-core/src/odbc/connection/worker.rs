use std::thread;

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

#[derive(Debug)]
pub(crate) struct ConnectionWorker {
    command_tx: flume::Sender<Command>,
}

enum Command {
    Ping {
        tx: oneshot::Sender<()>,
    },
    Shutdown {
        tx: oneshot::Sender<()>,
    },
    Begin {
        tx: oneshot::Sender<Result<(), Error>>,
    },
    Commit {
        tx: oneshot::Sender<Result<(), Error>>,
    },
    Rollback {
        tx: oneshot::Sender<Result<(), Error>>,
    },
    Execute {
        sql: Box<str>,
        args: Option<OdbcArguments>,
        tx: flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
    },
    Prepare {
        sql: Box<str>,
        tx: oneshot::Sender<Result<(u64, Vec<OdbcColumn>, usize), Error>>,
    },
}


impl ConnectionWorker {
    pub async fn establish(options: OdbcConnectOptions) -> Result<Self, Error> {
        let (establish_tx, establish_rx) = oneshot::channel();

        thread::Builder::new()
            .name("sqlx-odbc-conn".into())
            .spawn(move || {
                worker_thread_main(options, establish_tx);
            })?;

        establish_rx.await.map_err(|_| Error::WorkerCrashed)?
    }

    pub(crate) async fn ping(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.send_command(Command::Ping { tx }).await?;
        rx.await.map_err(|_| Error::WorkerCrashed)
    }

    pub(crate) async fn shutdown(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.send_command(Command::Shutdown { tx }).await?;
        rx.await.map_err(|_| Error::WorkerCrashed)
    }

    pub(crate) async fn begin(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.send_command(Command::Begin { tx }).await?;
        rx.await.map_err(|_| Error::WorkerCrashed)??;
        Ok(())
    }

    pub(crate) async fn commit(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.send_command(Command::Commit { tx }).await?;
        rx.await.map_err(|_| Error::WorkerCrashed)??;
        Ok(())
    }

    pub(crate) async fn rollback(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.send_command(Command::Rollback { tx }).await?;
        rx.await.map_err(|_| Error::WorkerCrashed)??;
        Ok(())
    }

    pub(crate) async fn execute_stream(
        &mut self,
        sql: &str,
        args: Option<OdbcArguments>,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        self.send_command(Command::Execute {
            sql: sql.into(),
            args,
            tx,
        })
        .await?;
        Ok(rx)
    }

    pub(crate) async fn prepare(
        &mut self,
        sql: &str,
    ) -> Result<(u64, Vec<OdbcColumn>, usize), Error> {
        let (tx, rx) = oneshot::channel();
        self.send_command(Command::Prepare {
            sql: sql.into(),
            tx,
        })
        .await?;
        rx.await.map_err(|_| Error::WorkerCrashed)?
    }

    async fn send_command(&mut self, cmd: Command) -> Result<(), Error> {
        self.command_tx
            .send_async(cmd)
            .await
            .map_err(|_| Error::WorkerCrashed)
    }
}

// Worker thread implementation
fn worker_thread_main(
    options: OdbcConnectOptions,
    establish_tx: oneshot::Sender<Result<ConnectionWorker, Error>>,
) {
    let (tx, rx) = flume::bounded(64);

    // Establish connection
    let conn = match establish_connection(&options) {
        Ok(conn) => conn,
        Err(e) => {
            let _ = establish_tx.send(Err(e));
            return;
        }
    };

    // Send back the worker handle
    if establish_tx
        .send(Ok(ConnectionWorker {
            command_tx: tx.clone(),
        }))
        .is_err()
    {
        return;
    }

    // Process commands
    for cmd in rx {
        if !process_command(cmd, &conn) {
            break;
        }
    }
}

fn establish_connection(
    options: &OdbcConnectOptions,
) -> Result<odbc_api::Connection<'static>, Error> {
    // Create environment and connect. We leak the environment to extend its lifetime
    // to 'static, as ODBC connection borrows it. This is acceptable for long-lived
    // process and mirrors SQLite approach to background workers.
    let env = Box::leak(Box::new(
        odbc_api::Environment::new()
            .map_err(|e| Error::Configuration(e.to_string().into()))?,
    ));

    env.connect_with_connection_string(options.connection_string(), Default::default())
        .map_err(|e| Error::Configuration(e.to_string().into()))
}

fn process_command(
    cmd: Command,
    conn: &odbc_api::Connection<'static>,
) -> bool {
    match cmd {
        Command::Ping { tx } => handle_ping(conn, tx),
        Command::Begin { tx } => handle_transaction(conn, "BEGIN", tx),
        Command::Commit { tx } => handle_transaction(conn, "COMMIT", tx),
        Command::Rollback { tx } => handle_transaction(conn, "ROLLBACK", tx),
        Command::Shutdown { tx } => {
            let _ = tx.send(());
            return false; // Signal to exit the loop
        }
        Command::Execute { sql, args, tx } => handle_execute(conn, sql, args, tx),
        Command::Prepare { sql, tx } => handle_prepare(conn, sql, tx),
    }
    true
}

// Command handlers
fn handle_ping(conn: &odbc_api::Connection<'static>, tx: oneshot::Sender<()>) {
    let _ = conn.execute("SELECT 1", (), None);
    let _ = tx.send(());
}

fn handle_transaction(
    conn: &odbc_api::Connection<'static>,
    sql: &str,
    tx: oneshot::Sender<Result<(), Error>>,
) {
    let result = execute_simple(conn, sql);
    let _ = tx.send(result);
}

fn handle_execute(
    conn: &odbc_api::Connection<'static>,
    sql: Box<str>,
    args: Option<OdbcArguments>,
    tx: flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) {
    execute_sql(conn, &sql, args, &tx);
}

fn handle_prepare(
    conn: &odbc_api::Connection<'static>,
    sql: Box<str>,
    tx: oneshot::Sender<Result<(u64, Vec<OdbcColumn>, usize), Error>>,
) {
    let result = match conn.prepare(&sql) {
        Ok(mut prepared) => {
            let columns = collect_columns(&mut prepared);
            let params = prepared.num_params().unwrap_or(0) as usize;
            Ok((0, columns, params))
        }
        Err(e) => Err(Error::from(e)),
    };
    
    let _ = tx.send(result);
}

// Helper functions
fn execute_simple(conn: &odbc_api::Connection<'static>, sql: &str) -> Result<(), Error> {
    match conn.execute(sql, (), None) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Configuration(e.to_string().into())),
    }
}


// SQL execution functions
fn execute_sql(
    conn: &odbc_api::Connection<'static>,
    sql: &str,
    args: Option<OdbcArguments>,
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) {
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
fn dispatch_execute<P>(
    conn: &odbc_api::Connection<'static>,
    sql: &str,
    params: P,
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) where
    P: odbc_api::ParameterCollectionRef,
{
    match conn.execute(sql, params, None) {
        Ok(Some(mut cursor)) => handle_cursor(&mut cursor, tx),
        Ok(None) => send_empty_result(tx),
        Err(e) => send_error(tx, Error::from(e)),
    }
}


fn handle_cursor<C>(
    cursor: &mut C,
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) where
    C: Cursor + ResultSetMetadata,
{
    let columns = collect_columns(cursor);
    
    if let Err(e) = stream_rows(cursor, &columns, tx) {
        send_error(tx, e);
        return;
    }
    
    send_empty_result(tx);
}

fn send_empty_result(
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) {
    let _ = tx.send(Ok(Either::Left(OdbcQueryResult { rows_affected: 0 })));
}

fn send_error(
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
    error: Error,
) {
    let _ = tx.send(Err(error));
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

fn stream_rows<C>(
    cursor: &mut C,
    columns: &[OdbcColumn],
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) -> Result<(), Error>
where
    C: Cursor,
{
    while let Some(mut row) = cursor.next_row()? {
        let values = collect_row_values(&mut row, columns)?;
        let row_data = OdbcRow {
            columns: columns.to_vec(),
            values,
        };
        
        if tx.send(Ok(Either::Right(row_data))).is_err() {
            // Receiver dropped, stop processing
            break;
        }
    }
    Ok(())
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

fn try_get_text(
    row: &mut CursorRow<'_>,
    col_idx: u16,
) -> Result<Option<Vec<u8>>, odbc_api::Error> {
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