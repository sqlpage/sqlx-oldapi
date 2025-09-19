//! Basic Snowflake connection example
//!
//! This example demonstrates the current state of Snowflake support in SQLx.
//!
//! Note: This example shows the API structure but requires proper RSA key-pair
//! authentication to work with a real Snowflake instance.

use sqlx_oldapi::snowflake::SnowflakeConnectOptions;
use sqlx_oldapi::{ConnectOptions, Connection, Executor};

#[tokio::main]
async fn main() -> Result<(), sqlx_oldapi::Error> {
    println!("🌨️  SQLx Snowflake Driver Example");
    println!("==================================");

    // Create connection options
    let options = SnowflakeConnectOptions::new()
        .account("your-account") // Replace with your Snowflake account
        .username("your-username") // Replace with your username
        .warehouse("your-warehouse") // Replace with your warehouse
        .database("your-database") // Replace with your database
        .schema("your-schema"); // Replace with your schema

    println!("📋 Configuration:");
    println!("   Account: {}", options.get_account());
    println!("   Username: {}", options.get_username());
    println!("   Warehouse: {:?}", options.get_warehouse());
    println!("   Database: {:?}", options.get_database());
    println!("   Schema: {:?}", options.get_schema());

    // Attempt connection
    println!("\n🔗 Connecting to Snowflake...");
    let mut connection = options.connect().await?;
    println!("✅ Connected successfully!");

    // Execute a simple query
    println!("\n📊 Executing query...");
    let result = connection.execute("SELECT CURRENT_VERSION()").await?;
    println!(
        "✅ Query executed! Rows affected: {}",
        result.rows_affected()
    );

    // Test connection ping
    println!("\n🏓 Testing connection ping...");
    connection.ping().await?;
    println!("✅ Ping successful!");

    // Close connection
    println!("\n🔌 Closing connection...");
    connection.close().await?;
    println!("✅ Connection closed!");

    println!("\n🎉 Example completed successfully!");

    Ok(())
}
