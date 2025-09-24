use crate::connection::{Connection, LogSettings};
use crate::error::Error;
use crate::odbc::blocking::run_blocking;
use crate::odbc::{Odbc, OdbcArguments, OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow};
use crate::transaction::Transaction;
use either::Either;
mod inner;
use futures_core::future::BoxFuture;
use futures_util::future;
use inner::{do_prepare, establish_connection, execute_sql, OdbcConn};
// no direct spawn_blocking here; use run_blocking helper
use std::sync::{Arc, Mutex};

mod executor;

/// A connection to an ODBC-accessible database.
///
/// ODBC uses a blocking C API, so we offload blocking calls to the runtime's blocking
/// thread-pool via `spawn_blocking` and synchronize access with a mutex.
#[derive(Debug)]
pub struct OdbcConnection {
    pub(crate) inner: Arc<Mutex<OdbcConn>>,
    pub(crate) log_settings: LogSettings,
}

impl OdbcConnection {
    #[inline]
    async fn with_conn<T, F>(&self, f: F) -> Result<T, Error>
    where
        T: Send + 'static,
        F: FnOnce(&mut OdbcConn) -> Result<T, Error> + Send + 'static,
    {
        let inner = self.inner.clone();
        run_blocking(move || {
            let mut conn = inner.lock().unwrap();
            f(&mut conn)
        })
        .await
    }

    #[inline]
    async fn with_conn_map<T, E, F>(&self, ctx: &'static str, f: F) -> Result<T, Error>
    where
        T: Send + 'static,
        E: std::fmt::Display,
        F: FnOnce(&mut OdbcConn) -> Result<T, E> + Send + 'static,
    {
        let inner = self.inner.clone();
        run_blocking(move || {
            let mut conn = inner.lock().unwrap();
            f(&mut conn).map_err(|e| Error::Protocol(format!("{}: {}", ctx, e)))
        })
        .await
    }

    pub(crate) async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let conn = run_blocking({
            let options = options.clone();
            move || establish_connection(&options)
        })
        .await?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            log_settings: LogSettings::default(),
        })
    }

    /// Returns the name of the actual Database Management System (DBMS) this
    /// connection is talking to as reported by the ODBC driver.
    pub async fn dbms_name(&mut self) -> Result<String, Error> {
        self.with_conn_map::<_, _, _>("Failed to get DBMS name", |conn| {
            conn.conn.database_management_system_name()
        })
        .await
    }

    pub(crate) async fn ping_blocking(&mut self) -> Result<(), Error> {
        self.with_conn_map::<_, _, _>("Ping failed", |conn| {
            conn.conn.execute("SELECT 1", (), None).map(|_| ())
        })
        .await
    }

    pub(crate) async fn begin_blocking(&mut self) -> Result<(), Error> {
        self.with_conn_map::<_, _, _>("Failed to begin transaction", |conn| {
            conn.conn.set_autocommit(false)
        })
        .await
    }

    pub(crate) async fn commit_blocking(&mut self) -> Result<(), Error> {
        self.with_conn_map::<_, _, _>("Failed to commit transaction", |conn| {
            conn.conn.commit()?;
            conn.conn.set_autocommit(true)
        })
        .await
    }

    pub(crate) async fn rollback_blocking(&mut self) -> Result<(), Error> {
        self.with_conn_map::<_, _, _>("Failed to rollback transaction", |conn| {
            conn.conn.rollback()?;
            conn.conn.set_autocommit(true)
        })
        .await
    }

    pub(crate) async fn execute_stream(
        &mut self,
        sql: &str,
        args: Option<OdbcArguments>,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        let sql = sql.to_string();
        let args_move = args;
        self.with_conn(move |conn| {
            if let Err(e) = execute_sql(conn, &sql, args_move, &tx) {
                let _ = tx.send(Err(e));
            }
            Ok(())
        })
        .await?;
        Ok(rx)
    }

    pub(crate) async fn prepare_metadata(
        &mut self,
        sql: &str,
    ) -> Result<(u64, Vec<OdbcColumn>, usize), Error> {
        let sql = sql.to_string();
        self.with_conn(move |conn| do_prepare(conn, sql.into()))
            .await
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

// moved helpers to connection/inner.rs
