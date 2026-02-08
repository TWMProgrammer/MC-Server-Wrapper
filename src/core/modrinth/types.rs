use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModrinthProjectType {
    #[serde(rename = "mod")]
    Mod,
    #[serde(rename = "plugin")]
    Plugin,
    #[serde(rename = "resourcepack")]
    ResourcePack,
    #[serde(rename = "datapack")]
    DataPack,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModrinthSortOrder {
    #[serde(rename = "relevance")]
    Relevance,
    #[serde(rename = "downloads")]
    Downloads,
    #[serde(rename = "follows")]
    Follows,
    #[serde(rename = "newest")]
    Newest,
    #[serde(rename = "updated")]
    Updated,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthSearchOptions {
    pub query: String,
    pub facets: Option<Vec<String>>,
    pub sort: Option<ModrinthSortOrder>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub project_type: Option<ModrinthProjectType>,
}

impl ModrinthSearchOptions {
    pub fn cache_key(&self) -> String {
        format!(
            "q:{}_f:{:?}_s:{:?}_o:{:?}_l:{:?}_v:{:?}_lo:{:?}_t:{:?}",
            self.query,
            self.facets,
            self.sort,
            self.offset,
            self.limit,
            self.game_version,
            self.loader,
            self.project_type
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthProject {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub screenshot_urls: Option<Vec<String>>,
    pub author: String,
    pub project_type: ModrinthProjectType,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub version_number: String,
    pub files: Vec<ModrinthFile>,
    pub loaders: Vec<String>,
    pub game_versions: Vec<String>,
    pub dependencies: Vec<ModrinthDependency>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
    pub hashes: ModrinthHashes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthHashes {
    pub sha1: Option<String>,
    pub sha512: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthDependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String, // "required", "optional", "incompatible", "embedded"
}
