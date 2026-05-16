use sqlx::pool::PoolOptions;
use sqlx::{Connection, Database, Pool};
use std::env;

pub fn setup_if_needed() {
    let _ = dotenvy::dotenv();
    let _ = env_logger::builder().is_test(true).try_init();
}

// Make a new connection
// Ensure [dotenvy] and [env_logger] have been setup
pub async fn new<DB>() -> anyhow::Result<DB::Connection>
where
    DB: Database,
{
    setup_if_needed();

    Ok(DB::Connection::connect(&env::var("DATABASE_URL")?).await?)
}

// Make a new pool
// Ensure [dotenvy] and [env_logger] have been setup
pub async fn pool<DB>() -> anyhow::Result<Pool<DB>>
where
    DB: Database,
{
    setup_if_needed();

    let pool = PoolOptions::<DB>::new()
        .min_connections(0)
        .max_connections(5)
        .test_before_acquire(true)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    Ok(pool)
}

// Test type encoding and decoding
#[macro_export]
macro_rules! test_type {
    ($name:ident<$ty:ty>($db:ident, $sql:literal, $($text:literal == $value:expr),+ $(,)?)) => {
        #[sqlx_macros::test]
        #[allow(non_snake_case)]
        async fn $name() -> anyhow::Result<()> {
            $crate::__test_prepared_type_body!($ty, $db, $sql, $($text == $value),+);
            $crate::__test_unprepared_type_body!($ty, $db, $($text == $value),+);

            Ok(())
        }
    };

    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        #[sqlx_macros::test]
        #[allow(non_snake_case)]
        async fn $name() -> anyhow::Result<()> {
            $crate::__test_prepared_type_body!($ty, $db, $crate::query_for_test_prepared_type!($db), $($text == $value),+);
            $crate::__test_unprepared_type_body!($ty, $db, $($text == $value),+);

            Ok(())
        }
    };

    ($name:ident($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::test_type!($name<$name>($db, $($text == $value),+));
    };
}

// Test type decoding only
#[macro_export]
macro_rules! test_decode_type {
    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        #[sqlx_macros::test]
        #[allow(non_snake_case)]
        async fn $name() -> anyhow::Result<()> {
            $crate::__test_prepared_decode_type_body!($ty, $db, $($text == $value),+);
            $crate::__test_unprepared_type_body!($ty, $db, $($text == $value),+);

            Ok(())
        }
    };

    ($name:ident($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::test_decode_type!($name<$name>($db, $($text == $value),+));
    };
}

// Test type encoding and decoding
#[macro_export]
macro_rules! test_prepared_type {
    ($name:ident<$ty:ty>($db:ident, $sql:literal, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::__test_prepared_type!($name<$ty>($db, $sql, $($text == $value),+));
    };

    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::__test_prepared_type!($name<$ty>($db, $crate::query_for_test_prepared_type!($db), $($text == $value),+));
    };

    ($name:ident($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        $crate::__test_prepared_type!($name<$name>($db, $($text == $value),+));
    };
}

// Test type decoding for the simple (unprepared) query API
#[macro_export]
macro_rules! test_unprepared_type {
    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        #[sqlx_macros::test]
        #[allow(non_snake_case)]
        async fn $name() -> anyhow::Result<()> {
            $crate::__test_unprepared_type_body!($ty, $db, $($text == $value),+);

            Ok(())
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __test_unprepared_type_body {
    ($ty:ty, $db:ident, $($text:literal == $value:expr),+ $(,)?) => {{
        use futures::TryStreamExt;
        use sqlx_oldapi::prelude::*;

        let mut conn = sqlx_test::new::<$db>().await?;

        $(
            let query = format!("SELECT {}", $text);
            let mut s = conn.fetch(&*query);
            let row = s.try_next().await?.unwrap();
            let rec = row.try_get::<$ty, _>(0)?;

            assert_eq!($value, rec, "Value mismatch on: Expected {:#?} but received {rec:#?} ({}) from db", $value, std::any::type_name::<$ty>());

            drop(s);
        )+
    }}
}

// Test type decoding only for the prepared query API
#[macro_export]
macro_rules! __test_prepared_decode_type {
    ($name:ident<$ty:ty>($db:ident, $($text:literal == $value:expr),+ $(,)?)) => {
        #[sqlx_macros::test]
        #[allow(non_snake_case)]
        async fn $name() -> anyhow::Result<()> {
            $crate::__test_prepared_decode_type_body!($ty, $db, $($text == $value),+);

            Ok(())
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __test_prepared_decode_type_body {
    ($ty:ty, $db:ident, $($text:literal == $value:expr),+ $(,)?) => {{
        use sqlx_oldapi::Row;

        let mut conn = sqlx_test::new::<$db>().await?;

        $(
            let query = format!("SELECT {}", $text);

            let row = sqlx_oldapi::query(&query)
                .fetch_one(&mut conn)
                .await?;

            let rec: $ty = row.try_get(0)?;

            assert_eq!($value, rec);
        )+
    }}
}

// Test type encoding and decoding for the prepared query API
#[macro_export]
macro_rules! __test_prepared_type {
    ($name:ident<$ty:ty>($db:ident, $sql:expr, $($text:literal == $value:expr),+ $(,)?)) => {
        #[sqlx_macros::test]
        #[allow(non_snake_case)]
        async fn $name() -> anyhow::Result<()> {
            $crate::__test_prepared_type_body!($ty, $db, $sql, $($text == $value),+);

            Ok(())
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __test_prepared_type_body {
    ($ty:ty, $db:ident, $sql:expr, $($text:literal == $value:expr),+ $(,)?) => {{
        use sqlx_oldapi::Row;

        let mut conn = sqlx_test::new::<$db>().await?;

        $(
            let query = format!($sql, $text);
            println!("{query}");

            let row = sqlx_oldapi::query(&query)
                .bind($value)
                .bind($value)
                .fetch_one(&mut conn)
                .await?;

            let matches: i32 = row.try_get(0)?;
            let returned: $ty = row.try_get(1)?;
            let round_trip: $ty = row.try_get(2)?;

            assert!(matches != 0,
                    "[1] DB value mismatch (SQL query returned non-zero value); given value: {:?}\n\
                     as returned: {:?}\n\
                     round-trip: {:?}",
                    $value, returned, round_trip);

            assert_eq!($value, returned,
                    "[2] DB value mismatch (given value does not match returned); given value: {:?}\n\
                             as returned: {:?}\n\
                             round-trip: {:?}",
                            $value, returned, round_trip);

            assert_eq!($value, round_trip,
                    "[3] DB value mismatch (given value does not match round-trip); given value: {:?}\n\
                             as returned: {:?}\n\
                             round-trip: {:?}",
                            $value, returned, round_trip);
        )+
    }}
}

#[macro_export]
macro_rules! query_for_test_prepared_type {
    (MySql) => {
        // MySQL 8.0.27 changed `<=>` to return an unsigned integer
        "SELECT CAST({0} <=> ? AS SIGNED INTEGER), {0}, ?"
    };

    (Mssql) => {
        "SELECT CASE WHEN {0} IS NULL AND @p1 IS NULL THEN 1 WHEN {0} = @p1 THEN 1 ELSE 0 END, {0}, @p2"
    };

    (Sqlite) => {
        "SELECT {0} is ?, {0}, ?"
    };

    (Postgres) => {
        "SELECT ({0} is not distinct from $1)::int4, {0}, $2"
    };

    (Odbc) => {
        // Most ODBC drivers support standard SQL syntax for NULL-safe comparison
        "SELECT CASE WHEN {0} IS NOT DISTINCT FROM ? THEN 1 ELSE 0 END, {0}, ?"
    };
}
