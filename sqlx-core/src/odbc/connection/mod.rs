use crate::connection::{Connection, LogSettings};
use crate::error::Error;
use crate::odbc::{Odbc, OdbcConnectOptions};
use crate::transaction::Transaction;
use futures_core::future::BoxFuture;
use futures_util::future;

mod executor;
mod worker;

pub(crate) use worker::ConnectionWorker;

/// A connection to an ODBC-accessible database.
///
/// ODBC uses a blocking C API, so we run all calls on a dedicated background thread
/// and communicate over channels to provide async access.
#[derive(Debug)]
pub struct OdbcConnection {
    pub(crate) worker: ConnectionWorker,
    pub(crate) log_settings: LogSettings,
}

impl OdbcConnection {
    pub(crate) async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let worker = ConnectionWorker::establish(options.clone()).await?;
        Ok(Self {
            worker,
            log_settings: LogSettings::default(),
        })
    }

    /// Returns the name of the actual Database Management System (DBMS) this
    /// connection is talking to as reported by the ODBC driver.
    ///
    /// This calls the underlying ODBC API `SQL_DBMS_NAME` via
    /// `odbc_api::Connection::database_management_system_name`.
    ///
    /// See: https://docs.rs/odbc-api/19.0.1/odbc_api/struct.Connection.html#method.database_management_system_name
    pub async fn dbms_name(&mut self) -> Result<String, Error> {
        self.worker.get_dbms_name().await
    }
}

impl Connection for OdbcConnection {
    type Database = Odbc;

    type Options = OdbcConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { self.worker.shutdown().await })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.worker.ping())
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
