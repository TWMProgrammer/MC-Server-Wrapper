use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE, USER_AGENT};
use regex::Regex;
use super::types::{Project, PluginProvider, ResolvedDependency};
use crate::cache::CacheManager;

pub struct SpigetClient {
    client: reqwest::Client,
    base_url: String,
    cache: Arc<CacheManager>,
}

impl SpigetClient {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self::with_base_url("https://api.spiget.org/v2".to_string(), cache)
    }

    pub fn with_base_url(base_url: String, cache: Arc<CacheManager>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("Failed to create reqwest client"),
            base_url,
            cache,
        }
    }

    pub async fn search(&self, options: &super::types::SearchOptions) -> Result<Vec<Project>> {
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
                    // Map common category names to Spiget IDs if possible, 
                    // or assume the facet is already the ID
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

    pub async fn download_resource(
        &self,
        resource_id: &str,
        target_dir: impl AsRef<Path>,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<String> {
        // Fetch project info first to check for external downloads and get the slug
        let url = format!("{}/resources/{}", self.base_url, resource_id);
        let info_response = self.client.get(&url).send().await?.error_for_status()?;
        let info_json = info_response.json::<serde_json::Value>().await?;

        let title = info_json["name"].as_str().unwrap_or("Unknown Plugin");
        let slug = info_json["name"]
            .as_str()
            .unwrap_or("plugin")
            .to_lowercase()
            .replace(' ', "-");

        if info_json["file"]["type"].as_str() == Some("external") {
            let external_url = info_json["file"]["external_url"]
                .as_str()
                .or_else(|| info_json["file"]["externalUrl"].as_str())
                .unwrap_or("unknown");

            if external_url.contains("github.com") {
                return self
                    .download_from_github(external_url, target_dir, title, game_version, loader)
                    .await;
            }

            return Err(anyhow::anyhow!(
                "Plugin '{}' (ID: {}) has an external download URL: {}. \
                Spiget cannot automatically download external resources. \
                Please download it manually and place it in the plugins folder.",
                title, resource_id, external_url
            ));
        }

        let download_url = format!("{}/resources/{}/download", self.base_url, resource_id);
        let response = self.client.get(&download_url).send().await?.error_for_status()?;
        
        // Check content type to ensure we're not downloading an error page or JSON
        let content_type = response.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
            
        if content_type.contains("text/html") || content_type.contains("application/json") {
             return Err(anyhow::anyhow!(
                "Spiget returned an unexpected content type for '{}': {}. \
                This usually means the resource is blocked by Cloudflare, requires a manual download, or redirected to an external site. \
                Try downloading it manually from SpigotMC.",
                title, content_type
            ));
        }

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        // Try to extract filename from Content-Disposition header
        let mut filename = None;
        if let Some(cd) = response.headers().get(CONTENT_DISPOSITION) {
            if let Ok(cd_str) = cd.to_str() {
                // Look for filename="name.jar" or filename=name.jar
                if let Some(f) = cd_str.split(';')
                    .find(|p| p.trim().starts_with("filename="))
                    .map(|p| p.trim().trim_start_matches("filename=").trim_matches('"')) {
                    if !f.is_empty() {
                        filename = Some(f.to_string());
                    }
                }
            }
        }

        // Fallback to slug-based filename if header is missing or invalid
        let filename = filename.unwrap_or_else(|| {
            let ext = info_json["file"]["type"].as_str().unwrap_or(".jar").trim_start_matches('.');
            format!("{}.{}", slug, ext)
        });

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
            return Err(anyhow::anyhow!("Downloaded file for '{}' is empty. This plugin might require a manual download or be blocked.", title));
        }

        Ok(filename)
    }

    async fn download_from_github(
        &self,
        url: &str,
        target_dir: impl AsRef<Path>,
        title: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<String> {
        let re = Regex::new(r"github\.com/([^/]+)/([^/]+)")?;
        let caps = re
            .captures(url)
            .ok_or_else(|| anyhow::anyhow!("Invalid GitHub URL: {}", url))?;

        let owner = &caps[1];
        let repo = &caps[2].trim_end_matches(".git");

        // Try to find a tag in the URL (e.g., /releases/tag/v1.0.0 or /releases/download/v1.0.0/...)
        let tag_re = Regex::new(r"/releases/(?:tag|download)/([^/]+)")?;
        let tag = tag_re.captures(url).map(|c| c[1].to_string());

        let api_url = match tag {
            Some(t) => format!(
                "https://api.github.com/repos/{}/{}/releases/tags/{}",
                owner, repo, t
            ),
            None => format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                owner, repo
            ),
        };

        let response = self
            .client
            .get(&api_url)
            .header(USER_AGENT, concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch release info from GitHub for '{}' ({}): {}. \
                The plugin might not have a public release or the URL is invalid.",
                title, api_url, response.status()
            ));
        }

        let release: serde_json::Value = response.json().await?;
        let assets = release["assets"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No assets found in GitHub release for '{}'", title))?;

        // Find the best asset:
        // 1. Ends in .jar and matches loader/version if provided
        // 2. Ends in .jar and contains the title (case-insensitive)
        // 3. Ends in .jar and contains the repo name (case-insensitive)
        // 4. Just the first .jar found
        let title_l = title.to_lowercase();
        let repo_l = repo.to_lowercase();
        let loader_l = loader.map(|l| l.to_lowercase());
        let gv_l = game_version.map(|gv| gv.to_lowercase());

        // Platform strings to exclude if they don't match the current loader
        let other_loaders = ["fabric", "forge", "neoforge", "quilt", "bungeecord", "velocity"];
        let exclusion_list: Vec<&str> = other_loaders
            .iter()
            .filter(|&&l| loader_l.as_ref().map_or(true, |curr| curr != l))
            .copied()
            .collect();

        // 1. Filter out non-jar and forbidden assets (wrong loader, "mod" strings on Paper, etc.)
        let candidates: Vec<&serde_json::Value> = assets
            .iter()
            .filter(|a| {
                let name = a["name"].as_str().unwrap_or_default().to_lowercase();
                if !name.ends_with(".jar") {
                    return false;
                }

                // Strictly exclude other loaders
                for excluded in &exclusion_list {
                    if name.contains(excluded) {
                        return false;
                    }
                }

                // Strictly exclude "mod" version if we are on a plugin loader (Paper/Spigot)
                if loader_l.as_ref().map_or(true, |l| l == "paper" || l == "spigot") {
                    if name.contains("-mod-") || name.contains(".mod.") {
                        return false;
                    }
                }

                true
            })
            .collect();

        // 2. Prioritized search among valid candidates
        let asset = candidates
            .iter()
            .find(|a| {
                let name = a["name"].as_str().unwrap_or_default().to_lowercase();
                let loader_match = loader_l.as_ref().map_or(true, |l| name.contains(l));
                let gv_match = gv_l.as_ref().map_or(true, |gv| name.contains(gv));
                loader_match && gv_match && (name.contains(&title_l) || name.contains(&repo_l))
            })
            .or_else(|| {
                // Second pass: match loader and version
                candidates.iter().find(|a| {
                    let name = a["name"].as_str().unwrap_or_default().to_lowercase();
                    let loader_match = loader_l.as_ref().map_or(true, |l| name.contains(l));
                    let gv_match = gv_l.as_ref().map_or(true, |gv| name.contains(gv));
                    loader_match && gv_match
                })
            })
            .or_else(|| {
                // Third pass: match title or repo
                candidates.iter().find(|a| {
                    let name = a["name"].as_str().unwrap_or_default().to_lowercase();
                    name.contains(&title_l) || name.contains(&repo_l)
                })
            })
            .or_else(|| candidates.first()) // Fallback to first valid candidate
            .copied()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No suitable JAR asset found in GitHub release for '{}' after filtering.",
                    title
                )
            })?;

        let download_url = asset["browser_download_url"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing download URL for asset in GitHub release"))?;
        let filename = asset["name"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing name for asset in GitHub release"))?
            .to_string();

        let response = self.client.get(download_url)
            .header(USER_AGENT, concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
            .send()
            .await?.error_for_status()?;

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

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
            return Err(anyhow::anyhow!("Downloaded GitHub asset for '{}' is empty.", title));
        }

        Ok(filename)
    }
}
