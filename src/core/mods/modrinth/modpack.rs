use super::ModrinthClient;
use crate::mods::types::ProjectVersion;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path;
use tokio::fs;
use zip::ZipArchive;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModpackProgress {
    pub current_step: String,
    pub progress: f32, // 0.0 to 1.0
    pub current_file: Option<String>,
    pub files_completed: Option<u32>,
    pub total_files: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthIndex {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<ModrinthIndexFile>,
    pub dependencies: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthIndexFile {
    pub path: String,
    pub hashes: std::collections::HashMap<String, String>,
    pub env: Option<ModrinthIndexEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModrinthEnvSupport {
    Required,
    Optional,
    Unsupported,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModrinthIndexEnv {
    pub client: ModrinthEnvSupport,
    pub server: ModrinthEnvSupport,
}

impl ModrinthClient {
    pub async fn install_modpack<F>(
        &self,
        instance_path: impl AsRef<Path>,
        version: &ProjectVersion,
        on_progress: F,
    ) -> Result<ModrinthIndex>
    where
        F: Fn(ModpackProgress) + Send + 'static,
    {
        let instance_path = instance_path.as_ref().to_path_buf();
        let temp_dir = instance_path.join(".temp_modpack");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).await?;
        }
        fs::create_dir_all(&temp_dir).await?;

        // 1. Download the .mrpack file
        on_progress(ModpackProgress {
            current_step: "Downloading modpack archive".to_string(),
            progress: 0.1,
            current_file: None,
            files_completed: None,
            total_files: None,
        });

        let primary_file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| anyhow!("No files found in modpack version"))?;

        let response = self
            .inner
            .cache
            .get_client()
            .get(&primary_file.url)
            .send()
            .await?;
        let bytes = response.bytes().await?;

        // 2. Extract the .mrpack file
        on_progress(ModpackProgress {
            current_step: "Extracting overrides".to_string(),
            progress: 0.3,
            current_file: None,
            files_completed: None,
            total_files: None,
        });

        let index = {
            let mut archive = ZipArchive::new(Cursor::new(bytes))?;

            // Read index first
            let index: ModrinthIndex = {
                let mut index_file = archive
                    .by_name("modrinth.index.json")
                    .context("modrinth.index.json not found in .mrpack")?;
                serde_json::from_reader(&mut index_file)?
            };

            // 3. Extract overrides
            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let name = file.name().to_string();

                if name.starts_with("overrides/") || name.starts_with("server-overrides/") {
                    let rel_path = if name.starts_with("overrides/") {
                        name.strip_prefix("overrides/").unwrap()
                    } else {
                        name.strip_prefix("server-overrides/").unwrap()
                    };

                    if rel_path.is_empty() {
                        continue;
                    }

                    let out_path = instance_path.join(rel_path);
                    if file.is_dir() {
                        std::fs::create_dir_all(&out_path)?;
                    } else {
                        if let Some(parent) = out_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        let mut out_file = std::fs::File::create(&out_path)?;
                        std::io::copy(&mut file, &mut out_file)?;
                    }
                }
            }
            index
        };

        // 4. Download files from index
        let total_files = index.files.len() as u32;
        for (i, mod_file) in index.files.iter().enumerate() {
            let i = i as u32;

            // Check if it's supported on server
            if let Some(env) = &mod_file.env {
                if matches!(env.server, ModrinthEnvSupport::Unsupported) {
                    continue;
                }
            }

            on_progress(ModpackProgress {
                current_step: format!("Downloading dependencies ({}/{})", i + 1, total_files),
                progress: 0.4 + (0.5 * (i as f32 / total_files as f32)),
                current_file: Some(mod_file.path.clone()),
                files_completed: Some(i),
                total_files: Some(total_files),
            });

            let dest_path = instance_path.join(&mod_file.path);
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            // Try downloads in order
            let mut success = false;
            for url in &mod_file.downloads {
                match self.inner.cache.get_client().get(url).send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            let mut out_file = fs::File::create(&dest_path).await?;
                            let mut content = Cursor::new(resp.bytes().await?);
                            tokio::io::copy(&mut content, &mut out_file).await?;
                            success = true;
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to download modpack file from {}: {}", url, e);
                    }
                }
            }

            if !success {
                return Err(anyhow!(
                    "Failed to download modpack file: {}",
                    mod_file.path
                ));
            }
        }

        on_progress(ModpackProgress {
            current_step: "Finishing installation".to_string(),
            progress: 1.0,
            current_file: None,
            files_completed: Some(total_files),
            total_files: Some(total_files),
        });

        // Clean up
        fs::remove_dir_all(&temp_dir).await.ok();

        Ok(index)
    }
}
