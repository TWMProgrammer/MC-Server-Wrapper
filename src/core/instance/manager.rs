use std::path::{Path, PathBuf};
use uuid::Uuid;
use anyhow::{Result, Context};
use tokio::fs;
use tracing::info;
use chrono::Utc;

use super::types::InstanceMetadata;
use super::archive::{extract_zip, extract_7z, copy_dir_all};
use super::types::InstanceSettings;

pub struct InstanceManager {
    pub(crate) base_dir: PathBuf,
    pub(crate) registry_path: PathBuf,
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
        self.create_instance_full(name, version, None, None).await
    }

    pub async fn create_instance_full(&self, name: &str, version: &str, mod_loader: Option<String>, loader_version: Option<String>) -> Result<InstanceMetadata> {
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
        };

        let mut instances = self.list_instances().await?;
        instances.push(metadata.clone());
        self.save_registry(&instances).await?;

        info!("Created new instance: {} (ID: {})", name, id);
        Ok(metadata)
    }

    pub async fn import_instance<F>(&self, name: &str, source_path: PathBuf, jar_name: String, mod_loader: Option<String>, root_within_zip: Option<String>, on_progress: F) -> Result<InstanceMetadata> 
    where F: Fn(u64, u64, String) + Send + Sync + 'static
    {
        let id = Uuid::new_v4();
        let instance_path = self.base_dir.join(id.to_string());
        fs::create_dir_all(&instance_path).await?;

        if source_path.is_dir() {
            copy_dir_all(&source_path, &instance_path, on_progress).await?;
        } else if source_path.is_file() {
            let extension = source_path.extension().map_or("", |ext| ext.to_str().unwrap_or("")).to_lowercase();
            if extension == "zip" {
                extract_zip(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else if extension == "7z" {
                extract_7z(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else {
                return Err(anyhow::anyhow!("Unsupported archive format: .{}", extension));
            }
        } else {
            return Err(anyhow::anyhow!("Source path must be a directory or a supported archive file (.zip, .7z)"));
        }

        let mut settings = InstanceSettings::default();
        settings.startup_line = format!("java -Xmx{{ram}}{{unit}} -jar {} nogui", jar_name);

        let metadata = InstanceMetadata {
            id,
            name: name.to_string(),
            version: "Imported".to_string(),
            mod_loader,
            loader_version: None,
            created_at: Utc::now(),
            last_run: None,
            path: instance_path,
            schedules: vec![],
            settings,
        };

        let mut instances = self.list_instances().await?;
        instances.push(metadata.clone());
        self.save_registry(&instances).await?;

        info!("Imported instance: {} (ID: {})", name, id);
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

    pub async fn get_instance_by_name(&self, name: &str) -> Result<Option<InstanceMetadata>> {
        let instances = self.list_instances().await?;
        Ok(instances.into_iter().find(|i| i.name == name))
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

    pub async fn delete_instance_by_name(&self, name: &str) -> Result<()> {
        let mut instances = self.list_instances().await?;
        if let Some(pos) = instances.iter().position(|i| i.name == name) {
            let instance = instances.remove(pos);
            if instance.path.exists() {
                fs::remove_dir_all(&instance.path).await?;
            }
            self.save_registry(&instances).await?;
            info!("Deleted instance by name: {} (ID: {})", instance.name, instance.id);
        }
        Ok(())
    }

    pub async fn clone_instance(&self, id: Uuid, new_name: &str) -> Result<InstanceMetadata> {
        let instance = self.get_instance(id).await?
            .context("Instance not found")?;
        
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
        };

        let mut instances = self.list_instances().await?;
        instances.push(new_metadata.clone());
        self.save_registry(&instances).await?;

        info!("Cloned instance: {} to {} (New ID: {})", instance.name, new_name, new_id);
        Ok(new_metadata)
    }

    pub fn get_base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    pub(crate) async fn save_registry(&self, instances: &[InstanceMetadata]) -> Result<()> {
        let content = serde_json::to_string_pretty(instances)?;
        fs::write(&self.registry_path, content).await?;
        Ok(())
    }
}
