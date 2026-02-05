use std::path::{Component, Path, PathBuf};

/// Logically normalizes a path without hitting the disk.
pub fn normalize_path(path: &Path) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::*;

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
