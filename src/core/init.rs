use std::path::{Path, PathBuf};
use tokio::fs;
use anyhow::{Result, Context};
use tracing::info;

/// Application directory structure
#[derive(Debug, Clone)]
pub struct AppDirs {
    pub backups: PathBuf,
    pub resources: PathBuf,
    pub server: PathBuf,
}

/// Initializes the application directory structure next to the executable.
/// 
/// If the folders do not exist, they will be created.
/// 
/// # Arguments
/// * `base_path` - The base path where the folders should be created (usually the exe directory).
pub async fn init_directories(base_path: &Path) -> Result<AppDirs> {
    let backups = base_path.join("backups");
    let resources = base_path.join("resources");
    let server = base_path.join("server");

    let dirs = [
        (&backups, "backups"),
        (&resources, "resources"),
        (&server, "server"),
    ];

    for (path, name) in dirs {
        if !path.exists() {
            fs::create_dir_all(path)
                .await
                .with_context(|| format!("Failed to create {} directory at {:?}", name, path))?;
            info!("Created {} directory: {:?}", name, path);
        }
    }

    Ok(AppDirs {
        backups,
        resources,
        server,
    })
}
