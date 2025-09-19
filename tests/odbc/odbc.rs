use futures::TryStreamExt;
use sqlx_oldapi::odbc::Odbc;
use sqlx_oldapi::Column;
use sqlx_oldapi::Connection;
use sqlx_oldapi::Executor;
use sqlx_oldapi::Row;
use sqlx_oldapi::Statement;
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
        saw_row = true;
    }
    assert!(saw_row);
    Ok(())
}

#[tokio::test]
async fn it_streams_multiple_rows() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;

    let mut s = conn.fetch("SELECT 1 AS v UNION ALL SELECT 2 UNION ALL SELECT 3");
    let mut row_count = 0;
    while let Some(_row) = s.try_next().await? {
        row_count += 1;
    }
    assert_eq!(row_count, 3);
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

    let v0 = row.try_get_raw(0)?; // 's'
    let v1 = row.try_get_raw(1)?; // 'z'

    assert!(!v0.is_null());
    assert!(v1.is_null());
    Ok(())
}

#[tokio::test]
async fn it_handles_basic_numeric_and_text_expressions() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let mut s = conn.fetch("SELECT 1 AS i, 1.5 AS f, 'hello' AS t");
    let row = s.try_next().await?.expect("row expected");

    // verify metadata is present and values are non-null
    assert_eq!(row.column(0).name(), "i");
    assert_eq!(row.column(1).name(), "f");
    assert_eq!(row.column(2).name(), "t");

    assert!(!row.try_get_raw(0)?.is_null());
    assert!(!row.try_get_raw(1)?.is_null());
    assert!(!row.try_get_raw(2)?.is_null());
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
    Ok(())
}
