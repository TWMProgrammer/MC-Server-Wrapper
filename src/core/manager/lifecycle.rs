use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
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

        let is_bedrock = instance.mod_loader.as_deref().map(|l| l.to_lowercase() == "bedrock").unwrap_or(false);
        let bedrock_exe = if cfg!(windows) { "bedrock_server.exe" } else { "bedrock_server" };
        let bedrock_path = instance.path.join(bedrock_exe);
        let jar_path = instance.path.join("server.jar");
        
        let exists = if is_bedrock { bedrock_path.exists() } else { jar_path.exists() };

        // Download jar/binary if missing
        if !exists {
            if let Some(loader) = &instance.mod_loader {
                let loader_lower = loader.to_lowercase();
                let is_fabric = loader_lower == "fabric";
                let is_forge = loader_lower == "forge";
                let is_neoforge = loader_lower == "neoforge";
                let _is_paper = loader_lower == "paper";
                let _is_purpur = loader_lower == "purpur";
                let _is_bedrock = loader_lower == "bedrock";

                if is_fabric {
                    self.install_fabric(Arc::clone(&server), &instance).await?;
                } else if is_forge {
                    self.install_forge(Arc::clone(&server), &instance).await?;
                } else if is_neoforge {
                    self.install_neoforge(Arc::clone(&server), &instance).await?;
                } else {
                    let server_clone = Arc::clone(&server);
                    let display_name = match loader_lower.as_str() {
                        "paper" => "Paper".to_string(),
                        "purpur" => "Purpur".to_string(),
                        "velocity" => "Velocity".to_string(),
                        "bungeecord" => "BungeeCord".to_string(),
                        "bedrock" => "Bedrock".to_string(),
                        _ => loader.clone(),
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

            // Also create eula.txt if it doesn't exist (Java only)
            if !is_bedrock {
                let eula_path = instance.path.join("eula.txt");
                if !eula_path.exists() {
                    tokio::fs::write(eula_path, "eula=true").await?;
                }
            }
        }

        // Basic config for now, in a real app this would be loaded from instance directory
        let mut final_jar_path = if is_bedrock { Some(bedrock_path) } else { Some(jar_path.clone()) };
        let run_script = None;
        let mut args = vec!["nogui".to_string()];

        let loader_lower = instance.mod_loader.as_deref().map(|l| l.to_lowercase());
        
        // Special case for Fabric: we want to run fabric-server.jar which then loads server.jar
        if loader_lower.as_deref() == Some("fabric") && instance.path.join("fabric-server.jar").exists() {
            final_jar_path = Some(instance.path.join("fabric-server.jar"));
        }

        // Special case for Proxies: they don't need nogui
        if let Some(loader) = &loader_lower {
            if loader == "velocity" || loader == "bungeecord" {
                args.clear();
            }
        }

        // Special case for Bedrock: it's not a jar and doesn't need nogui
        if is_bedrock {
            args.clear();
        }

        // Resolve Java path
        let mut java_path = None;
        if let Some(java_override) = &instance.settings.java_path_override {
            if !java_override.is_empty() && java_override != "java" {
                // Check if it's a managed version ID
                let settings = self.config_manager.load().await?;
                if let Some(managed) = settings.managed_java_versions.iter().find(|v| v.id == *java_override) {
                    java_path = Some(managed.path.clone());
                } else {
                    // Check if it's a valid path on disk
                    let path = std::path::Path::new(java_override);
                    if path.exists() {
                        java_path = Some(path.to_path_buf());
                    }
                }
            }
        }

        let mut config = ServerConfig {
            name: instance.name.clone(),
            max_memory: format!("{}{}", instance.settings.ram, instance.settings.ram_unit),
            min_memory: "1G".to_string(), // Could also be made configurable
            jar_path: final_jar_path,
            run_script,
            args,
            working_dir: instance.path.clone(),
            java_path,
            auto_restart: true,
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
        let server = {
            let servers = self.servers.lock().await;
            servers.get(&instance_id).cloned()
        };

        if let Some(server) = server {
            server.stop().await?;
            
            // Wait for it to stop (max 30 seconds)
            let mut attempts = 0;
            while attempts < 30 {
                let status = server.get_status().await;
                if status == ServerStatus::Stopped {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }
        
        self.start_server(instance_id).await?;
        Ok(())
    }
}
