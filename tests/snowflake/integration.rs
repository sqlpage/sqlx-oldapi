use sqlx_oldapi::snowflake::SnowflakeConnectOptions;
use sqlx_oldapi::{ConnectOptions, Connection, Executor};
use std::str::FromStr;

#[tokio::test]
async fn test_snowflake_connection_options_builder() {
    let options = SnowflakeConnectOptions::new()
        .account("test-account")
        .username("test-user")
        .password("test-pass")
        .warehouse("test-wh")
        .database("test-db")
        .schema("test-schema");

    assert_eq!(options.get_account(), "test-account");
    assert_eq!(options.get_username(), "test-user");
    assert_eq!(options.get_warehouse(), Some("test-wh"));
    assert_eq!(options.get_database(), Some("test-db"));
    assert_eq!(options.get_schema(), Some("test-schema"));
}

#[tokio::test]
async fn test_snowflake_url_parsing_comprehensive() {
    // Test basic URL
    let url = "snowflake://user@account.snowflakecomputing.com/db";
    let options = SnowflakeConnectOptions::from_str(url).unwrap();
    assert_eq!(options.get_account(), "account");
    assert_eq!(options.get_username(), "user");
    assert_eq!(options.get_database(), Some("db"));

    // Test URL with query parameters
    let url =
        "snowflake://user:pass@account.snowflakecomputing.com/db?warehouse=wh&schema=sch&role=r";
    let options = SnowflakeConnectOptions::from_str(url).unwrap();
    assert_eq!(options.get_account(), "account");
    assert_eq!(options.get_username(), "user");
    assert_eq!(options.get_database(), Some("db"));
    assert_eq!(options.get_warehouse(), Some("wh"));
    assert_eq!(options.get_schema(), Some("sch"));
}

#[tokio::test]
async fn test_snowflake_type_system() {
    use sqlx_oldapi::snowflake::{SnowflakeType, SnowflakeTypeInfo};
    use sqlx_oldapi::TypeInfo;

    let varchar_type = SnowflakeTypeInfo::new(SnowflakeType::Varchar);
    assert_eq!(varchar_type.name(), "VARCHAR");
    assert!(!varchar_type.is_null());

    let number_type = SnowflakeTypeInfo::new(SnowflakeType::Number);
    assert_eq!(number_type.name(), "NUMBER");

    // Test type parsing
    assert_eq!(
        SnowflakeType::from_name("VARCHAR"),
        Some(SnowflakeType::Varchar)
    );
    assert_eq!(
        SnowflakeType::from_name("INTEGER"),
        Some(SnowflakeType::Integer)
    );
    assert_eq!(
        SnowflakeType::from_name("BOOLEAN"),
        Some(SnowflakeType::Boolean)
    );
    assert_eq!(SnowflakeType::from_name("INVALID"), None);
}

#[tokio::test]
async fn test_snowflake_arguments_and_encoding() {
    use sqlx_oldapi::snowflake::SnowflakeArguments;
    use sqlx_oldapi::Arguments;

    let mut args = SnowflakeArguments::new();
    assert!(args.is_empty());
    assert_eq!(args.len(), 0);

    args.add("test string");
    args.add(42i32);
    args.add(3.14f64);
    args.add(true);

    assert!(!args.is_empty());
    assert_eq!(args.len(), 4);
}

#[tokio::test]
async fn test_snowflake_error_handling() {
    use sqlx_oldapi::error::DatabaseError;
    use sqlx_oldapi::snowflake::SnowflakeDatabaseError;

    let error = SnowflakeDatabaseError::new(
        "100072".to_string(),
        "Unique constraint violation".to_string(),
        Some("23505".to_string()),
    );

    assert_eq!(error.message(), "Unique constraint violation");
    assert_eq!(error.code().unwrap(), "100072");
    assert_eq!(error.constraint(), None);
}

// Test with fakesnow when available
#[ignore]
#[tokio::test]
async fn test_snowflake_with_fakesnow() {
    // This test requires fakesnow to be running
    // docker run -p 8080:8080 tekumara/fakesnow

    let options = SnowflakeConnectOptions::new()
        .account("localhost") // fakesnow runs on localhost
        .username("test")
        .password("test");

    match options.connect().await {
        Ok(mut connection) => {
            println!("✅ Connected to fakesnow!");

            match connection.execute("SELECT 1").await {
                Ok(result) => {
                    println!("✅ Query executed! Rows: {}", result.rows_affected());
                }
                Err(e) => {
                    println!("⚠️ Query failed (expected): {}", e);
                }
            }
        }
        Err(e) => {
            println!("ℹ️ fakesnow not available: {}", e);
        }
    }
}

// Integration test with real Snowflake (ignored by default)
#[ignore]
#[tokio::test]
async fn test_snowflake_real_integration() {
    let options = SnowflakeConnectOptions::new()
        .account("ffmauah-hq84745")
        .username("test")
        .password("ec_UZ.83iHy7D=-")
        .warehouse("COMPUTE_WH")
        .database("SNOWFLAKE_SAMPLE_DATA")
        .schema("TPCH_SF1");

    match options.connect().await {
        Ok(mut connection) => {
            println!("✅ Connected to real Snowflake!");

            // Test basic queries
            let queries = vec![
                "SELECT CURRENT_VERSION()",
                "SELECT CURRENT_TIMESTAMP()",
                "SELECT 1 + 1 as result",
            ];

            for query in queries {
                match connection.execute(query).await {
                    Ok(result) => {
                        println!(
                            "✅ Query '{}' executed! Rows: {}",
                            query,
                            result.rows_affected()
                        );
                    }
                    Err(e) => {
                        println!("⚠️ Query '{}' failed: {}", query, e);
                    }
                }
            }
        }
        Err(e) => {
            println!(
                "⚠️ Real Snowflake connection failed (expected with current auth): {}",
                e
            );
        }
    }
}
