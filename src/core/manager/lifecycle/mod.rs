use super::ServerManager;
use crate::server::{ResourceUsage, ServerHandle, ServerStatus};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use uuid::Uuid;

pub mod config;
pub mod installer;

impl ServerManager {
    pub async fn get_or_create_server(&self, instance_id: Uuid) -> Result<Arc<ServerHandle>> {
        let mut servers = self.servers.lock().await;

        if let Some(server) = servers.get(&instance_id) {
            return Ok(Arc::clone(server));
        }

        let instance = self
            .instance_manager
            .get_instance(instance_id)
            .await?
            .ok_or_else(|| anyhow!("Instance not found"))?;

        let config = self.build_server_config(&instance).await;
        let server = Arc::new(ServerHandle::new(config));
        servers.insert(instance_id, Arc::clone(&server));
        Ok(server)
    }

    pub async fn start_server(&self, instance_id: Uuid) -> Result<()> {
        let server = self.prepare_server(instance_id).await?;
        let status = server.get_status().await;

        if status != ServerStatus::Stopped && status != ServerStatus::Crashed {
            return Ok(());
        }

        server.start().await?;

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

    pub async fn kill_server(&self, instance_id: Uuid) -> Result<()> {
        let servers = self.servers.lock().await;
        if let Some(server) = servers.get(&instance_id) {
            server.kill().await?;
        }
        Ok(())
    }

    pub async fn send_command(&self, instance_id: Uuid, command: &str) -> Result<()> {
        let servers = self.servers.lock().await;
        if let Some(server) = servers.get(&instance_id) {
            server.send_command(command).await?;
        } else {
            return Err(anyhow!("Server not running"));
        }
        Ok(())
    }

    pub async fn get_server_status(&self, instance_id: Uuid) -> ServerStatus {
        let servers = self.servers.lock().await;
        if let Some(server) = servers.get(&instance_id) {
            server.get_status().await
        } else {
            ServerStatus::Stopped
        }
    }

    pub async fn get_server_usage(&self, instance_id: Uuid) -> Option<ResourceUsage> {
        let servers = self.servers.lock().await;
        if let Some(server) = servers.get(&instance_id) {
            Some(server.get_usage().await)
        } else {
            None
        }
    }

    pub async fn restart_server(&self, instance_id: Uuid) -> Result<()> {
        let server = {
            let servers = self.servers.lock().await;
            servers.get(&instance_id).cloned()
        };

        if let Some(server) = server {
            server.stop().await?;
        }

        self.start_server(instance_id).await?;
        Ok(())
    }
}
