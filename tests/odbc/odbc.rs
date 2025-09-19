use sqlx_oldapi::odbc::Odbc;
use sqlx_oldapi::Connection;
use sqlx_test::new;

#[sqlx_macros::test]
async fn it_connects_and_pings() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    conn.ping().await?;
    conn.close().await?;
    Ok(())
}

#[sqlx_macros::test]
async fn it_can_work_with_transactions() -> anyhow::Result<()> {
    let mut conn = new::<Odbc>().await?;
    let tx = conn.begin().await?;
    tx.rollback().await?;
    Ok(())
}
