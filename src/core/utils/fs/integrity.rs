use std::fs::File;
use std::path::Path;
use zip::ZipArchive;

/// Checks if a JAR file is valid by attempting to open it as a ZIP archive.
pub fn is_jar_valid(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    match ZipArchive::new(file) {
        Ok(archive) => archive.len() > 0,
        Err(_) => false,
    }
}
