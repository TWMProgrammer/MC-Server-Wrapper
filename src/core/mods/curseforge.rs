use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use super::types::{Project, ModProvider};

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
            provider: ModProvider::CurseForge,
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
