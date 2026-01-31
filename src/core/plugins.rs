use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;

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

pub struct ModrinthClient {
    client: reqwest::Client,
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ModrinthClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Project>> {
        let url = format!("https://api.modrinth.com/v2/search?query={}", query);
        let response = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        let hits = response["hits"].as_array().ok_or_else(|| anyhow!("Invalid response from Modrinth"))?;
        
        let projects = hits.iter().map(|h| Project {
            id: h["project_id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
            author: h["author"].as_str().unwrap_or_default().to_string(),
        }).collect();

        Ok(projects)
    }

    pub async fn get_versions(&self, project_id: &str) -> Result<Vec<ProjectVersion>> {
        let url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
        let versions = self.client.get(&url).send().await?.json::<Vec<ProjectVersion>>().await?;
        Ok(versions)
    }

    pub async fn download_version(&self, version: &ProjectVersion, target_dir: impl AsRef<Path>) -> Result<()> {
        let file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first())
            .ok_or_else(|| anyhow!("No files found for version"))?;

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let target_path = target_dir.as_ref().join(&file.filename);
        info!("Downloading mod from {}: {} ({} bytes)", file.url, file.filename, file.size);

        let response = self.client.get(&file.url).send().await?;
        let mut f = fs::File::create(&target_path).await?;

        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            f.write_all(&chunk).await?;
        }

        f.flush().await?;
        Ok(())
    }
}

pub struct SpigetClient {
    client: reqwest::Client,
}

impl Default for SpigetClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SpigetClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Project>> {
        let url = format!("https://api.spiget.org/v2/search/resources/{}?field=name&size=10", query);
        let response = self.client.get(&url).send().await?.json::<Vec<serde_json::Value>>().await?;
        
        let projects = response.into_iter().map(|h| Project {
            id: h["id"].as_u64().unwrap_or(0).to_string(),
            slug: h["name"].as_str().unwrap_or_default().to_lowercase().replace(' ', "-"),
            title: h["name"].as_str().unwrap_or_default().to_string(),
            description: h["tag"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon"]["url"].as_str().map(|s| s.to_string()),
            author: "SpigotMC".to_string(),
        }).collect();

        Ok(projects)
    }

    pub async fn download_resource(&self, resource_id: &str, target_dir: impl AsRef<Path>) -> Result<()> {
        let url = format!("https://api.spiget.org/v2/resources/{}/download", resource_id);
        let response = self.client.get(&url).send().await?;
        
        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let filename = format!("spigot-resource-{}.jar", resource_id);
        let target_path = target_dir.as_ref().join(filename);

        let mut f = fs::File::create(&target_path).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            f.write_all(&chunk).await?;
        }

        f.flush().await?;
        Ok(())
    }
}

pub struct CurseForgeClient {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl CurseForgeClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
            api_key,
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Project>> {
        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        
        let url = "https://api.curseforge.com/v1/mods/search";
        let response = self.client.get(url)
            .header("x-api-key", api_key)
            .query(&[
                ("gameId", "432"), // Minecraft
                ("searchFilter", query),
                ("classId", "6"), // Mods
                ("pageSize", "10"),
            ])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let data = response["data"].as_array().ok_or_else(|| anyhow!("Invalid response from CurseForge"))?;
        
        let projects = data.iter().map(|h| Project {
            id: h["id"].as_u64().unwrap_or(0).to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["name"].as_str().unwrap_or_default().to_string(),
            description: h["summary"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloadCount"].as_u64().unwrap_or(0),
            icon_url: h["logo"]["url"].as_str().map(|s| s.to_string()),
            author: h["authors"].as_array().and_then(|a| a.first()).and_then(|a| a["name"].as_str()).unwrap_or("Unknown").to_string(),
        }).collect();

        Ok(projects)
    }

    pub async fn download_mod(&self, mod_id: &str, file_id: &str, target_dir: impl AsRef<Path>) -> Result<()> {
        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        
        let url = format!("https://api.curseforge.com/v1/mods/{}/files/{}", mod_id, file_id);
        let response = self.client.get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let download_url = response["data"]["downloadUrl"].as_str()
            .ok_or_else(|| anyhow!("No download URL found for CurseForge mod"))?;
        let filename = response["data"]["fileName"].as_str().unwrap_or("mod.jar");

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let target_path = target_dir.as_ref().join(filename);
        let response = self.client.get(download_url).send().await?;
        
        let mut f = fs::File::create(&target_path).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            f.write_all(&chunk).await?;
        }

        f.flush().await?;
        Ok(())
    }
}

