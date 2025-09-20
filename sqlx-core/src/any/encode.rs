use crate::encode::Encode;
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

// Implements Encode for any T where T supports Encode for any database that has support currently
// compiled into SQLx
macro_rules! impl_any_encode {
    ($ty:ty) => {
        impl<'q> crate::encode::Encode<'q, crate::any::Any> for $ty
        where
            $ty: crate::any::AnyEncode<'q>,
        {
            fn encode_by_ref(
                &self,
                buf: &mut crate::any::AnyArgumentBuffer<'q>,
            ) -> crate::encode::IsNull {
                match &mut buf.0 {
                    #[cfg(feature = "postgres")]
                    crate::any::arguments::AnyArgumentBufferKind::Postgres(args, _) => {
                        args.add(self)
                    }

                    #[cfg(feature = "mysql")]
                    crate::any::arguments::AnyArgumentBufferKind::MySql(args, _) => args.add(self),

                    #[cfg(feature = "mssql")]
                    crate::any::arguments::AnyArgumentBufferKind::Mssql(args, _) => args.add(self),

                    #[cfg(feature = "sqlite")]
                    crate::any::arguments::AnyArgumentBufferKind::Sqlite(args) => args.add(self),

                    #[cfg(feature = "odbc")]
                    crate::any::arguments::AnyArgumentBufferKind::Odbc(args, _) => {
                        let _ = self.encode_by_ref(&mut args.values);
                    }
                }

                // unused
                crate::encode::IsNull::No
            }
        }
    };
}

// Macro to generate AnyEncode trait and implementation for a given set of databases
macro_rules! impl_any_encode_for_db {
    (
        $(#[$meta:meta])*
        $($db:ident),+
    ) => {
        $(#[$meta])*
        pub trait AnyEncode<'q>: $(Encode<'q, $db> + Type<$db> + )+ {}

        $(#[$meta])*
        impl<'q, T> AnyEncode<'q> for T
        where
            T: $(Encode<'q, $db> + Type<$db> + )+
        {}
    };
}

// Generate all combinations of databases
// The order is: Postgres, MySql, Mssql, Sqlite, Odbc

// All 5 databases
impl_any_encode_for_db! {
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
impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite",
        feature = "odbc",
        not(feature = "postgres")
    ))]
    MySql, Mssql, Sqlite, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        feature = "sqlite",
        feature = "odbc",
        not(feature = "mysql")
    ))]
    Postgres, Mssql, Sqlite, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "sqlite",
        feature = "odbc",
        not(feature = "mssql")
    ))]
    Postgres, MySql, Sqlite, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        feature = "odbc",
        not(feature = "sqlite")
    ))]
    Postgres, MySql, Mssql, Odbc
}

impl_any_encode_for_db! {
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
impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        feature = "sqlite",
        not(any(feature = "postgres", feature = "odbc"))
    ))]
    MySql, Mssql, Sqlite
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        feature = "odbc",
        not(any(feature = "postgres", feature = "sqlite"))
    ))]
    MySql, Mssql, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mssql"))
    ))]
    MySql, Sqlite, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mssql",
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql"))
    ))]
    Mssql, Sqlite, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        feature = "sqlite",
        not(any(feature = "mysql", feature = "odbc"))
    ))]
    Postgres, Mssql, Sqlite
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        feature = "odbc",
        not(any(feature = "mysql", feature = "sqlite"))
    ))]
    Postgres, Mssql, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "mysql", feature = "mssql"))
    ))]
    Postgres, Sqlite, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "sqlite",
        not(any(feature = "mssql", feature = "odbc"))
    ))]
    Postgres, MySql, Sqlite
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "odbc",
        not(any(feature = "mssql", feature = "sqlite"))
    ))]
    Postgres, MySql, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        feature = "mssql",
        not(any(feature = "sqlite", feature = "odbc"))
    ))]
    Postgres, MySql, Mssql
}

// 2 databases (10 combinations)
impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mysql",
        not(any(feature = "mssql", feature = "sqlite", feature = "odbc"))
    ))]
    Postgres, MySql
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "mssql",
        not(any(feature = "mysql", feature = "sqlite", feature = "odbc"))
    ))]
    Postgres, Mssql
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "sqlite",
        not(any(feature = "mysql", feature = "mssql", feature = "odbc"))
    ))]
    Postgres, Sqlite
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        feature = "odbc",
        not(any(feature = "mysql", feature = "mssql", feature = "sqlite"))
    ))]
    Postgres, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "mssql",
        not(any(feature = "postgres", feature = "sqlite", feature = "odbc"))
    ))]
    MySql, Mssql
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "sqlite",
        not(any(feature = "postgres", feature = "mssql", feature = "odbc"))
    ))]
    MySql, Sqlite
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mssql", feature = "sqlite"))
    ))]
    MySql, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mssql",
        feature = "sqlite",
        not(any(feature = "postgres", feature = "mysql", feature = "odbc"))
    ))]
    Mssql, Sqlite
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mssql",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql", feature = "sqlite"))
    ))]
    Mssql, Odbc
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "sqlite",
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql", feature = "mssql"))
    ))]
    Sqlite, Odbc
}

// 1 database (5 combinations)
impl_any_encode_for_db! {
    #[cfg(all(
        feature = "postgres",
        not(any(feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))
    ))]
    Postgres
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mysql",
        not(any(feature = "postgres", feature = "mssql", feature = "sqlite", feature = "odbc"))
    ))]
    MySql
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "mssql",
        not(any(feature = "postgres", feature = "mysql", feature = "sqlite", feature = "odbc"))
    ))]
    Mssql
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "sqlite",
        not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "odbc"))
    ))]
    Sqlite
}

impl_any_encode_for_db! {
    #[cfg(all(
        feature = "odbc",
        not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite"))
    ))]
    Odbc
}
