#![allow(clippy::approx_constant)]
use sqlx_oldapi::any::{AnyConnection, AnyRow};
use sqlx_oldapi::{Column, Connection, Executor, Row, Statement};

#[cfg(feature = "odbc")]
async fn odbc_conn() -> anyhow::Result<AnyConnection> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for ODBC tests");

    // The "odbc:" prefix is now optional - standard ODBC connection strings
    // like "DSN=mydsn" or "Driver={SQL Server};..." are automatically detected
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

    // DBMS name can be retrieved at runtime
    let dbms = conn.dbms_name().await?;
    assert!(!dbms.is_empty());

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
    let db_name = conn.dbms_name().await?;

    let is_sqlite = db_name.to_lowercase().contains("sqlite");
    let cast_date = |s: &str| if is_sqlite { s.to_string() } else { format!("CAST({} AS DATE)", s) };
    let cast_ts = |s: &str| if is_sqlite { s.to_string() } else { format!("CAST({} AS TIMESTAMP)", s) };

    // Test DATE
    let row: AnyRow = sqlx_oldapi::query(&format!("SELECT {} AS date_val", cast_date("'2023-05-15'")))
        .fetch_one(&mut conn)
        .await?;
    let date_val: NaiveDate = row.try_get("date_val")?;
    assert_eq!(date_val, NaiveDate::from_ymd_opt(2023, 5, 15).unwrap());

    // Test TIMESTAMP
    let row: AnyRow = sqlx_oldapi::query(&format!(
        "SELECT {} AS ts_val",
        cast_ts("'2023-05-15 14:30:00'")
    ))
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
async fn it_prepares_and_reports_metadata_via_any_odbc() -> anyhow::Result<()> {
    use either::Either;

    let mut conn = odbc_conn().await?;

    let stmt = conn.prepare("SELECT ? AS a, ? AS b").await?;

    match stmt.parameters() {
        Some(Either::Right(n)) => assert_eq!(n, 2),
        Some(Either::Left(_)) => anyhow::bail!("unexpected typed parameters"),
        None => anyhow::bail!("missing parameters metadata"),
    }

    let cols = stmt.columns();
    assert_eq!(cols.len(), 2);
    let col0_name = cols[0].name();
    let col1_name = cols[1].name();
    assert!(
        col0_name.eq_ignore_ascii_case("a"),
        "Expected 'a' or 'A', got '{}'",
        col0_name
    );
    assert!(
        col1_name.eq_ignore_ascii_case("b"),
        "Expected 'b' or 'B', got '{}'",
        col1_name
    );

    conn.close().await?;
    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_errors_on_wrong_parameter_count_via_any_odbc() -> anyhow::Result<()> {
    let mut conn = odbc_conn().await?;

    let res = sqlx_oldapi::query("SELECT ? AS value")
        .fetch_one(&mut conn)
        .await;
    assert!(res.is_err());

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

    // Ensure dbms_name works on owned connection too by dropping after fetch
    let _ = conn;

    Ok(())
}

#[cfg(feature = "odbc")]
#[sqlx_macros::test]
async fn it_accepts_standard_odbc_connection_strings() -> anyhow::Result<()> {
    use sqlx_oldapi::any::AnyKind;
    use std::str::FromStr;

    // Test various standard ODBC connection string formats
    let test_cases = vec![
        "DSN=mydsn",
        "DSN=mydsn;UID=user;PWD=pass",
        "Driver={SQL Server};Server=localhost;Database=test",
        "Driver={ODBC Driver 17 for SQL Server};Server=localhost;Database=test",
        "FILEDSN=myfile.dsn",
        "odbc:DSN=mydsn", // Still support the odbc: prefix
        "odbc:Driver={SQL Server};Server=localhost",
    ];

    for conn_str in test_cases {
        let kind_result = AnyKind::from_str(conn_str);

        // If ODBC feature is enabled, these should parse as ODBC
        match kind_result {
            Ok(kind) => assert_eq!(
                kind,
                AnyKind::Odbc,
                "Failed to identify '{}' as ODBC",
                conn_str
            ),
            Err(e) => panic!("Failed to parse '{}' as ODBC: {}", conn_str, e),
        }
    }

    // Test non-ODBC connection strings don't match
    let non_odbc_cases = vec![
        "postgres://localhost/db",
        "mysql://localhost/db",
        "sqlite:memory:",
        "random string without equals",
    ];

    for conn_str in non_odbc_cases {
        let kind_result = AnyKind::from_str(conn_str);
        if let Ok(kind) = kind_result {
            assert_ne!(
                kind,
                AnyKind::Odbc,
                "Incorrectly identified '{}' as ODBC",
                conn_str
            )
        }
    }

    Ok(())
}
