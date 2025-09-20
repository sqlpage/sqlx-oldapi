use futures::TryStreamExt;
use sqlx_oldapi::odbc::Odbc;
use sqlx_oldapi::Column;
use sqlx_oldapi::Connection;
use sqlx_oldapi::Executor;
use sqlx_oldapi::Row;
use sqlx_oldapi::Statement;
use sqlx_oldapi::Value;
use sqlx_oldapi::ValueRef;
use sqlx_test::new;

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
        assert_eq!(row.column(0).name(), "n");
        assert_eq!(row.column(1).name(), "s");
        assert_eq!(row.column(2).name(), "z");
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

    assert_eq!(row.column(0).name(), "i");
    assert_eq!(row.column(1).name(), "f");
    assert_eq!(row.column(2).name(), "t");

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
    let stmt = (&mut conn).prepare("SELECT 7 AS seven").await?;
    let row = stmt.query().fetch_one(&mut conn).await?;
    assert_eq!(row.column(0).name(), "seven");
    let v = row.try_get_raw(0)?.to_owned().decode::<i64>();
    assert_eq!(v, 7);
    Ok(())
}

#[tokio::test]
async fn it_can_prepare_then_query_with_params_integer_float_text() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let stmt = (&mut conn).prepare("SELECT ? AS i, ? AS f, ? AS t").await?;

    let row = stmt
        .query()
        .bind(5_i32)
        .bind(1.25_f64)
        .bind("hello")
        .fetch_one(&mut conn)
        .await?;

    assert_eq!(row.column(0).name(), "i");
    assert_eq!(row.column(1).name(), "f");
    assert_eq!(row.column(2).name(), "t");
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

    let stmt = (&mut conn).prepare(&sql).await?;

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

    let stmt = (&mut conn).prepare("SELECT ?, ?, ?, ?, ?").await?;

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
    let stmt = (&mut conn).prepare("SELECT ?, ?").await?;
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

    let str_int = row.try_get_raw(0)?.to_owned().decode::<i32>();
    let str_float = row.try_get_raw(1)?.to_owned().decode::<f64>();
    let str_bool = row.try_get_raw(2)?.to_owned().decode::<bool>();

    assert_eq!(str_int, 42);
    assert!((str_float - std::f64::consts::PI).abs() < 1e-10);
    assert!(str_bool);
    Ok(())
}

#[tokio::test]
async fn it_handles_large_strings() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    // Test a moderately large string
    let large_string = "a".repeat(1000);
    let stmt = (&mut conn).prepare("SELECT ? AS large_str").await?;
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
    let binary_data = vec![65u8, 66, 67, 68, 69]; // "ABCDE" in ASCII
    let stmt = (&mut conn).prepare("SELECT ? AS binary_data").await?;
    let row = stmt.query().bind(&binary_data).fetch_one(&mut conn).await?;

    let result = row.try_get_raw(0)?.to_owned().decode::<Vec<u8>>();
    assert_eq!(result, binary_data);
    Ok(())
}

#[tokio::test]
async fn it_handles_mixed_null_and_values() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let stmt = (&mut conn).prepare("SELECT ?, ?, ?, ?").await?;
    let row = stmt
        .query()
        .bind(42_i32)
        .bind(Option::<i32>::None)
        .bind("hello")
        .bind(Option::<String>::None)
        .fetch_one(&mut conn)
        .await?;

    let int_val = row.try_get_raw(0)?.to_owned().decode::<i32>();
    let null_int = row.try_get_raw(1)?.to_owned();
    let str_val = row.try_get_raw(2)?.to_owned().decode::<String>();
    let null_str = row.try_get_raw(3)?.to_owned();

    assert_eq!(int_val, 42);
    assert!(null_int.is_null());
    assert_eq!(str_val, "hello");
    assert!(null_str.is_null());
    Ok(())
}
