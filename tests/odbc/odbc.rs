use futures::TryStreamExt;
use sqlx_oldapi::odbc::Odbc;
use sqlx_oldapi::Column;
use sqlx_oldapi::Connection;
use sqlx_oldapi::Executor;
use sqlx_oldapi::Row;
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
