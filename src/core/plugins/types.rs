use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PluginProvider {
    Modrinth,
    Spiget,
    Hangar,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub facets: Option<Vec<String>>,
    pub sort: Option<SortOrder>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub game_version: Option<String>,
    pub loader: Option<String>,
}

impl SearchOptions {
    pub fn cache_key(&self) -> String {
        format!(
            "q:{}_f:{:?}_s:{:?}_o:{:?}_l:{:?}_v:{:?}_lo:{:?}",
            self.query,
            self.facets,
            self.sort,
            self.offset,
            self.limit,
            self.game_version,
            self.loader
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub screenshot_urls: Option<Vec<String>>,
    pub author: String,
    pub provider: PluginProvider,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResolvedDependency {
    pub project: Project,
    pub dependency_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginDependencies {
    pub mandatory: Vec<Project>,
    pub optional: Vec<Project>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub filename: Option<String>,
    pub dependency_type: String,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledPlugin {
    pub name: String,
    pub filename: String,
    pub enabled: bool,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub source: Option<PluginSource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginSource {
    pub project_id: String,
    pub provider: PluginProvider,
    pub current_version_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginUpdate {
    pub filename: String,
    pub current_version: Option<String>,
    pub latest_version: String,
    pub latest_version_id: String,
    pub project_id: String,
    pub provider: PluginProvider,
}
