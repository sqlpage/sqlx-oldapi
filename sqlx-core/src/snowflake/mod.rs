//! **Snowflake** database driver.
//!
//! This driver connects to Snowflake using the SQL API over HTTPS.

use crate::executor::Executor;

mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod options;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

#[cfg(feature = "migrate")]
mod migrate;

#[cfg(feature = "migrate")]
mod testing;

pub use arguments::SnowflakeArguments;
pub use column::SnowflakeColumn;
pub use connection::SnowflakeConnection;
pub use database::Snowflake;
pub use error::SnowflakeDatabaseError;
pub use options::{SnowflakeConnectOptions, SnowflakeSslMode};
pub use query_result::SnowflakeQueryResult;
pub use row::SnowflakeRow;
pub use statement::SnowflakeStatement;
pub use transaction::SnowflakeTransactionManager;
pub use type_info::{SnowflakeTypeInfo, SnowflakeType};
pub use value::{SnowflakeValue, SnowflakeValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for Snowflake.
pub type SnowflakePool = crate::pool::Pool<Snowflake>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for Snowflake.
pub type SnowflakePoolOptions = crate::pool::PoolOptions<Snowflake>;

/// An alias for [`Executor<'_, Database = Snowflake>`][Executor].
pub trait SnowflakeExecutor<'c>: Executor<'c, Database = Snowflake> {}
impl<'c, T: Executor<'c, Database = Snowflake>> SnowflakeExecutor<'c> for T {}

impl_into_arguments_for_arguments!(SnowflakeArguments);
impl_executor_for_pool_connection!(Snowflake, SnowflakeConnection, SnowflakeRow);
impl_executor_for_transaction!(Snowflake, SnowflakeRow);
impl_acquire!(Snowflake, SnowflakeConnection);
impl_column_index_for_row!(SnowflakeRow);
impl_column_index_for_statement!(SnowflakeStatement);
impl_into_maybe_pool!(Snowflake, SnowflakeConnection);
impl_encode_for_option!(Snowflake);