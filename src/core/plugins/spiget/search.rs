use anyhow::Result;
use super::SpigetClient;
use crate::plugins::types::{Project, PluginProvider, ResolvedDependency, SearchOptions};

impl SpigetClient {
    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let cache_key = format!("spiget_search_{}", options.cache_key());
        if let Ok(Some(cached)) = self.cache.get::<Vec<Project>>(&cache_key).await {
            return Ok(cached);
        }

        let size = options.limit.unwrap_or(20);
        let page = (options.offset.unwrap_or(0) / size) + 1;
        
        let url = if options.query.trim().is_empty() {
            if let Some(facets) = &options.facets {
                let category = facets.iter()
                    .find(|f| f.starts_with("categories:"))
                    .and_then(|f| f.strip_prefix("categories:"));
                
                if let Some(cat) = category {
                    let cat_id = match cat {
                        "chat" | "social" => "11",
                        "economy" => "12",
                        "gameplay" | "adventure" | "game-mechanics" | "magic" | "minigame" => "13",
                        "management" => "14",
                        "protection" => "15",
                        "utility" | "optimization" => "16",
                        "world-management" | "worldgen" => "17",
                        "library" => "19",
                        "admin" => "10",
                        "misc" => "18",
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
        
        let projects: Vec<Project> = json.into_iter().map(|h| Project {
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

        let _ = self.cache.set(cache_key, projects.clone()).await;
        Ok(projects)
    }

    pub async fn get_dependencies(&self, resource_id: &str) -> Result<Vec<ResolvedDependency>> {
        let cache_key = format!("spiget_dependencies_{}", resource_id);
        if let Ok(Some(cached)) = self.cache.get::<Vec<ResolvedDependency>>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/resources/{}/dependencies", self.base_url, resource_id);
        let response = self.client.get(&url).send().await?;
        
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }

        let response = response.error_for_status()?;
        let json = response.json::<Vec<serde_json::Value>>().await?;
        
        let mut resolved_deps = Vec::new();
        for dep in json {
            if let Some(id) = dep["id"].as_u64() {
                if let Ok(project) = self.get_project(&id.to_string()).await {
                    resolved_deps.push(ResolvedDependency {
                        project,
                        dependency_type: "required".to_string(),
                    });
                }
            }
        }

        let _ = self.cache.set(cache_key, resolved_deps.clone()).await;
        Ok(resolved_deps)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let cache_key = format!("spiget_project_{}", id);
        if let Ok(Some(cached)) = self.cache.get::<Project>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/resources/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let h = response.json::<serde_json::Value>().await?;
        
        let project = Project {
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
        };

        let _ = self.cache.set(cache_key, project.clone()).await;
        Ok(project)
    }

    pub async fn get_latest_version(&self, resource_id: &str) -> Result<(String, String)> {
        let url = format!("{}/resources/{}/versions/latest", self.base_url, resource_id);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let json = response.json::<serde_json::Value>().await?;
        
        let id = json["id"].as_u64().unwrap_or(0).to_string();
        let name = json["name"].as_str().unwrap_or_default().to_string();
        
        Ok((id, name))
    }
}
