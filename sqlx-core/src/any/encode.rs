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

// Callback macro that generates the actual trait and impl
macro_rules! impl_any_encode_for_databases {
    ($($db:ident),+) => {
        pub trait AnyEncode<'q>: $(Encode<'q, $db> + Type<$db> +)+ Send {}

        impl<'q, T> AnyEncode<'q> for T
        where
            T: $(Encode<'q, $db> + Type<$db> +)+ Send
        {}
    };
}

// Generate all combinations
for_all_feature_combinations! {
    entries: [
        ("postgres", Postgres),
        ("mysql", MySql),
        ("mssql", Mssql),
        ("sqlite", Sqlite),
        ("odbc", Odbc),
    ],
    callback: impl_any_encode_for_databases
}