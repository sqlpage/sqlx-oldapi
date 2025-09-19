use std::sync::Arc;
use std::thread;

use futures_channel::oneshot;
use futures_intrusive::sync::Mutex;

use crate::error::Error;
use crate::odbc::{OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow, OdbcTypeInfo};
use either::Either;
use odbc_api::Cursor;

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
                            // Using SELECT 1 as generic ping
                            if let Some(guard) = shared.conn.try_lock() {
                                let _ = guard.execute("SELECT 1", (), None);
                            }
                            let _ = tx.send(());
                        }
                        Command::Begin { tx } => {
                            let res = if let Some(guard) = shared.conn.try_lock() {
                                match guard.execute("BEGIN", (), None) {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(Error::Configuration(e.to_string().into())),
                                }
                            } else {
                                Ok(())
                            };
                            let _ = tx.send(res);
                        }
                        Command::Commit { tx } => {
                            let res = if let Some(guard) = shared.conn.try_lock() {
                                match guard.execute("COMMIT", (), None) {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(Error::Configuration(e.to_string().into())),
                                }
                            } else {
                                Ok(())
                            };
                            let _ = tx.send(res);
                        }
                        Command::Rollback { tx } => {
                            let res = if let Some(guard) = shared.conn.try_lock() {
                                match guard.execute("ROLLBACK", (), None) {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(Error::Configuration(e.to_string().into())),
                                }
                            } else {
                                Ok(())
                            };
                            let _ = tx.send(res);
                        }
                        Command::Shutdown { tx } => {
                            let _ = tx.send(());
                            return;
                        }
                        Command::Execute { sql, tx } => {
                            // Helper closure to process using a given connection reference
                            let process = |conn: &odbc_api::Connection<'static>| {
                                match conn.execute(&sql, (), None) {
                                    Ok(Some(mut cursor)) => {
                                        use odbc_api::ResultSetMetadata;
                                        let mut columns: Vec<OdbcColumn> = Vec::new();
                                        if let Ok(count) = cursor.num_result_cols() {
                                            for i in 1..=count {
                                                let mut cd = odbc_api::ColumnDescription::default();
                                                let _ = cursor.describe_col(i as u16, &mut cd);
                                                let name = String::from_utf8(cd.name)
                                                    .unwrap_or_else(|_| format!("col{}", i - 1));
                                                columns.push(OdbcColumn {
                                                    name,
                                                    type_info: OdbcTypeInfo { name: format!("{:?}", cd.data_type), is_null: false },
                                                    ordinal: (i - 1) as usize,
                                                });
                                            }
                                        }

                                        while let Ok(Some(mut row)) = cursor.next_row() {
                                            let mut values: Vec<(OdbcTypeInfo, Option<Vec<u8>>)> = Vec::with_capacity(columns.len());
                                            for i in 1..=columns.len() {
                                                let mut buf = Vec::new();
                                                match row.get_text(i as u16, &mut buf) {
                                                    Ok(true) => values.push((OdbcTypeInfo { name: "TEXT".into(), is_null: false }, Some(buf))),
                                                    Ok(false) => values.push((OdbcTypeInfo { name: "TEXT".into(), is_null: true }, None)),
                                                    Err(e) => {
                                                        let _ = tx.send(Err(Error::from(e)));
                                                        return;
                                                    }
                                                }
                                            }
                                            let _ = tx.send(Ok(Either::Right(OdbcRow { columns: columns.clone(), values })));
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
                            };

                            if let Some(conn) = shared.conn.try_lock() {
                                process(&conn);
                            } else {
                                let guard = futures_executor::block_on(shared.conn.lock());
                                process(&guard);
                            }
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
            .send_async(Command::Execute { sql: sql.into(), tx })
            .await
            .map_err(|_| Error::WorkerCrashed)?;
        Ok(rx)
    }
}
