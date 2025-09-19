//! ODBC database driver (via `odbc-api`).

use crate::executor::Executor;

mod connection;
mod database;
mod row;
mod column;
mod value;
mod type_info;
mod statement;
mod query_result;
mod transaction;
mod options;
mod error;
mod arguments;

pub use connection::OdbcConnection;
pub use database::Odbc;
pub use options::OdbcConnectOptions;
pub use query_result::OdbcQueryResult;
pub use row::OdbcRow;
pub use column::OdbcColumn;
pub use statement::OdbcStatement;
pub use transaction::OdbcTransactionManager;
pub use type_info::OdbcTypeInfo;
pub use value::{OdbcValue, OdbcValueRef};
pub use arguments::{OdbcArguments, OdbcArgumentValue};

/// An alias for [`Pool`][crate::pool::Pool], specialized for ODBC.
pub type OdbcPool = crate::pool::Pool<Odbc>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for ODBC.
pub type OdbcPoolOptions = crate::pool::PoolOptions<Odbc>;

/// An alias for [`Executor<'_, Database = Odbc>`][Executor].
pub trait OdbcExecutor<'c>: Executor<'c, Database = Odbc> {}
impl<'c, T: Executor<'c, Database = Odbc>> OdbcExecutor<'c> for T {}

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(crate::odbc::OdbcArguments<'q>);
impl_executor_for_pool_connection!(Odbc, OdbcConnection, OdbcRow);
impl_executor_for_transaction!(Odbc, OdbcRow);
impl_column_index_for_row!(OdbcRow);
impl_column_index_for_statement!(OdbcStatement);
impl_acquire!(Odbc, OdbcConnection);
impl_into_maybe_pool!(Odbc, OdbcConnection);

// required because some databases have a different handling of NULL
impl_encode_for_option!(Odbc);
