use std::time::Duration;
use tokio::time::sleep;



pub async fn run_with_retry<T, E>(retries: usize, delay: Duration, mut action: impl AsyncFnMut() -> Result<T, E>) -> Result<T, E> {
    for _ in 0..retries {
        if let Ok(result) = action().await  {
            return Ok(result)
        }
        sleep(delay).await;
    }

    action().await
}
