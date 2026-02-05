use anyhow::{Result, Context, anyhow};

/// A generic, reusable retry utility for async operations.
pub async fn retry_async<T, F, Fut>(
    operation: F,
    max_retries: usize,
    delay: std::time::Duration,
    name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = anyhow!("Unknown error");
    for attempt in 1..=max_retries {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_error = e;
                if attempt < max_retries {
                    tracing::warn!(
                        "{} failed (attempt {}/{}): {}. Retrying in {:?}...",
                        name,
                        attempt,
                        max_retries,
                        last_error,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    Err(last_error).context(format!("{} failed after {} attempts", name, max_retries))
}
