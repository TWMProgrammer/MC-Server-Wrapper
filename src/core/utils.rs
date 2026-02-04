use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow, Context};

/// Validates that a relative path does not contain path traversal components.
pub fn validate_rel_path(rel_path: &str) -> Result<()> {
    if rel_path.contains("..") || rel_path.starts_with('/') || rel_path.starts_with('\\') {
        return Err(anyhow!("Invalid path: Path traversal or absolute path not allowed: {}", rel_path));
    }
    Ok(())
}

/// Safely joins a base path with a relative path, ensuring no traversal.
pub fn safe_join(base: impl AsRef<Path>, rel: &str) -> Result<PathBuf> {
    validate_rel_path(rel)?;
    Ok(base.as_ref().join(rel))
}

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
