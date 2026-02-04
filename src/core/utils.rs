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

    #[test]
    fn test_safe_join() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();
        
        // Canonicalize base for comparison (tempdir might be in a symlinked path on some OS)
        let canonical_base = base.canonicalize().unwrap();

        // Safe join
        let joined = safe_join(base, "file.txt").unwrap();
        assert!(joined.starts_with(&canonical_base));
        assert!(joined.ends_with("file.txt"));

        // Safe join with existing file
        let file_path = base.join("existing.txt");
        std::fs::write(&file_path, "test").unwrap();
        let joined = safe_join(base, "existing.txt").unwrap();
        assert!(joined.starts_with(&canonical_base));
        assert!(joined.exists());

        // Safe join with subdirectory
        let sub = base.join("sub");
        std::fs::create_dir(&sub).unwrap();
        let joined = safe_join(base, "sub/file.txt").unwrap();
        assert!(joined.starts_with(&canonical_base));

        // Traversal should still be caught by validate_rel_path
        assert!(safe_join(base, "../outside.txt").is_err());
        assert!(safe_join(base, "sub/../../outside.txt").is_err());
    }

    #[test]
    fn test_normalize_path() {
        let p = Path::new("/a/b/../c/./d");
        assert_eq!(normalize_path(p), PathBuf::from("/a/c/d"));

        let p = Path::new("a/b/../c");
        assert_eq!(normalize_path(p), PathBuf::from("a/c"));

        #[cfg(windows)]
        {
            let p = Path::new("C:\\a\\..\\b");
            assert_eq!(normalize_path(p), PathBuf::from("C:\\b"));
        }
    }
}

/// Safely joins a base path with a relative path, ensuring no traversal.
/// 
/// This implementation follows a multi-step hardening process:
/// 1. Validates the relative path for traversal components and encoding.
/// 2. Canonicalizes the base path to resolve symlinks.
/// 3. Joins them and verifies the result stays within the base directory,
///    handling both existing and non-existing target paths.
pub fn safe_join(base: impl AsRef<Path>, rel: &str) -> Result<PathBuf> {
    validate_rel_path(rel)?;
    
    let base = base.as_ref();
    // 1. Canonicalize Base Path: Resolve any symbolic links or relative segments in the root directory.
    let canonical_base = base.canonicalize()
        .context(format!("Failed to canonicalize base path: {:?}", base))?;
    
    // 2. Logical Join: Join the canonicalized base with the normalized relative path.
    let joined = canonical_base.join(rel);
    
    // 3. Final Verification
    if joined.exists() {
        // If the resulting path exists, use std::fs::canonicalize on it.
        let canonical_joined = joined.canonicalize()
            .context(format!("Failed to canonicalize joined path: {:?}", joined))?;
        
        // Verify that the final absolute path starts with the canonicalized base path.
        if !canonical_joined.starts_with(&canonical_base) {
            return Err(anyhow!(
                "Security violation: joined path escaped base directory (canonical check). Base: {:?}, Joined: {:?}",
                canonical_base,
                canonical_joined
            ));
        }
        Ok(canonical_joined)
    } else {
        // If the path does not exist (e.g., for new file creation), 
        // ensure that the joined path's components do not escape the base directory.
        let normalized_joined = normalize_path(&joined);
        
        if !normalized_joined.starts_with(&canonical_base) {
            return Err(anyhow!(
                "Security violation: joined path escaped base directory (logical check). Base: {:?}, Joined: {:?}",
                canonical_base,
                normalized_joined
            ));
        }
        Ok(normalized_joined)
    }
}

/// Logically normalizes a path without hitting the disk.
/// Resolves '.' and '..' components.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek() {
        let buf = PathBuf::from(c.as_os_str());
        components.next();
        buf
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
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
