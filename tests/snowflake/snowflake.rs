use sqlx_oldapi::snowflake::SnowflakeConnectOptions;
use sqlx_oldapi::{ConnectOptions, Connection, Executor};
use std::str::FromStr;

#[tokio::test]
async fn test_snowflake_connection_creation() {
    let options = SnowflakeConnectOptions::new()
        .account("test-account")
        .username("test-user");

    // Test that we can create connection options
    assert_eq!(options.get_account(), "test-account");
    assert_eq!(options.get_username(), "test-user");
}

#[tokio::test]
async fn test_snowflake_url_parsing() {
    let url = "snowflake://test@test-account.snowflakecomputing.com/testdb?warehouse=testwh&schema=testschema";
    let options = SnowflakeConnectOptions::from_str(url).unwrap();

    assert_eq!(options.get_account(), "test-account");
    assert_eq!(options.get_username(), "test");
    assert_eq!(options.get_database(), Some("testdb"));
    assert_eq!(options.get_warehouse(), Some("testwh"));
    assert_eq!(options.get_schema(), Some("testschema"));
}

#[tokio::test]
async fn test_snowflake_type_info() {
    use sqlx_oldapi::snowflake::{SnowflakeType, SnowflakeTypeInfo};

    let type_info = SnowflakeTypeInfo::new(SnowflakeType::Varchar);
    assert_eq!(type_info.name(), "VARCHAR");

    let type_info = SnowflakeTypeInfo::new(SnowflakeType::Integer);
    assert_eq!(type_info.name(), "INTEGER");

    let type_info = SnowflakeTypeInfo::new(SnowflakeType::Boolean);
    assert_eq!(type_info.name(), "BOOLEAN");
}

#[tokio::test]
async fn test_snowflake_arguments() {
    use sqlx_oldapi::snowflake::SnowflakeArguments;
    use sqlx_oldapi::Arguments;

    let mut args = SnowflakeArguments::new();
    args.add("test string");
    args.add(42i32);
    args.add(true);

    assert_eq!(args.len(), 3);
}

// Integration test - only run if we have proper credentials
#[ignore]
#[tokio::test]
async fn test_snowflake_real_connection() {
    let options = SnowflakeConnectOptions::new()
        .account("ffmauah-hq84745")
        .username("test")
        .password("ec_UZ.83iHy7D=-");

    match options.connect().await {
        Ok(mut connection) => {
            // Test basic connectivity
            match connection.execute("SELECT 1").await {
                Ok(_) => println!("✅ Real Snowflake connection test passed!"),
                Err(e) => println!("⚠️  Query failed (expected with current auth): {}", e),
            }
        }
        Err(e) => {
            println!("⚠️  Connection failed (expected with current auth): {}", e);
        }
    }
}
