use crate::connection::Connection;
use crate::error::Error;
use crate::odbc::{
    Odbc, OdbcArguments, OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow, OdbcTypeInfo,
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
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod executor;

type PreparedStatement = Prepared<StatementConnection<SharedConnection<'static>>>;
type SharedPreparedStatement = Arc<Mutex<PreparedStatement>>;

fn collect_columns(prepared: &mut PreparedStatement) -> Vec<OdbcColumn> {
    let count = prepared.num_result_cols().unwrap_or(0);
    (1..=count)
        .map(|i| create_column(prepared, i as u16))
        .collect()
}

fn create_column(stmt: &mut PreparedStatement, index: u16) -> OdbcColumn {
    let mut cd = odbc_api::ColumnDescription::default();
    let _ = stmt.describe_col(index, &mut cd);

    OdbcColumn {
        name: decode_column_name(cd.name, index),
        type_info: OdbcTypeInfo::new(cd.data_type),
        ordinal: usize::from(index.checked_sub(1).unwrap()),
    }
}

fn decode_column_name(name_bytes: Vec<u8>, index: u16) -> String {
    String::from_utf8(name_bytes).unwrap_or_else(|_| format!("col{}", index - 1))
}

/// A connection to an ODBC-accessible database.
///
/// ODBC uses a blocking C API, so we offload blocking calls to the runtime's blocking
/// thread-pool via `spawn_blocking` and synchronize access with a mutex.
pub struct OdbcConnection {
    pub(crate) conn: SharedConnection<'static>,
    pub(crate) stmt_cache: HashMap<Arc<str>, SharedPreparedStatement>,
}

impl std::fmt::Debug for OdbcConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OdbcConnection")
            .field("conn", &self.conn)
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
            stmt_cache: HashMap::new(),
        })
    }

    // (dbms_name moved to the Connection trait implementation)

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

        let maybe_prepared = if let Some(prepared) = self.stmt_cache.get(sql) {
            MaybePrepared::Prepared(Arc::clone(prepared))
        } else {
            MaybePrepared::NotPrepared(sql.to_string())
        };

        let conn = Arc::clone(&self.conn);
        sqlx_rt::spawn(sqlx_rt::spawn_blocking(move || {
            let mut conn = conn.lock().expect("failed to lock connection");
            if let Err(e) = execute_sql(&mut conn, maybe_prepared, args, &tx) {
                let _ = tx.send(Err(e));
            }
        }));

        rx
    }

    pub(crate) async fn clear_cached_statements(&mut self) -> Result<(), Error> {
        // Clear the statement metadata cache
        self.stmt_cache.clear();
        Ok(())
    }

    pub async fn prepare<'a>(&mut self, sql: &'a str) -> Result<OdbcStatement<'a>, Error> {
        let conn = Arc::clone(&self.conn);
        let sql_arc = Arc::from(sql.to_string());
        let sql_clone = Arc::clone(&sql_arc);
        let (prepared, metadata) = spawn_blocking(move || {
            let mut prepared = conn.into_prepared(&sql_clone)?;
            let metadata = OdbcStatementMetadata {
                columns: collect_columns(&mut prepared),
                parameters: usize::from(prepared.num_params().unwrap_or(0)),
            };
            Ok::<_, Error>((prepared, metadata))
        })
        .await?;
        self.stmt_cache
            .insert(Arc::clone(&sql_arc), Arc::new(Mutex::new(prepared)));
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

// moved helpers to connection/inner.rs
