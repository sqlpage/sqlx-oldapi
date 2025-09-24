use crate::connection::Connection;
use crate::error::Error;
use crate::odbc::blocking::run_blocking;
use crate::odbc::{
    Odbc, OdbcArguments, OdbcColumn, OdbcConnectOptions, OdbcQueryResult, OdbcRow, OdbcTypeInfo,
};
use crate::transaction::Transaction;
use either::Either;
mod odbc_bridge;
use futures_core::future::BoxFuture;
use futures_util::future;
use odbc_bridge::{establish_connection, execute_sql};
// no direct spawn_blocking here; use run_blocking helper
use crate::odbc::{OdbcStatement, OdbcStatementMetadata};
use odbc_api::ResultSetMetadata;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

fn collect_columns(
    prepared: &mut odbc_api::Prepared<odbc_api::handles::StatementImpl<'_>>,
) -> Vec<OdbcColumn> {
    let count = prepared.num_result_cols().unwrap_or(0);
    (1..=count)
        .map(|i| create_column(prepared, i as u16))
        .collect()
}

fn create_column(
    stmt: &mut odbc_api::Prepared<odbc_api::handles::StatementImpl<'_>>,
    index: u16,
) -> OdbcColumn {
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

mod executor;

/// A connection to an ODBC-accessible database.
///
/// ODBC uses a blocking C API, so we offload blocking calls to the runtime's blocking
/// thread-pool via `spawn_blocking` and synchronize access with a mutex.
#[derive(Debug)]
pub struct OdbcConnection {
    pub(crate) conn: odbc_api::SharedConnection<'static>,
    pub(crate) stmt_cache: HashMap<u64, crate::odbc::statement::OdbcStatementMetadata>,
}

impl OdbcConnection {
    pub(crate) async fn with_conn<R, F, S>(&mut self, operation: S, f: F) -> Result<R, Error>
    where
        R: Send + 'static,
        F: FnOnce(&mut odbc_api::Connection<'static>) -> Result<R, Error> + Send + 'static,
        S: std::fmt::Display + Send + 'static,
    {
        let conn = Arc::clone(&self.conn);
        run_blocking(move || {
            let mut conn_guard = conn.lock().map_err(|_| {
                Error::Protocol(format!("ODBC {}: failed to lock connection", operation))
            })?;
            f(&mut conn_guard)
        })
        .await
    }

    pub(crate) async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let shared_conn = run_blocking({
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

    /// Returns the name of the actual Database Management System (DBMS) this
    /// connection is talking to as reported by the ODBC driver.
    pub async fn dbms_name(&mut self) -> Result<String, Error> {
        self.with_conn("dbms_name", move |conn| {
            Ok(conn.database_management_system_name()?)
        })
        .await
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

    pub(crate) async fn execute_stream(
        &mut self,
        sql: &str,
        args: Option<OdbcArguments>,
    ) -> Result<flume::Receiver<Result<Either<OdbcQueryResult, OdbcRow>, Error>>, Error> {
        let (tx, rx) = flume::bounded(64);
        let sql = sql.to_string();
        let args_move = args;

        self.with_conn("execute_stream", move |conn| {
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
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        sql.hash(&mut hasher);
        let key = hasher.finish();

        // Check cache first
        if let Some(metadata) = self.stmt_cache.get(&key) {
            return Ok((key, metadata.columns.clone(), metadata.parameters));
        }

        // Create new prepared statement to get metadata
        let sql = sql.to_string();
        self.with_conn("prepare_metadata", move |conn| {
            let mut prepared = conn.prepare(&sql)?;
            let columns = collect_columns(&mut prepared);
            let params = usize::from(prepared.num_params().unwrap_or(0));
            Ok((columns, params))
        })
        .await
        .map(|(columns, params)| {
            // Cache the metadata
            let metadata = crate::odbc::statement::OdbcStatementMetadata {
                columns: columns.clone(),
                parameters: params,
            };
            self.stmt_cache.insert(key, metadata);
            (key, columns, params)
        })
    }

    pub(crate) async fn clear_cached_statements(&mut self) -> Result<(), Error> {
        // Clear the statement metadata cache
        self.stmt_cache.clear();
        Ok(())
    }

    pub async fn prepare(&mut self, sql: &str) -> Result<OdbcStatement<'static>, Error> {
        let (_, columns, parameters) = self.prepare_metadata(sql).await?;
        let metadata = OdbcStatementMetadata {
            columns,
            parameters,
        };
        Ok(OdbcStatement {
            sql: Cow::Owned(sql.to_string()),
            metadata,
        })
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

    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.clear_cached_statements())
    }
}

// moved helpers to connection/inner.rs
