use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionInfo {
    pub id: String,
    pub r#type: String,
    pub url: String,
    #[serde(rename = "releaseTime")]
    pub release_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VersionDetail {
    pub downloads: Downloads,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub server: DownloadInfo,
}

#[derive(Debug, Deserialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}
