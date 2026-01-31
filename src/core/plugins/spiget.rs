use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use super::types::Project;

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
