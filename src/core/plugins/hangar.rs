use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;
use super::types::{Project, ProjectVersion, ProjectFile, PluginProvider, ResolvedDependency};

pub struct HangarClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for HangarClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HangarClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
            base_url: "https://hangar.papermc.io/api/v1".to_string(),
        }
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
        let mut url = format!("{}/projects?q={}", self.base_url, urlencoding::encode(&options.query));

        if let Some(offset) = options.offset {
            url.push_str(&format!("&offset={}", offset));
        }

        if let Some(limit) = options.limit {
            url.push_str(&format!("&limit={}", limit));
        }

        // Hangar doesn't have a direct "sort" parameter that matches ours exactly, 
        // but it has 'sort' parameter with values like 'stars', 'downloads', 'newest', 'updated', 'recent_views', 'recent_downloads'
        if let Some(sort) = options.sort {
            let sort_val = match sort {
                super::types::SortOrder::Relevance => "recent_downloads", // Default to recent downloads for relevance
                super::types::SortOrder::Downloads => "downloads",
                super::types::SortOrder::Follows => "stars",
                super::types::SortOrder::Newest => "newest",
                super::types::SortOrder::Updated => "updated",
            };
            url.push_str(&format!("&sort={}", sort_val));
        }

        let response: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        
        let result = response["result"].as_array().ok_or_else(|| anyhow!("Invalid response from Hangar: missing 'result' field"))?;
        
        let projects = result.iter().map(|h| {
            let owner = h["namespace"]["owner"].as_str().unwrap_or_default();
            let slug = h["namespace"]["slug"].as_str().unwrap_or_default();
            Project {
                // Use owner/slug as ID for Hangar to make API calls easier later
                id: format!("{}/{}", owner, slug),
                slug: slug.to_string(),
                title: h["name"].as_str().unwrap_or_default().to_string(),
                description: h["description"].as_str().unwrap_or_default().to_string(),
                downloads: h["stats"]["downloads"].as_u64().unwrap_or(0),
                icon_url: h["avatarUrl"].as_str().map(|s| s.to_string()),
                author: owner.to_string(),
                provider: PluginProvider::Hangar,
            }
        }).collect();

        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let url = format!("{}/projects/{}", self.base_url, id);
        let h: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        
        let owner = h["namespace"]["owner"].as_str().unwrap_or_default();
        let slug = h["namespace"]["slug"].as_str().unwrap_or_default();

        Ok(Project {
            id: id.to_string(),
            slug: slug.to_string(),
            title: h["name"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["stats"]["downloads"].as_u64().unwrap_or(0),
            icon_url: h["avatarUrl"].as_str().map(|s| s.to_string()),
            author: owner.to_string(),
            provider: PluginProvider::Hangar,
        })
    }

    pub async fn get_dependencies(&self, project_id: &str) -> Result<Vec<ResolvedDependency>> {
        // Hangar dependencies are listed per version. We'll get the latest version and its dependencies.
        let url = format!("{}/projects/{}/versions?limit=1", self.base_url, project_id);
        let response: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        
        let result = response["result"].as_array().ok_or_else(|| anyhow!("Invalid response from Hangar"))?;
        if result.is_empty() {
            return Ok(vec![]);
        }

        let latest_version = &result[0];
        let mut resolved_deps = Vec::new();

        // Check for plugin dependencies in PAPER platform
        if let Some(plugin_deps) = latest_version["pluginDependencies"].get("PAPER") {
            if let Some(deps_array) = plugin_deps.as_array() {
                for dep in deps_array {
                    let name = dep["name"].as_str().unwrap_or("Unknown").to_string();
                    let required = dep["required"].as_bool().unwrap_or(true);
                    let dependency_type = if required { "required" } else { "optional" }.to_string();

                    if let Some(namespace) = dep.get("namespace") {
                        let owner = namespace["owner"].as_str().unwrap_or_default();
                        let slug = namespace["slug"].as_str().unwrap_or_default();
                        
                        if !owner.is_empty() && !slug.is_empty() {
                            let dep_project_id = format!("{}/{}", owner, slug);
                            // We could fetch the full project info here, but for performance 
                            // we'll try to use what we have or just fetch it if needed.
                            // Since ResolvedDependency needs a full Project object, 
                            // let's fetch it to be safe and consistent with other providers.
                            if let Ok(project) = self.get_project(&dep_project_id).await {
                                resolved_deps.push(ResolvedDependency {
                                    project,
                                    dependency_type,
                                });
                            } else {
                                // Fallback to partial project if fetch fails
                                resolved_deps.push(ResolvedDependency {
                                    project: Project {
                                        id: dep_project_id,
                                        slug: slug.to_string(),
                                        title: name,
                                        description: String::new(),
                                        downloads: 0,
                                        icon_url: None,
                                        author: owner.to_string(),
                                        provider: PluginProvider::Hangar,
                                    },
                                    dependency_type,
                                });
                            }
                        }
                    } else if let Some(external_url) = dep["externalUrl"].as_str() {
                        // External dependency
                        resolved_deps.push(ResolvedDependency {
                            project: Project {
                                id: external_url.to_string(),
                                slug: name.to_lowercase().replace(' ', "-"),
                                title: name,
                                description: format!("External dependency: {}", external_url),
                                downloads: 0,
                                icon_url: None,
                                author: "External".to_string(),
                                provider: PluginProvider::Hangar,
                            },
                            dependency_type,
                        });
                    }
                }
            }
        }

        Ok(resolved_deps)
    }

    pub async fn get_versions(
        &self,
        project_id: &str,
        _game_version: Option<&str>,
        _loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let url = format!("{}/projects/{}/versions", self.base_url, project_id);
        let response: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        
        let result = response["result"].as_array().ok_or_else(|| anyhow!("Invalid response from Hangar"))?;
        
        let mut versions = Vec::new();
        for v in result {
            let version_name = v["name"].as_str().unwrap_or_default().to_string();
            
            let mut files = Vec::new();
            
            // Hangar can have multiple platforms (PAPER, WATERFALL, VELOCITY)
            // We'll look for PAPER first as it's the most common
            if let Some(paper_downloads) = v["downloads"].get("PAPER") {
                let download_url = paper_downloads["downloadUrl"].as_str();
                let external_url = paper_downloads["externalUrl"].as_str();
                
                if let Some(url) = download_url.or(external_url) {
                    files.push(ProjectFile {
                        url: url.to_string(),
                        filename: format!("{}-{}.jar", project_id.replace('/', "-"), version_name),
                        primary: true,
                        size: 0, // Hangar doesn't always provide size in this view
                    });
                }
            }

            if files.is_empty() {
                // Try other platforms if PAPER is not found
                for (_platform, platform_downloads) in v["downloads"].as_object().unwrap() {
                    let download_url = platform_downloads["downloadUrl"].as_str();
                    let external_url = platform_downloads["externalUrl"].as_str();
                    
                    if let Some(url) = download_url.or(external_url) {
                        files.push(ProjectFile {
                            url: url.to_string(),
                            filename: format!("{}-{}.jar", project_id.replace('/', "-"), version_name),
                            primary: true,
                            size: 0,
                        });
                        break; 
                    }
                }
            }

            if !files.is_empty() {
                versions.push(ProjectVersion {
                    id: version_name.clone(),
                    project_id: project_id.to_string(),
                    version_number: version_name,
                    files,
                    loaders: vec!["paper".to_string()], // Hangar is primarily for Paper
                    game_versions: vec![], // Hangar has complex game version mapping, leaving empty for now
                });
            }
        }

        Ok(versions)
    }

    pub async fn download_version(&self, version: &ProjectVersion, target_dir: impl AsRef<Path>) -> Result<String> {
        let file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first())
            .ok_or_else(|| anyhow!("No files found for version"))?;

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        // Hangar's downloadUrl might be a relative path or need a base URL
        let download_url = if file.url.starts_with('/') {
            format!("https://hangar.papermc.io{}", file.url)
        } else {
            file.url.clone()
        };

        let target_path = target_dir.as_ref().join(&file.filename);
        info!("Downloading plugin from {}: {}", download_url, file.filename);

        let response = self.client.get(&download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download plugin: HTTP {}", response.status()));
        }

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
