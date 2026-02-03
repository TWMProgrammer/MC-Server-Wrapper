use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;
use super::types::{Project, ProjectVersion, ModProvider, SearchOptions, SortOrder};

pub struct ModrinthClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ModrinthClient {
    pub fn new() -> Self {
        Self::with_base_url("https://api.modrinth.com/v2".to_string())
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
            base_url,
        }
    }

    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let mut url = format!("{}/search?query={}", self.base_url, urlencoding::encode(&options.query));

        let mut and_groups: Vec<Vec<String>> = Vec::new();

        if let Some(facets) = &options.facets {
            if !facets.is_empty() {
                for f in facets {
                    and_groups.push(vec![f.clone()]);
                }
            }
        }

        // Add project_type:mod if not already present
        let mut has_type = false;
        for group in &and_groups {
            if group.iter().any(|f| f.starts_with("project_type:")) {
                has_type = true;
                break;
            }
        }
        if !has_type {
            and_groups.push(vec!["project_type:mod".to_string()]);
        }

        if let Some(version) = &options.game_version {
            if !version.is_empty() {
                and_groups.push(vec![format!("versions:{}", version)]);
            }
        }

        if let Some(loader) = &options.loader {
            if !loader.is_empty() {
                and_groups.push(vec![format!("categories:{}", loader.to_lowercase())]);
            }
        }

        if !and_groups.is_empty() {
            let facets_json = serde_json::to_string(&and_groups)?;
            url.push_str(&format!("&facets={}", urlencoding::encode(&facets_json)));
        }

        if let Some(sort) = options.sort {
            let index = match sort {
                SortOrder::Relevance => "relevance",
                SortOrder::Downloads => "downloads",
                SortOrder::Follows => "follows",
                SortOrder::Newest => "newest",
                SortOrder::Updated => "updated",
            };
            url.push_str(&format!("&index={}", index));
        }

        if let Some(offset) = options.offset {
            url.push_str(&format!("&offset={}", offset));
        }

        if let Some(limit) = options.limit {
            url.push_str(&format!("&limit={}", limit));
        }

        let response_text = self.client.get(&url).send().await?.text().await?;
        let response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Modrinth response: {}. Body: {}", e, response_text))?;
        
        let hits = response["hits"].as_array().ok_or_else(|| anyhow!("Invalid response from Modrinth: missing 'hits' field"))?;
        
        let projects = hits.iter().map(|h| Project {
            id: h["project_id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
            author: h["author"].as_str().unwrap_or_default().to_string(),
            provider: ModProvider::Modrinth,
            categories: h["categories"].as_array().map(|cats| {
                cats.iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect()
            }),
        }).collect();

        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let url = format!("{}/project/{}", self.base_url, id);
        let h = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        Ok(Project {
            id: h["id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
            author: String::new(), // Author is in a separate field
            provider: ModProvider::Modrinth,
            categories: h["categories"].as_array().map(|cats| {
                cats.iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect()
            }),
        })
    }

    pub async fn get_dependencies(&self, project_id: &str) -> Result<Vec<Project>> {
        let url = format!("{}/project/{}/dependencies", self.base_url, project_id);
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
            provider: ModProvider::Modrinth,
            categories: h["categories"].as_array().map(|cats| {
                cats.iter().filter_map(|c| c.as_str().map(|s| s.to_string())).collect()
            }),
        }).collect();

        Ok(projects)
    }

    pub async fn get_versions(&self, project_id: &str) -> Result<Vec<ProjectVersion>> {
        let url = format!("{}/project/{}/version", self.base_url, project_id);
        let versions = self.client.get(&url).send().await?.json::<Vec<ProjectVersion>>().await?;
        Ok(versions)
    }

    pub async fn download_version(&self, version: &ProjectVersion, target_dir: impl AsRef<Path>) -> Result<String> {
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
        Ok(file.filename.clone())
    }
}