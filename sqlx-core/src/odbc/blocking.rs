use crate::error::Error;
use sqlx_rt::spawn_blocking;

pub async fn run_blocking<R, F>(f: F) -> Result<R, Error>
where
    R: Send + 'static,
    F: FnOnce() -> Result<R, Error> + Send + 'static,
{
    #[cfg(feature = "_rt-tokio")]
    {
        let join_result = spawn_blocking(f).await.map_err(|_| Error::WorkerCrashed)?;
        join_result
    }

    #[cfg(feature = "_rt-async-std")]
    {
        spawn_blocking(f).await
    }
}
