use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;
use super::types::{Project, ProjectVersion, PluginProvider};

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

    pub async fn search(&self, options: &super::types::SearchOptions) -> Result<Vec<Project>> {
        let mut url = format!("https://api.modrinth.com/v2/search?query={}", urlencoding::encode(&options.query));

        if let Some(facets) = &options.facets {
            if !facets.is_empty() {
                // Modrinth facets are a 2D array: [["AND_group1_OR_item1", "AND_group1_OR_item2"], ["AND_group2"]]
                // We treat each facet in our list as a separate AND group containing one item.
                let and_groups: Vec<Vec<&String>> = facets.iter().map(|f| vec![f]).collect();
                let facets_json = serde_json::to_string(&and_groups)?;
                url.push_str(&format!("&facets={}", urlencoding::encode(&facets_json)));
            }
        }

        if let Some(sort) = options.sort {
            let index = match sort {
                super::types::SortOrder::Relevance => "relevance",
                super::types::SortOrder::Downloads => "downloads",
                super::types::SortOrder::Follows => "follows",
                super::types::SortOrder::Newest => "newest",
                super::types::SortOrder::Updated => "updated",
            };
            url.push_str(&format!("&index={}", index));
        }

        if let Some(offset) = options.offset {
            url.push_str(&format!("&offset={}", offset));
        }

        if let Some(limit) = options.limit {
            url.push_str(&format!("&limit={}", limit));
        }

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
            provider: PluginProvider::Modrinth,
        }).collect();

        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let url = format!("https://api.modrinth.com/v2/project/{}", id);
        let h = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        Ok(Project {
            id: h["id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
            author: String::new(), // Author is in a separate field in project API
            provider: PluginProvider::Modrinth,
        })
    }

    pub async fn get_dependencies(&self, project_id: &str) -> Result<Vec<Project>> {
        let url = format!("https://api.modrinth.com/v2/project/{}/dependencies", project_id);
        let response = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        let projects_json = response["projects"].as_array().ok_or_else(|| anyhow!("Invalid dependencies response"))?;
        
        let projects = projects_json.iter().map(|h| Project {
            id: h["id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
            author: String::new(),
            provider: PluginProvider::Modrinth,
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
