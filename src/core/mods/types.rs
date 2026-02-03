use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledMod {
    pub name: String,
    pub filename: String,
    pub enabled: bool,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub loader: Option<String>, // Fabric, Forge, Quilt, NeoForge
    pub source: Option<ModSource>,
    pub icon_data: Option<String>, // Base64 encoded icon
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModSource {
    pub project_id: String,
    pub provider: ModProvider,
    pub current_version_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ModProvider {
    Modrinth,
    CurseForge,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub author: String,
    pub provider: ModProvider,
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResolvedDependency {
    pub project: Project,
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
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependency {
    pub project_id: Option<String>,
    pub version_id: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModConfig {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModCacheEntry {
    pub last_modified: u64,
    pub metadata: InstalledMod,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ModCache {
    pub entries: HashMap<String, ModCacheEntry>,
    pub sources: HashMap<String, ModSource>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModUpdate {
    pub filename: String,
    pub current_version: Option<String>,
    pub latest_version: String,
    pub latest_version_id: String,
    pub project_id: String,
    pub provider: ModProvider,
}
