#![allow(clippy::approx_constant)]
use sqlx_oldapi::any::{AnyConnection, AnyRow};
use sqlx_oldapi::{Connection, Executor, Row};

#[cfg(feature = "odbc")]
async fn odbc_conn() -> anyhow::Result<AnyConnection> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for ODBC tests");

    // Ensure the URL starts with "odbc:"
    let url = if !url.starts_with("odbc:") {
        format!("odbc:{}", url)
    } else {
        url
    };

    AnyConnection::connect(&url).await.map_err(Into::into)
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_connects_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    // Simple ping test
    conn.ping().await?;

    // Close the connection
    conn.close().await?;

    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_executes_simple_query_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    let row: AnyRow = sqlx_oldapi::query("SELECT 1 AS value")
        .fetch_one(&mut conn)
        .await?;

    let value: i32 = row.try_get("value")?;
    assert_eq!(value, 1);

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_handles_parameters_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    let row: AnyRow = sqlx_oldapi::query("SELECT ? AS value")
        .bind(42i32)
        .fetch_one(&mut conn)
        .await?;

    let value: i32 = row.try_get("value")?;
    assert_eq!(value, 42);

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_handles_multiple_types_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    // Test integers
    let row: AnyRow = sqlx_oldapi::query("SELECT 123 AS int_val")
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(row.try_get::<i32, _>("int_val")?, 123);

    // Test strings
    let row: AnyRow = sqlx_oldapi::query("SELECT 'hello' AS str_val")
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(row.try_get::<String, _>("str_val")?, "hello");

    // Test floats
    let row: AnyRow = sqlx_oldapi::query("SELECT 3.14 AS float_val")
        .fetch_one(&mut conn)
        .await?;
    let float_val: f64 = row.try_get("float_val")?;
    assert!((float_val - 3.14).abs() < 0.001);

    // Test NULL
    let row: AnyRow = sqlx_oldapi::query("SELECT NULL AS null_val")
        .fetch_one(&mut conn)
        .await?;
    let null_val: Option<i32> = row.try_get("null_val")?;
    assert!(null_val.is_none());

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_handles_multiple_rows_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    let rows: Vec<AnyRow> =
        sqlx_oldapi::query("SELECT 1 AS value UNION ALL SELECT 2 UNION ALL SELECT 3")
            .fetch_all(&mut conn)
            .await?;

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].try_get::<i32, _>("value")?, 1);
    assert_eq!(rows[1].try_get::<i32, _>("value")?, 2);
    assert_eq!(rows[2].try_get::<i32, _>("value")?, 3);

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_handles_optional_rows_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    // Query that returns a row
    let row: Option<AnyRow> = sqlx_oldapi::query("SELECT 1 AS value")
        .fetch_optional(&mut conn)
        .await?;
    assert!(row.is_some());
    assert_eq!(row.unwrap().try_get::<i32, _>("value")?, 1);

    // Query that returns no rows (using a condition that's always false)
    let row: Option<AnyRow> = sqlx_oldapi::query("SELECT 1 AS value WHERE 1 = 0")
        .fetch_optional(&mut conn)
        .await?;
    assert!(row.is_none());

    conn.close().await?;
    Ok(())
}

#[cfg(all(feature = "odbc", feature = "chrono"))]
#[sqlx_macros::test]
async fn it_handles_chrono_types_via_any_odbc() -> anyhow::Result<()> {
    use sqlx_oldapi::types::chrono::{NaiveDate, NaiveDateTime};

    let mut conn = odbc_conn().await?;

    // Test DATE
    let row: AnyRow = sqlx_oldapi::query("SELECT CAST('2023-05-15' AS DATE) AS date_val")
        .fetch_one(&mut conn)
        .await?;
    let date_val: NaiveDate = row.try_get("date_val")?;
    assert_eq!(date_val, NaiveDate::from_ymd_opt(2023, 5, 15).unwrap());

    // Test TIMESTAMP
    let row: AnyRow =
        sqlx_oldapi::query("SELECT CAST('2023-05-15 14:30:00' AS TIMESTAMP) AS ts_val")
            .fetch_one(&mut conn)
            .await?;
    let ts_val: NaiveDateTime = row.try_get("ts_val")?;
    assert_eq!(
        ts_val,
        NaiveDate::from_ymd_opt(2023, 5, 15)
            .unwrap()
            .and_hms_opt(14, 30, 0)
            .unwrap()
    );

    conn.close().await?;
    Ok(())
}

#[cfg(all(feature = "odbc", feature = "decimal"))]
#[sqlx_macros::test]
async fn it_handles_decimal_via_any_odbc() -> anyhow::Result<()> {
    use sqlx_oldapi::types::Decimal;
    use std::str::FromStr;

    let mut conn = odbc_conn().await?;

    let row: AnyRow = sqlx_oldapi::query("SELECT CAST(12345.67 AS DECIMAL(10,2)) AS dec_val")
        .fetch_one(&mut conn)
        .await?;

    let dec_val: Decimal = row.try_get("dec_val")?;
    assert_eq!(dec_val, Decimal::from_str("12345.67")?);

    conn.close().await?;
    Ok(())
}

#[cfg(all(feature = "odbc", feature = "uuid"))]
#[sqlx_macros::test]
async fn it_handles_uuid_via_any_odbc() -> anyhow::Result<()> {
    use sqlx_oldapi::types::Uuid;

    let mut conn = odbc_conn().await?;

    // PostgreSQL syntax for UUID
    let query = if std::env::var("DATABASE_URL")
        .unwrap_or_default()
        .contains("postgres")
    {
        "SELECT '550e8400-e29b-41d4-a716-446655440000'::uuid AS uuid_val"
    } else {
        // Generic syntax - might need adjustment for other databases
        "SELECT CAST('550e8400-e29b-41d4-a716-446655440000' AS VARCHAR(36)) AS uuid_val"
    };

    let row: AnyRow = sqlx_oldapi::query(query).fetch_one(&mut conn).await?;

    let uuid_val: Uuid = row.try_get("uuid_val")?;
    assert_eq!(
        uuid_val,
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")?
    );

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_handles_prepared_statements_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    // Prepare a statement
    let _stmt = conn.prepare("SELECT ? AS a, ? AS b").await?;

    // Execute it multiple times with different parameters
    for i in 1..=3 {
        let row: AnyRow = sqlx_oldapi::query("SELECT ? AS a, ? AS b")
            .bind(i)
            .bind(i * 10)
            .fetch_one(&mut conn)
            .await?;

        assert_eq!(row.try_get::<i32, _>("a")?, i);
        assert_eq!(row.try_get::<i32, _>("b")?, i * 10);
    }

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_handles_transactions_via_any_odbc() -> anyhow::Result<()> {
    use sqlx_oldapi::Connection;

    let mut conn = odbc_conn().await?;

    // Start a transaction
    let mut tx = conn.begin().await?;

    // Execute a query within the transaction
    let row: AnyRow = sqlx_oldapi::query("SELECT 42 AS value")
        .fetch_one(&mut *tx)
        .await?;
    assert_eq!(row.try_get::<i32, _>("value")?, 42);

    // Commit the transaction
    tx.commit().await?;

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_handles_errors_gracefully_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    // Try to execute an invalid query
    let result = sqlx_oldapi::query("SELECT * FROM nonexistent_table")
        .fetch_one(&mut conn)
        .await;

    assert!(result.is_err());

    // The connection should still be usable
    let row: AnyRow = sqlx_oldapi::query("SELECT 1 AS value")
        .fetch_one(&mut conn)
        .await?;
    assert_eq!(row.try_get::<i32, _>("value")?, 1);

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_matches_any_kind_odbc() -> anyhow::Result<()> {
    use sqlx_oldapi::any::AnyKind;

    let conn = odbc_conn().await?;

    // Check that the connection kind is ODBC
    assert_eq!(conn.kind(), AnyKind::Odbc);

    conn.close().await?;
    Ok(())
}
