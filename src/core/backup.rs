use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use zip::write::SimpleFileOptions;
use std::fs::File;
use std::io::{Write, Read};
use tracing::info;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}

pub struct BackupManager {
    base_dir: PathBuf,
}

impl BackupManager {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    fn get_instance_backup_dir(&self, instance_id: Uuid) -> PathBuf {
        self.base_dir.join(instance_id.to_string())
    }

    pub async fn create_backup(&self, instance_id: Uuid, source_dir: impl AsRef<Path>, name: &str) -> Result<BackupInfo> {
        let source_dir = source_dir.as_ref().to_path_buf();
        let backup_dir = self.get_instance_backup_dir(instance_id);
        
        if !backup_dir.exists() {
            tokio::fs::create_dir_all(&backup_dir).await?;
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}_{}.zip", name, timestamp);
        let backup_path = backup_dir.join(backup_filename);

        info!("Starting backup of {:?} to {:?}", source_dir, backup_path);

        let backup_path_clone = backup_path.clone();
        tokio::task::spawn_blocking(move || {
            let file = File::create(&backup_path_clone).context("Failed to create backup file")?;
            let mut zip = zip::ZipWriter::new(file);
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);

            let mut buffer = Vec::new();

            for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                let name = path.strip_prefix(&source_dir)
                    .context("Failed to strip prefix")?
                    .to_str()
                    .context("Failed to convert path to string")?;

                if path.is_file() {
                    zip.start_file(name, options).context("Failed to start file in zip")?;
                    let mut f = File::open(path).context("Failed to open file for backup")?;
                    f.read_to_end(&mut buffer).context("Failed to read file into buffer")?;
                    zip.write_all(&buffer).context("Failed to write file to zip")?;
                    buffer.clear();
                } else if !name.is_empty() {
                    zip.add_directory(name, options).context("Failed to add directory to zip")?;
                }
            }

            zip.finish().context("Failed to finish zip file")?;
            Ok::<(), anyhow::Error>(())
        }).await??;

        let metadata = tokio::fs::metadata(&backup_path).await?;
        let info = BackupInfo {
            name: backup_path.file_name().unwrap().to_string_lossy().into_owned(),
            path: backup_path.clone(),
            size: metadata.len(),
            created_at: Utc::now(),
        };

        info!("Backup completed successfully: {:?}", backup_path);
        Ok(info)
    }

    pub async fn list_backups(&self, instance_id: Uuid) -> Result<Vec<BackupInfo>> {
        let backup_dir = self.get_instance_backup_dir(instance_id);
        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();
        let mut entries = tokio::fs::read_dir(&backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("zip") {
                let metadata = entry.metadata().await?;
                let created_at: DateTime<Utc> = metadata.created()?.into();
                
                backups.push(BackupInfo {
                    name: path.file_name().unwrap().to_string_lossy().into_owned(),
                    path,
                    size: metadata.len(),
                    created_at,
                });
            }
        }

        // Sort by creation date descending
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    pub async fn delete_backup(&self, instance_id: Uuid, name: &str) -> Result<()> {
        let backup_dir = self.get_instance_backup_dir(instance_id);
        let path = backup_dir.join(name);
        if path.exists() {
            tokio::fs::remove_file(path).await?;
            info!("Deleted backup: {}", name);
        }
        Ok(())
    }

    pub async fn restore_backup(&self, instance_id: Uuid, backup_name: &str, target_dir: impl AsRef<Path>) -> Result<()> {
        let backup_dir = self.get_instance_backup_dir(instance_id);
        let backup_path = backup_dir.join(backup_name);
        let target_dir = target_dir.as_ref().to_path_buf();

        if !backup_path.exists() {
            return Err(anyhow::anyhow!("Backup not found: {}", backup_name));
        }

        info!("Restoring backup {:?} to {:?}", backup_path, target_dir);

        // Clear target directory first (safely)
        // We might want to be careful here, maybe just delete common folders
        if target_dir.exists() {
            // Optional: Backup current state before restoring?
            // For now, just clear it as requested by typical restore behavior
            tokio::fs::remove_dir_all(&target_dir).await?;
        }
        tokio::fs::create_dir_all(&target_dir).await?;

        tokio::task::spawn_blocking(move || {
            let file = File::open(&backup_path).context("Failed to open backup file")?;
            let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).context("Failed to get file from archive")?;
                let outpath = match file.enclosed_name() {
                    Some(path) => target_dir.join(path),
                    None => continue,
                };

                if (*file.name()).ends_with('/') {
                    std::fs::create_dir_all(&outpath).context("Failed to create directory")?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).context("Failed to create parent directory")?;
                        }
                    }
                    let mut outfile = File::create(&outpath).context("Failed to create output file")?;
                    std::io::copy(&mut file, &mut outfile).context("Failed to copy file")?;
                }

                // Get and Set permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = file.unix_mode() {
                        std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).ok();
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        }).await??;

        info!("Restore completed successfully");
        Ok(())
    }
}
