use super::InstanceManager;
use crate::instance::types::{InstanceMetadata, InstanceSettings};
use crate::mods::ModrinthClient;
use crate::mods::types::ProjectVersion;
use anyhow::{Result, anyhow};
use chrono::Utc;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

impl InstanceManager {
    pub async fn create_instance(&self, name: &str, version: &str) -> Result<InstanceMetadata> {
        self.create_instance_full(name, version, None, None).await
    }

    pub async fn create_instance_from_modpack<F>(
        &self,
        name: &str,
        version: &ProjectVersion,
        cache: std::sync::Arc<crate::cache::CacheManager>,
        on_progress: F,
    ) -> Result<InstanceMetadata>
    where
        F: Fn(crate::mods::modrinth::modpack::ModpackProgress) + Send + 'static,
    {
        let client = ModrinthClient::new(cache);

        let id = Uuid::new_v4();
        let instance_path = self.base_dir.join(id.to_string());
        fs::create_dir_all(&instance_path).await?;

        // Install modpack files
        let index = client
            .install_modpack(&instance_path, version, on_progress)
            .await?;

        // Extract game version and loader from index
        let game_version = index
            .dependencies
            .get("minecraft")
            .ok_or_else(|| anyhow!("Minecraft version not found in modpack index"))?;

        let (mod_loader, loader_version) =
            if let Some(fabric) = index.dependencies.get("fabric-loader") {
                (Some("fabric".to_string()), Some(fabric.clone()))
            } else if let Some(quilt) = index.dependencies.get("quilt-loader") {
                (Some("quilt".to_string()), Some(quilt.clone()))
            } else if let Some(forge) = index.dependencies.get("forge") {
                (Some("forge".to_string()), Some(forge.clone()))
            } else if let Some(neoforge) = index.dependencies.get("neoforge") {
                (Some("neoforge".to_string()), Some(neoforge.clone()))
            } else {
                (None, None)
            };

        let metadata = InstanceMetadata {
            id,
            name: name.to_string(),
            version: game_version.clone(),
            mod_loader,
            loader_version,
            created_at: Utc::now(),
            last_run: None,
            path: instance_path,
            schedules: vec![],
            settings: InstanceSettings::default(),
            status: crate::server::types::ServerStatus::Stopped,
            ip: None,
            port: None,
            max_players: None,
            description: index.summary,
        };

        self.save_instance_to_db(&metadata).await?;

        info!("Created new instance from modpack: {} (ID: {})", name, id);
        Ok(metadata)
    }

    pub async fn create_instance_full(
        &self,
        name: &str,
        version: &str,
        mod_loader: Option<String>,
        loader_version: Option<String>,
    ) -> Result<InstanceMetadata> {
        let id = Uuid::new_v4();
        let instance_path = self.base_dir.join(id.to_string());
        fs::create_dir_all(&instance_path).await?;

        let metadata = InstanceMetadata {
            id,
            name: name.to_string(),
            version: version.to_string(),
            mod_loader,
            loader_version,
            created_at: Utc::now(),
            last_run: None,
            path: instance_path,
            schedules: vec![],
            settings: InstanceSettings::default(),
            status: crate::server::types::ServerStatus::Stopped,
            ip: None,
            port: None,
            max_players: None,
            description: None,
        };

        self.save_instance_to_db(&metadata).await?;

        info!("Created new instance: {} (ID: {})", name, id);
        Ok(metadata)
    }
}
