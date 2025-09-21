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
                        let _ =
                            <$ty as crate::encode::Encode<'q, crate::odbc::Odbc>>::encode_by_ref(
                                self,
                                &mut args.values,
                            );
                    }
                }

                // unused
                crate::encode::IsNull::No
            }
        }
    };
}

// This macro generates the trait and impl based on which features are enabled
macro_rules! define_any_encode {
    (
        // List all possible feature combinations with their corresponding database lists
        $(
            #[cfg($($cfg:tt)*)]
            [$($db:ident),* $(,)?]
        ),* $(,)?
    ) => {
        $(
            #[cfg($($cfg)*)]
            pub trait AnyEncode<'q>: $(Encode<'q, $db> + Type<$db> +)* Send {}

            #[cfg($($cfg)*)]
            impl<'q, T> AnyEncode<'q> for T
            where
                T: $(Encode<'q, $db> + Type<$db> +)* Send
            {}
        )*
    };
}

// Define all combinations in a more compact, maintainable format
define_any_encode! {
    // 5 databases
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))]
    [Postgres, MySql, Mssql, Sqlite, Odbc],

    // 4 databases (5 combinations) - missing one each
    #[cfg(all(not(feature = "postgres"), feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))]
    [MySql, Mssql, Sqlite, Odbc],
    #[cfg(all(feature = "postgres", not(feature = "mysql"), feature = "mssql", feature = "sqlite", feature = "odbc"))]
    [Postgres, Mssql, Sqlite, Odbc],
    #[cfg(all(feature = "postgres", feature = "mysql", not(feature = "mssql"), feature = "sqlite", feature = "odbc"))]
    [Postgres, MySql, Sqlite, Odbc],
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql", not(feature = "sqlite"), feature = "odbc"))]
    [Postgres, MySql, Mssql, Odbc],
    #[cfg(all(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite", not(feature = "odbc")))]
    [Postgres, MySql, Mssql, Sqlite],

    // 3 databases (10 combinations)
    #[cfg(all(not(any(feature = "postgres", feature = "mysql")), feature = "mssql", feature = "sqlite", feature = "odbc"))]
    [Mssql, Sqlite, Odbc],
    #[cfg(all(not(any(feature = "postgres", feature = "mssql")), feature = "mysql", feature = "sqlite", feature = "odbc"))]
    [MySql, Sqlite, Odbc],
    #[cfg(all(not(any(feature = "postgres", feature = "sqlite")), feature = "mysql", feature = "mssql", feature = "odbc"))]
    [MySql, Mssql, Odbc],
    #[cfg(all(not(any(feature = "postgres", feature = "odbc")), feature = "mysql", feature = "mssql", feature = "sqlite"))]
    [MySql, Mssql, Sqlite],
    #[cfg(all(not(any(feature = "mysql", feature = "mssql")), feature = "postgres", feature = "sqlite", feature = "odbc"))]
    [Postgres, Sqlite, Odbc],
    #[cfg(all(not(any(feature = "mysql", feature = "sqlite")), feature = "postgres", feature = "mssql", feature = "odbc"))]
    [Postgres, Mssql, Odbc],
    #[cfg(all(not(any(feature = "mysql", feature = "odbc")), feature = "postgres", feature = "mssql", feature = "sqlite"))]
    [Postgres, Mssql, Sqlite],
    #[cfg(all(not(any(feature = "mssql", feature = "sqlite")), feature = "postgres", feature = "mysql", feature = "odbc"))]
    [Postgres, MySql, Odbc],
    #[cfg(all(not(any(feature = "mssql", feature = "odbc")), feature = "postgres", feature = "mysql", feature = "sqlite"))]
    [Postgres, MySql, Sqlite],
    #[cfg(all(not(any(feature = "sqlite", feature = "odbc")), feature = "postgres", feature = "mysql", feature = "mssql"))]
    [Postgres, MySql, Mssql],

    // 2 databases (10 combinations)
    #[cfg(all(feature = "postgres", feature = "mysql", not(any(feature = "mssql", feature = "sqlite", feature = "odbc"))))]
    [Postgres, MySql],
    #[cfg(all(feature = "postgres", feature = "mssql", not(any(feature = "mysql", feature = "sqlite", feature = "odbc"))))]
    [Postgres, Mssql],
    #[cfg(all(feature = "postgres", feature = "sqlite", not(any(feature = "mysql", feature = "mssql", feature = "odbc"))))]
    [Postgres, Sqlite],
    #[cfg(all(feature = "postgres", feature = "odbc", not(any(feature = "mysql", feature = "mssql", feature = "sqlite"))))]
    [Postgres, Odbc],
    #[cfg(all(feature = "mysql", feature = "mssql", not(any(feature = "postgres", feature = "sqlite", feature = "odbc"))))]
    [MySql, Mssql],
    #[cfg(all(feature = "mysql", feature = "sqlite", not(any(feature = "postgres", feature = "mssql", feature = "odbc"))))]
    [MySql, Sqlite],
    #[cfg(all(feature = "mysql", feature = "odbc", not(any(feature = "postgres", feature = "mssql", feature = "sqlite"))))]
    [MySql, Odbc],
    #[cfg(all(feature = "mssql", feature = "sqlite", not(any(feature = "postgres", feature = "mysql", feature = "odbc"))))]
    [Mssql, Sqlite],
    #[cfg(all(feature = "mssql", feature = "odbc", not(any(feature = "postgres", feature = "mysql", feature = "sqlite"))))]
    [Mssql, Odbc],
    #[cfg(all(feature = "sqlite", feature = "odbc", not(any(feature = "postgres", feature = "mysql", feature = "mssql"))))]
    [Sqlite, Odbc],

    // 1 database (5 combinations)
    #[cfg(all(feature = "postgres", not(any(feature = "mysql", feature = "mssql", feature = "sqlite", feature = "odbc"))))]
    [Postgres],
    #[cfg(all(feature = "mysql", not(any(feature = "postgres", feature = "mssql", feature = "sqlite", feature = "odbc"))))]
    [MySql],
    #[cfg(all(feature = "mssql", not(any(feature = "postgres", feature = "mysql", feature = "sqlite", feature = "odbc"))))]
    [Mssql],
    #[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "odbc"))))]
    [Sqlite],
    #[cfg(all(feature = "odbc", not(any(feature = "postgres", feature = "mysql", feature = "mssql", feature = "sqlite"))))]
    [Odbc],
}
