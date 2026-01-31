use std::sync::Arc;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tracing::info;
use super::ServerManager;
use super::super::server::{ServerHandle, ServerStatus};
use super::super::config::ServerConfig;

impl ServerManager {
    pub async fn get_or_create_server(&self, instance_id: Uuid) -> Result<Arc<ServerHandle>> {
        let mut servers = self.servers.lock().await;
        
        if let Some(server) = servers.get(&instance_id) {
            return Ok(Arc::clone(server));
        }

        let instance = self.instance_manager.get_instance(instance_id).await?
            .ok_or_else(|| anyhow!("Instance not found"))?;

        let mut config = ServerConfig {
            name: instance.name.clone(),
            working_dir: instance.path.clone(),
            ..Default::default()
        };

        // Check for run scripts (modern Forge/NeoForge)
        let run_script = if cfg!(windows) { "run.bat" } else { "run.sh" };
        if instance.path.join(run_script).exists() {
            config.run_script = Some(run_script.to_string());
            config.jar_path = None;
        }

        let server = Arc::new(ServerHandle::new(config));
        servers.insert(instance_id, Arc::clone(&server));
        Ok(server)
    }

    pub async fn start_server(&self, instance_id: Uuid) -> Result<()> {
        let server = self.get_or_create_server(instance_id).await?;
        let status = server.get_status().await;
        
        if status != ServerStatus::Stopped && status != ServerStatus::Crashed {
            return Ok(());
        }

        let instance = self.instance_manager.get_instance(instance_id).await?
            .ok_or_else(|| anyhow!("Instance not found"))?;

        let jar_path = instance.path.join("server.jar");
        
        // Download jar if missing
        if !jar_path.exists() {
            if let Some(loader) = &instance.mod_loader {
                let msg = format!("Downloading {} loader for version {}", loader, instance.version);
                info!("{}", msg);
                server.emit_log(msg);
                
                let is_forge = loader.to_lowercase() == "forge";
                let is_neoforge = loader.to_lowercase() == "neoforge";
                let is_fabric = loader.to_lowercase() == "fabric";

                if is_fabric {
                    self.install_fabric(Arc::clone(&server), &instance).await?;
                } else if is_forge {
                    self.install_forge(Arc::clone(&server), &instance).await?;
                } else if is_neoforge {
                    self.install_neoforge(Arc::clone(&server), &instance).await?;
                } else {
                    let server_clone = Arc::clone(&server);
                    let loader_type = loader.clone();
                    self.mod_loader_client.download_loader(
                        loader,
                        &instance.version,
                        instance.loader_version.as_deref(),
                        &jar_path,
                        move |current, total| {
                            server_clone.emit_progress(current, total, format!("Downloading {}...", loader_type));
                        }
                    ).await?;
                }
            } else {
                let msg = format!("Downloading vanilla server for version {}", instance.version);
                info!("{}", msg);
                server.emit_log(msg);
                let server_clone = Arc::clone(&server);
                self.downloader.download_server(&instance.version, &jar_path, move |current, total| {
                    server_clone.emit_progress(current, total, "Downloading vanilla server...".to_string());
                }).await?;
            }

            // Also create eula.txt if it doesn't exist
            let eula_path = instance.path.join("eula.txt");
            if !eula_path.exists() {
                tokio::fs::write(eula_path, "eula=true").await?;
            }
        }

        // Basic config for now, in a real app this would be loaded from instance directory
        let mut final_jar_path = jar_path.clone();
        
        // Special case for Fabric: we want to run fabric-server.jar which then loads server.jar
        let is_fabric = instance.mod_loader.as_deref().map(|l| l.to_lowercase() == "fabric").unwrap_or(false);
        if is_fabric && instance.path.join("fabric-server.jar").exists() {
            final_jar_path = instance.path.join("fabric-server.jar");
        }

        let mut config = ServerConfig {
            name: instance.name.clone(),
            max_memory: "2G".to_string(),
            min_memory: "1G".to_string(),
            jar_path: Some(final_jar_path),
            args: vec!["nogui".to_string()],
            working_dir: instance.path.clone(),
            java_path: None,
            auto_restart: true,
            run_script: None,
        };

        // Check for run scripts (modern Forge/NeoForge)
        let run_script = if cfg!(windows) { "run.bat" } else { "run.sh" };
        if instance.path.join(run_script).exists() {
            config.run_script = Some(run_script.to_string());
            config.jar_path = None;
        }

        server.update_config(config).await;
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
}
