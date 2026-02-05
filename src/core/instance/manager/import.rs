use super::InstanceManager;
use crate::instance::archive::{copy_dir_all, extract_7z, extract_zip};
use crate::instance::types::{InstanceMetadata, InstanceSettings};
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info};
use uuid::Uuid;

impl InstanceManager {
    pub async fn import_instance<F>(
        &self,
        name: &str,
        source_path: PathBuf,
        jar_name: String,
        mod_loader: Option<String>,
        root_within_zip: Option<String>,
        script_path: Option<String>,
        on_progress: F,
    ) -> Result<InstanceMetadata>
    where
        F: Fn(u64, u64, String) + Send + Sync + 'static,
    {
        let id = Uuid::new_v4();
        let instance_path = self.base_dir.join(id.to_string());
        fs::create_dir_all(&instance_path).await?;

        if source_path.is_dir() {
            copy_dir_all(&source_path, &instance_path, on_progress).await?;
        } else if source_path.is_file() {
            let extension = source_path
                .extension()
                .map_or("", |ext| ext.to_str().unwrap_or(""))
                .to_lowercase();
            if extension == "zip" {
                extract_zip(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else if extension == "7z" {
                extract_7z(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else {
                return Err(anyhow::anyhow!(
                    "Unsupported archive format: .{}",
                    extension
                ));
            }
        } else {
            return Err(anyhow::anyhow!(
                "Source path must be a directory or a supported archive file (.zip, .7z)"
            ));
        }

        let mut settings = InstanceSettings::default();

        // Parse script if provided
        if let Some(script) = script_path {
            let script_full_path = instance_path.join(&script);
            if script_full_path.exists() {
                if let Ok(content) = fs::read_to_string(script_full_path).await {
                    if let Some((min, min_u, max, max_u)) = self.parse_ram_from_script(&content) {
                        settings.min_ram = min;
                        settings.min_ram_unit = min_u;
                        settings.max_ram = max;
                        settings.max_ram_unit = max_u;
                    }
                }
            }
        }

        settings.startup_line = format!(
            "java -Xms{{min_ram}}{{min_unit}} -Xmx{{max_ram}}{{max_unit}} -jar {} nogui",
            jar_name
        );

        // Check for server-icon.png
        let icon_path = instance_path.join("server-icon.png");
        if icon_path.exists() {
            settings.icon_path = Some(icon_path.to_string_lossy().to_string());
        }

        let version = self
            .detect_minecraft_version(&instance_path, &jar_name)
            .await;

        let metadata = InstanceMetadata {
            id,
            name: name.to_string(),
            version,
            mod_loader,
            loader_version: None,
            created_at: Utc::now(),
            last_run: None,
            path: instance_path,
            schedules: vec![],
            settings,
            status: crate::server::types::ServerStatus::Stopped,
            ip: None,
            port: None,
            max_players: None,
            description: None,
        };

        self.save_instance_to_db(&metadata).await?;

        info!("Imported instance: {} (ID: {})", name, id);

        // Double check it's in the DB
        match self.get_instance(id).await {
            Ok(Some(_)) => info!("Verified instance {} exists in DB after import", name),
            Ok(None) => error!(
                "CRITICAL: Instance {} NOT found in DB immediately after save!",
                name
            ),
            Err(e) => error!("Error verifying instance {} in DB: {}", name, e),
        }

        Ok(metadata)
    }
}
