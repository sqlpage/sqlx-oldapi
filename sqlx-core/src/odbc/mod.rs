//! ODBC database driver (via `odbc-api`).
//!
//! ## Connection Strings
//!
//! When using the `Any` connection type, SQLx accepts standard ODBC connection strings:
//!
//! ```text
//! // DSN-based connection
//! DSN=MyDataSource;UID=myuser;PWD=mypassword
//!
//! // Driver-based connection
//! Driver={ODBC Driver 17 for SQL Server};Server=localhost;Database=test
//!
//! // File DSN
//! FILEDSN=/path/to/myfile.dsn
//! ```
//!
//! The `odbc:` URL scheme prefix is optional but still supported for backward compatibility:
//!
//! ```text
//! odbc:DSN=MyDataSource
//! ```

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

pub use arguments::{OdbcArgumentValue, OdbcArguments};
pub use column::OdbcColumn;
pub use connection::OdbcConnection;
pub use database::Odbc;
pub use options::OdbcConnectOptions;
pub use query_result::OdbcQueryResult;
pub use row::OdbcRow;
pub use statement::OdbcStatement;
pub use transaction::OdbcTransactionManager;
pub use type_info::{DataTypeExt, OdbcTypeInfo};
pub use value::{OdbcValue, OdbcValueRef};

/// An alias for [`Pool`][crate::pool::Pool], specialized for ODBC.
pub type OdbcPool = crate::pool::Pool<Odbc>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for ODBC.
pub type OdbcPoolOptions = crate::pool::PoolOptions<Odbc>;

/// An alias for [`Executor<'_, Database = Odbc>`][Executor].
pub trait OdbcExecutor<'c>: Executor<'c, Database = Odbc> {}
impl<'c, T: Executor<'c, Database = Odbc>> OdbcExecutor<'c> for T {}

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(crate::odbc::OdbcArguments);
impl_executor_for_pool_connection!(Odbc, OdbcConnection, OdbcRow);
impl_executor_for_transaction!(Odbc, OdbcRow);
impl_column_index_for_row!(OdbcRow);
impl_column_index_for_statement!(OdbcStatement);
impl_acquire!(Odbc, OdbcConnection);
impl_into_maybe_pool!(Odbc, OdbcConnection);

// custom Option<..> handling implemented in `arguments.rs`
