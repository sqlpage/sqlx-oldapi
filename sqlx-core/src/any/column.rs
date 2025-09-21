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

// Callback macro that generates the actual trait and impl
macro_rules! impl_any_column_index_for_databases {
    ($(($row:ident, $stmt:ident)),+) => {
        pub trait AnyColumnIndex: $(ColumnIndex<$row> + for<'q> ColumnIndex<$stmt<'q>> +)+ Sized {}

        impl<I: ?Sized> AnyColumnIndex for I
        where
            I: $(ColumnIndex<$row> + for<'q> ColumnIndex<$stmt<'q>> +)+ Sized
        {}
    };
}

// Generate all combinations
for_all_feature_combinations! {
    entries: [
        ("postgres", (PgRow, PgStatement)),
        ("mysql", (MySqlRow, MySqlStatement)),
        ("mssql", (MssqlRow, MssqlStatement)),
        ("sqlite", (SqliteRow, SqliteStatement)),
        ("odbc", (OdbcRow, OdbcStatement)),
    ],
    callback: impl_any_column_index_for_databases
}