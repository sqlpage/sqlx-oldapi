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
