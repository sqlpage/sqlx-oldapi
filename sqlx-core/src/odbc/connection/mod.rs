use crate::connection::{Connection, LogSettings};
use crate::error::Error;
use crate::odbc::{Odbc, OdbcArguments, OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow};
use crate::transaction::Transaction;
use either::Either;
use crate::odbc::blocking::run_blocking;
mod inner;
use inner::{do_prepare, establish_connection, execute_sql, OdbcConn};
use futures_core::future::BoxFuture;
use futures_util::future;
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
    pub(crate) async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let conn = run_blocking({
            let options = options.clone();
            move || establish_connection(&options)
        }).await?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
            log_settings: LogSettings::default(),
        })
    }

    /// Returns the name of the actual Database Management System (DBMS) this
    /// connection is talking to as reported by the ODBC driver.
    pub async fn dbms_name(&mut self) -> Result<String, Error> {
        let inner = self.inner.clone();
        run_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.database_management_system_name()
                .map_err(|e| Error::Protocol(format!("Failed to get DBMS name: {}", e)))
        }).await
    }

    pub(crate) async fn ping_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        run_blocking(move || {
            let conn = inner.lock().unwrap();
            let res = conn.execute("SELECT 1", (), None);
            match res {
                Ok(_) => Ok(()),
                Err(e) => Err(Error::Protocol(format!("Ping failed: {}", e))),
            }
        }).await
    }

    pub(crate) async fn begin_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        run_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.set_autocommit(false)
                .map_err(|e| Error::Protocol(format!("Failed to begin transaction: {}", e)))
        }).await
    }

    pub(crate) async fn commit_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        run_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.commit()
                .and_then(|_| conn.set_autocommit(true))
                .map_err(|e| Error::Protocol(format!("Failed to commit transaction: {}", e)))
        }).await
    }

    pub(crate) async fn rollback_blocking(&mut self) -> Result<(), Error> {
        let inner = self.inner.clone();
        run_blocking(move || {
            let conn = inner.lock().unwrap();
            conn.rollback()
                .and_then(|_| conn.set_autocommit(true))
                .map_err(|e| Error::Protocol(format!("Failed to rollback transaction: {}", e)))
        }).await
    }

    pub(crate) async fn execute_stream(
        &mut self,
        sql: &str,
        args: Option<OdbcArguments>,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        let inner = self.inner.clone();
        let sql = sql.to_string();
        run_blocking(move || {
            let mut guard = inner.lock().unwrap();
            if let Err(e) = execute_sql(&mut guard, &sql, args, &tx) {
                let _ = tx.send(Err(e));
            }
            Ok(())
        }).await?;
        Ok(rx)
    }

    pub(crate) async fn prepare_metadata(
        &mut self,
        sql: &str,
    ) -> Result<(u64, Vec<OdbcColumn>, usize), Error> {
        let inner = self.inner.clone();
        let sql = sql.to_string();
        run_blocking(move || do_prepare(&mut inner.lock().unwrap(), sql.into())).await
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
