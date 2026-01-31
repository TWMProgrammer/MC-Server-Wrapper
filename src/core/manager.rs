use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use super::server::ServerHandle;
use super::instance::{InstanceManager, InstanceMetadata};
use super::config::ServerConfig;
use super::downloader::VersionDownloader;
use super::mod_loaders::ModLoaderClient;

pub struct ServerManager {
    instance_manager: Arc<InstanceManager>,
    downloader: VersionDownloader,
    mod_loader_client: ModLoaderClient,
    servers: Arc<Mutex<HashMap<Uuid, Arc<ServerHandle>>>>,
}

impl ServerManager {
    pub fn new(instance_manager: Arc<InstanceManager>) -> Self {
        let cache_dir = instance_manager.get_base_dir().join("cache");
        Self {
            instance_manager,
            downloader: VersionDownloader::new(Some(cache_dir)),
            mod_loader_client: ModLoaderClient::new(),
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_downloader(&self) -> &VersionDownloader {
        &self.downloader
    }

    pub fn get_mod_loader_client(&self) -> &ModLoaderClient {
        &self.mod_loader_client
    }

    pub async fn create_instance_full(&self, name: &str, version: &str, mod_loader: Option<String>, loader_version: Option<String>) -> Result<InstanceMetadata> {
        let instance = self.instance_manager.create_instance(name, version).await?;
        
        // In a real app, we might want to trigger the download here
        // or let the UI handle progress. For now, we'll just return the metadata.
        // The jar will be downloaded when the server starts.
        
        // TODO: Handle mod loader installation logic here or in start_server
        
        Ok(instance)
    }

    pub async fn start_server(&self, instance_id: Uuid) -> Result<()> {
        let mut servers = self.servers.lock().await;
        
        if servers.contains_key(&instance_id) {
            let server = servers.get(&instance_id).unwrap();
            let status = server.get_status().await;
            if status != super::server::ServerStatus::Stopped && status != super::server::ServerStatus::Crashed {
                return Ok(());
            }
        }

        let instance = self.instance_manager.get_instance(instance_id).await?
            .ok_or_else(|| anyhow!("Instance not found"))?;

        let jar_path = instance.path.join("server.jar");
        
        // Download jar if missing
        if !jar_path.exists() {
            self.downloader.download_server(&instance.version, &jar_path).await?;
            // Also create eula.txt if it doesn't exist
            let eula_path = instance.path.join("eula.txt");
            if !eula_path.exists() {
                tokio::fs::write(eula_path, "eula=true").await?;
            }
        }

        // Basic config for now, in a real app this would be loaded from instance directory
        let config = ServerConfig {
            name: instance.name.clone(),
            max_memory: "2G".to_string(),
            min_memory: "1G".to_string(),
            jar_path,
            working_dir: instance.path.clone(),
            java_path: None,
            auto_restart: true,
        };

        let server = Arc::new(ServerHandle::new(config));
        server.start().await?;
        
        servers.insert(instance_id, server);
        self.instance_manager.update_last_run(instance_id).await?;
        
        Ok(())
    }

    pub async fn stop_server(&self, instance_id: Uuid) -> Result<()> {
        let servers = self.servers.lock().await;
        if let Some(server) = servers.get(&instance_id) {
            server.stop().await?;
        }
        Ok(())
    }

    pub async fn get_server(&self, instance_id: Uuid) -> Option<Arc<ServerHandle>> {
        let servers = self.servers.lock().await;
        servers.get(&instance_id).cloned()
    }
}
