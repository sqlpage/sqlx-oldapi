# How to contribute

So, you've decided to contribute, that's great!

You can use this document to figure out how and where to start.

## Getting started

- Make sure you have a [GitHub account](https://github.com/join).
- Take a look at [existing issues](https://github.com/launchbadge/sqlx/issues).
- If you need to create an issue:
  - Make sure to clearly describe it.
  - Including steps to reproduce when it is a bug.
  - Include the version of SQLx used.
  - Include the database driver and version.
  - Include the database version.

## Making changes

- Fork the repository on GitHub.
- Create a branch on your fork.
  - You can usually base it on the `main` branch.
  - Make sure not to commit directly to `main`.
- Make commits of logical and atomic units.
- Make sure you have added the necessary tests for your changes.
- Push your changes to a topic branch in your fork of the repository.
- Submit a pull request to the original repository.

## What to work on

We try to mark issues with a suggested level of experience (in Rust/SQL/SQLx).
Where possible we try to spell out how to go about implementing the feature.

To start with, check out:
- Issues labeled as ["good first issue"](https://github.com/launchbadge/sqlx/issues?q=is%3Aopen+is%3Aissue+label%3A%22good+first+issue%22).
- Issues labeled as ["Easy"](https://github.com/launchbadge/sqlx/issues?q=is%3Aopen+is%3Aissue+label%3AE-easy).

Additionally, it's always good to work on improving/adding examples and documentation.

## Adding Support for New Databases

Adding support for a new database to SQLx is a significant undertaking that requires implementing multiple traits and components. This section provides a comprehensive guide for adding a new database driver.

### Overview of SQLx Architecture

SQLx uses a trait-based architecture where each database implements a set of core traits:

- `Database`: The main trait that defines all associated types for a database
- `Connection`: Handles database connections and basic operations
- `Row`, `Column`, `Value`, `TypeInfo`: Handle data representation
- `Arguments`: Handle query parameter binding
- `Statement`: Handle prepared statements
- `QueryResult`: Handle query execution results
- `TransactionManager`: Handle database transactions

### Step 1: Project Structure Setup

1. **Create the database module directory:**
   ```
   sqlx-core/src/yourdb/
   ```

2. **Create the core module files:**
   ```
   sqlx-core/src/yourdb/
   ├── mod.rs                    # Module exports and main types
   ├── database.rs              # Database trait implementation
   ├── connection/
   │   ├── mod.rs               # Connection implementation
   │   ├── establish.rs         # Connection establishment logic
   │   └── executor.rs          # Query execution logic
   ├── arguments.rs             # Query arguments handling
   ├── column.rs               # Column metadata
   ├── row.rs                  # Row data access
   ├── value.rs                # Value encoding/decoding
   ├── type_info.rs            # Type system integration
   ├── statement.rs            # Prepared statements
   ├── query_result.rs         # Query results
   ├── transaction.rs          # Transaction management
   ├── options/
   │   ├── mod.rs              # Connection options
   │   ├── connect.rs          # Connection string parsing
   │   └── parse.rs            # URL parsing logic
   ├── error.rs                # Database-specific errors
   ├── migrate.rs              # Migration support (optional)
   ├── testing/
   │   └── mod.rs              # Test utilities
   └── types/                  # Type conversions
       ├── mod.rs
       ├── str.rs              # String types
       ├── int.rs              # Integer types
       ├── float.rs            # Float types
       ├── bool.rs             # Boolean types
       └── ...                 # Other type implementations
   ```

### Step 2: Define Core Types

1. **Create the database struct** (`database.rs`):
   ```rust
   use crate::database::{Database, HasArguments, HasStatement, HasStatementCache, HasValueRef};
   
   /// YourDB database driver.
   #[derive(Debug)]
   pub struct YourDb;
   
   impl Database for YourDb {
       type Connection = YourDbConnection;
       type TransactionManager = YourDbTransactionManager;
       type Row = YourDbRow;
       type QueryResult = YourDbQueryResult;
       type Column = YourDbColumn;
       type TypeInfo = YourDbTypeInfo;
       type Value = YourDbValue;
   }
   
   impl<'r> HasValueRef<'r> for YourDb {
       type Database = YourDb;
       type ValueRef = YourDbValueRef<'r>;
   }
   
   impl HasArguments<'_> for YourDb {
       type Database = YourDb;
       type Arguments = YourDbArguments;
       type ArgumentBuffer = YourDbArgumentBuffer; // Often Vec<u8> or custom type
   }
   
   impl<'q> HasStatement<'q> for YourDb {
       type Database = YourDb;
       type Statement = YourDbStatement<'q>;
   }
   
   // Only implement if your database supports prepared statement caching
   impl HasStatementCache for YourDb {}
   ```

2. **Define main module exports** (`mod.rs`):
   ```rust
   //! **YourDB** database driver.
   
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
   
   pub use arguments::{YourDbArgumentBuffer, YourDbArguments};
   pub use column::YourDbColumn;
   pub use connection::YourDbConnection;
   pub use database::YourDb;
   pub use error::YourDbDatabaseError;
   pub use options::YourDbConnectOptions;
   pub use query_result::YourDbQueryResult;
   pub use row::YourDbRow;
   pub use statement::YourDbStatement;
   pub use transaction::YourDbTransactionManager;
   pub use type_info::YourDbTypeInfo;
   pub use value::{YourDbValue, YourDbValueRef};
   
   /// An alias for [`Pool`][crate::pool::Pool], specialized for YourDB.
   pub type YourDbPool = crate::pool::Pool<YourDb>;
   
   /// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for YourDB.
   pub type YourDbPoolOptions = crate::pool::PoolOptions<YourDb>;
   
   /// An alias for [`Executor<'_, Database = YourDb>`][Executor].
   pub trait YourDbExecutor<'c>: Executor<'c, Database = YourDb> {}
   impl<'c, T: Executor<'c, Database = YourDb>> YourDbExecutor<'c> for T {}
   
   // Required macros for integration
   impl_into_arguments_for_arguments!(YourDbArguments<'q>);
   impl_executor_for_pool_connection!(YourDb, YourDbConnection, YourDbRow);
   impl_executor_for_transaction!(YourDb, YourDbRow);
   impl_acquire!(YourDb, YourDbConnection);
   impl_column_index_for_row!(YourDbRow);
   impl_column_index_for_statement!(YourDbStatement);
   impl_into_maybe_pool!(YourDb, YourDbConnection);
   ```

### Step 3: Implement Core Components

1. **Connection Implementation** (`connection/mod.rs`):
   ```rust
   use crate::connection::{Connection, ConnectOptions};
   use crate::database::Database;
   use crate::error::Error;
   use futures_core::future::BoxFuture;
   
   pub struct YourDbConnection {
       // Your connection fields (socket, stream, etc.)
   }
   
   impl Connection for YourDbConnection {
       type Database = YourDb;
       type Options = YourDbConnectOptions;
   
       fn close(self) -> BoxFuture<'static, Result<(), Error>> {
           // Implement graceful connection closing
       }
   
       fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
           // Implement immediate connection closing
       }
   
       fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
           // Implement connection health check
       }
   
       fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
       where
           Self: Sized,
       {
           Transaction::begin(self)
       }
   
       fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
           // Implement buffer flushing
       }
   
       fn should_flush(&self) -> bool {
           // Return true if buffers need flushing
       }
   }
   ```

2. **Arguments Implementation** (`arguments.rs`):
   ```rust
   use crate::arguments::Arguments;
   use crate::encode::{Encode, IsNull};
   use crate::types::Type;
   
   pub struct YourDbArguments {
       // Your argument storage (Vec<u8>, etc.)
   }
   
   impl<'q> Arguments<'q> for YourDbArguments {
       type Database = YourDb;
   
       fn reserve(&mut self, additional: usize, size: usize) {
           // Reserve space for arguments
       }
   
       fn add<T>(&mut self, value: T)
       where
           T: 'q + Send + Encode<'q, Self::Database> + Type<Self::Database>,
       {
           // Add argument to the buffer
       }
   }
   ```

3. **Row Implementation** (`row.rs`):
   ```rust
   use crate::column::ColumnIndex;
   use crate::database::HasValueRef;
   use crate::error::Error;
   use crate::row::Row;
   
   pub struct YourDbRow {
       // Row data storage
   }
   
   impl Row for YourDbRow {
       type Database = YourDb;
   
       fn columns(&self) -> &[YourDbColumn] {
           // Return column metadata
       }
   
       fn try_get_raw<I>(
           &self,
           index: I,
       ) -> Result<<YourDb as HasValueRef<'_>>::ValueRef, Error>
       where
           I: ColumnIndex<Self>,
       {
           // Get raw value by index
       }
   }
   ```

### Step 4: Type System Integration

1. **Implement basic type conversions** in `types/` directory:
   - `str.rs` for string types
   - `int.rs` for integer types  
   - `float.rs` for floating-point types
   - `bool.rs` for boolean types

2. **Each type implementation needs:**
   ```rust
   impl Type<YourDb> for i32 {
       fn type_info() -> YourDbTypeInfo {
           // Return type information
       }
   
       fn compatible(ty: &YourDbTypeInfo) -> bool {
           // Check type compatibility
       }
   }
   
   impl<'r> Decode<'r, YourDb> for i32 {
       fn decode(value: YourDbValueRef<'r>) -> Result<Self, BoxDynError> {
           // Decode value from database format
       }
   }
   
   impl<'q> Encode<'q, YourDb> for i32 {
       fn encode_by_ref(&self, buf: &mut YourDbArgumentBuffer) -> IsNull {
           // Encode value to database format
       }
   }
   ```

### Step 5: Integration with `Any` Database

To make your database work with the `Any` driver:

1. **Add your database to `AnyKind`** (`any/kind.rs`):
   ```rust
   #[derive(Copy, Clone, Debug, PartialEq, Eq)]
   pub enum AnyKind {
       // ... existing databases
       
       #[cfg(feature = "yourdb")]
       YourDb,
   }
   
   impl FromStr for AnyKind {
       fn from_str(url: &str) -> Result<Self, Self::Err> {
           match url {
               // ... existing cases
               
               #[cfg(feature = "yourdb")]
               _ if url.starts_with("yourdb:") => Ok(AnyKind::YourDb),
               
               #[cfg(not(feature = "yourdb"))]
               _ if url.starts_with("yourdb:") => {
                   Err(Error::Configuration("database URL has the scheme of a YourDB database but the `yourdb` feature is not enabled".into()))
               }
           }
       }
   }
   ```

2. **Add to `AnyConnectionKind`** (`any/connection/mod.rs`):
   ```rust
   pub enum AnyConnectionKind {
       // ... existing connections
       
       #[cfg(feature = "yourdb")]
       YourDb(yourdb::YourDbConnection),
   }
   ```

3. **Update delegation macros** in the same file to include your database in `delegate_to!` and `delegate_to_mut!` macros.

4. **Add `From` implementation**:
   ```rust
   #[cfg(feature = "yourdb")]
   impl From<yourdb::YourDbConnection> for AnyConnection {
       fn from(conn: yourdb::YourDbConnection) -> Self {
           AnyConnection(AnyConnectionKind::YourDb(conn))
       }
   }
   ```

5. **Update other `Any` components** (`any/arguments.rs`, `any/value.rs`, etc.) to include your database variants.

### Step 6: Feature Integration

1. **Add feature flag** to `Cargo.toml`:
   ```toml
   [features]
   yourdb = ["dep:yourdb-driver", "sqlx-core/yourdb"]
   ```

2. **Update `sqlx-core/Cargo.toml`**:
   ```toml
   [features]
   yourdb = ["dep:yourdb-driver"]
   
   [dependencies]
   yourdb-driver = { version = "1.0", optional = true }
   ```

3. **Add to main library** (`src/lib.rs`):
   ```rust
   #[cfg(feature = "yourdb")]
   #[cfg_attr(docsrs, doc(cfg(feature = "yourdb")))]
   pub use sqlx_core::yourdb::{self, YourDb};
   ```

### Step 7: Testing and Migration Support

1. **Implement testing utilities** (`testing/mod.rs`):
   ```rust
   use crate::testing::TestSupport;
   
   impl TestSupport for YourDb {
       fn test_context() -> &'static str {
           "YourDB test context"
       }
   
       fn cleanup_test() -> BoxFuture<'static, Result<(), Error>> {
           // Cleanup test data
       }
   }
   ```

2. **Add migration support** (`migrate.rs`) if your database supports schema migrations.

### Step 8: Documentation and Examples

1. **Add comprehensive documentation** to all public APIs
2. **Create examples** in `examples/yourdb/`
3. **Add integration tests** in `tests/yourdb/`
4. **Update main README** to include your database in supported databases list

### What Can Be Reused from Core

- **Pool implementation**: The connection pooling is database-agnostic
- **Query builder**: Basic query building functionality
- **Migration framework**: The migration system can be reused
- **Testing framework**: Test utilities and patterns
- **Error handling**: Base error types and patterns
- **Async runtime integration**: All the async/await infrastructure

### Common Pitfalls to Avoid

1. **Don't forget to implement `private_row::Sealed` for your `Row` type**
2. **Ensure all database-specific features are behind feature flags**
3. **Handle NULL values correctly in type implementations**
4. **Implement proper error conversion from your database's native errors**
5. **Test with both prepared statements and simple queries**
6. **Consider endianness when implementing binary protocols**
7. **Handle connection recovery and retry logic**

### Performance Considerations

- **Use prepared statements when possible** for better performance
- **Implement efficient binary protocols** rather than text-based when available
- **Consider connection pooling characteristics** of your database
- **Optimize type conversions** for common data types
- **Implement proper buffering** for network operations

This guide provides the foundation for implementing a new database driver. Each database has unique characteristics, so you'll need to adapt these patterns to your specific database's protocol, type system, and features.

## Communication

If you're unsure about your contribution or simply want to ask a question about anything, you can:
- Visit the [SQLx Discord server](https://discord.gg/uuruzJ7)
- Discuss something directly in the [Github issue](https://github.com/launchbadge/sqlx/issues).
