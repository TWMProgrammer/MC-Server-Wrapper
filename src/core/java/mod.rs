use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tokio::fs;

pub mod types;
pub mod detection;
pub mod download;

pub use types::*;

pub struct JavaManager {
    pub(crate) base_dir: PathBuf,
    pub(crate) client: reqwest::Client,
}

impl JavaManager {
    pub fn new() -> Result<Self> {
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;
        let base_dir = exe_path.parent()
            .context("Failed to get executable directory")?
            .join("java");
        
        let client = reqwest::Client::builder()
            .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self { base_dir, client })
    }

    /// Returns the path where Java versions are stored
    pub fn get_base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Deletes a managed Java version.
    pub async fn delete_version(&self, id: &str) -> Result<()> {
        let version_dir = self.base_dir.join(id);
        if version_dir.exists() {
            fs::remove_dir_all(&version_dir).await.context("Failed to delete Java version directory")?;
        }
        Ok(())
    }
}
