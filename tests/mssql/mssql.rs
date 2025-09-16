use futures::TryStreamExt;
use sqlx_core::mssql::MssqlRow;
use sqlx_oldapi::mssql::{Mssql, MssqlPoolOptions};
use sqlx_oldapi::{
    Column, Connection, Execute, Executor, MssqlConnection, Row, Statement, TypeInfo,
};
use sqlx_test::new;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

#[sqlx_macros::test]
async fn it_connects() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    conn.ping().await?;

    conn.close().await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_select_expression() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let row: MssqlRow = conn.fetch_one("SELECT 4").await?;
    let v: i32 = row.try_get(0)?;

    assert_eq!(v, 4);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_select_expression_by_name() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let row: MssqlRow = conn.fetch_one("SELECT 4 as _3").await?;
    let v: i32 = row.try_get("_3")?;

    assert_eq!(v, 4);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fail_to_connect() -> anyhow::Result<()> {
    let mut url = dotenvy::var("DATABASE_URL")?;
    url = url.replace("Password", "NotPassword");

    let res = MssqlConnection::connect(&url).await;
    let err = res.unwrap_err();
    let err = err.into_database_error().unwrap();

    assert_eq!(err.message(), "Login failed for user \'sa\'.");

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_inspect_errors() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let res: Result<_, sqlx_oldapi::Error> =
        sqlx_oldapi::query("select\nf").execute(&mut conn).await;
    let err = res.unwrap_err();

    assert_eq!(
        err.to_string(),
        "error returned from database: On line 2: Invalid column name 'f'."
    );

    // can also do [as_database_error] or use `match ..`
    let err = err.into_database_error().unwrap();

    assert_eq!(err.message(), "Invalid column name 'f'.");

    Ok(())
}

#[sqlx_macros::test]
async fn it_maths() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let value = sqlx_oldapi::query("SELECT 1 + @p1")
        .bind(5_i32)
        .try_map(|row: MssqlRow| row.try_get::<i32, _>(0))
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(6_i32, value);

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TABLE #users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=10_i32 {
        let done = sqlx_oldapi::query("INSERT INTO #users (id) VALUES (@p1)")
            .bind(index * 2)
            .execute(&mut conn)
            .await?;

        assert_eq!(done.rows_affected(), 1);
    }

    let sum: i32 = sqlx_oldapi::query("SELECT id FROM #users")
        .try_map(|row: MssqlRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 110);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_return_1000_rows() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let _ = conn
        .execute(
            r#"
CREATE TABLE #users (id INTEGER PRIMARY KEY);
            "#,
        )
        .await?;

    for index in 1..=1000_i32 {
        let done = sqlx_oldapi::query("INSERT INTO #users (id) VALUES (@p1)")
            .bind(index * 2)
            .execute(&mut conn)
            .await?;

        assert_eq!(done.rows_affected(), 1);
    }

    let sum: i32 = sqlx_oldapi::query("SELECT id FROM #users")
        .try_map(|row: MssqlRow| row.try_get::<i32, _>(0))
        .fetch(&mut conn)
        .try_fold(0_i32, |acc, x| async move { Ok(acc + x) })
        .await?;

    assert_eq!(sum, 1001000);

    Ok(())
}

#[sqlx_macros::test]
async fn it_selects_null() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let (_, val): (i32, Option<i32>) = sqlx_oldapi::query_as("SELECT 5, NULL")
        .fetch_one(&mut conn)
        .await?;

    assert!(val.is_none());

    let val: Option<i32> = conn.fetch_one("SELECT 10, NULL").await?.try_get(1)?;

    assert!(val.is_none());

    Ok(())
}

#[sqlx_macros::test]
async fn it_binds_empty_string_and_null() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let (val, val2): (String, Option<String>) = sqlx_oldapi::query_as("SELECT @p1, @p2")
        .bind("")
        .bind(None::<String>)
        .fetch_one(&mut conn)
        .await?;

    assert!(val.is_empty());
    assert!(val2.is_none());

    Ok(())
}

#[sqlx_macros::test]
async fn it_binds_string_with_special_chars() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;
    let s = "/\r\n\t\0\x1b!@#$/.%^\x1b&*()_+";
    let (val,): (String,) = sqlx_oldapi::query_as("SELECT @p1")
        .bind(s)
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(val, s);

    Ok(())
}

#[sqlx_macros::test]
async fn it_accepts_long_query_strings() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;
    // try a query that does not fit in a single TDS packet
    let (n,): (i32,) = sqlx_oldapi::query_as(&format!("SELECT {} 42", " ".repeat(3000)))
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(n, 42);
    Ok(())
}

#[sqlx_macros::test]
async fn it_can_work_with_transactions() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    conn.execute("IF OBJECT_ID('_sqlx_users_1922', 'U') IS NULL CREATE TABLE _sqlx_users_1922 (id INTEGER PRIMARY KEY)")
        .await?;

    conn.execute("DELETE FROM _sqlx_users_1922").await?;

    // begin .. rollback

    let mut tx = conn.begin().await?;

    sqlx_oldapi::query("INSERT INTO _sqlx_users_1922 (id) VALUES ($1)")
        .bind(10_i32)
        .execute(&mut tx)
        .await?;

    tx.rollback().await?;

    let (count,): (i32,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 0);

    // begin .. commit

    let mut tx = conn.begin().await?;

    sqlx_oldapi::query("INSERT INTO _sqlx_users_1922 (id) VALUES (@p1)")
        .bind(10_i32)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    let (count,): (i32,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 1);

    // begin .. (drop)

    {
        let mut tx = conn.begin().await?;

        sqlx_oldapi::query("INSERT INTO _sqlx_users_1922 (id) VALUES (@p1)")
            .bind(20_i32)
            .execute(&mut tx)
            .await?;
    }

    conn = new::<Mssql>().await?;

    let (count,): (i32,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_1922")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 1);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_work_with_nested_transactions() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    conn.execute("IF OBJECT_ID('_sqlx_users_2523', 'U') IS NULL CREATE TABLE _sqlx_users_2523 (id INTEGER PRIMARY KEY)")
        .await?;

    conn.execute("DELETE FROM _sqlx_users_2523").await?;

    // begin
    let mut tx = conn.begin().await?;

    // insert a user
    sqlx_oldapi::query("INSERT INTO _sqlx_users_2523 (id) VALUES (@p1)")
        .bind(50_i32)
        .execute(&mut tx)
        .await?;

    // begin once more
    let mut tx2 = tx.begin().await?;

    // insert another user
    sqlx_oldapi::query("INSERT INTO _sqlx_users_2523 (id) VALUES (@p1)")
        .bind(10_i32)
        .execute(&mut tx2)
        .await?;

    // never mind, rollback
    tx2.rollback().await?;

    // did we really?
    let (count,): (i32,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_2523")
        .fetch_one(&mut tx)
        .await?;

    assert_eq!(count, 1);

    // actually, commit
    tx.commit().await?;

    // did we really?
    let (count,): (i32,) = sqlx_oldapi::query_as("SELECT COUNT(*) FROM _sqlx_users_2523")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(count, 1);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_prepare_then_execute() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;
    let mut tx = conn.begin().await?;

    let tweet_id: i64 = sqlx_oldapi::query_scalar(
        "INSERT INTO tweet ( id, text ) OUTPUT INSERTED.id VALUES ( 50, 'Hello, World' )",
    )
    .fetch_one(&mut tx)
    .await?;

    let statement = tx.prepare("SELECT * FROM tweet WHERE id = @p1").await?;

    assert_eq!(statement.column(0).name(), "id");
    assert_eq!(statement.column(1).name(), "text");
    assert_eq!(statement.column(2).name(), "is_sent");
    assert_eq!(statement.column(3).name(), "owner_id");

    assert_eq!(statement.column(0).type_info().name(), "BIGINT");
    assert_eq!(statement.column(1).type_info().name(), "NVARCHAR");
    assert_eq!(statement.column(2).type_info().name(), "TINYINT");
    assert_eq!(statement.column(3).type_info().name(), "BIGINT");

    let row = statement.query().bind(tweet_id).fetch_one(&mut tx).await?;
    let tweet_text: String = row.try_get("text")?;

    assert_eq!(tweet_text, "Hello, World");

    Ok(())
}

// MSSQL-specific copy of the test case in `tests/any/pool.rs`
// because MSSQL has its own bespoke syntax for temporary tables.
#[sqlx_macros::test]
#[cfg(feature = "macros")]
async fn test_pool_callbacks() -> anyhow::Result<()> {
    #[derive(sqlx_oldapi::FromRow, Debug, PartialEq, Eq)]
    struct ConnStats {
        id: i32,
        before_acquire_calls: i32,
        after_release_calls: i32,
    }

    sqlx_test::setup_if_needed();

    let current_id = AtomicI32::new(0);

    let pool = MssqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .after_connect(move |conn, meta| {
            assert_eq!(meta.age, Duration::ZERO);
            assert_eq!(meta.idle_for, Duration::ZERO);

            let id = current_id.fetch_add(1, Ordering::AcqRel);

            Box::pin(async move {
                let statement = format!(
                    // language=MSSQL
                    r#"
                    CREATE TABLE #conn_stats(
                        id int primary key,
                        before_acquire_calls int default 0,
                        after_release_calls int default 0 
                    );
                    INSERT INTO #conn_stats(id) VALUES ({});
                    "#,
                    // Until we have generalized bind parameters
                    id
                );

                conn.execute(&statement[..]).await?;
                Ok(())
            })
        })
        .before_acquire(|conn, meta| {
            // `age` and `idle_for` should both be nonzero
            assert_ne!(meta.age, Duration::ZERO);
            assert_ne!(meta.idle_for, Duration::ZERO);

            Box::pin(async move {
                // MSSQL doesn't support UPDATE ... RETURNING either
                sqlx_oldapi::query(
                    r#"
                        UPDATE #conn_stats 
                        SET before_acquire_calls = before_acquire_calls + 1
                    "#,
                )
                .execute(&mut *conn)
                .await?;

                let stats: ConnStats = sqlx_oldapi::query_as("SELECT * FROM #conn_stats")
                    .fetch_one(conn)
                    .await?;

                // For even IDs, cap by the number of before_acquire calls.
                // Ignore the check for odd IDs.
                Ok((stats.id & 1) == 1 || stats.before_acquire_calls < 3)
            })
        })
        .after_release(|conn, meta| {
            // `age` should be nonzero but `idle_for` should be zero.
            assert_ne!(meta.age, Duration::ZERO);
            assert_eq!(meta.idle_for, Duration::ZERO);

            Box::pin(async move {
                sqlx_oldapi::query(
                    r#"
                        UPDATE #conn_stats 
                        SET after_release_calls = after_release_calls + 1
                    "#,
                )
                .execute(&mut *conn)
                .await?;

                let stats: ConnStats = sqlx_oldapi::query_as("SELECT * FROM #conn_stats")
                    .fetch_one(conn)
                    .await?;

                // For odd IDs, cap by the number of before_release calls.
                // Ignore the check for even IDs.
                Ok((stats.id & 1) == 0 || stats.after_release_calls < 4)
            })
        })
        // Don't establish a connection yet.
        .connect_lazy(&std::env::var("DATABASE_URL")?)?;

    // Expected pattern of (id, before_acquire_calls, after_release_calls)
    let pattern = [
        // The connection pool starts empty.
        (0, 0, 0),
        (0, 1, 1),
        (0, 2, 2),
        (1, 0, 0),
        (1, 1, 1),
        (1, 2, 2),
        // We should expect one more `acquire` because the ID is odd
        (1, 3, 3),
        (2, 0, 0),
        (2, 1, 1),
        (2, 2, 2),
        (3, 0, 0),
    ];

    for (id, before_acquire_calls, after_release_calls) in pattern {
        let conn_stats: ConnStats = sqlx_oldapi::query_as("SELECT * FROM #conn_stats")
            .fetch_one(&pool)
            .await?;

        assert_eq!(
            conn_stats,
            ConnStats {
                id,
                before_acquire_calls,
                after_release_calls
            }
        );
    }

    pool.close().await;

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_decode_tinyint_as_i16() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let row: MssqlRow = conn.fetch_one("SELECT CAST(42 AS TINYINT) as val").await?;
    let v: i16 = row.try_get("val")?;

    assert_eq!(v, 42);

    Ok(())
}

#[sqlx_macros::test]
#[cfg(feature = "macros")]
async fn it_works_with_query_builder() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    // Create a temporary table
    conn.execute(
        r#"
CREATE TABLE #qb_test (
    id INT PRIMARY KEY,
    name NVARCHAR(50)
);
        "#,
    )
    .await?;

    // Insert data using QueryBuilder
    let mut insert_builder = sqlx_oldapi::QueryBuilder::new("INSERT INTO #qb_test (id, name) ");

    #[derive(sqlx_oldapi::FromRow, Debug, PartialEq)]
    struct TestItem {
        id: i32,
        name: String,
    }

    let items_to_insert = [
        TestItem {
            id: 1,
            name: "Alice".to_string(),
        },
        TestItem {
            id: 2,
            name: "Bob".to_string(),
        },
        TestItem {
            id: 3,
            name: "Charlie".to_string(),
        },
    ];

    insert_builder.push_values(items_to_insert.iter(), |mut b, item| {
        b.push_bind(item.id).push_bind(&item.name);
    });

    let insert_query = insert_builder.build();
    eprintln!("Generated INSERT SQL: {}", insert_query.sql()); // Debug print
    conn.execute(insert_query).await?;

    // Select data using QueryBuilder
    let mut select_builder =
        sqlx_oldapi::QueryBuilder::<'_, Mssql>::new("SELECT id, name FROM #qb_test WHERE id = ");
    select_builder.push_bind(2i32);
    let select_query = select_builder.build_query_as::<TestItem>();

    let selected_item: TestItem = select_query.fetch_one(&mut conn).await?;

    assert_eq!(selected_item.id, 2);
    assert_eq!(selected_item.name, "Bob");

    // Select multiple items
    let mut select_all_builder = sqlx_oldapi::QueryBuilder::<'_, Mssql>::new(
        "SELECT id, name FROM #qb_test WHERE name LIKE ",
    );
    select_all_builder.push_bind("B%"); // Names starting with B
    let select_all_query = select_all_builder.build_query_as::<TestItem>();

    let all_b_items: Vec<TestItem> = select_all_query.fetch_all(&mut conn).await?;
    assert_eq!(all_b_items.len(), 1);
    assert_eq!(all_b_items[0].id, 2);
    assert_eq!(all_b_items[0].name, "Bob");

    conn.close().await?;
    Ok(())
}

#[sqlx_macros::test]
#[cfg(feature = "macros")]
async fn it_executes_query_from_issue_11() -> anyhow::Result<()> {
    // https://github.com/sqlpage/sqlx-oldapi/issues/11
    let mut conn = new::<Mssql>().await?;

    // Create a temporary table similar to the one in the issue
    conn.execute(
        r#"
CREATE TABLE #temp_issue_table (
    id INT PRIMARY KEY,
    name NVARCHAR(50)
);
        "#,
    )
    .await?;

    // Insert some data
    let insert_id1 = 100;
    let insert_name1 = "test_user_1";
    let insert_id2 = 200;
    let insert_name2 = "test_user_2";

    sqlx_oldapi::query("INSERT INTO #temp_issue_table (id, name) VALUES (@p1, @p2), (@p3, @p4)")
        .bind(insert_id1)
        .bind(insert_name1)
        .bind(insert_id2)
        .bind(insert_name2)
        .execute(&mut conn)
        .await?;

    // Define a struct to map the query results
    #[derive(sqlx_oldapi::FromRow, Debug, PartialEq)]
    struct TableRow {
        id: i32,
        name: String,
    }

    // Use QueryBuilder as in the issue report
    let id_to_select = insert_id1;
    let name_to_select = insert_name1;

    let mut builder = sqlx_oldapi::QueryBuilder::new("SELECT id, name FROM #temp_issue_table ");
    builder
        .push("WHERE id=")
        .push_bind(id_to_select) // Bind the specific id we want to find
        .push(" AND name=")
        .push_bind(name_to_select); // Bind the specific name

    let query = builder.build_query_as::<TableRow>();
    let sql = query.sql();
    eprintln!("Generated SQL for issue report test: {}", sql);
    assert_eq!(
        sql,
        "SELECT id, name FROM #temp_issue_table WHERE id=@p1 AND name=@p2"
    );

    let selected_row: TableRow = query.fetch_one(&mut conn).await?;

    assert_eq!(selected_row.id, id_to_select);
    assert_eq!(selected_row.name, name_to_select);

    // Test selecting a non-existent row to ensure a different id/name fails as expected
    let mut builder_no_match =
        sqlx_oldapi::QueryBuilder::new("SELECT id, name FROM #temp_issue_table ");
    builder_no_match
        .push("WHERE id=")
        .push_bind(999) // Non-existent ID
        .push(" AND name=")
        .push_bind("no_such_user");

    let query_no_match = builder_no_match.build_query_as::<TableRow>();
    let result_no_match = query_no_match.fetch_optional(&mut conn).await?;
    assert!(
        result_no_match.is_none(),
        "Query should not have found a match for non-existent data"
    );

    conn.close().await?;
    Ok(())
}
