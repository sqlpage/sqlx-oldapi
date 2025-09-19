use crate::error::Error;
use crate::transaction::TransactionManager;
use crate::odbc::Odbc;
use futures_core::future::BoxFuture;
use futures_util::future;

pub struct OdbcTransactionManager;

impl TransactionManager for OdbcTransactionManager {
    type Database = Odbc;

    fn begin(conn: &mut <Self::Database as crate::database::Database>::Connection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { conn.worker.begin().await })
    }

    fn commit(conn: &mut <Self::Database as crate::database::Database>::Connection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { conn.worker.commit().await })
    }

    fn rollback(conn: &mut <Self::Database as crate::database::Database>::Connection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { conn.worker.rollback().await })
    }

    fn start_rollback(_conn: &mut <Self::Database as crate::database::Database>::Connection) {
        // no-op best effort
    }
}
