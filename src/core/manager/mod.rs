use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use anyhow::Result;
use super::instance::{InstanceManager, InstanceMetadata};
use super::app_config::GlobalConfigManager;
use super::downloader::VersionDownloader;
use super::mod_loaders::ModLoaderClient;
use super::server::ServerHandle;

mod lifecycle;
mod install;

pub struct ServerManager {
    pub(crate) instance_manager: Arc<InstanceManager>,
    pub(crate) config_manager: Arc<GlobalConfigManager>,
    pub(crate) downloader: VersionDownloader,
    pub(crate) mod_loader_client: ModLoaderClient,
    pub(crate) servers: Arc<Mutex<HashMap<Uuid, Arc<ServerHandle>>>>,
}

impl ServerManager {
    pub fn new(instance_manager: Arc<InstanceManager>, config_manager: Arc<GlobalConfigManager>) -> Self {
        let cache_dir = instance_manager.get_base_dir().join("cache");
        Self {
            instance_manager,
            config_manager,
            downloader: VersionDownloader::new(Some(cache_dir.clone())),
            mod_loader_client: ModLoaderClient::new(Some(cache_dir)),
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_downloader(&self) -> &VersionDownloader {
        &self.downloader
    }

    pub fn get_mod_loader_client(&self) -> &ModLoaderClient {
        &self.mod_loader_client
    }

    pub async fn get_server(&self, instance_id: Uuid) -> Option<Arc<ServerHandle>> {
        let servers = self.servers.lock().await;
        servers.get(&instance_id).cloned()
    }

    pub async fn create_instance_full(&self, name: &str, version: &str, mod_loader: Option<String>, loader_version: Option<String>) -> Result<InstanceMetadata> {
        let instance = self.instance_manager.create_instance_full(name, version, mod_loader, loader_version).await?;
        Ok(instance)
    }

    pub async fn get_bedrock_versions(&self) -> Result<crate::downloader::VersionManifest> {
        self.mod_loader_client.get_bedrock_versions().await
    }

    pub async fn get_velocity_versions(&self) -> Result<Vec<String>> {
        self.mod_loader_client.get_velocity_versions().await
    }

    pub async fn get_velocity_builds(&self, version: &str) -> Result<Vec<String>> {
        self.mod_loader_client.get_velocity_builds(version).await
    }

    pub async fn get_bungeecord_versions(&self) -> Result<Vec<String>> {
        self.mod_loader_client.get_bungeecord_versions().await
    }
}
