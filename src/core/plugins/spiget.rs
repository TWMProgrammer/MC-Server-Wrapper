use anyhow::Result;
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use super::types::{Project, PluginProvider, ResolvedDependency};

pub struct SpigetClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for SpigetClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SpigetClient {
    pub fn new() -> Self {
        Self::with_base_url("https://api.spiget.org/v2".to_string())
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

    pub async fn search(&self, options: &super::types::SearchOptions) -> Result<Vec<Project>> {
        let size = options.limit.unwrap_or(20);
        let page = (options.offset.unwrap_or(0) / size) + 1;
        
        let url = if options.query.trim().is_empty() {
            if let Some(facets) = &options.facets {
                let category = facets.iter()
                    .find(|f| f.starts_with("categories:"))
                    .and_then(|f| f.strip_prefix("categories:"));
                
                if let Some(cat) = category {
                    // Map common category names to Spiget IDs if possible, 
                    // or assume the facet is already the ID
                    let cat_id = match cat {
                        "administration" => "10",
                        "chat" => "11",
                        "economy" => "12",
                        "gameplay" => "13",
                        "management" => "14",
                        "utility" => "16",
                        "world-management" => "17",
                        _ => cat
                    };
                    format!("{}/categories/{}/resources?size={}&page={}&sort=-downloads", self.base_url, cat_id, size, page)
                } else {
                    format!("{}/resources?size={}&page={}&sort=-downloads", self.base_url, size, page)
                }
            } else {
                format!("{}/resources?size={}&page={}&sort=-downloads", self.base_url, size, page)
            }
        } else {
            format!("{}/search/resources/{}?field=name&size={}&page={}", 
                self.base_url, urlencoding::encode(&options.query), size, page)
        };

        let response = self.client.get(&url).send().await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }

        let response = response.error_for_status()?;
        let json = response.json::<Vec<serde_json::Value>>().await?;
        
        let projects = json.into_iter().map(|h| Project {
            id: h["id"].as_u64().unwrap_or(0).to_string(),
            slug: h["name"].as_str().unwrap_or_default().to_lowercase().replace(' ', "-"),
            title: h["name"].as_str().unwrap_or_default().to_string(),
            description: h["tag"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon"]["url"].as_str()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    if s.starts_with("http") {
                        s.to_string()
                    } else {
                        format!("https://www.spigotmc.org/{}", s)
                    }
                }),
            author: format!("User {}", h["author"]["id"]),
            provider: PluginProvider::Spiget,
        }).collect();

        Ok(projects)
    }

    pub async fn get_dependencies(&self, resource_id: &str) -> Result<Vec<ResolvedDependency>> {
        let url = format!("{}/resources/{}/dependencies", self.base_url, resource_id);
        let response = self.client.get(&url).send().await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }

        let response = response.error_for_status()?;
        let json = response.json::<Vec<serde_json::Value>>().await?;
        
        let mut resolved_deps = Vec::new();
        for dep in json {
            // Spiget dependencies can be internal (numeric ID) or external (URL/Name)
            // For internal ones, we can fetch the project info
            if let Some(id) = dep["id"].as_u64() {
                if let Ok(project) = self.get_project(&id.to_string()).await {
                    resolved_deps.push(ResolvedDependency {
                        project,
                        dependency_type: "required".to_string(), // Spiget doesn't specify, assume required
                    });
                }
            }
        }

        Ok(resolved_deps)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let url = format!("{}/resources/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let h = response.json::<serde_json::Value>().await?;
        
        Ok(Project {
            id: h["id"].as_u64().unwrap_or(0).to_string(),
            slug: h["name"].as_str().unwrap_or_default().to_lowercase().replace(' ', "-"),
            title: h["name"].as_str().unwrap_or_default().to_string(),
            description: h["tag"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon"]["url"].as_str()
                .filter(|s| !s.is_empty())
                .map(|s| {
                    if s.starts_with("http") {
                        s.to_string()
                    } else {
                        format!("https://www.spigotmc.org/{}", s)
                    }
                }),
            author: format!("User {}", h["author"]["id"]),
            provider: PluginProvider::Spiget,
        })
    }

    pub async fn get_latest_version(&self, resource_id: &str) -> Result<(String, String)> {
        let url = format!("{}/resources/{}/versions/latest", self.base_url, resource_id);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let json = response.json::<serde_json::Value>().await?;
        
        let id = json["id"].as_u64().unwrap_or(0).to_string();
        let name = json["name"].as_str().unwrap_or_default().to_string();
        
        Ok((id, name))
    }

    pub async fn download_resource(&self, resource_id: &str, target_dir: impl AsRef<Path>) -> Result<String> {
        let url = format!("{}/resources/{}/download", self.base_url, resource_id);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        
        // Some resources might redirect to an external site that doesn't return a JAR
        // We should at least check the content type if possible, but for now let's just proceed
        
        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let filename = format!("spigot-resource-{}.jar", resource_id);
        let target_path = target_dir.as_ref().join(&filename);

        let mut f = fs::File::create(&target_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            f.write_all(&chunk).await?;
            downloaded += chunk.len();
        }

        f.flush().await?;
        
        if downloaded == 0 {
            return Err(anyhow::anyhow!("Downloaded file is empty. This plugin might require a manual download or be blocked by Cloudflare."));
        }

        Ok(filename)
    }
}
