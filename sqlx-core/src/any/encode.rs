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

// Macro to generate all feature combinations
macro_rules! for_all_feature_combinations {
    // Entry point
    ( $callback:ident ) => {
        for_all_feature_combinations!(@parse_databases [
            ("postgres", Postgres),
            ("mysql", MySql),
            ("mssql", Mssql),
            ("sqlite", Sqlite),
            ("odbc", Odbc)
        ] $callback);
    };

    // Convert the database list format to tokens suitable for recursion
    (@parse_databases [ $(($feat:literal, $ty:ident)),* ] $callback:ident) => {
        for_all_feature_combinations!(@recurse [] [] [$( ($feat, $ty) )*] $callback);
    };

    // Recursive case: process each database
    (@recurse [$($yes:tt)*] [$($no:tt)*] [($feat:literal, $ty:ident) $($rest:tt)*] $callback:ident) => {
        // Include this database
        for_all_feature_combinations!(@recurse
            [$($yes)* ($feat, $ty)]
            [$($no)*]
            [$($rest)*]
            $callback
        );

        // Exclude this database
        for_all_feature_combinations!(@recurse
            [$($yes)*]
            [$($no)* $feat]
            [$($rest)*]
            $callback
        );
    };

    // Base case: no more databases, generate the implementation if we have at least one
    (@recurse [$(($feat:literal, $ty:ident))+] [$($no:literal)*] [] $callback:ident) => {
        #[cfg(all($(feature = $feat),+ $(, not(feature = $no))*))]
        $callback! { $($ty),+ }
    };
    
    // Base case: no databases selected, skip
    (@recurse [] [$($no:literal)*] [] $callback:ident) => {
        // Don't generate anything for zero databases
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
for_all_feature_combinations!(impl_any_encode_for_databases);