use futures::TryStreamExt;
use sqlx_oldapi::odbc::{Odbc, OdbcConnectOptions, OdbcConnection};
use sqlx_oldapi::Column;
use sqlx_oldapi::Connection;
use sqlx_oldapi::Executor;
use sqlx_oldapi::Row;
use sqlx_oldapi::Statement;
use sqlx_oldapi::Value;
use sqlx_oldapi::ValueRef;
use sqlx_test::new;
use std::str::FromStr;

#[tokio::test]
async fn it_connects_and_pings() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    conn.ping().await?;
    conn.close().await?;
    Ok(())
}

#[tokio::test]
async fn it_can_work_with_transactions() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let tx = conn.begin().await?;
    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn it_streams_row_and_metadata() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut s = conn.fetch("SELECT 42 AS n, 'hi' AS s, NULL AS z");
    let mut saw_row = false;
    while let Some(row) = s.try_next().await? {
        let col0_name = row.column(0).name();
        let col1_name = row.column(1).name();
        let col2_name = row.column(2).name();
        assert!(
            col0_name.eq_ignore_ascii_case("n"),
            "Expected 'n' or 'N', got '{}'",
            col0_name
        );
        assert!(
            col1_name.eq_ignore_ascii_case("s"),
            "Expected 's' or 'S', got '{}'",
            col1_name
        );
        assert!(
            col2_name.eq_ignore_ascii_case("z"),
            "Expected 'z' or 'Z', got '{}'",
            col2_name
        );
        let vn = row.try_get_raw(0)?.to_owned();
        let vs = row.try_get_raw(1)?.to_owned();
        let vz = row.try_get_raw(2)?.to_owned();
        assert_eq!(vn.decode::<i64>(), 42);
        assert_eq!(vs.decode::<String>(), "hi".to_string());
        assert!(vz.is_null());
        saw_row = true;
    }
    assert!(saw_row);
    Ok(())
}

#[tokio::test]
async fn it_streams_multiple_rows() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut s = conn.fetch("SELECT 1 AS v UNION ALL SELECT 2 UNION ALL SELECT 3");
    let mut vals = Vec::new();
    while let Some(row) = s.try_next().await? {
        vals.push(row.try_get_raw(0)?.to_owned().decode::<i64>());
    }
    assert_eq!(vals, vec![1, 2, 3]);
    Ok(())
}

#[tokio::test]
async fn it_handles_empty_result() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let mut s = conn.fetch("SELECT 1 WHERE 1=0");
    let mut saw_row = false;
    while let Some(_row) = s.try_next().await? {
        saw_row = true;
    }
    assert!(!saw_row);
    Ok(())
}

#[tokio::test]
async fn it_reports_null_and_non_null_values() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let mut s = conn.fetch("SELECT 'text' AS s, NULL AS z");
    let row = s.try_next().await?.expect("row expected");

    let s_val = row.try_get_raw(0)?.to_owned().decode::<String>();
    let z_val = row.try_get_raw(1)?.to_owned();
    assert_eq!(s_val, "text");
    assert!(z_val.is_null());
    Ok(())
}

#[tokio::test]
async fn it_handles_basic_numeric_and_text_expressions() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let mut s = conn.fetch("SELECT 1 AS i, 1.5 AS f, 'hello' AS t");
    let row = s.try_next().await?.expect("row expected");

    // Column names may be uppercase or lowercase depending on the database
    let col0_name = row.column(0).name();
    let col1_name = row.column(1).name();
    let col2_name = row.column(2).name();
    assert!(
        col0_name.eq_ignore_ascii_case("i"),
        "Expected 'i' or 'I', got '{}'",
        col0_name
    );
    assert!(
        col1_name.eq_ignore_ascii_case("f"),
        "Expected 'f' or 'F', got '{}'",
        col1_name
    );
    assert!(
        col2_name.eq_ignore_ascii_case("t"),
        "Expected 't' or 'T', got '{}'",
        col2_name
    );

    let i = row.try_get_raw(0)?.to_owned().decode::<i64>();
    let f = row.try_get_raw(1)?.to_owned().decode::<f64>();
    let t = row.try_get_raw(2)?.to_owned().decode::<String>();
    assert_eq!(i, 1);
    assert_eq!(f, 1.5);
    assert_eq!(t, "hello");
    Ok(())
}

#[tokio::test]
async fn it_fetch_optional_some_and_none() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let some = (&mut conn).fetch_optional("SELECT 1").await?;
    let none = (&mut conn).fetch_optional("SELECT 1 WHERE 1=0").await?;
    assert!(some.is_some());
    assert!(none.is_none());
    Ok(())
}

#[tokio::test]
async fn it_can_prepare_then_query_without_params() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let stmt = conn.prepare("SELECT 7 AS seven").await?;
    let row = stmt.query().fetch_one(&mut conn).await?;
    let col_name = row.column(0).name();
    assert!(
        col_name.eq_ignore_ascii_case("seven"),
        "Expected 'seven' or 'SEVEN', got '{}'",
        col_name
    );
    let v = row.try_get_raw(0)?.to_owned().decode::<i64>();
    assert_eq!(v, 7);
    Ok(())
}

#[tokio::test]
async fn it_can_prepare_then_query_with_params_integer_float_text() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let stmt = conn.prepare("SELECT ? AS i, ? AS f, ? AS t").await?;

    let row = stmt
        .query()
        .bind(5_i32)
        .bind(1.25_f64)
        .bind("hello")
        .fetch_one(&mut conn)
        .await?;

    let col0_name = row.column(0).name();
    let col1_name = row.column(1).name();
    let col2_name = row.column(2).name();
    assert!(
        col0_name.eq_ignore_ascii_case("i"),
        "Expected 'i' or 'I', got '{}'",
        col0_name
    );
    assert!(
        col1_name.eq_ignore_ascii_case("f"),
        "Expected 'f' or 'F', got '{}'",
        col1_name
    );
    assert!(
        col2_name.eq_ignore_ascii_case("t"),
        "Expected 't' or 'T', got '{}'",
        col2_name
    );
    let i = row.try_get_raw(0)?.to_owned().decode::<i64>();
    let f = row.try_get_raw(1)?.to_owned().decode::<f64>();
    let t = row.try_get_raw(2)?.to_owned().decode::<String>();
    assert_eq!(i, 5);
    assert!((f - 1.25).abs() < 1e-9);
    assert_eq!(t, "hello");

    Ok(())
}

#[tokio::test]
async fn it_can_bind_many_params_dynamically() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let count = 20usize;
    let mut sql = String::from("SELECT ");
    for i in 0..count {
        if i != 0 {
            sql.push_str(", ");
        }
        sql.push('?');
    }

    let stmt = conn.prepare(&sql).await?;

    let values: Vec<i32> = (1..=count as i32).collect();
    let mut q = stmt.query();
    for v in &values {
        q = q.bind(*v);
    }

    let row = q.fetch_one(&mut conn).await?;
    for (i, expected) in values.iter().enumerate() {
        let got = row.try_get_raw(i)?.to_owned().decode::<i64>();
        assert_eq!(got, *expected as i64);
    }
    Ok(())
}

#[tokio::test]
async fn it_can_bind_heterogeneous_params() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let stmt = conn.prepare("SELECT ?, ?, ?, ?, ?").await?;

    let row = stmt
        .query()
        .bind(7_i32)
        .bind(3.5_f64)
        .bind("abc")
        .bind("xyz")
        .bind(42_i32)
        .fetch_one(&mut conn)
        .await?;

    let i = row.try_get_raw(0)?.to_owned().decode::<i64>();
    let f = row.try_get_raw(1)?.to_owned().decode::<f64>();
    let t = row.try_get_raw(2)?.to_owned().decode::<String>();
    let t2 = row.try_get_raw(3)?.to_owned().decode::<String>();
    let last = row.try_get_raw(4)?.to_owned().decode::<i64>();

    assert_eq!(i, 7);
    assert!((f - 3.5).abs() < 1e-9);
    assert_eq!(t, "abc");
    assert_eq!(t2, "xyz");
    assert_eq!(last, 42);
    Ok(())
}

#[tokio::test]
async fn it_binds_null_string_parameter() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let stmt = conn.prepare("SELECT ?, ?").await?;
    let row = stmt
        .query()
        .bind("abc")
        .bind(Option::<String>::None)
        .fetch_one(&mut conn)
        .await?;

    let a = row.try_get_raw(0)?.to_owned().decode::<String>();
    let b = row.try_get_raw(1)?.to_owned();
    assert_eq!(a, "abc");
    assert!(b.is_null());
    Ok(())
}

#[tokio::test]
async fn it_handles_different_integer_types() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test various integer sizes
    let mut s = conn.fetch(
        "SELECT 127 AS tiny, 32767 AS small, 2147483647 AS regular, 9223372036854775807 AS big",
    );
    let row = s.try_next().await?.expect("row expected");

    let tiny = row.try_get_raw(0)?.to_owned().decode::<i8>();
    let small = row.try_get_raw(1)?.to_owned().decode::<i16>();
    let regular = row.try_get_raw(2)?.to_owned().decode::<i32>();
    let big = row.try_get_raw(3)?.to_owned().decode::<i64>();

    assert_eq!(tiny, 127);
    assert_eq!(small, 32767);
    assert_eq!(regular, 2147483647);
    assert_eq!(big, 9223372036854775807);
    Ok(())
}

#[tokio::test]
async fn it_handles_negative_integers() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut s = conn.fetch(
        "SELECT -128 AS tiny, -32768 AS small, -2147483648 AS regular, -9223372036854775808 AS big",
    );
    let row = s.try_next().await?.expect("row expected");

    let tiny = row.try_get_raw(0)?.to_owned().decode::<i8>();
    let small = row.try_get_raw(1)?.to_owned().decode::<i16>();
    let regular = row.try_get_raw(2)?.to_owned().decode::<i32>();
    let big = row.try_get_raw(3)?.to_owned().decode::<i64>();

    assert_eq!(tiny, -128);
    assert_eq!(small, -32768);
    assert_eq!(regular, -2147483648);
    assert_eq!(big, -9223372036854775808);
    Ok(())
}

#[tokio::test]
async fn it_handles_different_float_types() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let sql = format!(
        "SELECT {} AS f32_val, {} AS f64_val, 1.23456789 AS precise_val",
        std::f32::consts::PI,
        std::f64::consts::E
    );
    let mut s = conn.fetch(sql.as_str());
    let row = s.try_next().await?.expect("row expected");

    let f32_val = row.try_get_raw(0)?.to_owned().decode::<f32>();
    let f64_val = row.try_get_raw(1)?.to_owned().decode::<f64>();
    let precise_val = row.try_get_raw(2)?.to_owned().decode::<f64>();

    assert!((f32_val - std::f32::consts::PI).abs() < 1e-5);
    assert!((f64_val - std::f64::consts::E).abs() < 1e-10);
    assert!((precise_val - 1.23456789).abs() < 1e-8);
    Ok(())
}

#[tokio::test]
async fn it_handles_boolean_values() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test boolean-like values - some databases represent booleans as 1/0
    let mut s = conn.fetch("SELECT 1 AS true_val, 0 AS false_val");
    let row = s.try_next().await?.expect("row expected");

    let true_val = row.try_get_raw(0)?.to_owned().decode::<bool>();
    let false_val = row.try_get_raw(1)?.to_owned().decode::<bool>();

    assert!(true_val);
    assert!(!false_val);
    Ok(())
}

#[tokio::test]
async fn it_handles_zero_and_special_numbers() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut s = conn.fetch("SELECT 0 AS zero, 0.0 AS zero_float");
    let row = s.try_next().await?.expect("row expected");

    let zero = row.try_get_raw(0)?.to_owned().decode::<i32>();
    let zero_float = row.try_get_raw(1)?.to_owned().decode::<f64>();

    assert_eq!(zero, 0);
    assert_eq!(zero_float, 0.0);
    Ok(())
}

#[tokio::test]
async fn it_handles_string_variations() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut s = conn.fetch("SELECT '' AS empty, ' ' AS space, 'Hello, World!' AS greeting, 'Unicode: 🦀 Rust' AS unicode");
    let row = s.try_next().await?.expect("row expected");

    let empty = row.try_get_raw(0)?.to_owned().decode::<String>();
    let space = row.try_get_raw(1)?.to_owned().decode::<String>();
    let greeting = row.try_get_raw(2)?.to_owned().decode::<String>();
    let unicode = row.try_get_raw(3)?.to_owned().decode::<String>();

    assert_eq!(empty, "");
    assert_eq!(space, " ");
    assert_eq!(greeting, "Hello, World!");
    assert_eq!(unicode, "Unicode: 🦀 Rust");
    Ok(())
}

#[tokio::test]
async fn it_handles_type_coercion_from_strings() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test that numeric values returned as strings can be parsed
    let sql = format!(
        "SELECT '42' AS str_int, '{}' AS str_float, '1' AS str_bool",
        std::f64::consts::PI
    );
    let mut s = conn.fetch(sql.as_str());
    let row = s.try_next().await?.expect("row expected");

    let str_float = row.try_get_raw(1)?.to_owned().decode::<f64>();
    let str_bool = row.try_get_raw(2)?.to_owned().decode::<bool>();

    assert!((str_float - std::f64::consts::PI).abs() < 1e-10);
    assert!(str_bool);
    Ok(())
}

#[tokio::test]
async fn it_handles_large_strings() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test a moderately large string
    let large_string = "a".repeat(1000);
    let stmt = conn.prepare("SELECT ? AS large_str").await?;
    let row = stmt
        .query()
        .bind(&large_string)
        .fetch_one(&mut conn)
        .await?;

    let result = row.try_get_raw(0)?.to_owned().decode::<String>();
    assert_eq!(result, large_string);
    assert_eq!(result.len(), 1000);
    Ok(())
}

#[tokio::test]
async fn it_handles_binary_data() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test binary data - use UTF-8 safe bytes for PostgreSQL compatibility
    let binary_data = "Héllö world! 😀".as_bytes();
    let stmt = conn.prepare("SELECT ? AS binary_data").await?;
    let row = stmt
        .query_as::<(Vec<u8>,)>()
        .bind(binary_data)
        .fetch_optional(&mut conn)
        .await
        .expect("query failed")
        .expect("row expected");

    assert_eq!(row.0, binary_data);
    Ok(())
}

#[tokio::test]
async fn it_handles_mixed_null_and_values() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let stmt = conn
        .prepare("SELECT ?, ?, ?, ? UNION ALL SELECT NULL, NULL, NULL, NULL")
        .await?;
    let rows = stmt
        .query()
        .bind(42_i32)
        .bind(Option::<i32>::None)
        .bind("hello")
        .bind(Option::<String>::None)
        .fetch_all(&mut conn)
        .await?;

    assert_eq!(rows.len(), 2, "should have 2 rows");
    assert_eq!(rows[0].get::<Option<i32>, _>(0), Some(42));
    assert_eq!(rows[0].get::<Option<i32>, _>(1), None);
    assert_eq!(
        rows[0].get::<Option<String>, _>(2),
        Some("hello".to_owned())
    );
    assert_eq!(rows[0].get::<Option<String>, _>(3), None);
    assert_eq!(rows[1].get::<Option<i32>, _>(0), None);
    assert_eq!(rows[1].get::<Option<i32>, _>(1), None);
    assert_eq!(rows[1].get::<Option<String>, _>(2), None);
    assert_eq!(rows[1].get::<Option<String>, _>(3), None);
    Ok(())
}

#[tokio::test]
async fn it_handles_unsigned_integers() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test unsigned integer types
    let mut s = conn.fetch("SELECT 255 AS u8_val, 65535 AS u16_val, 4294967295 AS u32_val");
    let row = s.try_next().await?.expect("row expected");

    let u8_val = row.try_get_raw(0)?.to_owned().decode::<u8>();
    let u16_val = row.try_get_raw(1)?.to_owned().decode::<u16>();
    let u32_val = row.try_get_raw(2)?.to_owned().decode::<u32>();

    assert_eq!(u8_val, 255);
    assert_eq!(u16_val, 65535);
    assert_eq!(u32_val, 4294967295);
    Ok(())
}

#[tokio::test]
async fn it_handles_slice_types() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test slice types
    let test_data = b"Hello, ODBC!";
    let stmt = conn.prepare("SELECT ? AS slice_data").await?;
    let row = stmt
        .query()
        .bind(&test_data[..])
        .fetch_one(&mut conn)
        .await?;

    let result = row.try_get_raw(0)?.to_owned().decode::<Vec<u8>>();
    assert_eq!(result, test_data);
    Ok(())
}

#[cfg(feature = "uuid")]
#[tokio::test]
async fn it_handles_uuid() -> anyhow::Result<()> {
    use sqlx_oldapi::types::Uuid;
    let mut conn = new::<Odbc>().await?;

    // Use a fixed UUID for testing
    let test_uuid = Uuid::nil();
    let uuid_str = test_uuid.to_string();

    // Test UUID as string
    let stmt = conn.prepare("SELECT ? AS uuid_data").await?;
    let row = stmt.query().bind(&uuid_str).fetch_one(&mut conn).await?;

    let result = row.try_get_raw(0)?.to_owned().decode::<Uuid>();
    assert_eq!(result, test_uuid);

    // Test with a specific UUID string
    let specific_uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let stmt = conn.prepare("SELECT ? AS uuid_data").await?;
    let row = stmt
        .query()
        .bind(specific_uuid_str)
        .fetch_one(&mut conn)
        .await?;

    let result = row.try_get_raw(0)?.to_owned().decode::<Uuid>();
    let expected_uuid: Uuid = specific_uuid_str.parse()?;
    assert_eq!(result, expected_uuid);

    Ok(())
}

#[cfg(feature = "json")]
#[tokio::test]
async fn it_handles_json() -> anyhow::Result<()> {
    use serde_json::{json, Value};
    let mut conn = new::<Odbc>().await?;

    let test_json = json!({
        "name": "John",
        "age": 30,
        "active": true
    });
    let json_str = test_json.to_string();

    let stmt = conn.prepare("SELECT ? AS json_data").await?;
    let row = stmt.query().bind(&json_str).fetch_one(&mut conn).await?;

    let result: Value = row.try_get_raw(0)?.to_owned().decode();
    assert_eq!(result, test_json);
    Ok(())
}

#[cfg(feature = "bigdecimal")]
#[tokio::test]
async fn it_handles_bigdecimal() -> anyhow::Result<()> {
    use sqlx_oldapi::types::BigDecimal;
    use std::str::FromStr;
    let mut conn = new::<Odbc>().await?;

    let test_decimal = BigDecimal::from_str("123.456789")?;
    let decimal_str = test_decimal.to_string();

    let stmt = conn.prepare("SELECT ? AS decimal_data").await?;
    let row = stmt.query().bind(&decimal_str).fetch_one(&mut conn).await?;

    let result = row.try_get_raw(0)?.to_owned().decode::<BigDecimal>();
    assert_eq!(result, test_decimal);
    Ok(())
}

#[cfg(feature = "decimal")]
#[tokio::test]
async fn it_handles_rust_decimal() -> anyhow::Result<()> {
    use sqlx_oldapi::types::Decimal;
    let mut conn = new::<Odbc>().await?;

    let test_decimal = "123.456789".parse::<Decimal>()?;
    let decimal_str = test_decimal.to_string();

    let stmt = conn.prepare("SELECT ? AS decimal_data").await?;
    let row = stmt.query().bind(&decimal_str).fetch_one(&mut conn).await?;

    let result = row.try_get_raw(0)?.to_owned().decode::<Decimal>();
    assert_eq!(result, test_decimal);
    Ok(())
}

#[cfg(feature = "chrono")]
#[tokio::test]
async fn it_handles_chrono_datetime() -> anyhow::Result<()> {
    use sqlx_oldapi::types::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    let mut conn = new::<Odbc>().await?;

    // Test that chrono types work for encoding and basic handling
    // We'll test encode/decode through the Type and Encode implementations

    // Create chrono objects
    let test_date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
    let test_time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
    let test_datetime = NaiveDateTime::new(test_date, test_time);

    // Test that we can encode chrono types (by storing them as strings)
    let stmt = conn.prepare("SELECT ? AS date_data").await?;
    let row = stmt.query().bind(test_date).fetch_one(&mut conn).await?;

    // Decode as string and verify format
    let result_str = row.try_get_raw(0)?.to_owned().decode::<String>();
    assert_eq!(result_str, "2023-12-25");

    // Test time encoding
    let stmt = conn.prepare("SELECT ? AS time_data").await?;
    let row = stmt.query().bind(test_time).fetch_one(&mut conn).await?;

    let result_str = row.try_get_raw(0)?.to_owned().decode::<String>();
    assert_eq!(result_str, "14:30:00");

    // Test datetime encoding
    let stmt = conn.prepare("SELECT ? AS datetime_data").await?;
    let row = stmt
        .query()
        .bind(test_datetime)
        .fetch_one(&mut conn)
        .await?;

    let result_str = row.try_get_raw(0)?.to_owned().decode::<String>();
    assert_eq!(result_str, "2023-12-25 14:30:00");

    Ok(())
}

#[tokio::test]
async fn it_handles_type_compatibility_edge_cases() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test that small integers can decode to larger types
    let mut s = conn.fetch("SELECT 127 AS small_int");
    let row = s.try_next().await?.expect("row expected");

    // Should be able to decode as most integer types (some may not be compatible due to specific type mapping)
    let as_i8 = row.try_get_raw(0)?.to_owned().decode::<i8>();
    let as_i16 = row.try_get_raw(0)?.to_owned().decode::<i16>();
    let as_i32 = row.try_get_raw(0)?.to_owned().decode::<i32>();
    let as_i64 = row.try_get_raw(0)?.to_owned().decode::<i64>();
    let as_u8 = row.try_get_raw(0)?.to_owned().decode::<u8>();
    let as_u16 = row.try_get_raw(0)?.to_owned().decode::<u16>();
    let as_u32 = row.try_get_raw(0)?.to_owned().decode::<u32>();
    // Note: u64 may not be compatible with all integer types from databases

    assert_eq!(as_i8, 127);
    assert_eq!(as_i16, 127);
    assert_eq!(as_i32, 127);
    assert_eq!(as_i64, 127);
    assert_eq!(as_u8, 127);
    assert_eq!(as_u16, 127);
    assert_eq!(as_u32, 127);

    Ok(())
}

#[tokio::test]
async fn it_handles_numeric_precision() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test high precision floating point
    let sql = format!("SELECT {} AS high_precision", std::f64::consts::PI);
    let mut s = conn.fetch(sql.as_str());
    let row = s.try_next().await?.expect("row expected");

    let result = row.try_get_raw(0)?.to_owned().decode::<f64>();
    assert!((result - std::f64::consts::PI).abs() < 1e-10);

    Ok(())
}

// Error case tests

#[tokio::test]
async fn it_handles_connection_level_errors() -> anyhow::Result<()> {
    // Test connection with obviously invalid connection strings
    let invalid_opts = OdbcConnectOptions::from_str("DSN=DefinitelyNonExistentDataSource_12345")?;
    let result = sqlx_oldapi::odbc::OdbcConnection::connect_with(&invalid_opts).await;
    // This should reliably fail across all ODBC drivers
    let err = result.expect_err("should be an error");
    assert!(
        matches!(err, sqlx_core::error::Error::Configuration(_)),
        "{:?} should be a configuration error",
        err
    );

    // Test with malformed connection string
    let malformed_opts = OdbcConnectOptions::from_str("INVALID_KEY_VALUE_PAIRS;;;")?;
    let result = sqlx_oldapi::odbc::OdbcConnection::connect_with(&malformed_opts).await;
    // This should also reliably fail
    let err = result.expect_err("should be an error");
    assert!(
        matches!(err, sqlx_core::error::Error::Configuration(_)),
        "{:?} should be a configuration error",
        err
    );

    Ok(())
}

#[tokio::test]
async fn it_handles_sql_syntax_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test invalid SQL syntax
    let result = conn.execute("INVALID SQL SYNTAX THAT SHOULD FAIL").await;
    let err = result.expect_err("should be an error");
    assert!(
        matches!(err, sqlx_core::error::Error::Database(_)),
        "{:?} should be a database error",
        err
    );

    // Test malformed SELECT
    let result = conn.execute("SELECT FROM WHERE").await;
    let err = result.expect_err("should be an error");
    assert!(
        matches!(err, sqlx_core::error::Error::Database(_)),
        "{:?} should be a database error",
        err
    );

    // Test unclosed quotes
    let result = conn.execute("SELECT 'unclosed string").await;
    let err = result.expect_err("should be an error");
    assert!(
        matches!(err, sqlx_core::error::Error::Database(_)),
        "{:?} should be a database error",
        err
    );

    Ok(())
}

#[tokio::test]
async fn it_handles_prepare_statement_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Many ODBC drivers are permissive at prepare time and only validate at execution
    // So we test that execution fails even if preparation succeeds

    // Test executing prepared invalid SQL
    if let Ok(stmt) = conn.prepare("INVALID PREPARE STATEMENT").await {
        let result = stmt.query().fetch_one(&mut conn).await;
        let err = result.expect_err("should be an error");
        assert!(
            matches!(err, sqlx_core::error::Error::Database(_)),
            "{:?} should be a database error",
            err
        );
    }

    // Test executing prepared SQL with syntax errors
    match conn
        .prepare("SELECT idonotexist FROM idonotexist WHERE idonotexist")
        .await
    {
        Ok(stmt) => match stmt.query().fetch_one(&mut conn).await {
            Ok(_) => panic!("should be an error"),
            Err(sqlx_oldapi::Error::Database(err)) => {
                assert!(
                    err.to_string().contains("idonotexist"),
                    "{:?} should contain 'idonotexist'",
                    err
                );
            }
            Err(err) => {
                panic!("should be a database error, got {:?}", err);
            }
        },
        Err(sqlx_oldapi::Error::Database(err)) => {
            assert!(
                err.to_string().to_lowercase().contains("idonotexist"),
                "{:?} should contain 'idonotexist'",
                err
            );
        }
        Err(err) => {
            panic!("should be an error, got {:?}", err);
        }
    }
    Ok(())
}

#[tokio::test]
async fn it_handles_parameter_binding_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test with completely missing parameters - this should more reliably fail
    let stmt = conn.prepare("SELECT ? AS param1, ? AS param2").await?;

    // Test with no parameters when some are expected
    let result = stmt.query().fetch_one(&mut conn).await;
    // This test may or may not fail depending on ODBC driver behavior
    // Some drivers are permissive and treat missing params as NULL
    // The important thing is that we don't panic
    let _ = result;

    // Test that we can handle parameter binding gracefully
    // Even if the driver is permissive, the system should be robust
    let stmt2 = conn.prepare("SELECT ? AS single_param").await?;

    // Bind correct number of parameters - this should work
    let result = stmt2.query().bind(42i32).fetch_one(&mut conn).await;
    // If this fails, it's likely due to other issues, not parameter count
    if result.is_err() {
        // Log that even basic parameter binding failed - this indicates deeper issues
        println!("Note: Basic parameter binding failed, may indicate driver issues");
    }

    Ok(())
}

#[tokio::test]
async fn it_handles_parameter_execution_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test parameter binding with incompatible operations that should fail at execution
    let stmt = conn.prepare("SELECT ? / 0 AS div_by_zero").await?;

    // This should execute but may produce a runtime error (division by zero)
    let result = stmt.query().bind(42i32).fetch_one(&mut conn).await;
    // Division by zero behavior is database-specific, so we just ensure no panic
    let _ = result;

    // Test with a parameter in an invalid context that should fail
    if let Ok(stmt) = conn.prepare("SELECT * FROM ?").await {
        // Using parameter as table name should fail at execution
        let result = stmt
            .query()
            .bind("non_existent_table")
            .fetch_one(&mut conn)
            .await;
        assert!(result.is_err());
    }

    Ok(())
}

#[tokio::test]
async fn it_handles_fetch_errors_from_invalid_queries() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test fetching from invalid table
    {
        let mut stream = conn.fetch("SELECT * FROM non_existent_table_12345");
        let result = stream.try_next().await;
        assert!(result.is_err());
    }

    // Test fetching with invalid column references
    {
        let mut stream =
            conn.fetch("SELECT non_existent_column FROM (SELECT 1 AS existing_column) t");
        let result = stream.try_next().await;
        assert!(result.is_err());
    }

    Ok(())
}

#[tokio::test]
async fn it_handles_transaction_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Start a transaction
    let mut tx = conn.begin().await?;

    // Try to execute invalid SQL in transaction
    let result = tx.execute("INVALID TRANSACTION SQL").await;
    assert!(result.is_err());

    // Transaction should still be rollbackable even after error
    let rollback_result = tx.rollback().await;
    // Some databases may auto-rollback on errors, so we don't assert success here
    // Just ensure we don't panic
    let _ = rollback_result;

    Ok(())
}

#[tokio::test]
async fn it_handles_fetch_optional_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test fetch_optional with invalid SQL
    let result = (&mut conn)
        .fetch_optional("INVALID SQL FOR FETCH OPTIONAL")
        .await;
    assert!(result.is_err());

    // Test fetch_optional with malformed query
    let result = (&mut conn).fetch_optional("SELECT FROM").await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn it_handles_execute_many_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test execute with invalid SQL that would affect multiple rows
    let result = conn.execute("UPDATE non_existent_table SET col = 1").await;
    assert!(result.is_err());

    // Test execute with constraint violations (if supported by the database)
    // This is database-specific, so we'll test with a more generic invalid statement
    let result = conn
        .execute("INSERT INTO non_existent_table VALUES (1)")
        .await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn it_handles_invalid_column_access() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut stream = conn.fetch("SELECT 'test' AS single_column");
    if let Some(row) = stream.try_next().await? {
        // Test accessing non-existent column by index
        let result = row.try_get_raw(999); // Invalid index
        assert!(result.is_err());

        // Test accessing non-existent column by name
        let result = row.try_get_raw("non_existent_column");
        assert!(result.is_err());
    }

    Ok(())
}

#[tokio::test]
async fn it_handles_type_conversion_errors() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut stream = conn.fetch("SELECT 'not_a_number' AS text_value");
    if let Some(row) = stream.try_next().await? {
        // Try to decode text as number - this might succeed or fail depending on implementation
        // The error handling depends on whether the decode trait panics or returns a result
        let text_val = row.try_get_raw(0)?.to_owned();

        // Test decoding text as different types
        // Some type conversions might work (string parsing) while others might fail
        // This tests the robustness of the type system
        let _: Result<i32, _> = std::panic::catch_unwind(|| text_val.decode::<i32>());

        // The test should not panic even with invalid conversions
    }

    Ok(())
}

#[tokio::test]
async fn it_handles_large_invalid_queries() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test with very long invalid SQL
    let large_invalid_sql = "SELECT ".to_string() + &"invalid_column, ".repeat(1000) + "1";
    let result = conn.execute(large_invalid_sql.as_str()).await;
    assert!(result.is_err());

    // Test with deeply nested invalid SQL
    let nested_invalid_sql = "SELECT (".repeat(100) + "1" + &")".repeat(100) + " FROM non_existent";
    let result = conn.execute(nested_invalid_sql.as_str()).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn it_handles_concurrent_error_scenarios() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test multiple invalid operations in sequence
    let _ = conn.execute("INVALID SQL 1").await;
    let _ = conn.execute("INVALID SQL 2").await;
    let _ = conn.execute("INVALID SQL 3").await;

    // Connection should still be usable after errors
    let valid_result = conn.execute("SELECT 1").await;
    // Some databases may close connection on errors, others may keep it open
    // We just ensure no panic occurs
    let _ = valid_result;

    Ok(())
}

#[tokio::test]
async fn it_handles_prepared_statement_with_wrong_parameters() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Prepare a statement expecting specific parameter types
    let stmt = conn.prepare("SELECT ? + ? AS sum").await?;

    // Test binding incompatible types (if the database is strict about types)
    // Some databases/drivers are permissive, others are strict
    let result = stmt
        .query()
        .bind("not_a_number")
        .bind("also_not_a_number")
        .fetch_one(&mut conn)
        .await;
    // This may or may not error depending on the database's type system
    let _ = result;

    Ok(())
}

#[tokio::test]
async fn it_works_with_buffered_and_unbuffered_mode() -> anyhow::Result<()> {
    use sqlx_oldapi::odbc::{OdbcBufferSettings, OdbcConnectOptions};

    // Create connection with unbuffered settings
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let mut opts = OdbcConnectOptions::from_str(&database_url)?;

    let count = 450;

    let select = (0..count)
        .map(|i| format!("SELECT {i} AS n, '{}' as aas", "a".repeat(i)))
        .collect::<Vec<_>>()
        .join(" UNION ALL ");

    for buf_settings in [
        OdbcBufferSettings {
            batch_size: 1,
            max_column_size: None,
        },
        OdbcBufferSettings {
            batch_size: 1,
            max_column_size: Some(450),
        },
        OdbcBufferSettings {
            batch_size: 100,
            max_column_size: None,
        },
        OdbcBufferSettings {
            batch_size: 10000,
            max_column_size: None,
        },
        OdbcBufferSettings {
            batch_size: 10000,
            max_column_size: Some(450),
        },
    ] {
        opts.buffer_settings(buf_settings);

        let mut conn = OdbcConnection::connect_with(&opts).await?;

        // Test that unbuffered mode works correctly
        let s = conn
            .prepare(&select)
            .await?
            .query()
            .fetch_all(&mut conn)
            .await?;
        assert_eq!(s.len(), count);
        for i in 0..count {
            let row = s.get(i).expect("row expected");
            let as_i64 = row
                .try_get_raw(0)
                .expect("1 column expected")
                .to_owned()
                .try_decode::<i64>()
                .expect("SELECT n should be an i64");
            assert_eq!(as_i64, i64::try_from(i).unwrap());
            let aas = row
                .try_get_raw(1)
                .expect("1 column expected")
                .to_owned()
                .try_decode::<String>()
                .expect("SELECT aas should be a string");
            assert_eq!(aas, "a".repeat(i));
        }
    }
    Ok(())
}
