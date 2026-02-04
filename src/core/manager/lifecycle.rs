use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tracing::info;
use super::ServerManager;
use super::super::server::{ServerHandle, ServerStatus, ResourceUsage, generate_ascii_bar};
use super::super::config::ServerConfig;

impl ServerManager {
    async fn build_server_config(&self, instance: &crate::instance::InstanceMetadata) -> ServerConfig {
        let is_bedrock = instance.mod_loader.as_deref().map(|l| l.to_lowercase() == "bedrock").unwrap_or(false);
        let bedrock_exe = if cfg!(windows) { "bedrock_server.exe" } else { "bedrock_server" };
        let bedrock_path = instance.path.join(bedrock_exe);
        let jar_path = instance.path.join("server.jar");

        let mut final_jar_path = if is_bedrock { Some(bedrock_path) } else { Some(jar_path) };
        let mut final_run_script = None;
        let mut args = vec!["nogui".to_string()];

        let loader_lower = instance.mod_loader.as_deref().map(|l| l.to_lowercase());

        // Check for Fabric server
        if loader_lower.as_deref() == Some("fabric") && instance.path.join("fabric-server.jar").exists() {
            final_jar_path = Some(instance.path.join("fabric-server.jar"));
        }

        // Check for run scripts (modern Forge/NeoForge)
        let run_script_name = if cfg!(windows) { "run.bat" } else { "run.sh" };
        if instance.path.join(run_script_name).exists() {
            final_run_script = Some(run_script_name.to_string());
            final_jar_path = None;
        }

        // Apply custom launch settings if it's an imported instance or has custom settings
        use crate::instance::LaunchMethod;
        match instance.settings.launch_method {
            LaunchMethod::BatFile => {
                if let Some(bat) = &instance.settings.bat_file {
                    final_run_script = Some(bat.clone());
                    final_jar_path = None;
                    args.clear();
                }
            },
            LaunchMethod::StartupLine => {
                let is_imported = instance.version == "Imported";
                let has_specialized = final_run_script.is_some() || 
                    (loader_lower.as_deref() == Some("fabric") && instance.path.join("fabric-server.jar").exists());

                if is_imported || !has_specialized {
                    if let Some(jar_idx) = instance.settings.startup_line.find("-jar ") {
                        let after_jar = &instance.settings.startup_line[jar_idx + 5..];
                        let mut parts = after_jar.split_whitespace();
                        if let Some(jar_name) = parts.next() {
                            final_jar_path = Some(instance.path.join(jar_name));
                        }
                        args = parts.map(|s| s.to_string()).collect();
                    }
                }
            }
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
                if let Ok(settings) = self.config_manager.load().await {
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
        }

        let min_ram_unit = match instance.settings.min_ram_unit.as_str() {
            "GB" => "G",
            "MB" => "M",
            u => u,
        };
        let max_ram_unit = match instance.settings.max_ram_unit.as_str() {
            "GB" => "G",
            "MB" => "M",
            u => u,
        };

        ServerConfig {
            name: instance.name.clone(),
            max_memory: format!("{}{}", instance.settings.max_ram, max_ram_unit),
            min_memory: format!("{}{}", instance.settings.min_ram, min_ram_unit),
            jar_path: final_jar_path,
            run_script: final_run_script,
            args,
            working_dir: instance.path.clone(),
            java_path,
            crash_handling: instance.settings.crash_handling.clone(),
            stop_timeout: 30,
        }
    }

    pub async fn get_or_create_server(&self, instance_id: Uuid) -> Result<Arc<ServerHandle>> {
        let mut servers = self.servers.lock().await;
        
        if let Some(server) = servers.get(&instance_id) {
            return Ok(Arc::clone(server));
        }

        let instance = self.instance_manager.get_instance(instance_id).await?
            .ok_or_else(|| anyhow!("Instance not found"))?;

        let config = self.build_server_config(&instance).await;
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
        
        let loader_lower = instance.mod_loader.as_deref().map(|l| l.to_lowercase());
        let run_script_name = if cfg!(windows) { "run.bat" } else { "run.sh" };
        
        // Robust check for existing installation
        let mut exists = if is_bedrock { 
            bedrock_path.exists() 
        } else { 
            // Check for server.jar
            jar_path.exists() || 
            // Check for Fabric server
            (loader_lower.as_deref() == Some("fabric") && instance.path.join("fabric-server.jar").exists()) ||
            // Check for modern Forge/NeoForge run scripts
            instance.path.join(run_script_name).exists()
        };

        // For imported instances, check if the configured executable exists
        if instance.version == "Imported" {
            use crate::instance::LaunchMethod;
            exists = match instance.settings.launch_method {
                LaunchMethod::BatFile => {
                    instance.settings.bat_file.as_ref()
                        .map(|bat| instance.path.join(bat).exists())
                        .unwrap_or(false)
                },
                LaunchMethod::StartupLine => {
                    // Check if startup line has a -jar argument
                    if let Some(jar_idx) = instance.settings.startup_line.find("-jar ") {
                        let after_jar = &instance.settings.startup_line[jar_idx + 5..];
                        let jar_name = after_jar.split_whitespace().next().unwrap_or("");
                        !jar_name.is_empty() && instance.path.join(jar_name).exists()
                    } else {
                        // If no -jar, check if the startup line itself might be a direct executable path (unlikely but possible)
                        // Or fallback to directory not empty check
                        let exe_name = instance.settings.startup_line.split_whitespace().next().unwrap_or("");
                        (!exe_name.is_empty() && instance.path.join(exe_name).exists()) || 
                        (instance.path.exists() && std::fs::read_dir(&instance.path).map(|mut d| d.next().is_some()).unwrap_or(false))
                    }
                }
            };
        }

        // Download jar/binary if missing
        if !exists {
            if instance.version == "Imported" {
                return Err(anyhow!("Imported instance is missing its executable (jar or bat file). Please check the instance settings."));
            }
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

        // Update server config after potential installation
        let config = self.build_server_config(&instance).await;
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
