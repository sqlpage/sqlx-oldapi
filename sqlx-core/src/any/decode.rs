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

// Callback macro that generates the actual trait and impl
macro_rules! impl_any_decode_for_databases {
    ($($db:ident),+) => {
        pub trait AnyDecode<'r>: $(Decode<'r, $db> + Type<$db> +)+ 'r {}

        impl<'r, T> AnyDecode<'r> for T
        where
            T: $(Decode<'r, $db> + Type<$db> +)+ 'r
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
    callback: impl_any_decode_for_databases
}
