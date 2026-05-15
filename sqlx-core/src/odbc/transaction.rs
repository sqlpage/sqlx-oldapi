use crate::error::Error;
use crate::odbc::{Odbc, OdbcConnection};
use crate::transaction::TransactionManager;
use futures_core::future::BoxFuture;

pub struct OdbcTransactionManager;

impl TransactionManager for OdbcTransactionManager {
    type Database = Odbc;

    fn begin(
        conn: &mut <Self::Database as crate::database::Database>::Connection,
    ) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { conn.begin_blocking().await })
    }

    fn commit(
        conn: &mut <Self::Database as crate::database::Database>::Connection,
    ) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { conn.commit_blocking().await })
    }

    fn rollback(
        conn: &mut <Self::Database as crate::database::Database>::Connection,
    ) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { conn.rollback_blocking().await })
    }

    fn start_rollback(conn: &mut OdbcConnection) {
        if let Ok(conn) = conn.conn.lock() {
            if conn.rollback().is_ok() {
                let _ = conn.set_autocommit(true);
            }
        }
    }
}
