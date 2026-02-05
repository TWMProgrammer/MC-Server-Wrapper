use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}
