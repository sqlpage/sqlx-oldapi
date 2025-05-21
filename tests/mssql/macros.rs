#[allow(unused_imports)]
use sqlx_oldapi as sqlx;
use sqlx_oldapi::Mssql;
use sqlx_test::new;

#[sqlx_macros::test]
async fn test_query_simple() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let account =
        sqlx_oldapi::query!("select * from (select (1) as id, 'Herp Derpinson' as name, cast(null as char) as email, CAST(1 as bit) as deleted) accounts")
            .fetch_one(&mut conn)
            .await?;

    assert_eq!(account.id, 1);
    assert_eq!(account.name, "Herp Derpinson");
    assert_eq!(account.email, None);
    assert_eq!(account.deleted, Some(true));

    Ok(())
}

#[sqlx_macros::test]
async fn test_query_datetime() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    // Define the expected NaiveDateTime value
    let expected_naive_dt = sqlx_oldapi::types::chrono::NaiveDate::from_ymd_opt(2024, 7, 15)
        .expect("Invalid date")
        .and_hms_milli_opt(10, 30, 0, 123)
        .expect("Invalid time");

    // Use DATETIME2(3) for precise millisecond storage in MSSQL.
    // The query! macro requires a string literal.
    let record =
        sqlx_oldapi::query!("SELECT CAST('2024-07-15 10:30:00.123' AS DATETIME2(3)) as dt")
            .fetch_one(&mut conn)
            .await?;

    assert_eq!(record.dt, Some(expected_naive_dt));

    Ok(())
}

#[derive(sqlx_oldapi::FromRow, Debug, Clone, PartialEq)]
pub struct LogNotificationConfig {
    pub id: i32,
    pub config_key: String,
    pub config_value: String,
    pub created_on: Option<sqlx_oldapi::types::chrono::NaiveDateTime>,
    pub last_updated: Option<sqlx_oldapi::types::chrono::NaiveDateTime>,
}

#[sqlx_macros::test]
async fn test_query_as_from_issue() -> anyhow::Result<()> {
    let mut conn = new::<Mssql>().await?;

    let expected_created_on = sqlx_oldapi::types::chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
        .unwrap()
        .and_hms_milli_opt(10, 0, 0, 0)
        .unwrap();
    let expected_last_updated = sqlx_oldapi::types::chrono::NaiveDate::from_ymd_opt(2023, 1, 2)
        .unwrap()
        .and_hms_milli_opt(11, 30, 0, 500)
        .unwrap();

    let result = sqlx_oldapi::query_as!(
        LogNotificationConfig,
        r#"
            SELECT 
                1 AS id, 
                'test_key' AS config_key, 
                'test_value' AS config_value, 
                CAST('2023-01-01 10:00:00.000' AS DATETIME2(3)) AS created_on, 
                CAST('2023-01-02 11:30:00.500' AS DATETIME2(3)) AS last_updated
        "#
    )
    .fetch_one(&mut conn)
    .await?;

    assert_eq!(result.id, 1);
    assert_eq!(result.config_key, "test_key");
    assert_eq!(result.config_value, "test_value");
    assert_eq!(result.created_on, Some(expected_created_on));
    assert_eq!(result.last_updated, Some(expected_last_updated));

    Ok(())
}
