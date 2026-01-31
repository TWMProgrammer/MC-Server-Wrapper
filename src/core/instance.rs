use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tokio::fs;
use chrono::{DateTime, Utc};
use tracing::info;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub path: PathBuf,
}

pub struct InstanceManager {
    base_dir: PathBuf,
    registry_path: PathBuf,
}

impl InstanceManager {
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).await?;
        }
        let registry_path = base_dir.join("instances.json");
        Ok(Self { base_dir, registry_path })
    }

    pub async fn create_instance(&self, name: &str, version: &str) -> Result<InstanceMetadata> {
        let id = Uuid::new_v4();
        let instance_path = self.base_dir.join(id.to_string());
        fs::create_dir_all(&instance_path).await?;

        let metadata = InstanceMetadata {
            id,
            name: name.to_string(),
            version: version.to_string(),
            created_at: Utc::now(),
            last_run: None,
            path: instance_path,
        };

        let mut instances = self.list_instances().await?;
        instances.push(metadata.clone());
        self.save_registry(&instances).await?;

        info!("Created new instance: {} (ID: {})", name, id);
        Ok(metadata)
    }

    pub async fn list_instances(&self) -> Result<Vec<InstanceMetadata>> {
        if !self.registry_path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&self.registry_path).await?;
        let instances: Vec<InstanceMetadata> = serde_json::from_str(&content)
            .context("Failed to parse instances registry")?;
        Ok(instances)
    }

    pub async fn get_instance(&self, id: Uuid) -> Result<Option<InstanceMetadata>> {
        let instances = self.list_instances().await?;
        Ok(instances.into_iter().find(|i| i.id == id))
    }

    pub async fn delete_instance(&self, id: Uuid) -> Result<()> {
        let mut instances = self.list_instances().await?;
        if let Some(pos) = instances.iter().position(|i| i.id == id) {
            let instance = instances.remove(pos);
            if instance.path.exists() {
                fs::remove_dir_all(&instance.path).await?;
            }
            self.save_registry(&instances).await?;
            info!("Deleted instance: {} (ID: {})", instance.name, id);
        }
        Ok(())
    }

    pub async fn update_last_run(&self, id: Uuid) -> Result<()> {
        let mut instances = self.list_instances().await?;
        if let Some(instance) = instances.iter_mut().find(|i| i.id == id) {
            instance.last_run = Some(Utc::now());
            self.save_registry(&instances).await?;
        }
        Ok(())
    }

    pub fn get_base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    async fn save_registry(&self, instances: &[InstanceMetadata]) -> Result<()> {
        let content = serde_json::to_string_pretty(instances)?;
        fs::write(&self.registry_path, content).await?;
        Ok(())
    }
}
