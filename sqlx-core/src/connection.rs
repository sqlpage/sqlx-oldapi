use crate::database::{Database, HasStatementCache};
use crate::error::Error;
use crate::transaction::Transaction;
use futures_core::future::BoxFuture;
use log::LevelFilter;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::Duration;

/// Represents a single database connection.
pub trait Connection: Send {
    type Database: Database;

    type Options: ConnectOptions<Connection = Self>;

    /// Explicitly close this database connection.
    ///
    /// This method is **not required** for safe and consistent operation. However, it is
    /// recommended to call it instead of letting a connection `drop` as the database backend
    /// will be faster at cleaning up resources.
    fn close(self) -> BoxFuture<'static, Result<(), Error>>;

    /// Immediately close the connection without sending a graceful shutdown.
    ///
    /// This should still at least send a TCP `FIN` frame to let the server know we're dying.
    #[doc(hidden)]
    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>>;

    /// Checks if a connection to the database is still valid.
    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>>;

    /// Begin a new transaction or establish a savepoint within the active transaction.
    ///
    /// Returns a [`Transaction`] for controlling and tracking the new transaction.
    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized;

    /// Execute the function inside a transaction.
    ///
    /// If the function returns an error, the transaction will be rolled back. If it does not
    /// return an error, the transaction will be committed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use sqlx_core_oldapi::connection::Connection;
    /// use sqlx_core_oldapi::error::Error;
    /// use sqlx_core_oldapi::executor::Executor;
    /// use sqlx_core_oldapi::postgres::{PgConnection, PgRow};
    /// use sqlx_core_oldapi::query::query;
    ///
    /// # pub async fn _f(conn: &mut PgConnection) -> Result<Vec<PgRow>, Error> {
    /// conn.transaction(|conn|Box::pin(async move {
    ///     query("select * from ..").fetch_all(conn).await
    /// })).await
    /// # }
    /// ```
    fn transaction<'a, F, R, E>(&'a mut self, callback: F) -> BoxFuture<'a, Result<R, E>>
    where
        for<'c> F: FnOnce(&'c mut Transaction<'_, Self::Database>) -> BoxFuture<'c, Result<R, E>>
            + 'a
            + Send
            + Sync,
        Self: Sized,
        R: Send,
        E: From<Error> + Send,
    {
        Box::pin(async move {
            let mut transaction = self.begin().await?;
            let ret = callback(&mut transaction).await;

            match ret {
                Ok(ret) => {
                    transaction.commit().await?;

                    Ok(ret)
                }
                Err(err) => {
                    transaction.rollback().await?;

                    Err(err)
                }
            }
        })
    }

    /// The number of statements currently cached in the connection.
    fn cached_statements_size(&self) -> usize
    where
        Self::Database: HasStatementCache,
    {
        0
    }

    /// Removes all statements from the cache, closing them on the server if
    /// needed.
    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>>
    where
        Self::Database: HasStatementCache,
    {
        Box::pin(async move { Ok(()) })
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>>;

    #[doc(hidden)]
    fn should_flush(&self) -> bool;

    /// Establish a new database connection.
    ///
    /// A value of [`Options`][Self::Options] is parsed from the provided connection string. This parsing
    /// is database-specific.
    #[inline]
    fn connect(url: &str) -> BoxFuture<'static, Result<Self, Error>>
    where
        Self: Sized,
        Self::Options: FromStr<Err = Error>,
    {
        let options = url.parse();

        Box::pin(async move { Self::connect_with(&options?).await })
    }

    /// Establish a new database connection with the provided options.
    fn connect_with(options: &Self::Options) -> BoxFuture<'_, Result<Self, Error>>
    where
        Self: Sized,
    {
        options.connect()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LogSettings {
    pub(crate) statements_level: LevelFilter,
    pub(crate) slow_statements_level: LevelFilter,
    pub(crate) slow_statements_duration: Duration,
}

impl Default for LogSettings {
    fn default() -> Self {
        LogSettings {
            statements_level: LevelFilter::Info,
            slow_statements_level: LevelFilter::Warn,
            slow_statements_duration: Duration::from_secs(1),
        }
    }
}

impl LogSettings {
    pub(crate) fn log_statements(&mut self, level: LevelFilter) {
        self.statements_level = level;
    }
    pub(crate) fn log_slow_statements(&mut self, level: LevelFilter, duration: Duration) {
        self.slow_statements_level = level;
        self.slow_statements_duration = duration;
    }
}

pub trait ConnectOptions: Sized + Send + Sync + 'static {
    type Connection: Connection;

    fn from_url(url: &str) -> Result<Self, Error>
    where
        Self: std::str::FromStr<Err = Error>,
    {
        Self::from_str(url)
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>;

    fn connect_with(options: &Self) -> BoxFuture<'_, Result<Self::Connection, Error>> {
        options.connect()
    }

    fn from_env() -> Result<Self, Error>
    where
        Self: std::str::FromStr<Err = Error>,
    {
        let url = std::env::var("DATABASE_URL").map_err(Error::config)?;
        Self::from_str(&url)
    }

    fn log_statements(&mut self, level: LevelFilter) -> &mut Self;

    fn log_slow_statements(&mut self, level: LevelFilter, duration: Duration) -> &mut Self;
}
