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
