use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use chrono::Utc;
use zip::write::SimpleFileOptions;
use std::fs::File;
use std::io::{Write, Read};
use tracing::info;
use walkdir::WalkDir;

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new(backup_dir: impl AsRef<Path>) -> Self {
        Self {
            backup_dir: backup_dir.as_ref().to_path_buf(),
        }
    }

    pub async fn create_backup(&self, source_dir: impl AsRef<Path>, name: &str) -> Result<PathBuf> {
        let source_dir = source_dir.as_ref().to_path_buf();
        if !self.backup_dir.exists() {
            tokio::fs::create_dir_all(&self.backup_dir).await?;
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}_{}.zip", name, timestamp);
        let backup_path = self.backup_dir.join(backup_filename);

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

        info!("Backup completed successfully: {:?}", backup_path);
        Ok(backup_path)
    }
}
