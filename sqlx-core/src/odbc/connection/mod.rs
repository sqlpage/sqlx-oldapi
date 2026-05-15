use crate::common::StatementCache;
use crate::connection::{Connection, LogSettings};
use crate::error::Error;
use crate::odbc::{
    Odbc, OdbcArguments, OdbcBufferSettings, OdbcColumn, OdbcConnectOptions, OdbcQueryResult,
    OdbcRow, OdbcTypeInfo,
};
use crate::transaction::Transaction;
use either::Either;
use sqlx_rt::spawn_blocking;
mod odbc_bridge;
use crate::odbc::{OdbcStatement, OdbcStatementMetadata};
use futures_core::future::BoxFuture;
use futures_util::future;
use odbc_api::ConnectionTransitions;
use odbc_api::{handles::StatementConnection, Prepared, ResultSetMetadata, SharedConnection};
use odbc_bridge::{establish_connection, execute_sql};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};

mod executor;

type PreparedStatement = Prepared<StatementConnection<SharedConnection<'static>>>;
type SharedPreparedStatement = Arc<Mutex<PreparedStatement>>;

fn collect_columns(prepared: &mut PreparedStatement) -> Result<Vec<OdbcColumn>, Error> {
    let count = prepared.num_result_cols()?;
    let mut columns = Vec::with_capacity(count as usize);
    for i in 1..=count {
        columns.push(describe_column(prepared, i as u16)?);
    }
    Ok(columns)
}

fn collect_statement_metadata(
    prepared: &mut PreparedStatement,
) -> Result<OdbcStatementMetadata, Error> {
    Ok(OdbcStatementMetadata {
        columns: collect_columns(prepared)?,
        parameters: usize::from(prepared.num_params()?),
    })
}

pub(super) fn describe_column<S>(stmt: &mut S, index: u16) -> Result<OdbcColumn, Error>
where
    S: ResultSetMetadata,
{
    let mut cd = odbc_api::ColumnDescription::default();
    stmt.describe_col(index, &mut cd)?;

    Ok(OdbcColumn {
        name: decode_column_name(cd.name, index),
        type_info: OdbcTypeInfo::new(cd.data_type),
        ordinal: usize::from(index.checked_sub(1).unwrap()),
    })
}

pub(super) trait ColumnNameDecode {
    fn decode_or_default(self, index: u16) -> String;
}

impl ColumnNameDecode for Vec<u8> {
    fn decode_or_default(self, index: u16) -> String {
        String::from_utf8(self).unwrap_or_else(|_| format!("col{}", index - 1))
    }
}

impl ColumnNameDecode for Vec<u16> {
    fn decode_or_default(self, index: u16) -> String {
        String::from_utf16(&self).unwrap_or_else(|_| format!("col{}", index - 1))
    }
}

pub(super) fn decode_column_name<T: ColumnNameDecode>(name: T, index: u16) -> String {
    name.decode_or_default(index)
}

/// A connection to an ODBC-accessible database.
///
/// ODBC uses a blocking C API, so we offload blocking calls to the runtime's blocking
/// thread-pool via `spawn_blocking` and synchronize access with a mutex.
pub struct OdbcConnection {
    pub(crate) conn: SharedConnection<'static>,
    pub(crate) stmt_cache: StatementCache<SharedPreparedStatement>,
    pub(crate) buffer_settings: OdbcBufferSettings,
    pub(crate) log_settings: LogSettings,
}

impl std::fmt::Debug for OdbcConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OdbcConnection")
            .field("conn", &self.conn)
            .field("buffer_settings", &self.buffer_settings)
            .finish()
    }
}

impl OdbcConnection {
    pub(crate) async fn with_conn<R, F, S>(&mut self, operation: S, f: F) -> Result<R, Error>
    where
        R: Send + 'static,
        F: FnOnce(&mut odbc_api::Connection<'static>) -> Result<R, Error> + Send + 'static,
        S: std::fmt::Display + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        spawn_blocking(move || {
            let mut conn_guard = conn.lock().map_err(|_| {
                Error::Protocol(format!("ODBC {}: failed to lock connection", operation))
            })?;
            f(&mut conn_guard)
        })
        .await
    }

    pub(crate) async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let shared_conn = spawn_blocking({
            let options = options.clone();
            move || {
                let conn = establish_connection(&options)?;
                let shared_conn = odbc_api::SharedConnection::new(std::sync::Mutex::new(conn));
                Ok::<_, Error>(shared_conn)
            }
        })
        .await?;

        Ok(Self {
            conn: shared_conn,
            stmt_cache: StatementCache::new(options.statement_cache_capacity),
            buffer_settings: options.buffer_settings,
            log_settings: options.log_settings.clone(),
        })
    }

    pub(crate) async fn ping_blocking(&mut self) -> Result<(), Error> {
        self.with_conn("ping", move |conn| {
            conn.execute("SELECT 1", (), None)?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn begin_blocking(&mut self) -> Result<(), Error> {
        self.with_conn("begin", move |conn| {
            conn.set_autocommit(false)?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn commit_blocking(&mut self) -> Result<(), Error> {
        self.with_conn("commit", move |conn| {
            conn.commit()?;
            conn.set_autocommit(true)?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn rollback_blocking(&mut self) -> Result<(), Error> {
        self.with_conn("rollback", move |conn| {
            conn.rollback()?;
            conn.set_autocommit(true)?;
            Ok(())
        })
        .await
    }

    /// Launches a background task to execute the SQL statement and send the results to the returned channel.
    pub(crate) fn execute_stream(
        &mut self,
        sql: &str,
        args: Option<OdbcArguments>,
    ) -> flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>> {
        let (tx, rx) = flume::bounded(64);

        let sql_owned = sql.to_string();
        let maybe_prepared = if let Some(prepared) = self.stmt_cache.get_mut(sql) {
            MaybePrepared::Prepared(Arc::clone(prepared))
        } else {
            MaybePrepared::NotPrepared(sql_owned.clone())
        };

        let conn = Arc::clone(&self.conn);
        let buffer_settings = self.buffer_settings;
        let log_settings = self.log_settings.clone();
        sqlx_rt::spawn(sqlx_rt::spawn_blocking(move || {
            let mut logger = crate::logger::QueryLogger::new(&sql_owned, log_settings);
            let result = conn
                .lock()
                .map_err(|_| Error::Protocol("ODBC execute: failed to lock connection".into()))
                .and_then(|mut conn| {
                    execute_sql(
                        &mut conn,
                        maybe_prepared,
                        args,
                        &tx,
                        buffer_settings,
                        &mut logger,
                    )
                });

            if let Err(e) = result {
                let _ = tx.send(Err(e));
            }
        }));

        rx
    }

    pub(crate) async fn clear_cached_statements(&mut self) -> Result<(), Error> {
        while self.stmt_cache.remove_lru().is_some() {}
        Ok(())
    }

    pub async fn prepare<'a>(&mut self, sql: &'a str) -> Result<OdbcStatement<'a>, Error> {
        let cached = self
            .stmt_cache
            .get_mut(sql)
            .map(|prepared| Arc::clone(prepared));

        if let Some(prepared) = cached {
            let metadata = spawn_blocking(move || {
                let mut prepared = prepared.lock().map_err(|_| {
                    Error::Protocol("ODBC prepare: failed to lock prepared statement".into())
                })?;
                collect_statement_metadata(&mut prepared)
            })
            .await?;

            return Ok(OdbcStatement {
                sql: Cow::Borrowed(sql),
                metadata,
            });
        }

        let conn = Arc::clone(&self.conn);
        let sql_owned = sql.to_string();
        let sql_clone = sql_owned.clone();
        let (prepared, metadata) = spawn_blocking(move || {
            let mut prepared = conn.into_prepared(&sql_clone)?;
            let metadata = collect_statement_metadata(&mut prepared)?;
            Ok::<_, Error>((prepared, metadata))
        })
        .await?;

        if self.stmt_cache.is_enabled() {
            self.stmt_cache
                .insert(&sql_owned, Arc::new(Mutex::new(prepared)));
        }

        Ok(OdbcStatement {
            sql: Cow::Borrowed(sql),
            metadata,
        })
    }
}

pub(crate) enum MaybePrepared {
    Prepared(SharedPreparedStatement),
    NotPrepared(String),
}

impl std::fmt::Debug for MaybePrepared {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybePrepared::Prepared(_) => f.debug_tuple("Prepared").finish(),
            MaybePrepared::NotPrepared(sql) => f.debug_tuple("NotPrepared").field(sql).finish(),
        }
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

    fn cached_statements_size(&self) -> usize {
        self.stmt_cache.len()
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(future::ok(()))
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        false
    }

    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.clear_cached_statements())
    }

    fn dbms_name(&mut self) -> BoxFuture<'_, Result<String, Error>> {
        Box::pin(async move {
            self.with_conn("dbms_name", move |conn| {
                Ok(conn.database_management_system_name()?)
            })
            .await
        })
    }
}
