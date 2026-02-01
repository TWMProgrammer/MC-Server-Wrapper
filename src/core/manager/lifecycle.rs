use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tracing::info;
use super::ServerManager;
use super::super::server::{ServerHandle, ServerStatus, generate_ascii_bar};
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

    pub async fn prepare_server(&self, instance_id: Uuid) -> Result<Arc<ServerHandle>> {
        let server = self.get_or_create_server(instance_id).await?;
        let instance = self.instance_manager.get_instance(instance_id).await?
            .ok_or_else(|| anyhow!("Instance not found"))?;

        let jar_path = instance.path.join("server.jar");
        
        // Download jar if missing
        if !jar_path.exists() {
            if let Some(loader) = &instance.mod_loader {
                let is_forge = loader.to_lowercase() == "forge";
                let is_neoforge = loader.to_lowercase() == "neoforge";
                let is_fabric = loader.to_lowercase() == "fabric";
                let is_paper = loader.to_lowercase() == "paper";
                let is_purpur = loader.to_lowercase() == "purpur";

                if is_fabric {
                    self.install_fabric(Arc::clone(&server), &instance).await?;
                } else if is_forge {
                    self.install_forge(Arc::clone(&server), &instance).await?;
                } else if is_neoforge {
                    self.install_neoforge(Arc::clone(&server), &instance).await?;
                } else {
                    let server_clone = Arc::clone(&server);
                    let display_name = if is_paper { 
                        "Paper".to_string() 
                    } else if is_purpur { 
                        "Purpur".to_string() 
                    } else { 
                        loader.clone() 
                    };
                    
                    let msg = format!("Starting download of {} for version {}", display_name, instance.version);
                    info!("{}", msg);
                    server.emit_log(msg);

                    let last_percent = Arc::new(AtomicU32::new(0));
                    let display_name_clone = display_name.clone();
                    self.mod_loader_client.download_loader(
                        loader,
                        &instance.version,
                        instance.loader_version.as_deref(),
                        &jar_path,
                        move |current, total| {
                            let percent = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };
                            let prev = last_percent.load(Ordering::Relaxed);
                            if percent >= prev + 5 || percent == 100 {
                                last_percent.store(percent, Ordering::Relaxed);
                                let bar = generate_ascii_bar(current, total);
                                server_clone.emit_log(format!("Downloading {}... {}", display_name_clone, bar));
                            }
                            server_clone.emit_progress(current, total, format!("Downloading {}...", display_name_clone));
                        }
                    ).await?;
                    server.emit_log("Download complete!".to_string());
                }
            } else {
                let msg = format!("Starting download of vanilla server for version {}", instance.version);
                info!("{}", msg);
                server.emit_log(msg);
                let server_clone = Arc::clone(&server);
                let last_percent = Arc::new(AtomicU32::new(0));
                self.downloader.download_server(&instance.version, &jar_path, move |current, total| {
                    let percent = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };
                    let prev = last_percent.load(Ordering::Relaxed);
                    if percent >= prev + 5 || percent == 100 {
                        last_percent.store(percent, Ordering::Relaxed);
                        let bar = generate_ascii_bar(current, total);
                        server_clone.emit_log(format!("Downloading vanilla server... {}", bar));
                    }
                    server_clone.emit_progress(current, total, "Downloading vanilla server...".to_string());
                }).await?;
                server.emit_log("Download complete!".to_string());
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

    pub async fn restart_server(&self, instance_id: Uuid) -> Result<()> {
        self.stop_server(instance_id).await?;
        self.start_server(instance_id).await?;
        Ok(())
    }
}
