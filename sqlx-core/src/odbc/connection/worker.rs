use std::sync::Arc;
use std::thread;

use futures_channel::oneshot;
use futures_intrusive::sync::Mutex;

use crate::error::Error;
use crate::odbc::{
    OdbcArgumentValue, OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow, OdbcTypeInfo,
};
#[allow(unused_imports)]
use crate::row::Row as SqlxRow;
use either::Either;
use odbc_api::{Cursor, CursorRow, IntoParameter, ResultSetMetadata};

#[derive(Debug)]
pub(crate) struct ConnectionWorker {
    command_tx: flume::Sender<Command>,
    pub(crate) shared: Arc<Shared>,
}

#[derive(Debug)]
pub(crate) struct Shared {
    pub(crate) conn: Mutex<odbc_api::Connection<'static>>, // see establish for 'static explanation
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
        tx: flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
    },
    ExecuteWithArgs {
        sql: Box<str>,
        args: Vec<OdbcArgumentValue<'static>>,
        tx: flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
    },
}

impl ConnectionWorker {
    pub async fn establish(options: OdbcConnectOptions) -> Result<Self, Error> {
        let (establish_tx, establish_rx) = oneshot::channel();

        thread::Builder::new()
            .name("sqlx-odbc-conn".into())
            .spawn(move || {
                let (tx, rx) = flume::bounded(64);

                // Create environment and connect. We leak the environment to extend its lifetime
                // to 'static, as ODBC connection borrows it. This is acceptable for long-lived
                // process and mirrors SQLite approach to background workers.
                let env = Box::leak(Box::new(odbc_api::Environment::new().unwrap()));
                let conn = match env
                    .connect_with_connection_string(options.connection_string(), Default::default())
                {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = establish_tx.send(Err(Error::Configuration(e.to_string().into())));
                        return;
                    }
                };

                let shared = Arc::new(Shared {
                    conn: Mutex::new(conn, true),
                });

                if establish_tx
                    .send(Ok(Self {
                        command_tx: tx.clone(),
                        shared: Arc::clone(&shared),
                    }))
                    .is_err()
                {
                    return;
                }

                for cmd in rx {
                    match cmd {
                        Command::Ping { tx } => {
                            with_conn(&shared, |conn| {
                                let _ = conn.execute("SELECT 1", (), None);
                            });
                            let _ = tx.send(());
                        }
                        Command::Begin { tx } => {
                            let res = exec_simple(&shared, "BEGIN");
                            let _ = tx.send(res);
                        }
                        Command::Commit { tx } => {
                            let res = exec_simple(&shared, "COMMIT");
                            let _ = tx.send(res);
                        }
                        Command::Rollback { tx } => {
                            let res = exec_simple(&shared, "ROLLBACK");
                            let _ = tx.send(res);
                        }
                        Command::Shutdown { tx } => {
                            let _ = tx.send(());
                            return;
                        }
                        Command::Execute { sql, tx } => {
                            with_conn(&shared, |conn| execute_sql(conn, &sql, &tx));
                        }
                        Command::ExecuteWithArgs { sql, args, tx } => {
                            with_conn(&shared, |conn| {
                                execute_sql_with_params(conn, &sql, args, &tx)
                            });
                        }
                    }
                }
            })?;

        establish_rx.await.map_err(|_| Error::WorkerCrashed)?
    }

    pub(crate) async fn ping(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send_async(Command::Ping { tx })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        rx.await.map_err(|_| Error::WorkerCrashed)
    }

    pub(crate) async fn shutdown(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send_async(Command::Shutdown { tx })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        rx.await.map_err(|_| Error::WorkerCrashed)
    }

    pub(crate) async fn begin(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send_async(Command::Begin { tx })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        rx.await.map_err(|_| Error::WorkerCrashed)??;
        Ok(())
    }

    pub(crate) async fn commit(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send_async(Command::Commit { tx })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        rx.await.map_err(|_| Error::WorkerCrashed)??;
        Ok(())
    }

    pub(crate) async fn rollback(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send_async(Command::Rollback { tx })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        rx.await.map_err(|_| Error::WorkerCrashed)??;
        Ok(())
    }

    pub(crate) async fn execute_stream(
        &mut self,
        sql: &str,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        self.command_tx
            .send_async(Command::Execute {
                sql: sql.into(),
                tx,
            })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        Ok(rx)
    }

    pub(crate) async fn execute_stream_with_args(
        &mut self,
        sql: &str,
        args: Vec<OdbcArgumentValue<'static>>,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        self.command_tx
            .send_async(Command::ExecuteWithArgs {
                sql: sql.into(),
                args,
                tx,
            })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        Ok(rx)
    }
}

fn with_conn<F>(shared: &Shared, f: F)
where
    F: FnOnce(&odbc_api::Connection<'static>),
{
    if let Some(conn) = shared.conn.try_lock() {
        f(&conn);
    } else {
        let guard = futures_executor::block_on(shared.conn.lock());
        f(&guard);
    }
}

fn exec_simple(shared: &Shared, sql: &str) -> Result<(), Error> {
    let mut result: Result<(), Error> = Ok(());
    with_conn(shared, |conn| match conn.execute(sql, (), None) {
        Ok(_) => result = Ok(()),
        Err(e) => result = Err(Error::Configuration(e.to_string().into())),
    });
    result
}

fn execute_sql(
    conn: &odbc_api::Connection<'static>,
    sql: &str,
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) {
    match conn.execute(sql, (), None) {
        Ok(Some(mut cursor)) => {
            let columns = collect_columns(&mut cursor);
            if let Err(e) = stream_rows(&mut cursor, &columns, tx) {
                let _ = tx.send(Err(e));
                return;
            }
            let _ = tx.send(Ok(Either::Left(OdbcQueryResult { rows_affected: 0 })));
        }
        Ok(None) => {
            let _ = tx.send(Ok(Either::Left(OdbcQueryResult { rows_affected: 0 })));
        }
        Err(e) => {
            let _ = tx.send(Err(Error::from(e)));
        }
    }
}

fn execute_sql_with_params(
    conn: &odbc_api::Connection<'static>,
    sql: &str,
    args: Vec<OdbcArgumentValue<'static>>,
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) {
    if args.is_empty() {
        dispatch_execute(conn, sql, (), tx);
        return;
    }

    let mut params: Vec<Box<dyn odbc_api::parameter::InputParameter>> =
        Vec::with_capacity(args.len());
    for a in dbg!(args) {
        params.push(to_param(a));
    }
    dispatch_execute(conn, sql, &params[..], tx);
}

fn to_param(
    arg: OdbcArgumentValue<'static>,
) -> Box<dyn odbc_api::parameter::InputParameter + 'static> {
    match arg {
        OdbcArgumentValue::Int(i) => Box::new(i.into_parameter()),
        OdbcArgumentValue::Float(f) => Box::new(f.into_parameter()),
        OdbcArgumentValue::Text(s) => Box::new(s.into_parameter()),
        OdbcArgumentValue::Bytes(b) => Box::new(b.into_parameter()),
        OdbcArgumentValue::Null | OdbcArgumentValue::Phantom(_) => {
            Box::new(Option::<String>::None.into_parameter())
        }
    }
}

fn dispatch_execute<P>(
    conn: &odbc_api::Connection<'static>,
    sql: &str,
    params: P,
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) where
    P: odbc_api::ParameterCollectionRef,
{
    match conn.execute(sql, params, None) {
        Ok(Some(mut cursor)) => {
            let columns = collect_columns(&mut cursor);
            if let Err(e) = stream_rows(&mut cursor, &columns, tx) {
                let _ = tx.send(Err(e));
                return;
            }
            let _ = tx.send(Ok(Either::Left(OdbcQueryResult { rows_affected: 0 })));
        }
        Ok(None) => {
            let _ = tx.send(Ok(Either::Left(OdbcQueryResult { rows_affected: 0 })));
        }
        Err(e) => {
            let _ = tx.send(Err(Error::from(e)));
        }
    }
}

fn collect_columns<C>(cursor: &mut C) -> Vec<OdbcColumn>
where
    C: ResultSetMetadata,
{
    let mut columns: Vec<OdbcColumn> = Vec::new();
    if let Ok(count) = cursor.num_result_cols() {
        for i in 1..=count {
            let mut cd = odbc_api::ColumnDescription::default();
            let _ = cursor.describe_col(i as u16, &mut cd);
            let name = String::from_utf8(cd.name).unwrap_or_else(|_| format!("col{}", i - 1));
            columns.push(OdbcColumn {
                name,
                type_info: OdbcTypeInfo::new(cd.data_type),
                ordinal: (i - 1) as usize,
            });
        }
    }
    columns
}

fn stream_rows<C>(
    cursor: &mut C,
    columns: &[OdbcColumn],
    tx: &flume::Sender<Result<Either<OdbcQueryResult, OdbcRow>, Error>>,
) -> Result<(), Error>
where
    C: Cursor,
{
    loop {
        match cursor.next_row() {
            Ok(Some(mut row)) => {
                let values = collect_row_values(&mut row, columns)?;
                let _ = tx.send(Ok(Either::Right(OdbcRow {
                    columns: columns.to_vec(),
                    values,
                })));
            }
            Ok(None) => break,
            Err(e) => return Err(Error::from(e)),
        }
    }
    Ok(())
}

fn collect_row_values(
    row: &mut CursorRow<'_>,
    columns: &[OdbcColumn],
) -> Result<Vec<(OdbcTypeInfo, Option<Vec<u8>>)>, Error> {
    let mut values: Vec<(OdbcTypeInfo, Option<Vec<u8>>)> = Vec::with_capacity(columns.len());
    for (i, column) in columns.iter().enumerate() {
        let col_idx = (i + 1) as u16;
        let mut buf = Vec::new();
        match row.get_text(col_idx, &mut buf) {
            Ok(true) => values.push((column.type_info.clone(), Some(buf))),
            Ok(false) => values.push((column.type_info.clone(), None)),
            Err(_) => {
                let mut bin = Vec::new();
                match row.get_binary(col_idx, &mut bin) {
                    Ok(true) => values.push((column.type_info.clone(), Some(bin))),
                    Ok(false) => values.push((column.type_info.clone(), None)),
                    Err(e) => return Err(Error::from(e)),
                }
            }
        }
    }
    Ok(values)
}
