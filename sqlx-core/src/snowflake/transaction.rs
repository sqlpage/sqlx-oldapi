use crate::snowflake::{Snowflake, SnowflakeConnection};
use crate::transaction::TransactionManager;
use futures_core::future::BoxFuture;
use std::borrow::Cow;

/// Implementation of [`TransactionManager`] for Snowflake.
#[derive(Debug)]
pub struct SnowflakeTransactionManager;

impl TransactionManager for SnowflakeTransactionManager {
    type Database = Snowflake;

    fn begin(conn: &mut SnowflakeConnection) -> BoxFuture<'_, Result<(), crate::error::Error>> {
        Box::pin(async move {
            // Snowflake uses standard SQL transaction commands
            conn.execute("BEGIN").await?;
            Ok(())
        })
    }

    fn commit(conn: &mut SnowflakeConnection) -> BoxFuture<'_, Result<(), crate::error::Error>> {
        Box::pin(async move {
            conn.execute("COMMIT").await?;
            Ok(())
        })
    }

    fn rollback(conn: &mut SnowflakeConnection) -> BoxFuture<'_, Result<(), crate::error::Error>> {
        Box::pin(async move {
            conn.execute("ROLLBACK").await?;
            Ok(())
        })
    }

    fn start_rollback(conn: &mut SnowflakeConnection) {
        // For Snowflake, we can immediately start the rollback
        // This is a best-effort operation
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                let _ = conn.execute("ROLLBACK").await;
            });
        }
    }
}