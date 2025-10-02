use sqlx_oldapi::any::AnyRow;
use sqlx_oldapi::{Any, Column, Connection, Decode, Executor, Row, Statement, Type};
use sqlx_test::new;

async fn get_val<T>(expr: &str) -> anyhow::Result<T>
where
    for<'r> T: Decode<'r, Any> + Type<Any> + std::marker::Unpin + std::marker::Send + 'static,
{
    let mut conn = new::<Any>().await?;
    let val = sqlx_oldapi::query(&format!("select {}", expr))
        .try_map(|row: AnyRow| row.try_get::<T, _>(0))
        .fetch_one(&mut conn)
        .await?;
    conn.close().await?;
    Ok(val)
}

#[sqlx_macros::test]
async fn it_has_all_the_types() -> anyhow::Result<()> {
    assert_eq!(6, get_val::<i32>("5 + 1").await?);
    assert_eq!(1234567890123, get_val::<i64>("1234567890123").await?);
    assert_eq!(
        "hello world".to_owned(),
        get_val::<String>("'hello world'").await?
    );
    assert_eq!(None, get_val::<Option<i32>>("NULL").await?);
    assert_eq!(1e-40, get_val::<f64>("1e-40").await?);
    assert_eq!(12.5f64, get_val::<f64>("CAST(12.5 AS DECIMAL(9,2))").await?);
    assert_eq!(
        125125.125f64,
        get_val::<f64>("CAST(125125.125 AS DECIMAL(10,3))").await?
    );
    assert_eq!(
        -1234567890.125,
        get_val::<f64>("CAST(-1234567890.125 AS DECIMAL(15,3))").await?
    );
    Ok(())
}

#[cfg(feature = "chrono")]
#[sqlx_macros::test]
async fn it_has_chrono() -> anyhow::Result<()> {
    use sqlx_oldapi::types::chrono::NaiveDate;
    let mut conn = crate::new::<sqlx_oldapi::Any>().await?;
    let dbms_name = conn.dbms_name().await.unwrap_or_default();
    let sql_date = if dbms_name.to_lowercase().contains("sqlite") {
        "'2020-01-02'"
    } else {
        "CAST('2020-01-02' AS DATE)"
    };
    let expected_date = NaiveDate::from_ymd_opt(2020, 1, 2).unwrap();
    let actual = conn.fetch_one(&*format!("SELECT {}", sql_date)).await?;
    assert_eq!(expected_date, actual.try_get::<NaiveDate, _>(0)?);
    Ok(())
}

#[cfg(feature = "chrono")]
#[sqlx_macros::test]
async fn it_has_chrono_fixed_offset() -> anyhow::Result<()> {
    use sqlx_oldapi::types::chrono::{DateTime, FixedOffset};
    assert_eq!(
        DateTime::<FixedOffset>::parse_from_rfc3339("2020-01-02T12:00:00+02:00").unwrap(),
        get_val::<DateTime<FixedOffset>>(if cfg!(feature = "sqlite") {
            "'2020-01-02 12:00:00+02:00'"
        } else if cfg!(feature = "mssql") {
            "CAST('2020-01-02 12:00:00+02:00' AS DATETIMEOFFSET)"
        } else if cfg!(feature = "postgres") {
            "'2020-01-02 12:00:00+02:00'::timestamptz"
        } else {
            eprintln!("DBMS not supported");
            return Ok(());
        })
        .await?
    );
    Ok(())
}

#[cfg(feature = "bigdecimal")]
#[sqlx_macros::test]
async fn it_has_bigdecimal() -> anyhow::Result<()> {
    use sqlx_oldapi::types::BigDecimal;
    use std::str::FromStr;
    assert_eq!(
        BigDecimal::from_str("1234567.25")?,
        get_val::<BigDecimal>("CAST('1234567.25' AS DECIMAL(9,2))").await?
    );
    Ok(())
}

#[cfg(feature = "decimal")]
#[sqlx_macros::test]
async fn it_has_decimal() -> anyhow::Result<()> {
    use sqlx_oldapi::types::Decimal;
    use std::str::FromStr;
    assert_eq!(
        Decimal::from_str("1234567.25")?,
        get_val::<Decimal>("CAST('1234567.25' AS DECIMAL(9,2))").await?
    );
    Ok(())
}

#[cfg(feature = "json")]
#[sqlx_macros::test]
async fn it_has_json() -> anyhow::Result<()> {
    use serde_json::json;

    let databases_without_json = ["sqlite", "microsoft sql server", "snowflake"];
    let mut conn = crate::new::<sqlx_oldapi::Any>().await?;
    let dbms_name = conn.dbms_name().await.unwrap_or_default();
    let json_sql = if databases_without_json.contains(&dbms_name.to_lowercase().as_str()) {
        "select '{\"foo\": \"bar\"}'"
    } else {
        "select CAST('{\"foo\": \"bar\"}' AS JSON)"
    };

    let expected_json = json!({"foo": "bar"});
    let actual = conn
        .fetch_one(json_sql)
        .await?
        .try_get::<serde_json::Value, _>(0)?;
    assert_eq!(expected_json, actual, "Json value for {}", json_sql);
    Ok(())
}

#[cfg(feature = "uuid")]
#[sqlx_macros::test]
async fn it_has_uuid() -> anyhow::Result<()> {
    use sqlx_oldapi::types::Uuid;
    let mut conn = new::<Any>().await?;
    let dbms_name = conn.dbms_name().await?.to_lowercase();
    let expected_uuid = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000")?;

    let sql = if dbms_name.contains("mssql") || dbms_name.contains("sql server") {
        "select CONVERT(uniqueidentifier, '123e4567-e89b-12d3-a456-426614174000')"
    } else if dbms_name.contains("postgres") {
        "select '123e4567-e89b-12d3-a456-426614174000'::uuid"
    } else {
        "select x'123e4567e89b12d3a456426614174000'"
    };
    let actual = conn.fetch_one(sql).await?.try_get::<Uuid, _>(0)?;
    assert_eq!(expected_uuid, actual, "UUID value for {}", sql);
    Ok(())
}

#[sqlx_macros::test]
async fn it_pings() -> anyhow::Result<()> {
    let mut conn = new::<Any>().await?;

    conn.ping().await?;

    Ok(())
}

#[sqlx_macros::test]
async fn it_executes_one_statement_with_pool() -> anyhow::Result<()> {
    let pool = sqlx_test::pool::<Any>().await?;

    let rows = pool.fetch_all("SELECT 1").await?;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].try_get::<i32, _>(0)?, 1);

    Ok(())
}

/// ODBC does not support multiple statements in a single query
#[cfg(not(feature = "odbc"))]
#[sqlx_macros::test]
async fn it_executes_two_statements_with_pool() -> anyhow::Result<()> {
    let pool = sqlx_test::pool::<Any>().await?;

    let rows = pool.fetch_all("SELECT 1; SElECT 2").await?;

    assert_eq!(rows.len(), 2);

    Ok(())
}

#[sqlx_macros::test]
async fn it_does_not_stop_stream_after_decoding_error() -> anyhow::Result<()> {
    use futures::stream::StreamExt;
    // see https://github.com/launchbadge/sqlx/issues/1884
    let pool = sqlx_test::pool::<Any>().await?;

    #[derive(Debug, PartialEq)]
    struct MyType;
    impl<'a> sqlx_oldapi::FromRow<'a, AnyRow> for MyType {
        fn from_row(row: &'a AnyRow) -> sqlx_oldapi::Result<Self> {
            let n = row.try_get::<i32, _>(0)?;
            if n == 1 {
                Err(sqlx_oldapi::Error::RowNotFound)
            } else {
                Ok(MyType)
            }
        }
    }

    let rows = sqlx_oldapi::query_as("SELECT 0 UNION ALL SELECT 1 UNION ALL SELECT 2")
        .fetch(&pool)
        .map(|r| r.ok())
        .collect::<Vec<_>>()
        .await;

    assert_eq!(rows, vec![Some(MyType), None, Some(MyType)]);
    Ok(())
}

#[sqlx_macros::test]
async fn it_gets_by_name() -> anyhow::Result<()> {
    let mut conn = new::<Any>().await?;

    let row = conn.fetch_one("SELECT 1 as _1").await?;
    let val: i32 = row.get("_1");

    assert_eq!(val, 1);

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fail_and_recover() -> anyhow::Result<()> {
    let mut conn = new::<Any>().await?;

    for i in 0..10 {
        // make a query that will fail
        let res = conn
            .execute("INSERT INTO not_found (column) VALUES (10)")
            .await;

        assert!(res.is_err());

        // now try and use the connection
        let val: i32 = conn
            .fetch_one(&*format!("SELECT {}", i))
            .await?
            .get_unchecked(0);

        assert_eq!(val, i);
    }

    Ok(())
}

#[sqlx_macros::test]
async fn it_can_fail_and_recover_with_pool() -> anyhow::Result<()> {
    let pool = sqlx_test::pool::<Any>().await?;

    for i in 0..10 {
        // make a query that will fail
        let res = pool
            .execute("INSERT INTO not_found (column) VALUES (10)")
            .await;

        assert!(res.is_err());

        // now try and use the connection
        let val: i32 = pool
            .fetch_one(&*format!("SELECT {}", i))
            .await?
            .get_unchecked(0);

        assert_eq!(val, i);
    }

    Ok(())
}

#[sqlx_macros::test]
async fn it_has_unsigned_integers() -> anyhow::Result<()> {
    let max_value = 9223372036854775807;
    let expr = if cfg!(feature = "mysql") {
        format!("CAST({} AS UNSIGNED)", max_value)
    } else {
        max_value.to_string()
    };
    assert_eq!(max_value, get_val::<u64>(&expr).await?);
    Ok(())
}

#[sqlx_macros::test]
async fn it_reports_rows_affected() -> anyhow::Result<()> {
    let mut conn = new::<Any>().await?;

    let dbms = conn.dbms_name().await.unwrap_or_default().to_lowercase();
    let (create_sql, insert_sql, delete_sql) =
        if dbms.contains("mssql") || dbms.contains("sql server") {
            (
                "CREATE TABLE #temp_rows_affected (id INT PRIMARY KEY)",
                "INSERT INTO #temp_rows_affected (id) VALUES (1)",
                "DELETE FROM #temp_rows_affected WHERE id = 1",
            )
        } else {
            (
                "CREATE TEMPORARY TABLE temp_rows_affected (id INTEGER PRIMARY KEY)",
                "INSERT INTO temp_rows_affected (id) VALUES (1)",
                "DELETE FROM temp_rows_affected WHERE id = 1",
            )
        };

    conn.execute(create_sql).await?;

    let insert_done = conn.execute(insert_sql).await?;
    assert_eq!(insert_done.rows_affected(), 1);

    let delete_done = conn.execute(delete_sql).await?;
    assert_eq!(delete_done.rows_affected(), 1);

    Ok(())
}

#[sqlx_macros::test]
async fn it_prepares_statements() -> anyhow::Result<()> {
    let mut conn = new::<Any>().await?;

    let stmt = conn.prepare("SELECT 42 AS answer").await?;

    assert_eq!(stmt.columns().len(), 1);
    assert_eq!(stmt.columns()[0].name(), "answer");

    let row = stmt.query().fetch_one(&mut conn).await?;
    let answer: i32 = row.try_get("answer")?;
    assert_eq!(answer, 42);

    Ok(())
}

#[sqlx_macros::test]
async fn it_fails_to_prepare_invalid_statements() -> anyhow::Result<()> {
    let mut conn = new::<Any>().await?;

    let result = conn.prepare("SELECT * FROM table_does_not_exist").await;

    assert!(
        result.is_err(),
        "Expected error when preparing statement for non-existent table"
    );

    Ok(())
}
