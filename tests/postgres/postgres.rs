use futures::{StreamExt, TryStreamExt};
use sqlx_oldapi::postgres::types::Oid;
use sqlx_oldapi::postgres::{
    PgAdvisoryLock, PgConnectOptions, PgConnection, PgDatabaseError, PgErrorPosition, PgListener,
    PgPoolOptions, PgRow, PgSeverity, Postgres,
};
use sqlx_oldapi::{Column, Connection, Executor, Row, Statement, TypeInfo};
use sqlx_test::{new, pool, setup_if_needed};
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[sqlx_macros::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let value = sqlx_oldapi::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(2i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_select_void() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // pg_notify just happens to be a function that returns void
    let _: () = sqlx_oldapi::query_scalar("select pg_notify('chan', 'message');")
        .fetch_one(&mut conn)
        .await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_pings() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.ping().await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_pings_after_suspended_query() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute("create temporary table processed_row(val int4 primary key)")
        .await?;

    // This query wants to return 50 rows but we only read the first one.
    // This will return a `SuspendedPortal` that the driver currently ignores.
    let _: i32 = sqlx_oldapi::query_scalar(
        r#"
            insert into processed_row(val)
            select * from generate_series(1, 50)
            returning val
        "#,
    )
    .fetch_one(&mut conn)
    .await?;

    // `Sync` closes the current autocommit transaction which presumably includes closing any
    // suspended portals.
    conn.ping().await?;

    // Make sure that all the values got inserted even though we only read the first one back.
    let count: i64 = sqlx_oldapi::query_scalar("select count(*) from processed_row")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 50);

    Ok(())
}

#[sqlx_macros::test]
async fn it_maths() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let value = sqlx_oldapi::query("select 1 + $1::int")
        .bind(5_i32)
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(6i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_inspect_errors() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let res: Result<_, sqlx_oldapi::Error> =
        sqlx_oldapi::query("select f").execute(&mut conn).await;
    let err = res.unwrap_err();

    // can also do [as_database_error] or use `match ..`
    let err = err.into_database_error().unwrap();

    assert_eq!(err.message(), "column \"f\" does not exist");
    assert_eq!(err.code().as_deref(), Some("42703"));

    // can also do [downcast_ref]
    let err: Box<PgDatabaseError> = err.downcast();

    assert_eq!(err.severity(), PgSeverity::Error);
    assert_eq!(err.message(), "column \"f\" does not exist");
    assert_eq!(err.code(), "42703");
    assert_eq!(err.position(), Some(PgErrorPosition::Original(8)));
    assert_eq!(err.routine(), Some("errorMissingColumn"));
    assert_eq!(err.constraint(), None);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_inspect_constraint_errors() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let res: Result<_, sqlx_oldapi::Error> =
        sqlx_oldapi::query("INSERT INTO products VALUES (1, 'Product 1', 0);")
            .execute(&mut conn)
            .await;
    let err = res.unwrap_err();

    // can also do [as_database_error] or use `match ..`
    let err = err.into_database_error().unwrap();

    assert_eq!(
        err.message(),
        "new row for relation \"products\" violates check constraint \"products_price_check\""
    );
    assert_eq!(err.code().as_deref(), Some("23514"));

    // can also do [downcast_ref]
    let err: Box<PgDatabaseError> = err.downcast();

    assert_eq!(err.severity(), PgSeverity::Error);
    assert_eq!(
        err.message(),
        "new row for relation \"products\" violates check constraint \"products_price_check\""
    );
    assert_eq!(err.code(), "23514");
    assert_eq!(err.position(), None);
    assert_eq!(err.routine(), Some("ExecConstraints"));
    assert_eq!(err.constraint(), Some("products_price_check"));

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let done = sqlx_oldapi::query("INSERT INTO users (id) VALUES ($1)")
            .bind(index)
            .execute(&mut conn)
            .await?;

        assert_eq!(done.rows_affected(), 1);
    }

    let sum: i32 = sqlx_oldapi::query("SELECT id FROM users")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 55);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_nest_map() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let res = sqlx_oldapi::query("SELECT 5")
        .map(|row: PgRow| row.get(0))
        .map(|int: i32| int.to_string())
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(res, "5");

    Ok(())
}

#[cfg(feature = "json")]
#[sqlx_macros::test]
async fn it_describes_and_inserts_json_and_jsonb() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TEMPORARY TABLE json_stuff (obj json, obj2 jsonb);
            "#,
        )
        .await?;

    let query = "INSERT INTO json_stuff (obj, obj2) VALUES ($1, $2)";
    let _ = conn.describe(query).await?;

    let done = sqlx_oldapi::query(query)
        .bind(serde_json::json!({ "a": "a" }))
        .bind(serde_json::json!({ "a": "a" }))
        .execute(&mut conn)
        .await?;

    assert_eq!(done.rows_affected(), 1);

    Ok(())
}

#[sqlx_macros::test]
async fn it_works_with_cache_disabled() -> anyhow::Result<()> {
    setup_if_needed();

    let mut url = url::Url::parse(&env::var("DATABASE_URL")?)?;
    url.query_pairs_mut()
        .append_pair("statement-cache-capacity", "0");

    let mut conn = PgConnection::connect(url.as_ref()).await?;

    for index in 1..=10_i32 {
        let _ = sqlx_oldapi::query("SELECT $1")
            .bind(index)
            .execute(&mut conn)
            .await?;
    }

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes_with_pool() -> anyhow::Result<()> {
    let pool = sqlx_test::pool::<Postgres>().await?;

    let rows = pool.fetch_all("SELECT 1; SElECT 2").await?;

    assert_eq!(rows.len(), 2);

    Ok(())
}

// https://github.com/launchbadge/sqlx/issues/104
#[sqlx_macros::test]
async fn it_can_return_interleaved_nulls_issue_104() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let tuple =
        sqlx_oldapi::query("SELECT NULL, 10::INT, NULL, 20::INT, NULL, 40::INT, NULL, 80::INT")
            .map(|row: PgRow| {
                (
                    row.get::<Option<i32>, _>(0),
                    row.get::<Option<i32>, _>(1),
                    row.get::<Option<i32>, _>(2),
                    row.get::<Option<i32>, _>(3),
                    row.get::<Option<i32>, _>(4),
                    row.get::<Option<i32>, _>(5),
                    row.get::<Option<i32>, _>(6),
                    row.get::<Option<i32>, _>(7),
                )
            })
            .fetch_one(&mut conn)
            .await?;

    assert_eq!(tuple.0, None);
    assert_eq!(tuple.1, Some(10));
    assert_eq!(tuple.2, None);
    assert_eq!(tuple.3, Some(20));
    assert_eq!(tuple.4, None);
    assert_eq!(tuple.5, Some(40));
    assert_eq!(tuple.6, None);
    assert_eq!(tuple.7, Some(80));

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fail_and_recover() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    for i in 0..10 {
        // make a query that will fail
        let res = conn
            .execute("INSERT INTO not_found (column) VALUES (10)")
            .await;

        assert!(res.is_err());

        // now try and use the connection
        let val: i32 = conn
            .fetch_one(&*format!("SELECT {}::int4", i))
            .await?
            .get(0);

        assert_eq!(val, i);
    }

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fail_and_recover_with_pool() -> anyhow::Result<()> {
    let pool = sqlx_test::pool::<Postgres>().await?;

    for i in 0..10 {
        // make a query that will fail
        let res = pool
            .execute("INSERT INTO not_found (column) VALUES (10)")
            .await;

        assert!(res.is_err());

        // now try and use the connection
        let val: i32 = pool
            .fetch_one(&*format!("SELECT {}::int4", i))
            .await?
            .get(0);

        assert_eq!(val, i);
    }

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_query_scalar() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let scalar: i32 = sqlx_oldapi::query_scalar("SELECT 42")
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(scalar, 42);

    let scalar: Option<i32> = sqlx_oldapi::query_scalar("SELECT 42")
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(scalar, Some(42));

    let scalar: Option<i32> = sqlx_oldapi::query_scalar("SELECT NULL")
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(scalar, None);

    let scalar: Option<i64> = sqlx_oldapi::query_scalar("SELECT 42::bigint")
        .fetch_optional(&mut conn)
        .await?;
    assert_eq!(scalar, Some(42));

    let scalar: Option<i16> = sqlx_oldapi::query_scalar("")
        .fetch_optional(&mut conn)
        .await?;
    assert_eq!(scalar, None);

    Ok(())
}

#[sqlx_macros::test]
/// This is separate from `it_can_query_scalar` because while implementing it I ran into a
/// bug which that prevented `Vec<i32>` from compiling but allowed Vec<Option<i32>>.
async fn it_can_query_all_scalar() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let scalar: Vec<i32> = sqlx_oldapi::query_scalar("SELECT $1")
        .bind(42)
        .fetch_all(&mut conn)
        .await?;
    assert_eq!(scalar, vec![42]);

    let scalar: Vec<Option<i32>> = sqlx_oldapi::query_scalar("SELECT $1 UNION ALL SELECT NULL")
        .bind(42)
        .fetch_all(&mut conn)
        .await?;
    assert_eq!(scalar, vec![Some(42), None]);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_work_with_transactions() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute("CREATE TABLE IF NOT EXISTS _sqlx_users_1922 (id INTEGER PRIMARY KEY)")
        .await?;

    conn.execute("TRUNCATE _sqlx_users_1922").await?;

    // begin .. rollback

    let mut tx = conn.begin().await?;

    sqlx_oldapi::query("INSERT INTO _sqlx_users_1922 (id) VALUES ($1)")
        .bind(10_i32)
        .execute(&mut tx)
        .await?;

    tx.rollback().await?;

    let (count,): (i64,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 0);

    // begin .. commit

    let mut tx = conn.begin().await?;

    sqlx_oldapi::query("INSERT INTO _sqlx_users_1922 (id) VALUES ($1)")
        .bind(10_i32)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    let (count,): (i64,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 1);

    // begin .. (drop)

    {
        let mut tx = conn.begin().await?;

        sqlx_oldapi::query("INSERT INTO _sqlx_users_1922 (id) VALUES ($1)")
            .bind(20_i32)
            .execute(&mut tx)
            .await?;
    }

    conn = new::<Postgres>().await?;

    let (count,): (i64,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 1);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_work_with_nested_transactions() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute("CREATE TABLE IF NOT EXISTS _sqlx_users_2523 (id INTEGER PRIMARY KEY)")
        .await?;

    conn.execute("TRUNCATE _sqlx_users_2523").await?;

    // begin
    let mut tx = conn.begin().await?; // transaction

    // insert a user
    sqlx_oldapi::query("INSERT INTO _sqlx_users_2523 (id) VALUES ($1)")
        .bind(50_i32)
        .execute(&mut tx)
        .await?;

    // begin once more
    let mut tx2 = tx.begin().await?; // savepoint

    // insert another user
    sqlx_oldapi::query("INSERT INTO _sqlx_users_2523 (id) VALUES ($1)")
        .bind(10_i32)
        .execute(&mut tx2)
        .await?;

    // never mind, rollback
    tx2.rollback().await?; // roll that one back

    // did we really?
    let (count,): (i64,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_2523")
        .fetch_one(&mut tx)
        .await?;

    assert_eq!(count, 1);

    // actually, commit
    tx.commit().await?;

    // did we really?
    let (count,): (i64,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_2523")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 1);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_drop_multiple_transactions() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute("CREATE TABLE IF NOT EXISTS _sqlx_users_3952 (id INTEGER PRIMARY KEY)")
        .await?;

    conn.execute("TRUNCATE _sqlx_users_3952").await?;

    // begin .. (drop)

    // run 2 times to see what happens if we drop transactions repeatedly
    for _ in 0..2 {
        {
            let mut tx = conn.begin().await?;

            // do actually something before dropping
            let _user =
                sqlx_oldapi::query("INSERT INTO _sqlx_users_3952 (id) VALUES ($1) RETURNING id")
                    .bind(20_i32)
                    .fetch_one(&mut tx)
                    .await?;
        }

        let (count,): (i64,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_3952")
            .fetch_one(&mut conn)
            .await?;

        assert_eq!(count, 0);
    }

    Ok(())
}

// run with `cargo test --features postgres -- --ignored --nocapture pool_smoke_test`
#[ignore]
#[sqlx_macros::test]
async fn pool_smoke_test() -> anyhow::Result<()> {
    use futures::{future, task::Poll, Future};

    eprintln!("starting pool");

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(5))
        .min_connections(1)
        .max_connections(1)
        .connect(&dotenvy::var("DATABASE_URL")?)
        .await?;

    // spin up more tasks than connections available, and ensure we don't deadlock
    for i in 0..200 {
        let pool = pool.clone();
        sqlx_rt::spawn(async move {
            for j in 0.. {
                if let Err(e) = sqlx_oldapi::query("select 1 + 1").execute(&pool).await {
                    // normal error at termination of the test
                    if matches!(e, sqlx_oldapi::Error::PoolClosed) {
                        eprintln!("pool task {} exiting normally after {} iterations", i, j);
                    } else {
                        eprintln!("pool task {} dying due to {} after {} iterations", i, e, j);
                    }
                    break;
                }

                // shouldn't be necessary if the pool is fair
                // sqlx_rt::yield_now().await;
            }
        });
    }

    // spawn a bunch of tasks that attempt to acquire but give up to ensure correct handling
    // of cancellations
    for _ in 0..50 {
        let pool = pool.clone();
        sqlx_rt::spawn(async move {
            while !pool.is_closed() {
                let acquire = pool.acquire();
                futures::pin_mut!(acquire);

                // poll the acquire future once to put the waiter in the queue
                future::poll_fn(move |cx| {
                    let _ = acquire.as_mut().poll(cx);
                    Poll::Ready(())
                })
                .await;

                // this one is necessary since this is a hot loop,
                // otherwise this task will never be descheduled
                sqlx_rt::yield_now().await;
            }
        });
    }

    eprintln!("sleeping for 30 seconds");

    sqlx_rt::sleep(Duration::from_secs(30)).await;

    // assert_eq!(pool.size(), 10);

    eprintln!("closing pool");

    sqlx_rt::timeout(Duration::from_secs(30), pool.close()).await?;

    eprintln!("pool closed successfully");

    Ok(())
}

#[sqlx_macros::test]
async fn test_invalid_query() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    conn.execute("definitely not a correct query")
        .await
        .unwrap_err();

    let mut s = conn.fetch("select 1");
    let row = s.try_next().await?.unwrap();

    assert_eq!(row.get::<i32, _>(0), 1i32);

    Ok(())
}

/// Tests the edge case of executing a completely empty query string.
///
/// This gets flagged as an `EmptyQueryResponse` in Postgres. We
/// catch this and just return no rows.
#[sqlx_macros::test]
async fn test_empty_query() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    let done = conn.execute("").await?;

    assert_eq!(done.rows_affected(), 0);

    Ok(())
}

/// Test a simple select expression. This should return the row.
#[sqlx_macros::test]
async fn test_select_expression() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let mut s = conn.fetch("SELECT 5");
    let row = s.try_next().await?.unwrap();

    assert!(5i32 == row.try_get::<i32, _>(0)?);

    Ok(())
}

/// Test that we can interleave reads and writes to the database
/// in one simple query. Using the `Cursor` API we should be
/// able to fetch from both queries in sequence.
#[sqlx_macros::test]
async fn test_multi_read_write() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    let mut s = conn.fetch(
        "
CREATE TABLE IF NOT EXISTS _sqlx_test_postgres_5112 (
    id BIGSERIAL PRIMARY KEY,
    text TEXT NOT NULL
);

SELECT 'Hello World' as _1;

INSERT INTO _sqlx_test_postgres_5112 (text) VALUES ('this is a test');

SELECT id, text FROM _sqlx_test_postgres_5112;
    ",
    );

    let row = s.try_next().await?.unwrap();

    assert!("Hello World" == row.try_get::<&str, _>("_1")?);

    let row = s.try_next().await?.unwrap();

    let id: i64 = row.try_get("id")?;
    let text: &str = row.try_get("text")?;

    assert_eq!(1_i64, id);
    assert_eq!("this is a test", text);

    Ok(())
}

#[sqlx_macros::test]
async fn it_caches_statements() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    for i in 0..2 {
        let row = sqlx_oldapi::query("SELECT $1 AS val")
            .bind(Oid(i))
            .persistent(true)
            .fetch_one(&mut conn)
            .await?;

        let val: Oid = row.get("val");

        assert_eq!(Oid(i), val);
    }

    assert_eq!(1, conn.cached_statements_size());
    conn.clear_cached_statements().await?;
    assert_eq!(0, conn.cached_statements_size());

    for i in 0..2 {
        let row = sqlx_oldapi::query("SELECT $1 AS val")
            .bind(Oid(i))
            .persistent(false)
            .fetch_one(&mut conn)
            .await?;

        let val: Oid = row.get("val");

        assert_eq!(Oid(i), val);
    }

    assert_eq!(0, conn.cached_statements_size());

    Ok(())
}

#[sqlx_macros::test]
async fn it_closes_statement_from_cache_issue_470() -> anyhow::Result<()> {
    sqlx_test::setup_if_needed();

    let mut options: PgConnectOptions = env::var("DATABASE_URL")?.parse().unwrap();

    // a capacity of 1 means that before each statement (after the first)
    // we will close the previous statement
    options = options.statement_cache_capacity(1);

    let mut conn = PgConnection::connect_with(&options).await?;

    for i in 0..5 {
        let row = sqlx_oldapi::query("SELECT $1::int4 AS val")
            .bind(i)
            .fetch_one(&mut conn)
            .await?;

        let val: i32 = row.get("val");

        assert_eq!(i, val);
    }

    assert_eq!(1, conn.cached_statements_size());

    Ok(())
}

#[sqlx_macros::test]
async fn it_sets_application_name() -> anyhow::Result<()> {
    sqlx_test::setup_if_needed();

    let mut options: PgConnectOptions = env::var("DATABASE_URL")?.parse().unwrap();
    options = options.application_name("some-name");

    let mut conn = PgConnection::connect_with(&options).await?;

    let row = sqlx_oldapi::query("select current_setting('application_name') as app_name")
        .fetch_one(&mut conn)
        .await?;

    let val: String = row.get("app_name");

    assert_eq!("some-name", &val);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_handle_parameter_status_message_issue_484() -> anyhow::Result<()> {
    new::<Postgres>().await?.execute("SET NAMES 'UTF8'").await?;
    Ok(())
}

#[sqlx_macros::test]
async fn it_can_prepare_then_execute() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    let mut tx = conn.begin().await?;

    let tweet_id: i64 = sqlx_oldapi::query_scalar(
        "INSERT INTO tweet ( text ) VALUES ( 'Hello, World' ) RETURNING id",
    )
    .fetch_one(&mut tx)
    .await?;

    let statement = tx.prepare("SELECT * FROM tweet WHERE id = $1").await?;

    assert_eq!(statement.column(0).name(), "id");
    assert_eq!(statement.column(1).name(), "created_at");
    assert_eq!(statement.column(2).name(), "text");
    assert_eq!(statement.column(3).name(), "owner_id");

    assert_eq!(statement.column(0).type_info().name(), "INT8");
    assert_eq!(statement.column(1).type_info().name(), "TIMESTAMPTZ");
    assert_eq!(statement.column(2).type_info().name(), "TEXT");
    assert_eq!(statement.column(3).type_info().name(), "INT8");

    let row = statement.query().bind(tweet_id).fetch_one(&mut tx).await?;
    let tweet_text: &str = row.try_get("text")?;

    assert_eq!(tweet_text, "Hello, World");

    Ok(())
}

// repro is more reliable with the basic scheduler used by `#[tokio::test]`
#[cfg(feature = "_rt-tokio")]
#[tokio::test]
async fn test_issue_622() -> anyhow::Result<()> {
    use std::time::Instant;

    setup_if_needed();

    let pool = PgPoolOptions::new()
        .max_connections(1) // also fails with higher counts, e.g. 5
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await?;

    println!("pool state: {:?}", pool);

    let mut handles = vec![];

    // given repro spawned 100 tasks but I found it reliably reproduced with 3
    for i in 0..3 {
        let pool = pool.clone();

        handles.push(sqlx_rt::spawn(async move {
            {
                let mut conn = pool.acquire().await.unwrap();

                let _ = sqlx_oldapi::query("SELECT 1")
                    .fetch_one(&mut conn)
                    .await
                    .unwrap();

                // conn gets dropped here and should be returned to the pool
            }

            // (do some other work here without holding on to a connection)
            // this actually fixes the issue, depending on the timeout used
            // sqlx_rt::sleep(Duration::from_millis(500)).await;

            {
                let start = Instant::now();
                match pool.acquire().await {
                    Ok(conn) => {
                        println!("{} acquire took {:?}", i, start.elapsed());
                        drop(conn);
                    }
                    Err(e) => panic!("{} acquire returned error: {} pool state: {:?}", i, e, pool),
                }
            }

            Result::<(), anyhow::Error>::Ok(())
        }));
    }

    futures::future::try_join_all(handles).await?;

    Ok(())
}

#[sqlx_macros::test]
async fn test_describe_outer_join_nullable() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    // test nullability inference for various joins

    // inner join, nullability should not be overridden
    // language=PostgreSQL
    let describe = conn
        .describe(
            "select tweet.id
    from tweet
    inner join products on products.name = tweet.text",
        )
        .await?;

    assert_eq!(describe.nullable(0), Some(false));

    // language=PostgreSQL
    let describe = conn
        .describe(
            "select tweet.id
from (values (null)) vals(val)
         left join tweet on false",
        )
        .await?;

    // tweet.id is marked NOT NULL but it's brought in from a left-join here
    // which should make it nullable
    assert_eq!(describe.nullable(0), Some(true));

    // make sure we don't mis-infer for the outer half of the join
    // language=PostgreSQL
    let describe = conn
        .describe(
            "select tweet1.id, tweet2.id
    from tweet tweet1
    left join tweet tweet2 on false",
        )
        .await?;

    assert_eq!(describe.nullable(0), Some(false));
    assert_eq!(describe.nullable(1), Some(true));

    // right join, nullability should be inverted
    // language=PostgreSQL
    let describe = conn
        .describe(
            "select tweet1.id, tweet2.id
    from tweet tweet1
    right join tweet tweet2 on false",
        )
        .await?;

    assert_eq!(describe.nullable(0), Some(true));
    assert_eq!(describe.nullable(1), Some(false));

    // full outer join, both tables are nullable
    // language=PostgreSQL
    let describe = conn
        .describe(
            "select tweet1.id, tweet2.id
    from tweet tweet1
    full join tweet tweet2 on false",
        )
        .await?;

    assert_eq!(describe.nullable(0), Some(true));
    assert_eq!(describe.nullable(1), Some(true));

    Ok(())
}

#[cfg(feature = "_rt-tokio")] // TODO: fix stack overflow on async-std
#[sqlx_macros::test]
async fn test_listener_cleanup() -> anyhow::Result<()> {
    #[cfg(feature = "_rt-tokio")]
    use tokio::time::timeout;

    #[cfg(feature = "_rt-async-std")]
    use async_std::future::timeout;

    use sqlx_oldapi::pool::PoolOptions;
    use sqlx_oldapi::postgres::PgListener;

    // Create a connection on which to send notifications
    let mut notify_conn = new::<Postgres>().await?;

    // Create a pool with exactly one connection so we can
    // deterministically test the cleanup.
    let pool = PoolOptions::<Postgres>::new()
        .min_connections(1)
        .max_connections(1)
        .test_before_acquire(true)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("test_channel").await?;

    let ms = Duration::from_millis(100);

    // Check no notification is received before one is sent
    timeout(ms, listener.recv())
        .await
        .expect_err("Notification not sent");

    // Check notification is sent and received
    notify_conn.execute("NOTIFY test_channel, 'x'").await?;
    assert_eq!(
        timeout(ms, listener.recv())
            .await
            .unwrap()
            .unwrap()
            .payload(),
        "x",
        "Notification sent and received"
    );

    timeout(ms, listener.recv())
        .await
        .expect_err("Notification is not duplicated");

    // Test that cleanup stops listening on the channel
    drop(listener);
    let mut listener = PgListener::connect_with(&pool).await?;

    // Check notification is sent but not received
    notify_conn.execute("NOTIFY test_channel").await?;
    timeout(ms, listener.recv())
        .await
        .expect_err("Notification is not received on fresh listener");

    Ok(())
}

#[sqlx_macros::test]
async fn test_pg_listener_allows_pool_to_close() -> anyhow::Result<()> {
    let pool = pool::<Postgres>().await?;

    // acquires and holds a connection which would normally prevent the pool from closing
    let mut listener = PgListener::connect_with(&pool).await?;

    sqlx_rt::spawn(async move {
        listener.recv().await.unwrap();
    });

    // would previously hang forever since `PgListener` had no way to know the pool wanted to close
    pool.close().await;

    Ok(())
}

#[sqlx_macros::test]
async fn it_supports_domain_types_in_composite_domain_types() -> anyhow::Result<()> {
    // Only supported in Postgres 11+
    let mut conn = new::<Postgres>().await?;
    if matches!(conn.server_version_num(), Some(version) if version < 110000) {
        return Ok(());
    }

    conn.execute(
        r#"
DROP TABLE IF EXISTS heating_bills;
DROP DOMAIN IF EXISTS winter_year_month;
DROP TYPE IF EXISTS year_month;
DROP DOMAIN IF EXISTS month_id;

CREATE DOMAIN month_id AS INT2 CHECK (1 <= value AND value <= 12);
CREATE TYPE year_month AS (year INT4, month month_id);
CREATE DOMAIN winter_year_month AS year_month CHECK ((value).month <= 3);
CREATE TABLE heating_bills (
  month winter_year_month NOT NULL PRIMARY KEY,
  cost INT4 NOT NULL
);
    "#,
    )
    .await?;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct MonthId(i16);

    impl sqlx_oldapi::Type<Postgres> for MonthId {
        fn type_info() -> sqlx_oldapi::postgres::PgTypeInfo {
            sqlx_oldapi::postgres::PgTypeInfo::with_name("month_id")
        }

        fn compatible(ty: &sqlx_oldapi::postgres::PgTypeInfo) -> bool {
            *ty == Self::type_info()
        }
    }

    impl<'r> sqlx_oldapi::Decode<'r, Postgres> for MonthId {
        fn decode(
            value: sqlx_oldapi::postgres::PgValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            Ok(Self(<i16 as sqlx_oldapi::Decode<Postgres>>::decode(value)?))
        }
    }

    impl<'q> sqlx_oldapi::Encode<'q, Postgres> for MonthId {
        fn encode_by_ref(
            &self,
            buf: &mut sqlx_oldapi::postgres::PgArgumentBuffer,
        ) -> sqlx_oldapi::encode::IsNull {
            <i16 as sqlx_oldapi::Encode<Postgres>>::encode(self.0, buf)
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct WinterYearMonth {
        year: i32,
        month: MonthId,
    }

    impl sqlx_oldapi::Type<Postgres> for WinterYearMonth {
        fn type_info() -> sqlx_oldapi::postgres::PgTypeInfo {
            sqlx_oldapi::postgres::PgTypeInfo::with_name("winter_year_month")
        }

        fn compatible(ty: &sqlx_oldapi::postgres::PgTypeInfo) -> bool {
            *ty == Self::type_info()
        }
    }

    impl<'r> sqlx_oldapi::Decode<'r, Postgres> for WinterYearMonth {
        fn decode(
            value: sqlx_oldapi::postgres::PgValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            let mut decoder = sqlx_oldapi::postgres::types::PgRecordDecoder::new(value)?;

            let year = decoder.try_decode::<i32>()?;
            let month = decoder.try_decode::<MonthId>()?;

            Ok(Self { year, month })
        }
    }

    impl<'q> sqlx_oldapi::Encode<'q, Postgres> for WinterYearMonth {
        fn encode_by_ref(
            &self,
            buf: &mut sqlx_oldapi::postgres::PgArgumentBuffer,
        ) -> sqlx_oldapi::encode::IsNull {
            let mut encoder = sqlx_oldapi::postgres::types::PgRecordEncoder::new(buf);
            encoder.encode(self.year);
            encoder.encode(self.month);
            encoder.finish();
            sqlx_oldapi::encode::IsNull::No
        }
    }
    let mut conn = new::<Postgres>().await?;

    let result = sqlx_oldapi::query("DELETE FROM heating_bills;")
        .execute(&mut conn)
        .await;

    let result = result.unwrap();
    assert_eq!(result.rows_affected(), 0);

    let result = sqlx_oldapi::query(
        "INSERT INTO heating_bills(month, cost) VALUES($1::winter_year_month, 100);",
    )
    .bind(WinterYearMonth {
        year: 2021,
        month: MonthId(1),
    })
    .execute(&mut conn)
    .await;

    let result = result.unwrap();
    assert_eq!(result.rows_affected(), 1);

    let result = sqlx_oldapi::query("DELETE FROM heating_bills;")
        .execute(&mut conn)
        .await;

    let result = result.unwrap();
    assert_eq!(result.rows_affected(), 1);

    Ok(())
}

#[sqlx_macros::test]
#[cfg(feature = "macros")]
async fn it_resolves_custom_type_in_array() -> anyhow::Result<()> {
    // Only supported in Postgres 11+
    let mut conn = new::<Postgres>().await?;
    if matches!(conn.server_version_num(), Some(version) if version < 110000) {
        return Ok(());
    }

    // language=PostgreSQL
    conn.execute(
        r#"
DROP TABLE IF EXISTS pets;
DROP TYPE IF EXISTS pet_name_and_race;

CREATE TYPE pet_name_and_race AS (
  name TEXT,
  race TEXT
);
CREATE TABLE pets (
  owner TEXT NOT NULL,
  name TEXT NOT NULL,
  race TEXT NOT NULL,
  PRIMARY KEY (owner, name)
);
INSERT INTO pets(owner, name, race)
VALUES
  ('Alice', 'Foo', 'cat');
INSERT INTO pets(owner, name, race)
VALUES
  ('Alice', 'Bar', 'dog');
    "#,
    )
    .await?;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct PetNameAndRace {
        name: String,
        race: String,
    }

    impl sqlx_oldapi::Type<Postgres> for PetNameAndRace {
        fn type_info() -> sqlx_oldapi::postgres::PgTypeInfo {
            sqlx_oldapi::postgres::PgTypeInfo::with_name("pet_name_and_race")
        }
    }

    impl<'r> sqlx_oldapi::Decode<'r, Postgres> for PetNameAndRace {
        fn decode(
            value: sqlx_oldapi::postgres::PgValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            let mut decoder = sqlx_oldapi::postgres::types::PgRecordDecoder::new(value)?;
            let name = decoder.try_decode::<String>()?;
            let race = decoder.try_decode::<String>()?;
            Ok(Self { name, race })
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct PetNameAndRaceArray(Vec<PetNameAndRace>);

    impl sqlx_oldapi::Type<Postgres> for PetNameAndRaceArray {
        fn type_info() -> sqlx_oldapi::postgres::PgTypeInfo {
            // Array type name is the name of the element type prefixed with `_`
            sqlx_oldapi::postgres::PgTypeInfo::with_name("_pet_name_and_race")
        }
    }

    impl<'r> sqlx_oldapi::Decode<'r, Postgres> for PetNameAndRaceArray {
        fn decode(
            value: sqlx_oldapi::postgres::PgValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            Ok(Self(Vec::<PetNameAndRace>::decode(value)?))
        }
    }

    let mut conn = new::<Postgres>().await?;

    let row = sqlx_oldapi::query("select owner, array_agg(row(name, race)::pet_name_and_race) as pets from pets group by owner")
        .fetch_one(&mut conn)
        .await?;

    let pets: PetNameAndRaceArray = row.get("pets");

    assert_eq!(pets.0.len(), 2);
    Ok(())
}

#[sqlx_macros::test]
#[cfg(feature = "macros")]
async fn it_resolves_custom_types_in_anonymous_records() -> anyhow::Result<()> {
    use sqlx_core::error::Error;
    // This request involves nested records and array types.

    // Only supported in Postgres 11+
    let mut conn = new::<Postgres>().await?;
    if matches!(conn.server_version_num(), Some(version) if version < 110000) {
        return Ok(());
    }

    // language=PostgreSQL
    conn.execute(
        r#"
DROP TABLE IF EXISTS repo_users;
DROP TABLE IF EXISTS repositories;
DROP TABLE IF EXISTS repo_memberships;
DROP TYPE IF EXISTS repo_member;

CREATE TABLE repo_users (
  user_id INT4 NOT NULL,
  username TEXT NOT NULL,
  PRIMARY KEY (user_id)
);
CREATE TABLE repositories (
  repo_id INT4 NOT NULL,
  repo_name TEXT NOT NULL,
  PRIMARY KEY (repo_id)
);
CREATE TABLE repo_memberships (
  repo_id INT4 NOT NULL,
  user_id INT4 NOT NULL,
  permission TEXT NOT NULL,
  PRIMARY KEY (repo_id, user_id)
);
CREATE TYPE repo_member AS (
  user_id INT4,
  permission TEXT
);
INSERT INTO repo_users(user_id, username)
VALUES
  (101, 'alice'),
  (102, 'bob'),
  (103, 'charlie');
INSERT INTO repositories(repo_id, repo_name)
VALUES
  (201, 'rust'),
  (202, 'sqlx'),
  (203, 'hello-world');
INSERT INTO repo_memberships(repo_id, user_id, permission)
VALUES
  (201, 101, 'admin'),
  (201, 102, 'write'),
  (201, 103, 'read'),
  (202, 102, 'admin');
"#,
    )
    .await?;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct RepoMember {
        user_id: i32,
        permission: String,
    }

    impl sqlx_oldapi::Type<Postgres> for RepoMember {
        fn type_info() -> sqlx_oldapi::postgres::PgTypeInfo {
            sqlx_oldapi::postgres::PgTypeInfo::with_name("repo_member")
        }
    }

    impl<'r> sqlx_oldapi::Decode<'r, Postgres> for RepoMember {
        fn decode(
            value: sqlx_oldapi::postgres::PgValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            let mut decoder = sqlx_oldapi::postgres::types::PgRecordDecoder::new(value)?;
            let user_id = decoder.try_decode::<i32>()?;
            let permission = decoder.try_decode::<String>()?;
            Ok(Self {
                user_id,
                permission,
            })
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct RepoMemberArray(Vec<RepoMember>);

    impl sqlx_oldapi::Type<Postgres> for RepoMemberArray {
        fn type_info() -> sqlx_oldapi::postgres::PgTypeInfo {
            // Array type name is the name of the element type prefixed with `_`
            sqlx_oldapi::postgres::PgTypeInfo::with_name("_repo_member")
        }
    }

    impl<'r> sqlx_oldapi::Decode<'r, Postgres> for RepoMemberArray {
        fn decode(
            value: sqlx_oldapi::postgres::PgValueRef<'r>,
        ) -> Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            Ok(Self(Vec::<RepoMember>::decode(value)?))
        }
    }

    let mut conn = new::<Postgres>().await?;

    #[derive(Debug, sqlx_oldapi::FromRow)]
    #[allow(dead_code)] // We don't actually read these fields.
    struct Row {
        count: i64,
        items: Vec<(i32, String, RepoMemberArray)>,
    }
    // language=PostgreSQL
    let row: Result<Row, Error> = conn
        .prepare(
            r"
        WITH
          members_by_repo AS (
            SELECT repo_id,
              ARRAY_AGG(ROW (user_id, permission)::repo_member) AS members
            FROM repo_memberships
            GROUP BY repo_id
          ),
          repos AS (
            SELECT repo_id, repo_name, COALESCE(members, '{}') AS members
            FROM repositories
              LEFT OUTER JOIN members_by_repo USING (repo_id)
            ORDER BY repo_id
          ),
          repo_array AS (
            SELECT COALESCE(ARRAY_AGG(repos.*), '{}') AS items
            FROM repos
          ),
          repo_count AS (
            SELECT COUNT(*) AS count
            FROM repos
          )
        SELECT count, items
        FROM repo_count, repo_array
        ;
    ",
        )
        .await?
        .query_as::<Row>()
        .fetch_one(&mut conn)
        .await;

    // This test currently tests mitigations for `#1672` (use regular errors
    // instead of panics). Once we fully support custom types, it should be
    // updated accordingly.
    match row {
        Ok(_) => panic!("full support for custom types is not implemented yet"),
        Err(e) => assert!(e
            .to_string()
            .contains("custom types in records are not fully supported yet")),
    }
    Ok(())
}

#[sqlx_macros::test]
async fn test_pg_server_num() -> anyhow::Result<()> {
    let conn = new::<Postgres>().await?;

    assert!(conn.server_version_num().is_some());

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_copy_in() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    conn.execute(
        r#"
        CREATE TEMPORARY TABLE users (id INTEGER NOT NULL);
    "#,
    )
    .await?;

    let mut copy = conn
        .copy_in_raw(
            r#"
        COPY users (id) FROM STDIN WITH (FORMAT CSV, HEADER);
    "#,
        )
        .await?;

    copy.send("id\n1\n2\n".as_bytes()).await?;
    let rows = copy.finish().await?;
    assert_eq!(rows, 2);

    // conn is safe for reuse
    let value = sqlx_oldapi::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(2i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_abort_copy_in() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    conn.execute(
        r#"
        CREATE TEMPORARY TABLE users (id INTEGER NOT NULL);
    "#,
    )
    .await?;

    let copy = conn
        .copy_in_raw(
            r#"
        COPY users (id) FROM STDIN WITH (FORMAT CSV, HEADER);
    "#,
        )
        .await?;

    copy.abort("this is only a test").await?;

    // conn is safe for reuse
    let value = sqlx_oldapi::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(2i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_copy_out() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;

    {
        let mut copy = conn
            .copy_out_raw(
                "
            COPY (SELECT generate_series(1, 2) AS id) TO STDOUT WITH (FORMAT CSV, HEADER);
        ",
            )
            .await?;

        assert_eq!(copy.next().await.unwrap().unwrap(), "id\n");
        assert_eq!(copy.next().await.unwrap().unwrap(), "1\n");
        assert_eq!(copy.next().await.unwrap().unwrap(), "2\n");
        if copy.next().await.is_some() {
            anyhow::bail!("Unexpected data from COPY");
        }
    }

    // conn is safe for reuse
    let value = sqlx_oldapi::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(2i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_encodes_custom_array_issue_1504() -> anyhow::Result<()> {
    use sqlx_oldapi::encode::IsNull;
    use sqlx_oldapi::postgres::{PgArgumentBuffer, PgTypeInfo};
    use sqlx_oldapi::{Decode, Encode, Type, ValueRef};

    #[derive(Debug, PartialEq)]
    enum Value {
        String(String),
        Number(i32),
        Array(Vec<Value>),
    }

    impl<'r> Decode<'r, Postgres> for Value {
        fn decode(
            value: sqlx_oldapi::postgres::PgValueRef<'r>,
        ) -> std::result::Result<Self, Box<dyn std::error::Error + 'static + Send + Sync>> {
            let typ = value.type_info().into_owned();

            if typ == PgTypeInfo::with_name("text") {
                let s = <String as Decode<'_, Postgres>>::decode(value)?;

                Ok(Self::String(s))
            } else if typ == PgTypeInfo::with_name("int4") {
                let n = <i32 as Decode<'_, Postgres>>::decode(value)?;

                Ok(Self::Number(n))
            } else if typ == PgTypeInfo::with_name("_text") {
                let arr = Vec::<String>::decode(value)?;
                let v = arr.into_iter().map(Value::String).collect();

                Ok(Self::Array(v))
            } else if typ == PgTypeInfo::with_name("_int4") {
                let arr = Vec::<i32>::decode(value)?;
                let v = arr.into_iter().map(Value::Number).collect();

                Ok(Self::Array(v))
            } else {
                Err("unknown type".into())
            }
        }
    }

    impl Encode<'_, Postgres> for Value {
        fn produces(&self) -> Option<PgTypeInfo> {
            match self {
                Self::Array(a) => {
                    if a.is_empty() {
                        return Some(PgTypeInfo::with_name("_text"));
                    }

                    match a[0] {
                        Self::String(_) => Some(PgTypeInfo::with_name("_text")),
                        Self::Number(_) => Some(PgTypeInfo::with_name("_int4")),
                        Self::Array(_) => None,
                    }
                }
                Self::String(_) => Some(PgTypeInfo::with_name("text")),
                Self::Number(_) => Some(PgTypeInfo::with_name("int4")),
            }
        }

        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
            match self {
                Value::String(s) => <String as Encode<'_, Postgres>>::encode_by_ref(s, buf),
                Value::Number(n) => <i32 as Encode<'_, Postgres>>::encode_by_ref(n, buf),
                Value::Array(arr) => arr.encode(buf),
            }
        }
    }

    impl Type<Postgres> for Value {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_name("unknown")
        }

        fn compatible(ty: &PgTypeInfo) -> bool {
            [
                PgTypeInfo::with_name("text"),
                PgTypeInfo::with_name("_text"),
                PgTypeInfo::with_name("int4"),
                PgTypeInfo::with_name("_int4"),
            ]
            .contains(ty)
        }
    }

    let mut conn = new::<Postgres>().await?;

    let (row,): (Value,) = sqlx_oldapi::query_as("SELECT $1::text[] as Dummy")
        .bind(Value::Array(vec![
            Value::String("Test 0".to_string()),
            Value::String("Test 1".to_string()),
        ]))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(
        row,
        Value::Array(vec![
            Value::String("Test 0".to_string()),
            Value::String("Test 1".to_string()),
        ])
    );

    let (row,): (Value,) = sqlx_oldapi::query_as("SELECT $1::int4[] as Dummy")
        .bind(Value::Array(vec![
            Value::Number(3),
            Value::Number(2),
            Value::Number(1),
        ]))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(
        row,
        Value::Array(vec![Value::Number(3), Value::Number(2), Value::Number(1)])
    );

    Ok(())
}

#[sqlx_macros::test]
#[cfg(feature = "macros")]
async fn test_issue_1254() -> anyhow::Result<()> {
    #[derive(sqlx_oldapi::Type)]
    #[sqlx(type_name = "pair")]
    struct Pair {
        one: i32,
        two: i32,
    }

    // array for custom type is not supported, use wrapper
    #[derive(sqlx_oldapi::Type)]
    #[sqlx(type_name = "_pair")]
    struct Pairs(Vec<Pair>);

    let mut conn = new::<Postgres>().await?;
    conn.execute(
        "
DROP TABLE IF EXISTS issue_1254;
DROP TYPE IF EXISTS pair;

CREATE TYPE pair AS (one INT4, two INT4);
CREATE TABLE issue_1254 (id INT4 PRIMARY KEY, pairs PAIR[]);
",
    )
    .await?;

    let result = sqlx_oldapi::query("INSERT INTO issue_1254 VALUES($1, $2)")
        .bind(0)
        .bind(Pairs(vec![Pair { one: 94, two: 87 }]))
        .execute(&mut conn)
        .await?;
    assert_eq!(result.rows_affected(), 1);

    Ok(())
}

#[sqlx_macros::test]
async fn test_advisory_locks() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&dotenvy::var("DATABASE_URL")?)
        .await?;

    let lock1 = Arc::new(PgAdvisoryLock::new("sqlx-postgres-tests-1"));
    let lock2 = Arc::new(PgAdvisoryLock::new("sqlx-postgres-tests-2"));

    let conn1 = pool.acquire().await?;
    let mut conn1_lock1 = lock1.acquire(conn1).await?;

    // try acquiring a recursive lock through a mutable reference then dropping
    drop(lock1.acquire(&mut conn1_lock1).await?);

    let conn2 = pool.acquire().await?;
    // leak so we can take it across the task boundary
    let conn2_lock2 = lock2.acquire(conn2).await?.leak();

    sqlx_rt::spawn({
        let lock1 = lock1.clone();
        let lock2 = lock2.clone();

        async move {
            let conn2_lock2 = lock1.try_acquire(conn2_lock2).await?.right_or_else(|_| {
                panic!(
                    "acquired lock but wasn't supposed to! Key: {:?}",
                    lock1.key()
                )
            });

            let (conn2, released) = lock2.force_release(conn2_lock2).await?;
            assert!(released);

            // acquire both locks but let the pool release them
            let conn2_lock1 = lock1.acquire(conn2).await?;
            let _conn2_lock1and2 = lock2.acquire(conn2_lock1).await?;

            anyhow::Ok(())
        }
    });

    // acquire lock2 on conn1, we leak the lock1 guard so we can manually release it before lock2
    let conn1_lock1and2 = lock2.acquire(conn1_lock1.leak()).await?;

    // release lock1 while holding lock2
    let (conn1_lock2, released) = lock1.force_release(conn1_lock1and2).await?;
    assert!(released);

    let conn1 = conn1_lock2.release_now().await?;

    // acquire both locks to be sure they were released
    {
        let conn1_lock1 = lock1.acquire(conn1).await?;
        let _conn1_lock1and2 = lock2.acquire(conn1_lock1).await?;
    }

    pool.close().await;

    Ok(())
}

#[sqlx_macros::test]
async fn test_postgres_bytea_hex_deserialization_errors() -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    conn.execute("SET bytea_output = 'escape';").await?;
    for value in ["", "DEADBEEF"] {
        let query = format!("SELECT '\\x{}'::bytea", value);
        let res: sqlx_oldapi::Result<Vec<u8>> =
            conn.fetch_one(query.as_str()).await?.try_get(0usize);
        // Deserialization only supports hex format so this should error and definitely not panic.
        res.unwrap_err();
    }
    Ok(())
}

async fn test_copy_in_error_case(query: &str, expected_error: &str) -> anyhow::Result<()> {
    let mut conn = new::<Postgres>().await?;
    conn.execute("CREATE TEMPORARY TABLE IF NOT EXISTS invalid_copy_target (id int4)")
        .await?;

    // Try the COPY operation
    match conn.copy_in_raw(query).await {
        Ok(_) => anyhow::bail!("expected error"),
        Err(e) => assert!(
            e.to_string().contains(expected_error),
            "expected error to contain: {expected_error}, got: {e:?}"
        ),
    }

    // Verify connection is still usable
    let value = sqlx_oldapi::query("select 1 + 1")
        .try_map(|row: PgRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(2i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_recover_from_copy_in_to_missing_table() -> anyhow::Result<()> {
    test_copy_in_error_case(
        r#"
        COPY nonexistent_table (id) FROM STDIN WITH (FORMAT CSV, HEADER);
        "#,
        "does not exist",
    )
    .await
}

#[sqlx_macros::test]
async fn it_can_recover_from_copy_in_empty_query() -> anyhow::Result<()> {
    test_copy_in_error_case("", "EmptyQuery").await
}

#[sqlx_macros::test]
async fn it_can_recover_from_copy_in_syntax_error() -> anyhow::Result<()> {
    test_copy_in_error_case(
        r#"
        COPY FROM STDIN WITH (FORMAT CSV);
        "#,
        "syntax error",
    )
    .await
}

#[sqlx_macros::test]
async fn it_can_recover_from_copy_in_invalid_params() -> anyhow::Result<()> {
    test_copy_in_error_case(
        r#"
        COPY invalid_copy_target FROM STDIN WITH (FORMAT CSV, INVALID_PARAM true);
        "#,
        "invalid_param",
    )
    .await
}
