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

Adding support for a new database to SQLx is a significant undertaking that requires implementing multiple traits and components. This guide provides a step-by-step approach to building a database driver progressively, with testing at each stage.

### Overview of SQLx Architecture

SQLx uses a trait-based architecture where each database implements a set of core traits:

- `Database`: The main trait that defines all associated types for a database
- `Connection`: Handles database connections and basic operations
- `Row`, `Column`, `Value`, `TypeInfo`: Handle data representation
- `Arguments`: Handle query parameter binding
- `Statement`: Handle prepared statements
- `QueryResult`: Handle query execution results
- `TransactionManager`: Handle database transactions

### Prerequisites

Before starting, ensure you have:
1. A working database server/client library to connect to your database
2. Understanding of your database's wire protocol or client API
3. Knowledge of your database's type system and SQL dialect

### Step 1: Initial Setup and Feature Configuration

**1.1 Add the feature to the main `Cargo.toml`:**

```toml
[features]
default = ["runtime-tokio-rustls", "migrate", "postgres", "mysql", "sqlite"]
yourdb = ["sqlx-core/yourdb", "sqlx-macros/yourdb"]

# Add to the all-features list
all-features = ["...", "yourdb"]
```

**1.2 Update `sqlx-core/Cargo.toml`:**

```toml
[features]
yourdb = ["dep:yourdb-native-client"]

[dependencies]
# Add your database's client library
yourdb-native-client = { version = "1.0", optional = true }
```

**1.3 Update `sqlx-macros/Cargo.toml`:**

```toml
[features]
yourdb = ["sqlx-core/yourdb"]
```

**1.4 Create the basic module structure:**

```bash
mkdir -p sqlx-core/src/yourdb
```

**Test at this stage:** Ensure the project compiles with your new feature:
```bash
cargo check --features yourdb
```

### Step 2: Minimal Database Implementation

**2.1 Create `sqlx-core/src/yourdb/mod.rs`:**

```rust
//! **YourDB** database driver.

// Start with just the database struct
mod database;

pub use database::YourDb;

/// An alias for [`Pool`][crate::pool::Pool], specialized for YourDB.
pub type YourDbPool = crate::pool::Pool<YourDb>;
```

**2.2 Create `sqlx-core/src/yourdb/database.rs`:**

```rust
/// YourDB database driver.
#[derive(Debug)]
pub struct YourDb;

// We'll implement the Database trait in later steps
```

**2.3 Add the module to `sqlx-core/src/lib.rs`:**

```rust
#[cfg(feature = "yourdb")]
pub mod yourdb;
```

**2.4 Add to main `src/lib.rs`:**

```rust
#[cfg(feature = "yourdb")]
#[cfg_attr(docsrs, doc(cfg(feature = "yourdb")))]
pub use sqlx_core::yourdb::{self, YourDb, YourDbPool};
```

**Test at this stage:**
```bash
cargo check --features yourdb
```

### Step 3: Type System Foundation

**3.1 Create the core types (add these to `yourdb/mod.rs`):**

```rust
mod type_info;
mod value;
mod error;

pub use type_info::YourDbTypeInfo;
pub use value::{YourDbValue, YourDbValueRef};
pub use error::YourDbDatabaseError;
```

**3.2 Create `sqlx-core/src/yourdb/type_info.rs`:**

```rust
use std::fmt::{self, Display, Formatter};
use crate::type_info::TypeInfo;

/// Type information for YourDB.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct YourDbTypeInfo {
    pub(crate) name: String,
    pub(crate) id: u32, // or whatever your DB uses for type IDs
}

impl Display for YourDbTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TypeInfo for YourDbTypeInfo {
    fn is_null(&self) -> bool {
        false // Implement based on your DB's null handling
    }

    fn name(&self) -> &str {
        &self.name
    }
}
```

**3.3 Create `sqlx-core/src/yourdb/value.rs`:**

```rust
use crate::database::HasValueRef;
use crate::value::{Value, ValueRef};
use crate::yourdb::{YourDb, YourDbTypeInfo};

/// An owned value from YourDB.
#[derive(Debug, Clone)]
pub struct YourDbValue {
    pub(crate) type_info: YourDbTypeInfo,
    pub(crate) data: Vec<u8>, // Adjust based on your needs
}

/// A borrowed value from YourDB.
#[derive(Debug)]
pub struct YourDbValueRef<'r> {
    pub(crate) type_info: &'r YourDbTypeInfo,
    pub(crate) data: &'r [u8], // Adjust based on your needs
}

impl Value for YourDbValue {
    type Database = YourDb;

    fn as_ref(&self) -> <Self::Database as HasValueRef<'_>>::ValueRef {
        YourDbValueRef {
            type_info: &self.type_info,
            data: &self.data,
        }
    }

    fn type_info(&self) -> std::borrow::Cow<'_, YourDbTypeInfo> {
        std::borrow::Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        // Implement based on your DB's null representation
        self.data.is_empty() // Placeholder
    }
}

impl<'r> ValueRef<'r> for YourDbValueRef<'r> {
    type Database = YourDb;

    fn to_owned(&self) -> YourDbValue {
        YourDbValue {
            type_info: self.type_info.clone(),
            data: self.data.to_vec(),
        }
    }

    fn type_info(&self) -> std::borrow::Cow<'_, YourDbTypeInfo> {
        std::borrow::Cow::Borrowed(self.type_info)
    }

    fn is_null(&self) -> bool {
        // Implement based on your DB's null representation
        self.data.is_empty() // Placeholder
    }
}
```

**3.4 Create `sqlx-core/src/yourdb/error.rs`:**

```rust
use crate::error::DatabaseError;
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

/// An error returned from YourDB.
#[derive(Debug)]
pub struct YourDbDatabaseError {
    message: String,
    code: Option<String>,
}

impl Display for YourDbDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for YourDbDatabaseError {}

impl DatabaseError for YourDbDatabaseError {
    fn message(&self) -> &str {
        &self.message
    }

    fn code(&self) -> Option<Cow<'_, str>> {
        self.code.as_ref().map(|c| Cow::Borrowed(c.as_str()))
    }

    // Implement other methods as needed for your database
}
```

**Test at this stage:**
```bash
cargo check --features yourdb
```

### Step 4: Database Trait Implementation

**4.1 Update `sqlx-core/src/yourdb/database.rs`:**

```rust
use crate::database::{Database, HasArguments, HasStatement, HasValueRef};
use crate::yourdb::{YourDbValue, YourDbValueRef, YourDbTypeInfo};

/// YourDB database driver.
#[derive(Debug)]
pub struct YourDb;

// We'll add the other associated types as we implement them
impl<'r> HasValueRef<'r> for YourDb {
    type Database = YourDb;
    type ValueRef = YourDbValueRef<'r>;
}

// Placeholder implementations - we'll complete these in later steps
pub struct YourDbConnection;
pub struct YourDbTransactionManager;
pub struct YourDbRow;
pub struct YourDbQueryResult;
pub struct YourDbColumn;
pub struct YourDbArguments<'q>(std::marker::PhantomData<&'q ()>);
pub struct YourDbStatement<'q>(std::marker::PhantomData<&'q ()>);
pub type YourDbArgumentBuffer = Vec<u8>; // Adjust as needed

impl Database for YourDb {
    type Connection = YourDbConnection;
    type TransactionManager = YourDbTransactionManager;
    type Row = YourDbRow;
    type QueryResult = YourDbQueryResult;
    type Column = YourDbColumn;
    type TypeInfo = YourDbTypeInfo;
    type Value = YourDbValue;
}

impl HasArguments<'_> for YourDb {
    type Database = YourDb;
    type Arguments = YourDbArguments<'_>;
    type ArgumentBuffer = YourDbArgumentBuffer;
}

impl<'q> HasStatement<'q> for YourDb {
    type Database = YourDb;
    type Statement = YourDbStatement<'q>;
}
```

**Test at this stage:**
```bash
cargo check --features yourdb
```

### Step 5: Connection Implementation (Core Networking)

**5.1 Create `sqlx-core/src/yourdb/connection/mod.rs`:**

```rust
use crate::connection::{Connection, ConnectOptions, LogSettings};
use crate::error::Error;
use crate::transaction::Transaction;
use crate::yourdb::{YourDb, YourDbConnectOptions};
use futures_core::future::BoxFuture;
use std::fmt::{self, Debug, Formatter};

pub struct YourDbConnection {
    // Add your connection fields here
    // For example: socket, stream, buffer, etc.
    _placeholder: (),
}

impl Debug for YourDbConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("YourDbConnection").finish()
    }
}

impl Connection for YourDbConnection {
    type Database = YourDb;
    type Options = YourDbConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            // Implement graceful connection closing
            Ok(())
        })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            // Implement immediate connection closing
            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            // Implement connection health check
            // For now, just return Ok
            Ok(())
        })
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            // Implement buffer flushing
            Ok(())
        })
    }

    fn should_flush(&self) -> bool {
        // Return true if buffers need flushing
        false
    }
}

// Placeholder for connection options
pub struct YourDbConnectOptions {
    pub(crate) log_settings: LogSettings,
}

impl Debug for YourDbConnectOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("YourDbConnectOptions").finish()
    }
}

impl Clone for YourDbConnectOptions {
    fn clone(&self) -> Self {
        Self {
            log_settings: self.log_settings.clone(),
        }
    }
}

impl std::str::FromStr for YourDbConnectOptions {
    type Err = Error;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        // Parse connection string - implement based on your DB's URL format
        Ok(Self {
            log_settings: LogSettings::default(),
        })
    }
}

impl ConnectOptions for YourDbConnectOptions {
    type Connection = YourDbConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move {
            // Implement actual connection logic
            // For now, return a placeholder
            Ok(YourDbConnection {
                _placeholder: (),
            })
        })
    }

    fn log_statements(&mut self, level: log::LevelFilter) -> &mut Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(&mut self, level: log::LevelFilter, duration: std::time::Duration) -> &mut Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
```

**5.2 Update `yourdb/mod.rs` to include connection:**

```rust
mod connection;
pub use connection::{YourDbConnection, YourDbConnectOptions};
```

**5.3 Update `yourdb/database.rs` to remove the placeholder:**

```rust
// Remove: pub struct YourDbConnection;
// It's now imported from the connection module
```

**Test at this stage:**
```bash
cargo check --features yourdb
# Try to create a simple connection test
```

### Step 6: Basic Type Support

**6.1 Create `sqlx-core/src/yourdb/types/mod.rs`:**

```rust
//! Conversions between Rust and YourDB types.

mod str;

// Re-export the types for easy access
pub use str::*;
```

**6.2 Create `sqlx-core/src/yourdb/types/str.rs`:**

```rust
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::yourdb::{YourDb, YourDbArgumentBuffer, YourDbTypeInfo, YourDbValueRef};

impl Type<YourDb> for String {
    fn type_info() -> YourDbTypeInfo {
        YourDbTypeInfo {
            name: "TEXT".to_string(), // Adjust for your DB
            id: 1, // Use appropriate type ID
        }
    }

    fn compatible(ty: &YourDbTypeInfo) -> bool {
        ty.name == "TEXT" || ty.name == "VARCHAR" // Adjust for your DB
    }
}

impl Type<YourDb> for str {
    fn type_info() -> YourDbTypeInfo {
        String::type_info()
    }

    fn compatible(ty: &YourDbTypeInfo) -> bool {
        String::compatible(ty)
    }
}

impl<'r> Decode<'r, YourDb> for String {
    fn decode(value: YourDbValueRef<'r>) -> Result<Self, BoxDynError> {
        // Implement decoding from your database format
        // This is a placeholder - adjust for your protocol
        Ok(String::from_utf8(value.data.to_vec())?)
    }
}

impl<'r> Decode<'r, YourDb> for &'r str {
    fn decode(value: YourDbValueRef<'r>) -> Result<Self, BoxDynError> {
        // Implement decoding from your database format
        Ok(std::str::from_utf8(value.data)?)
    }
}

impl<'q> Encode<'q, YourDb> for String {
    fn encode_by_ref(&self, buf: &mut YourDbArgumentBuffer) -> IsNull {
        buf.extend_from_slice(self.as_bytes());
        IsNull::No
    }
}

impl<'q> Encode<'q, YourDb> for &'q str {
    fn encode_by_ref(&self, buf: &mut YourDbArgumentBuffer) -> IsNull {
        buf.extend_from_slice(self.as_bytes());
        IsNull::No
    }
}
```

**Test at this stage:** Create a simple test to ensure type conversions work:

```rust
// Add to yourdb/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;

    #[test]
    fn test_string_type_info() {
        let type_info = String::type_info();
        assert_eq!(type_info.name, "TEXT");
    }
}
```

Run: `cargo test --features yourdb yourdb::tests`

### Step 7: Arguments and Query Execution

**7.1 Create `sqlx-core/src/yourdb/arguments.rs`:**

```rust
use crate::arguments::Arguments;
use crate::encode::{Encode, IsNull};
use crate::types::Type;
use crate::yourdb::YourDb;

pub type YourDbArgumentBuffer = Vec<u8>;

#[derive(Debug, Default)]
pub struct YourDbArguments<'q> {
    buffer: YourDbArgumentBuffer,
    _phantom: std::marker::PhantomData<&'q ()>,
}

impl<'q> Arguments<'q> for YourDbArguments<'q> {
    type Database = YourDb;

    fn reserve(&mut self, additional: usize, size: usize) {
        self.buffer.reserve(size);
    }

    fn add<T>(&mut self, value: T)
    where
        T: 'q + Send + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        let _ = value.encode_by_ref(&mut self.buffer);
    }
}
```

**7.2 Update `yourdb/mod.rs`:**

```rust
mod arguments;
pub use arguments::{YourDbArguments, YourDbArgumentBuffer};
```

**7.3 Create basic executor functionality in `connection/executor.rs`:**

```rust
use crate::executor::Executor;
use crate::yourdb::{YourDbConnection, YourDbRow, YourDb};
use futures_core::stream::BoxStream;
use futures_util::stream;
use either::Either;
use crate::error::Error;

impl<'c> Executor<'c> for &'c mut YourDbConnection {
    type Database = YourDb;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        _query: E,
    ) -> BoxStream<'e, Result<Either<<Self::Database as crate::database::Database>::QueryResult, <Self::Database as crate::database::Database>::Row>, Error>>
    where
        'c: 'e,
        E: crate::executor::Execute<'q, Self::Database> + 'q,
    {
        // Placeholder implementation
        Box::pin(stream::empty())
    }
}
```

**Test at this stage:** Test basic argument handling:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::arguments::Arguments;

    #[test]
    fn test_arguments() {
        let mut args = YourDbArguments::default();
        args.add("test");
        // Verify the argument was added correctly
    }
}
```

### Step 8: CI and Testing Infrastructure

**8.1 Update `.github/workflows/ci.yml`:**

Add your database to the test matrix:

```yaml
strategy:
  matrix:
    include:
      # ... existing entries
      - runtime: tokio
        database: yourdb
        os: ubuntu-20.04
        tls: native-tls

# Add a job for your database
test-yourdb:
  name: YourDB
  runs-on: ubuntu-20.04
  services:
    yourdb:
      image: yourdb/yourdb:latest  # Use appropriate image
      ports:
        - 5432:5432  # Use appropriate port
      env:
        YOURDB_PASSWORD: password
      options: >-
        --health-cmd "yourdb-health-check"
        --health-interval 10s
        --health-timeout 5s
        --health-retries 5

  steps:
    - uses: actions/checkout@v2
    - uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true
    - uses: actions-rs/cargo@v1
      with:
        command: test
        args: --features yourdb
      env:
        DATABASE_URL: yourdb://user:password@localhost/test
```

**8.2 Create `tests/yourdb/mod.rs`:**

```rust
use sqlx::YourDb;
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<YourDb>().await?;
    
    // Test basic connectivity
    sqlx::query("SELECT 1").execute(&mut conn).await?;
    
    Ok(())
}
```

### Step 9: Complete the Implementation

Now you need to implement the remaining components in order:

**9.1 Row, Column, and QueryResult:**
- Implement data retrieval and column metadata
- Test with simple SELECT queries

**9.2 Statement preparation:**
- Implement prepared statements if your DB supports them
- Test with parameterized queries

**9.3 Transaction management:**
- Implement BEGIN, COMMIT, ROLLBACK
- Test transaction behavior

**9.4 Additional type support:**
- Add support for integers, floats, dates, etc.
- Test type conversions thoroughly

**9.5 Integration with `Any` driver:**
- Add your database to the `Any` enum and implementations
- Test runtime database selection

### Step 10: Documentation and Polish

**10.1 Add comprehensive documentation:**
- Document all public APIs
- Add examples for common use cases
- Update the main README

**10.2 Create examples:**
- Basic connection and querying
- Transaction usage
- Type conversions

**10.3 Performance testing:**
- Benchmark against native drivers
- Optimize hot paths

### Testing Strategy

At each step, create tests that verify:

1. **Compilation**: `cargo check --features yourdb`
2. **Unit tests**: Test individual components in isolation
3. **Integration tests**: Test database connectivity and operations
4. **Type safety**: Ensure compile-time type checking works
5. **Runtime behavior**: Test actual database operations

### Common Implementation Patterns

- **Start simple**: Begin with basic string queries before adding prepared statements
- **Incremental testing**: Test each component as you build it
- **Error handling**: Convert database errors to SQLx errors consistently
- **Memory safety**: Be careful with lifetimes, especially in async contexts
- **Protocol efficiency**: Use binary protocols when possible for performance

This progressive approach ensures you can test and validate each component before moving to the next, making the development process more manageable and less error-prone.

## Communication

If you're unsure about your contribution or simply want to ask a question about anything, you can:
- Visit the [SQLx Discord server](https://discord.gg/uuruzJ7)
- Discuss something directly in the [Github issue](https://github.com/launchbadge/sqlx/issues).
