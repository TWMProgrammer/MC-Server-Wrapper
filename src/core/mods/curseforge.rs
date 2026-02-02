use super::types::{Project, ModProvider, SearchOptions, SortOrder, ProjectVersion, ProjectFile};
use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

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

    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        
        let mut query_params = vec![
            ("gameId", "432".to_string()), // Minecraft
            ("searchFilter", options.query.clone()),
            ("classId", "6".to_string()), // Mods
        ];

        if let Some(sort) = options.sort {
            let sort_field = match sort {
                SortOrder::Relevance => "0", // Featured
                SortOrder::Downloads => "2", // Popularity
                SortOrder::Follows => "2", // Popularity (closest match)
                SortOrder::Newest => "1", // Newest
                SortOrder::Updated => "3", // Updated
            };
            query_params.push(("sortField", sort_field.to_string()));
            query_params.push(("sortOrder", "desc".to_string()));
        }

        if let Some(offset) = options.offset {
            query_params.push(("index", offset.to_string()));
        }

        if let Some(limit) = options.limit {
            query_params.push(("pageSize", limit.to_string()));
        }

        if let Some(version) = &options.game_version {
            query_params.push(("gameVersion", version.clone()));
        }

        if let Some(loader) = &options.loader {
            let loader_type = match loader.to_lowercase().as_str() {
                "forge" => "1",
                "fabric" => "4",
                "quilt" => "5",
                "neoforge" => "6",
                _ => "0",
            };
            if loader_type != "0" {
                query_params.push(("modLoaderType", loader_type.to_string()));
            }
        }

        let url = "https://api.curseforge.com/v1/mods/search";
        let response = self.client.get(url)
            .header("x-api-key", api_key)
            .query(&query_params)
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
            categories: h["categories"].as_array().map(|cats| {
                cats.iter().filter_map(|c| c["name"].as_str().map(|s| s.to_string())).collect()
            }),
        }).collect();

        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        let url = format!("https://api.curseforge.com/v1/mods/{}", id);
        let response = self.client.get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let h = &response["data"];
        Ok(Project {
            id: h["id"].as_u64().unwrap_or(0).to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["name"].as_str().unwrap_or_default().to_string(),
            description: h["summary"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloadCount"].as_u64().unwrap_or(0),
            icon_url: h["logo"]["url"].as_str().map(|s| s.to_string()),
            author: h["authors"].as_array().and_then(|a| a.first()).and_then(|a| a["name"].as_str()).unwrap_or("Unknown").to_string(),
            provider: ModProvider::CurseForge,
            categories: h["categories"].as_array().map(|cats| {
                cats.iter().filter_map(|c| c["name"].as_str().map(|s| s.to_string())).collect()
            }),
        })
    }

    pub async fn get_dependencies(&self, project_id: &str) -> Result<Vec<Project>> {
        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        let url = format!("https://api.curseforge.com/v1/mods/{}", project_id);
        let response = self.client.get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let dependencies = response["data"]["dependencies"].as_array();
        if let Some(deps) = dependencies {
            let mut project_deps = Vec::new();
            for dep in deps {
                // Dependency type 3 is "Required"
                if dep["relationType"].as_u64() == Some(3) {
                    if let Some(mod_id) = dep["modId"].as_u64() {
                        if let Ok(project) = self.get_project(&mod_id.to_string()).await {
                            project_deps.push(project);
                        }
                    }
                }
            }
            Ok(project_deps)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_versions(&self, project_id: &str) -> Result<Vec<ProjectVersion>> {
        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        let url = format!("https://api.curseforge.com/v1/mods/{}/files", project_id);
        let response = self.client.get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let data = response["data"].as_array().ok_or_else(|| anyhow!("Invalid response from CurseForge"))?;
        
        let versions = data.iter().map(|v| ProjectVersion {
            id: v["id"].as_u64().unwrap_or(0).to_string(),
            project_id: project_id.to_string(),
            version_number: v["displayName"].as_str().unwrap_or_default().to_string(),
            files: vec![ProjectFile {
                url: v["downloadUrl"].as_str().unwrap_or_default().to_string(),
                filename: v["fileName"].as_str().unwrap_or_default().to_string(),
                primary: true,
                size: v["fileLength"].as_u64().unwrap_or(0),
            }],
            loaders: v["gameVersions"].as_array().map(|gv| {
                gv.iter().filter_map(|s: &serde_json::Value| s.as_str().map(|s| s.to_string()))
                  .filter(|s: &String| ["Forge", "Fabric", "Quilt", "NeoForge"].contains(&s.as_str()))
                  .collect()
            }).unwrap_or_default(),
            game_versions: v["gameVersions"].as_array().map(|gv| {
                gv.iter().filter_map(|s: &serde_json::Value| s.as_str().map(|s| s.to_string()))
                  .filter(|s: &String| s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                  .collect()
            }).unwrap_or_default(),
        }).collect();

        Ok(versions)
    }

    pub async fn download_mod(&self, _mod_id: &str, file_id: &str, _target_dir: impl AsRef<Path>) -> Result<String> {
        let _api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        
        // In CurseForge v1, we can get file info by file_id directly if we have the mod_id, 
        // but often we just need the download URL from the version info.
        // For simplicity, let's assume we get the version first.
        
        // If we only have file_id, we need to fetch it.
        let _url = format!("https://api.curseforge.com/v1/mods/0/files/{}", file_id); // modId 0 works for some endpoints
        // Actually, the v1 API requires modId. Let's assume we have it or use a different endpoint.
        // Wait, the version already has the download URL.
        
        // Let's implement a more robust download if we only have IDs.
        // But for now, let's assume we use the download_url from ProjectFile.
        
        Err(anyhow!("Use download_file instead with the URL from ProjectFile"))
    }

    pub async fn download_file(&self, url: &str, filename: &str, target_dir: impl AsRef<Path>) -> Result<String> {
        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let target_path = target_dir.as_ref().join(filename);
        let response = self.client.get(url).send().await?.error_for_status()?;
        
        let mut f = fs::File::create(&target_path).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            f.write_all(&chunk).await?;
        }

        f.flush().await?;
        Ok(filename.to_string())
    }
}
