# Running tests

Install Rust through rustup and [Docker Compose](https://docs.docker.com/compose/install/).
Run the local test script from any directory:

```sh
./test.sh          # all backends, stopping at the first failure
./test.sh sqlite   # SQLite only; no Docker needed
./test.sh postgres # PostgreSQL 14
./test.sh mysql    # MySQL 8
./test.sh mssql    # SQL Server 2022
./test.sh odbc     # ODBC against local PostgreSQL 16
```

Each database run starts a disposable container, waits for its test schema, and
publishes a randomly assigned port on loopback. The script uses that container's
connection string and removes it when the test command exits, including failures.
SQLite runs against a temporary copy of its fixture database.
The script does not use an existing `DATABASE_URL`, a saved ODBC DSN, or modify `~/.odbc.ini`.
The ODBC run requires unixODBC and the **PostgreSQL Unicode** driver on the host.

The PostgreSQL client-certificate authentication configuration is tested separately
by the **Postgres with SSL client cert** CI job.

For custom databases, set `DATABASE_URL` explicitly when invoking Cargo. These tests
create and modify database objects: use a disposable test database.

```sh
DATABASE_URL=mysql://root:password@127.0.0.1:3306/sqlx \
  cargo test --locked --no-default-features \
  --features macros,offline,any,all-types,mysql,native-tls
```
