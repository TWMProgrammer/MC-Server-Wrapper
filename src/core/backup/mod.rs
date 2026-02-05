use std::path::{Path, PathBuf};
use uuid::Uuid;

pub mod types;
pub mod operations;

pub use types::BackupInfo;

pub struct BackupManager {
    pub(crate) base_dir: PathBuf,
}

impl BackupManager {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    pub(crate) fn get_instance_backup_dir(&self, instance_id: Uuid) -> PathBuf {
        self.base_dir.join(instance_id.to_string())
    }
}
