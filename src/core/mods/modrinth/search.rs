use anyhow::{Result, anyhow};
use std::time::Duration;
use crate::utils::retry_async;
use super::ModrinthClient;
use crate::mods::types::{Project, SearchOptions, SortOrder, ModProvider};

impl ModrinthClient {
    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let cache_key = format!("modrinth_mods_search_{}", options.cache_key());
        if let Ok(Some(cached)) = self.cache.get::<Vec<Project>>(&cache_key).await {
            return Ok(cached);
        }

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

        let response_text: String = retry_async(
            || async {
                let res = self.client.get(&url)
                    .send()
                    .await?;
                let text = res.text()
                    .await
                    .map_err(|e| anyhow!(e))?;
                Ok(text)
            },
            3,
            Duration::from_secs(2),
            &format!("Modrinth search: {}", options.query)
        ).await?;

        let response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Modrinth response: {}. Body: {}", e, response_text))?;
        
        let hits = response["hits"].as_array().ok_or_else(|| anyhow!("Invalid response from Modrinth: missing 'hits' field"))?;
        
        let projects: Vec<Project> = hits.iter().map(|h| Project {
            id: h["project_id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s: &str| s.to_string()),
            author: h["author"].as_str().unwrap_or_default().to_string(),
            provider: ModProvider::Modrinth,
            categories: h["categories"].as_array().map(|cats: &Vec<serde_json::Value>| {
                cats.iter().filter_map(|c: &serde_json::Value| c.as_str().map(|s: &str| s.to_string())).collect()
            }),
        }).collect();

        let _ = self.cache.set(cache_key, projects.clone()).await;
        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let cache_key = format!("modrinth_mods_project_{}", id);
        if let Ok(Some(cached)) = self.cache.get::<Project>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/project/{}", self.base_url, id);
        let h = retry_async(
            || async {
                self.client.get(&url)
                    .send()
                    .await?
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            &format!("Get Modrinth project: {}", id)
        ).await?;
        
        let project = Project {
            id: h["id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s: &str| s.to_string()),
            author: String::new(), // Author is in a separate field
            provider: ModProvider::Modrinth,
            categories: h["categories"].as_array().map(|cats: &Vec<serde_json::Value>| {
                cats.iter().filter_map(|c: &serde_json::Value| c.as_str().map(|s: &str| s.to_string())).collect()
            }),
        };

        let _ = self.cache.set(cache_key, project.clone()).await;
        Ok(project)
    }
}
