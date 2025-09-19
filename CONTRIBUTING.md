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

- **[`Database`](sqlx-core/src/database.rs)**: The main trait that defines all associated types for a database. This is the central trait that ties everything together and must be implemented for your database struct.

- **[`Connection`](sqlx-core/src/connection.rs)**: Handles database connections and basic operations like connecting, closing, pinging, and transaction management. See examples: [PostgreSQL](sqlx-core/src/postgres/connection/mod.rs), [MySQL](sqlx-core/src/mysql/connection/mod.rs), [SQLite](sqlx-core/src/sqlite/connection/mod.rs).

- **[`Row`](sqlx-core/src/row.rs)**: Represents a single row from a query result, providing access to column data by index or name. Examples: [PgRow](sqlx-core/src/postgres/row.rs), [MySqlRow](sqlx-core/src/mysql/row.rs).

- **[`Column`](sqlx-core/src/column.rs)**: Provides metadata about columns (name, type, etc.). Examples: [PgColumn](sqlx-core/src/postgres/column.rs), [MySqlColumn](sqlx-core/src/mysql/column.rs).

- **[`Value`](sqlx-core/src/value.rs) and [`ValueRef`](sqlx-core/src/value.rs)**: Handle owned and borrowed values from the database. Examples: [PgValue](sqlx-core/src/postgres/value.rs), [MySqlValue](sqlx-core/src/mysql/value.rs).

- **[`TypeInfo`](sqlx-core/src/type_info.rs)**: Provides information about database types for the type system. Examples: [PgTypeInfo](sqlx-core/src/postgres/type_info.rs), [MySqlTypeInfo](sqlx-core/src/mysql/type_info.rs).

- **[`Arguments`](sqlx-core/src/arguments.rs)**: Handles query parameter binding and encoding. Examples: [PgArguments](sqlx-core/src/postgres/arguments.rs), [MySqlArguments](sqlx-core/src/mysql/arguments.rs).

- **[`Statement`](sqlx-core/src/statement.rs)**: Handles prepared statements and their metadata. Examples: [PgStatement](sqlx-core/src/postgres/statement.rs), [MySqlStatement](sqlx-core/src/mysql/statement.rs).

- **Query execution**: Implement [`Executor`](sqlx-core/src/executor.rs) for your connection type to handle query execution and result streaming.

### Prerequisites

Before starting, ensure you have:
1. A working database server/client library to connect to your database
2. Understanding of your database's wire protocol or client API
3. Knowledge of your database's type system and SQL dialect

### Step 1: Initial Setup and Feature Configuration

**1.1 Add feature flags to Cargo.toml files:**

Add your database feature to the main `Cargo.toml`, `sqlx-core/Cargo.toml`, and `sqlx-macros/Cargo.toml`. Follow the pattern used by existing databases like `postgres` or `mysql`. Include any native client library dependencies as optional dependencies.

**1.2 Create the basic module structure:**
```bash
mkdir -p sqlx-core/src/yourdb
```

**1.3 Add the module to `sqlx-core/src/lib.rs` and main `src/lib.rs`** with appropriate feature gates.

**Test:** `cargo check --features yourdb`

### Step 2: Database Struct and Core Types

**2.1 Create your database struct** that will implement the [`Database`](sqlx-core/src/database.rs) trait. Look at [Postgres](sqlx-core/src/postgres/database.rs), [MySQL](sqlx-core/src/mysql/database.rs), or [SQLite](sqlx-core/src/sqlite/database.rs) for examples.

**2.2 Implement core types:**
- **TypeInfo**: Represents your database's type system. Study existing implementations to understand how to map database types to Rust types.
- **Value and ValueRef**: Handle data storage and retrieval. These work with your database's binary or text protocol.
- **DatabaseError**: Convert your database's native errors to SQLx's error system.

**Test:** `cargo check --features yourdb`

### Step 3: Connection Implementation

**3.1 Implement [`Connection`](sqlx-core/src/connection.rs)** for your database. This handles:
- Connection establishment and URL parsing ([`ConnectOptions`](sqlx-core/src/connection.rs))
- Connection lifecycle (open, close, ping)
- Basic connection management

Study the connection implementations in existing drivers, particularly how they handle:
- Network protocols (see [PostgreSQL stream handling](sqlx-core/src/postgres/connection/stream.rs))
- Authentication (see [MySQL auth](sqlx-core/src/mysql/connection/auth.rs))
- Connection options parsing (see [PostgreSQL options](sqlx-core/src/postgres/options/mod.rs))

**Test:** Basic connection establishment

### Step 4: Type System Integration

**4.1 Implement [`Type`](sqlx-core/src/types/mod.rs), [`Encode`](sqlx-core/src/encode.rs), and [`Decode`](sqlx-core/src/decode.rs)** traits for basic Rust types.

Start with simple types like strings and integers. Look at existing type implementations:
- [PostgreSQL types](sqlx-core/src/postgres/types/)
- [MySQL types](sqlx-core/src/mysql/types/)
- [SQLite types](sqlx-core/src/sqlite/types/)

Each type needs:
- `Type` implementation to provide type metadata
- `Encode` implementation to convert Rust values to database format
- `Decode` implementation to convert database values to Rust types

**Test:** Type conversion unit tests

### Step 5: Query Arguments

**5.1 Implement [`Arguments`](sqlx-core/src/arguments.rs)** for your database. This handles parameter binding in prepared statements.

Study how existing databases handle parameter encoding:
- [PostgreSQL arguments](sqlx-core/src/postgres/arguments.rs) (binary protocol)
- [MySQL arguments](sqlx-core/src/mysql/arguments.rs) (binary protocol)
- [SQLite arguments](sqlx-core/src/sqlite/arguments.rs) (uses native SQLite binding)

**Test:** Parameter binding and encoding

### Step 6: Query Execution

**6.1 Implement [`Executor`](sqlx-core/src/executor.rs)** for your connection type. This is where queries are actually sent to the database and results are processed.

Look at executor implementations:
- [PostgreSQL executor](sqlx-core/src/postgres/connection/executor.rs)
- [MySQL executor](sqlx-core/src/mysql/connection/executor.rs)
- [SQLite executor](sqlx-core/src/sqlite/connection/executor.rs)

**6.2 Implement Row, Column, and QueryResult types** to handle query results and metadata.

**Test:** Basic query execution (`SELECT 1`, simple queries)

### Step 7: Statement Preparation

**7.1 Implement [`Statement`](sqlx-core/src/statement.rs)** if your database supports prepared statements.

Study existing statement implementations to understand:
- Statement preparation and caching
- Parameter metadata
- Column metadata

**Test:** Prepared statement execution with parameters

### Step 8: Transaction Management

**8.1 Implement transaction support** by implementing the transaction-related methods in your `Connection` and creating a `TransactionManager`.

Look at existing transaction implementations:
- [PostgreSQL transactions](sqlx-core/src/postgres/transaction.rs)
- [MySQL transactions](sqlx-core/src/mysql/transaction.rs)

**Test:** BEGIN, COMMIT, ROLLBACK operations

### Step 9: Integration with Any Driver

**9.1 Add your database to the [`Any`](sqlx-core/src/any/) driver** to support runtime database selection.

This involves:
- Adding your database to [`AnyKind`](sqlx-core/src/any/kind.rs)
- Adding connection type to [`AnyConnectionKind`](sqlx-core/src/any/connection/mod.rs)
- Updating delegation macros in Any implementations
- Adding to other Any components (arguments, values, etc.)

Study how existing databases are integrated into the Any driver.

**Test:** Runtime database selection with Any driver

### Step 10: CI and Testing Infrastructure

**10.1 Add CI support** by updating `.github/workflows/ci.yml` with:
- Your database service in GitHub Actions
- Test job for your database
- Appropriate environment variables and health checks

**10.2 Create integration tests** in `tests/yourdb/` following the pattern of existing database tests.

**10.3 Add testing utilities** by implementing [`TestSupport`](sqlx-core/src/testing/mod.rs) for your database.

### Step 11: Advanced Features (Optional)

**11.1 Migration support**: Implement [`MigrateDatabase`](sqlx-core/src/migrate/migrate.rs) if your database supports schema migrations.

**11.2 Listen/Notify**: If your database supports real-time notifications, implement listener functionality (see [PostgreSQL listener](sqlx-core/src/postgres/listener.rs)).

**11.3 Additional type support**: Add support for database-specific types, arrays, JSON, etc.

### Step 12: Documentation and Examples

**12.1 Add comprehensive documentation** to all public APIs with examples.

**12.2 Create examples** in `examples/yourdb/` showing common usage patterns.

**12.3 Update the main README** to include your database in the supported databases list.

### Testing Strategy

At each step, create tests that verify:

1. **Compilation**: `cargo check --features yourdb`
2. **Unit tests**: Test individual components in isolation
3. **Integration tests**: Test database connectivity and operations  
4. **Type safety**: Ensure compile-time type checking works
5. **Runtime behavior**: Test actual database operations

### Implementation Tips

- **Study existing implementations**: The PostgreSQL, MySQL, and SQLite drivers provide excellent examples of different approaches (network protocols vs embedded databases, binary vs text protocols, etc.).

- **Start simple**: Begin with basic string queries before adding prepared statements, transactions, and complex types.

- **Incremental development**: Test each component thoroughly before moving to the next.

- **Protocol efficiency**: Use binary protocols when available for better performance.

- **Error handling**: Provide clear error messages and proper error type conversions.

- **Memory safety**: Pay careful attention to lifetimes, especially in async contexts.

### Common Patterns

- **Module organization**: Follow the established pattern of separate modules for connection, types, arguments, etc.
- **Feature gating**: Ensure all database-specific code is behind feature flags.
- **Async patterns**: Use `BoxFuture` for async trait methods and `BoxStream` for result streaming.
- **Protocol handling**: Implement proper buffering and message framing for network protocols.

This progressive approach ensures you can test and validate each component before moving to the next, making the development process manageable and reducing the likelihood of errors.

## Communication

If you're unsure about your contribution or simply want to ask a question about anything, you can:
- Visit the [SQLx Discord server](https://discord.gg/uuruzJ7)
- Discuss something directly in the [Github issue](https://github.com/launchbadge/sqlx/issues).
