use super::InstanceManager;
use crate::instance::archive::copy_dir_all;
use crate::instance::types::InstanceMetadata;
use anyhow::{Context, Result};
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

impl InstanceManager {
    pub async fn clone_instance(&self, id: Uuid, new_name: &str) -> Result<InstanceMetadata> {
        let instance = self.get_instance(id).await?.context("Instance not found")?;

        let new_id = Uuid::new_v4();
        let new_path = self.base_dir.join(new_id.to_string());

        // Copy directory recursively
        copy_dir_all(&instance.path, &new_path, |_, _, _| {}).await?;

        let new_metadata = InstanceMetadata {
            id: new_id,
            name: new_name.to_string(),
            version: instance.version.clone(),
            mod_loader: instance.mod_loader.clone(),
            loader_version: instance.loader_version.clone(),
            created_at: Utc::now(),
            last_run: None,
            path: new_path,
            schedules: instance.schedules.clone(),
            settings: instance.settings.clone(),
            status: crate::server::types::ServerStatus::Stopped,
            ip: None,
            port: None,
            max_players: None,
            description: None,
        };

        self.save_instance_to_db(&new_metadata).await?;

        info!(
            "Cloned instance: {} to {} (New ID: {})",
            instance.name, new_name, new_id
        );
        Ok(new_metadata)
    }
}
