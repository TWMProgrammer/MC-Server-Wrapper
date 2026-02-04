use std::path::{Path, PathBuf, Component};
use anyhow::{Result, anyhow, Context};

/// Validates that a relative path does not contain path traversal components.
/// This function handles URL encoding and logical normalization.
pub fn validate_rel_path(rel_path: &str) -> Result<()> {
    // 1. Handle Encoded Characters: Ensure that common traversal encodings 
    // (e.g., %2e%2e%2f, ..%5c) are decoded before checks.
    let decoded = urlencoding::decode(rel_path)
        .map_err(|_| anyhow!("Invalid URL encoding in path: {}", rel_path))?;
    
    // Normalize slashes for cross-platform consistency during validation
    let normalized_input = decoded.replace('\\', "/");

    // 2. Reject Absolute Paths and Prefixes
    let path = Path::new(&normalized_input);
    if path.is_absolute() {
        return Err(anyhow!("Invalid path: Absolute paths not allowed: {}", rel_path));
    }

    // 3. Normalize Input & Reject Traversal Components
    // Use logical path normalization to resolve . and .. components.
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::Normal(c) => components.push(c),
            Component::ParentDir => {
                // If we try to pop and there's nothing, it means we're climbing above root
                if components.pop().is_none() {
                    return Err(anyhow!("Invalid path: Path traversal detected (climbing above root): {}", rel_path));
                }
            }
            Component::CurDir => {}, // . is ignored in logical normalization
            Component::RootDir | Component::Prefix(_) => {
                return Err(anyhow!("Invalid path: Absolute paths or prefixes not allowed: {}", rel_path));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_rel_path_basic() {
        assert!(validate_rel_path("safe/path/file.txt").is_ok());
        assert!(validate_rel_path("file.txt").is_ok());
        assert!(validate_rel_path("subdir/").is_ok());
    }

    #[test]
    fn test_validate_rel_path_traversal() {
        assert!(validate_rel_path("../traversal").is_err());
        assert!(validate_rel_path("a/../../traversal").is_err());
        assert!(validate_rel_path("a/b/../../../traversal").is_err());
    }

    #[test]
    fn test_validate_rel_path_normalization() {
        assert!(validate_rel_path("a/./b").is_ok());
        assert!(validate_rel_path("a/../b").is_ok()); // This resolves to "b" which is safe
        assert!(validate_rel_path("a/b/../c").is_ok()); // Resolves to "a/c" which is safe
    }

    #[test]
    fn test_validate_rel_path_absolute() {
        assert!(validate_rel_path("/absolute/path").is_err());
        #[cfg(windows)]
        {
            assert!(validate_rel_path("C:\\Windows").is_err());
            assert!(validate_rel_path("\\\\server\\share").is_err());
        }
    }

    #[test]
    fn test_validate_rel_path_encoded() {
        // %2e is .
        assert!(validate_rel_path("%2e%2e/traversal").is_err());
        assert!(validate_rel_path("a/%2e%2e/%2e%2e/traversal").is_err());
        // %2f is /
        assert!(validate_rel_path("a%2fb").is_ok());
        // %5c is \
        assert!(validate_rel_path("a%5cb").is_ok());
        // Mixed
        assert!(validate_rel_path("..%5ctraversal").is_err());
        assert!(validate_rel_path("%2e%2e%2fhidden").is_err());
    }
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
