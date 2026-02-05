use super::InstanceManager;
use crate::instance::types::{InstanceMetadata, InstanceSettings};
use anyhow::Result;
use chrono::Utc;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

impl InstanceManager {
    pub async fn create_instance(&self, name: &str, version: &str) -> Result<InstanceMetadata> {
        self.create_instance_full(name, version, None, None).await
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
