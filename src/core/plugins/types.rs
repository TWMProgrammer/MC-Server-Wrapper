use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectVersion {
    pub id: String,
    pub project_id: String,
    pub version_number: String,
    pub files: Vec<ProjectFile>,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
}
