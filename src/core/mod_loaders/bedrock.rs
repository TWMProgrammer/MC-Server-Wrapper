use anyhow::Result;
use super::ModLoaderClient;
use std::io::Cursor;
use zip::ZipArchive;
use std::path::Path;
use tokio::fs;

impl ModLoaderClient {
    pub async fn get_bedrock_versions(&self) -> Result<crate::downloader::VersionManifest> {
        // Check cache first (TTL 1 hour)
        if let Some(cache_dir) = &self.cache_dir {
            let cache_file = cache_dir.join("bedrock_manifest.json");
            if cache_file.exists() {
                if let Ok(metadata) = std::fs::metadata(&cache_file) {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().map(|e| e.as_secs() < 3600).unwrap_or(false) {
                            if let Ok(content) = std::fs::read_to_string(&cache_file) {
                                if let Ok(manifest) = serde_json::from_str::<crate::downloader::VersionManifest>(&content) {
                                    return Ok(manifest);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fetch versions from Bedrock-OSS/BDS-Versions repository
        let contents_url = "https://api.github.com/repos/Bedrock-OSS/BDS-Versions/contents/linux";
        
        #[derive(Debug, serde::Deserialize)]
        struct GitHubContent {
            name: String,
            r#type: String,
        }

        let contents_response = self.client.get(contents_url)
            .header("User-Agent", "Minecraft-Server-Wrapper")
            .send()
            .await?;

        if !contents_response.status().is_success() {
            // Fallback to some hardcoded versions if API fails
            let versions = vec![
                "1.21.60.10", "1.21.50.07", "1.21.44.01", "1.21.40.01"
            ];
            let version_infos = versions.into_iter().map(|v| crate::downloader::VersionInfo {
                id: v.to_string(),
                r#type: "release".to_string(),
                url: "".to_string(),
                release_date: chrono::Utc::now(),
            }).collect();

            return Ok(crate::downloader::VersionManifest {
                latest: crate::downloader::LatestVersions {
                    release: "1.21.60.10".to_string(),
                    snapshot: "1.21.60.10".to_string(),
                },
                versions: version_infos,
            });
        }

        let contents: Vec<GitHubContent> = contents_response.json().await?;
        let versions: Vec<String> = contents.into_iter()
            .filter(|c| c.r#type == "file" && c.name.ends_with(".json") && c.name != "versions.json")
            .map(|c| c.name.replace(".json", ""))
            .collect();

        // Fetch commits to get release dates
        let commits_url = "https://api.github.com/repos/Bedrock-OSS/BDS-Versions/commits?path=linux&per_page=100";
        
        #[derive(Debug, serde::Deserialize)]
        struct GitHubCommit {
            commit: CommitDetails,
        }
        #[derive(Debug, serde::Deserialize)]
        struct CommitDetails {
            message: String,
            committer: Committer,
        }
        #[derive(Debug, serde::Deserialize)]
        struct Committer {
            date: chrono::DateTime<chrono::Utc>,
        }

        let mut version_dates = std::collections::HashMap::new();
        if let Ok(commits_response) = self.client.get(commits_url)
            .header("User-Agent", "Minecraft-Server-Wrapper")
            .send()
            .await 
        {
            if commits_response.status().is_success() {
                if let Ok(commits) = commits_response.json::<Vec<GitHubCommit>>().await {
                    let re = regex::Regex::new(r"(\d+\.\d+\.\d+\.\d+)").unwrap();
                    for commit in commits {
                        if let Some(cap) = re.captures(&commit.commit.message) {
                            let version = cap[1].to_string();
                            version_dates.entry(version).or_insert(commit.commit.committer.date);
                        }
                    }
                }
            }
        }

        let mut version_infos: Vec<crate::downloader::VersionInfo> = versions.into_iter().map(|v| {
            let date = version_dates.get(&v).cloned().unwrap_or_else(|| chrono::Utc::now());
            crate::downloader::VersionInfo {
                id: v,
                r#type: "release".to_string(),
                url: "".to_string(),
                release_date: date,
            }
        }).collect();

        // Sort versions descending
        version_infos.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.id.split('.').filter_map(|p| p.parse().ok()).collect();
            let b_parts: Vec<u32> = b.id.split('.').filter_map(|p| p.parse().ok()).collect();
            b_parts.cmp(&a_parts)
        });

        let manifest = crate::downloader::VersionManifest {
            latest: crate::downloader::LatestVersions {
                release: version_infos.first().map(|v| v.id.clone()).unwrap_or_default(),
                snapshot: version_infos.first().map(|v| v.id.clone()).unwrap_or_default(),
            },
            versions: version_infos,
        };

        // Save to cache
        if let Some(cache_dir) = &self.cache_dir {
            let _ = std::fs::create_dir_all(cache_dir);
            let cache_file = cache_dir.join("bedrock_manifest.json");
            if let Ok(content) = serde_json::to_string_pretty(&manifest) {
                let _ = std::fs::write(cache_file, content);
            }
        }

        Ok(manifest)
    }

    pub async fn download_bedrock<F>(&self, version: &str, target_dir: impl AsRef<Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        let os = if cfg!(windows) { "win" } else { "linux" };
        let url = format!("https://minecraft.azureedge.net/bin-{}/bedrock-server-{}.zip", os, version);
        
        let temp_zip = target_dir.as_ref().join("bedrock-server.zip");
        self.download_with_progress(&url, &temp_zip, on_progress).await?;

        // Extract ZIP
        let zip_content = fs::read(&temp_zip).await?;
        let mut archive = ZipArchive::new(Cursor::new(zip_content))?;
        
        for i in 0..archive.len() {
            let (_name, is_dir, outpath) = {
                let file = archive.by_index(i)?;
                let name = file.name().to_string();
                let is_dir = name.ends_with('/');
                let outpath = match file.enclosed_name() {
                    Some(path) => target_dir.as_ref().join(path),
                    None => continue,
                };
                (name, is_dir, outpath)
            };

            if is_dir {
                fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).await?;
                    }
                }
                
                // Extract file content in a separate block to ensure ZipFile is dropped
                let mut content = Vec::new();
                {
                    let mut file = archive.by_index(i)?;
                    std::io::copy(&mut file, &mut content)?;
                }
                
                fs::write(&outpath, content).await?;
            }
        }

        // Clean up ZIP
        fs::remove_file(temp_zip).await?;

        Ok(())
    }
}
