use anyhow::{anyhow, Result};
use std::path::{Component, Path};

/// Validates that a relative path does not contain path traversal components.
pub fn validate_rel_path(rel_path: &str) -> Result<()> {
    let decoded = urlencoding::decode(rel_path)
        .map_err(|_| anyhow!("Invalid URL encoding in path: {}", rel_path))?;

    let normalized_input = decoded.replace('\\', "/");

    if normalized_input.starts_with('/') {
        return Err(anyhow!(
            "Invalid path: Absolute paths not allowed: {}",
            rel_path
        ));
    }

    if normalized_input.len() >= 2 {
        let mut chars = normalized_input.chars();
        let first = chars.next().unwrap();
        let second = chars.next().unwrap();
        if first.is_ascii_alphabetic() && second == ':' {
            return Err(anyhow!(
                "Invalid path: Windows drive prefixes not allowed: {}",
                rel_path
            ));
        }
    }

    if normalized_input.starts_with("//") {
        return Err(anyhow!("Invalid path: UNC paths not allowed: {}", rel_path));
    }

    let path = Path::new(&normalized_input);
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::Normal(c) => components.push(c),
            Component::ParentDir => {
                if components.pop().is_none() {
                    return Err(anyhow!(
                        "Invalid path: Path traversal detected (climbing above root): {}",
                        rel_path
                    ));
                }
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {
                return Err(anyhow!(
                    "Invalid path: Absolute paths or prefixes not allowed: {}",
                    rel_path
                ));
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
        assert!(validate_rel_path("a/../b").is_ok());
        assert!(validate_rel_path("a/b/../c").is_ok());
    }

    #[test]
    fn test_validate_rel_path_absolute() {
        assert!(validate_rel_path("/absolute/path").is_err());
    }

    #[test]
    fn test_validate_rel_path_encoded() {
        assert!(validate_rel_path("%2e%2e/traversal").is_err());
        assert!(validate_rel_path("a/%2e%2e/%2e%2e/traversal").is_err());
        assert!(validate_rel_path("a%2fb").is_ok());
        assert!(validate_rel_path("a%5cb").is_ok());
        assert!(validate_rel_path("..%5ctraversal").is_err());
        assert!(validate_rel_path("%2e%2e%2fhidden").is_err());
    }
}
