use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};

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
