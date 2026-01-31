use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;
use super::types::{Project, ProjectVersion};

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
