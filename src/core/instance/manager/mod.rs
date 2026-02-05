use crate::database::Database;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::warn;

pub mod clone;
pub mod create;
pub mod delete;
pub mod detection;
pub mod import;
pub mod persistence;
pub mod query;

pub struct InstanceManager {
    pub(crate) base_dir: PathBuf,
    pub(crate) db: Arc<Database>,
}

impl InstanceManager {
    pub async fn new(base_dir: impl AsRef<Path>, db: Arc<Database>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).await?;
        }
        let manager = Self { base_dir, db };
        if let Err(e) = manager.migrate_from_json().await {
            warn!("Failed to migrate instances from JSON: {}", e);
        }
        Ok(manager)
    }

    pub fn get_base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }
}
