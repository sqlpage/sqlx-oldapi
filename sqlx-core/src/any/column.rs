use crate::any::{Any, AnyTypeInfo};
use crate::column::{Column, ColumnIndex};

#[cfg(feature = "postgres")]
use crate::postgres::{PgColumn, PgRow, PgStatement};

#[cfg(feature = "mysql")]
use crate::mysql::{MySqlColumn, MySqlRow, MySqlStatement};

#[cfg(feature = "sqlite")]
use crate::sqlite::{SqliteColumn, SqliteRow, SqliteStatement};

#[cfg(feature = "mssql")]
use crate::mssql::{MssqlColumn, MssqlRow, MssqlStatement};

#[cfg(feature = "odbc")]
use crate::odbc::{OdbcColumn, OdbcRow, OdbcStatement};

#[derive(Debug, Clone)]
pub struct AnyColumn {
    pub(crate) kind: AnyColumnKind,
    pub(crate) type_info: AnyTypeInfo,
}

impl crate::column::private_column::Sealed for AnyColumn {}

#[derive(Debug, Clone)]
pub(crate) enum AnyColumnKind {
    #[cfg(feature = "postgres")]
    Postgres(PgColumn),

    #[cfg(feature = "mysql")]
    MySql(MySqlColumn),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteColumn),

    #[cfg(feature = "mssql")]
    Mssql(MssqlColumn),

    #[cfg(feature = "odbc")]
    Odbc(OdbcColumn),
}

impl Column for AnyColumn {
    type Database = Any;

    fn ordinal(&self) -> usize {
        match &self.kind {
            #[cfg(feature = "postgres")]
            AnyColumnKind::Postgres(row) => row.ordinal(),

            #[cfg(feature = "mysql")]
            AnyColumnKind::MySql(row) => row.ordinal(),

            #[cfg(feature = "sqlite")]
            AnyColumnKind::Sqlite(row) => row.ordinal(),

            #[cfg(feature = "mssql")]
            AnyColumnKind::Mssql(row) => row.ordinal(),

            #[cfg(feature = "odbc")]
            AnyColumnKind::Odbc(row) => row.ordinal(),
        }
    }

    fn name(&self) -> &str {
        match &self.kind {
            #[cfg(feature = "postgres")]
            AnyColumnKind::Postgres(row) => row.name(),

            #[cfg(feature = "mysql")]
            AnyColumnKind::MySql(row) => row.name(),

            #[cfg(feature = "sqlite")]
            AnyColumnKind::Sqlite(row) => row.name(),

            #[cfg(feature = "mssql")]
            AnyColumnKind::Mssql(row) => row.name(),

            #[cfg(feature = "odbc")]
            AnyColumnKind::Odbc(row) => row.name(),
        }
    }

    fn type_info(&self) -> &AnyTypeInfo {
        &self.type_info
    }
}

// Macro to generate AnyColumnIndex trait and impl based on enabled features
macro_rules! define_any_column_index {
    (
        // List all possible feature combinations with their corresponding bounds
        $(
            #[cfg($($cfg:tt)*)]
            [$($bounds:tt)*]
        ),* $(,)?
    ) => {
        $(
            #[cfg($($cfg)*)]
            pub trait AnyColumnIndex: $($bounds)* {}

            #[cfg($($cfg)*)]
            impl<I: ?Sized> AnyColumnIndex for I where I: $($bounds)* {}
        )*
    };
}

// Define all combinations in a compact format
define_any_column_index! {
    // 5 databases
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 4 databases - missing postgres
    #[cfg(all(not(feature = "postgres"), feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 4 databases - missing mysql
    #[cfg(all(feature = "postgres", not(feature = "mysql"), feature = "mssql", feature = "sqlite", feature = "odbc"))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 4 databases - missing mssql
    #[cfg(all(feature = "postgres", feature = "mysql", not(feature = "mssql"), feature = "sqlite", feature = "odbc"))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 4 databases - missing sqlite
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql", not(feature = "sqlite"), feature = "odbc"))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 4 databases - missing odbc
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite", not(feature = "odbc")))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 3 databases - postgres, mysql, mssql
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql", not(any(feature = "sqlite", feature = "odbc"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>>],

    // 3 databases - postgres, mysql, sqlite
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "sqlite", not(any(feature = "mssql", feature = "odbc"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 3 databases - postgres, mysql, odbc
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "odbc", not(any(feature = "mssql", feature = "sqlite"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 3 databases - postgres, mssql, sqlite
    #[cfg(all(feature = "postgres", feature = "mssql", feature = "sqlite", not(any(feature = "mysql", feature = "odbc"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 3 databases - postgres, mssql, odbc
    #[cfg(all(feature = "postgres", feature = "mssql", feature = "odbc", not(any(feature = "mysql", feature = "sqlite"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 3 databases - postgres, sqlite, odbc
    #[cfg(all(feature = "postgres", feature = "sqlite", feature = "odbc", not(any(feature = "mysql", feature = "mssql"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 3 databases - mysql, mssql, sqlite
    #[cfg(all(feature = "mysql", feature = "mssql", feature = "sqlite", not(any(feature = "postgres", feature = "odbc"))))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 3 databases - mysql, mssql, odbc
    #[cfg(all(feature = "mysql", feature = "mssql", feature = "odbc", not(any(feature = "postgres", feature = "sqlite"))))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 3 databases - mysql, sqlite, odbc
    #[cfg(all(feature = "mysql", feature = "sqlite", feature = "odbc", not(any(feature = "postgres", feature = "mssql"))))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 3 databases - mssql, sqlite, odbc
    #[cfg(all(feature = "mssql", feature = "sqlite", feature = "odbc", not(any(feature = "postgres", feature = "mysql"))))]
    [ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 2 databases - postgres, mysql
    #[cfg(all(feature = "postgres", feature = "mysql", not(any(feature = "mssql", feature = "sqlite", feature = "odbc"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>>],

    // 2 databases - postgres, mssql
    #[cfg(all(feature = "postgres", feature = "mssql", not(any(feature = "mysql", feature = "sqlite", feature = "odbc"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>>],

    // 2 databases - postgres, sqlite
    #[cfg(all(feature = "postgres", feature = "sqlite", not(any(feature = "mysql", feature = "mssql", feature = "odbc"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 2 databases - postgres, odbc
    #[cfg(all(feature = "postgres", feature = "odbc", not(any(feature = "mysql", feature = "mssql", feature = "sqlite"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 2 databases - mysql, mssql
    #[cfg(all(feature = "mysql", feature = "mssql", not(any(feature = "postgres", feature = "sqlite", feature = "odbc"))))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>>],

    // 2 databases - mysql, sqlite
    #[cfg(all(feature = "mysql", feature = "sqlite", not(any(feature = "postgres", feature = "mssql", feature = "odbc"))))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 2 databases - mysql, odbc
    #[cfg(all(feature = "mysql", feature = "odbc", not(any(feature = "postgres", feature = "mssql", feature = "sqlite"))))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 2 databases - mssql, sqlite
    #[cfg(all(feature = "mssql", feature = "sqlite", not(any(feature = "postgres", feature = "mysql", feature = "odbc"))))]
    [ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 2 databases - mssql, odbc
    #[cfg(all(feature = "mssql", feature = "odbc", not(any(feature = "postgres", feature = "mysql", feature = "sqlite"))))]
    [ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 2 databases - sqlite, odbc
    #[cfg(all(feature = "sqlite", feature = "odbc", not(any(feature = "postgres", feature = "mysql", feature = "mssql"))))]
    [ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>> + ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],

    // 1 database - postgres
    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))))]
    [ColumnIndex<PgRow> + for<'q> ColumnIndex<PgStatement<'q>>],

    // 1 database - mysql
    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "mssql", feature = "sqlite", feature = "odbc"))))]
    [ColumnIndex<MySqlRow> + for<'q> ColumnIndex<MySqlStatement<'q>>],

    // 1 database - mssql
    #[cfg(all(feature = "mssql", not(any(feature = "postgres", feature = "mysql", feature = "sqlite", feature = "odbc"))))]
    [ColumnIndex<MssqlRow> + for<'q> ColumnIndex<MssqlStatement<'q>>],

    // 1 database - sqlite
    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "odbc"))))]
    [ColumnIndex<SqliteRow> + for<'q> ColumnIndex<SqliteStatement<'q>>],

    // 1 database - odbc
    #[cfg(all(feature = "odbc", not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite"))))]
    [ColumnIndex<OdbcRow> + for<'q> ColumnIndex<OdbcStatement<'q>>],
}
