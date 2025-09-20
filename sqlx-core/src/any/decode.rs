use crate::decode::Decode;
use crate::types::Type;

#[cfg(feature = "odbc")]
use crate::odbc::Odbc;

#[cfg(feature = "postgres")]
use crate::postgres::Postgres;

#[cfg(feature = "mysql")]
use crate::mysql::MySql;

#[cfg(feature = "mssql")]
use crate::mssql::Mssql;

#[cfg(feature = "sqlite")]
use crate::sqlite::Sqlite;

// Implements Decode for any T where T supports Decode for any database that has support currently
// compiled into SQLx
macro_rules! impl_any_decode {
    ($ty:ty) => {
        impl<'r> crate::decode::Decode<'r, crate::any::Any> for $ty
        where
            $ty: crate::any::AnyDecode<'r>,
        {
            fn decode(
                value: crate::any::AnyValueRef<'r>,
            ) -> Result<Self, crate::error::BoxDynError> {
                match value.kind {
                    #[cfg(feature = "mysql")]
                    crate::any::value::AnyValueRefKind::MySql(value) => {
                        <$ty as crate::decode::Decode<'r, crate::mysql::MySql>>::decode(value)
                    }

                    #[cfg(feature = "sqlite")]
                    crate::any::value::AnyValueRefKind::Sqlite(value) => {
                        <$ty as crate::decode::Decode<'r, crate::sqlite::Sqlite>>::decode(value)
                    }

                    #[cfg(feature = "mssql")]
                    crate::any::value::AnyValueRefKind::Mssql(value) => {
                        <$ty as crate::decode::Decode<'r, crate::mssql::Mssql>>::decode(value)
                    }

                    #[cfg(feature = "postgres")]
                    crate::any::value::AnyValueRefKind::Postgres(value) => {
                        <$ty as crate::decode::Decode<'r, crate::postgres::Postgres>>::decode(value)
                    }

                    #[cfg(feature = "odbc")]
                    crate::any::value::AnyValueRefKind::Odbc(value) => {
                        <$ty as crate::decode::Decode<'r, crate::odbc::Odbc>>::decode(value)
                    }
                }
            }
        }
    };
}

// Macro to generate AnyDecode trait and implementation for a given set of databases
macro_rules! impl_any_decode_for_db {
    (
        $(#[$meta:meta])*
        $($db:ident),+
    ) => {
        $(#[$meta])*
        pub trait AnyDecode<'r>: $(Decode<'r, $db> + Type<$db> + )+ {}

        $(#[$meta])*
        impl<'r, T> AnyDecode<'r> for T
        where
            T: $(Decode<'r, $db> + Type<$db> + )+
        {}
    };
}

// Generate all combinations of databases
// The order is: Postgres, MySql, Mssql, Sqlite, Odbc

// All 5 databases
impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite",
        feature = "odbc"
    ))]
    Postgres, MySql, Mssql, Sqlite, Odbc
}

// 4 databases (5 combinations)
impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite",
        feature = "odbc",
        not(feature = "postgres")
    ))]
    MySql, Mssql, Sqlite, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        feature = "sqlite",
        feature = "odbc",
        not(feature = "mysql")
    ))]
    Postgres, Mssql, Sqlite, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "sqlite",
        feature = "odbc",
        not(feature = "mssql")
    ))]
    Postgres, MySql, Sqlite, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "odbc",
        not(feature = "sqlite")
    ))]
    Postgres, MySql, Mssql, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite",
        not(feature = "odbc")
    ))]
    Postgres, MySql, Mssql, Sqlite
}

// 3 databases (10 combinations)
impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite",
        not(any(feature = "postgres", feature = "odbc"))
    ))]
    MySql, Mssql, Sqlite
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        feature = "odbc",
        not(any(feature = "postgres", feature = "sqlite"))
    ))]
    MySql, Mssql, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mssql"))
    ))]
    MySql, Sqlite, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mssql",
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql"))
    ))]
    Mssql, Sqlite, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        feature = "sqlite",
        not(any(feature = "mysql", feature = "odbc"))
    ))]
    Postgres, Mssql, Sqlite
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        feature = "odbc",
        not(any(feature = "mysql", feature = "sqlite"))
    ))]
    Postgres, Mssql, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "mysql", feature = "mssql"))
    ))]
    Postgres, Sqlite, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "sqlite",
        not(any(feature = "mssql", feature = "odbc"))
    ))]
    Postgres, MySql, Sqlite
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "odbc",
        not(any(feature = "mssql", feature = "sqlite"))
    ))]
    Postgres, MySql, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        not(any(feature = "sqlite", feature = "odbc"))
    ))]
    Postgres, MySql, Mssql
}

// 2 databases (10 combinations)
impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        not(any(feature = "mssql", feature = "sqlite", feature = "odbc"))
    ))]
    Postgres, MySql
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        not(any(feature = "mysql", feature = "sqlite", feature = "odbc"))
    ))]
    Postgres, Mssql
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "sqlite",
        not(any(feature = "mysql", feature = "mssql", feature = "odbc"))
    ))]
    Postgres, Sqlite
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "odbc",
        not(any(feature = "mysql", feature = "mssql", feature = "sqlite"))
    ))]
    Postgres, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        not(any(feature = "postgres", feature = "sqlite", feature = "odbc"))
    ))]
    MySql, Mssql
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "sqlite",
        not(any(feature = "postgres", feature = "mssql", feature = "odbc"))
    ))]
    MySql, Sqlite
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mssql", feature = "sqlite"))
    ))]
    MySql, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mssql",
        feature = "sqlite",
        not(any(feature = "postgres", feature = "mysql", feature = "odbc"))
    ))]
    Mssql, Sqlite
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mssql",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql", feature = "sqlite"))
    ))]
    Mssql, Odbc
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql", feature = "mssql"))
    ))]
    Sqlite, Odbc
}

// 1 database (5 combinations)
impl_any_decode_for_db! {
    #[cfg(all(
        feature = "postgres",
        not(any(feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))
    ))]
    Postgres
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mysql",
        not(any(feature = "postgres", feature = "mssql", feature = "sqlite", feature = "odbc"))
    ))]
    MySql
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "mssql",
        not(any(feature = "postgres", feature = "mysql", feature = "sqlite", feature = "odbc"))
    ))]
    Mssql
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "sqlite",
        not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "odbc"))
    ))]
    Sqlite
}

impl_any_decode_for_db! {
    #[cfg(all(
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite"))
    ))]
    Odbc
}
