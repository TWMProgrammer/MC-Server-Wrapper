use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use super::validation::validate_rel_path;
use super::normalization::normalize_path;

/// Safely joins a base path with a relative path, ensuring no traversal.
pub fn safe_join(base: impl AsRef<Path>, rel: &str) -> Result<PathBuf> {
    validate_rel_path(rel)?;

    let base = base.as_ref();
    let canonical_base = base
        .canonicalize()
        .context(format!("Failed to canonicalize base path: {:?}", base))?;

    let joined = canonical_base.join(rel);

    if joined.exists() {
        let canonical_joined = joined
            .canonicalize()
            .context(format!("Failed to canonicalize joined path: {:?}", joined))?;

        if !canonical_joined.starts_with(&canonical_base) {
            return Err(anyhow!(
                "Security violation: joined path escaped base directory (canonical check). Base: {:?}, Joined: {:?}",
                canonical_base,
                canonical_joined
            ));
        }
        Ok(canonical_joined)
    } else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_join() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path();
        let canonical_base = base.canonicalize().unwrap();

        let joined = safe_join(base, "file.txt").unwrap();
        assert!(joined.starts_with(&canonical_base));
        assert!(joined.ends_with("file.txt"));

        let file_path = base.join("existing.txt");
        std::fs::write(&file_path, "test").unwrap();
        let joined = safe_join(base, "existing.txt").unwrap();
        assert!(joined.starts_with(&canonical_base));
        assert!(joined.exists());

        let sub = base.join("sub");
        std::fs::create_dir(&sub).unwrap();
        let joined = safe_join(base, "sub/file.txt").unwrap();
        assert!(joined.starts_with(&canonical_base));

        assert!(safe_join(base, "../outside.txt").is_err());
        assert!(safe_join(base, "sub/../../outside.txt").is_err());
    }
}
