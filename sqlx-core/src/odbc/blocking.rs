use crate::error::Error;
use sqlx_rt::spawn_blocking;

pub async fn run_blocking<R, F>(f: F) -> Result<R, Error>
where
    R: Send + 'static,
    F: FnOnce() -> Result<R, Error> + Send + 'static,
{
    let res = spawn_blocking(f).await.map_err(|_| Error::WorkerCrashed)?;
    res
}
