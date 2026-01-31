use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tracing::info;
use tokio::io::AsyncBufReadExt;
use std::process::Stdio;
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

    pub async fn get_server(&self, instance_id: Uuid) -> Option<Arc<ServerHandle>> {
        let servers = self.servers.lock().await;
        servers.get(&instance_id).cloned()
    }

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

    pub async fn create_instance_full(&self, name: &str, version: &str, mod_loader: Option<String>, loader_version: Option<String>) -> Result<InstanceMetadata> {
        let instance = self.instance_manager.create_instance_full(name, version, mod_loader, loader_version).await?;
        
        // In a real app, we might want to trigger the download here
        // or let the UI handle progress. For now, we'll just return the metadata.
        // The jar will be downloaded when the server starts.
        
        Ok(instance)
    }

    pub async fn start_server(&self, instance_id: Uuid) -> Result<()> {
        let server = self.get_or_create_server(instance_id).await?;
        let status = server.get_status().await;
        
        if status != super::server::ServerStatus::Stopped && status != super::server::ServerStatus::Crashed {
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

    async fn install_fabric(&self, server: Arc<ServerHandle>, instance: &InstanceMetadata) -> Result<()> {
        let installer_name = "fabric-installer.jar";
        let installer_path = instance.path.join(installer_name);
        
        // Get latest fabric installer version
        let installer_versions = self.mod_loader_client.get_fabric_installer_versions().await?;
        let latest_installer = installer_versions.first()
            .ok_or_else(|| anyhow!("No Fabric installer versions found"))?;
        
        let server_clone = Arc::clone(&server);
        self.mod_loader_client.download_fabric_installer(
            latest_installer,
            &installer_path,
            move |current, total| {
                server_clone.emit_progress(current, total, "Downloading Fabric installer...".to_string());
            }
        ).await?;

        let mut cmd = tokio::process::Command::new("java");
        cmd.current_dir(&instance.path)
            .arg("-jar")
            .arg(&installer_path)
            .arg("server")
            .arg("-mcversion").arg(&instance.version)
            .arg("-downloadMinecraft");
        
        if let Some(loader_ver) = &instance.loader_version {
            cmd.arg("-loader").arg(loader_ver);
        }

        self.run_installer_command(cmd, server, "Fabric").await?;
        let _ = tokio::fs::remove_file(installer_path).await;

        // Find and rename loader jar
        let loader_jar = std::fs::read_dir(&instance.path)?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                (name.contains("fabric-server-launch") || name.contains("fabric-loader")) && name.ends_with(".jar")
            });

        if let Some(jar) = loader_jar {
            let target_path = instance.path.join("fabric-server.jar");
            tokio::fs::rename(jar.path(), target_path).await?;
        }

        Ok(())
    }

    async fn install_forge(&self, server: Arc<ServerHandle>, instance: &InstanceMetadata) -> Result<()> {
        let loader_version = instance.loader_version.as_deref()
            .ok_or_else(|| anyhow!("Forge requires a loader version"))?;
        
        let installer_name = "forge-installer.jar";
        let installer_path = instance.path.join(installer_name);
        
        let server_clone = Arc::clone(&server);
        self.mod_loader_client.download_forge(
            &instance.version,
            loader_version,
            &installer_path,
            move |current, total| {
                server_clone.emit_progress(current, total, "Downloading Forge installer...".to_string());
            }
        ).await?;

        let mut cmd = tokio::process::Command::new("java");
        cmd.current_dir(&instance.path)
            .arg("-jar")
            .arg(&installer_path)
            .arg("--installServer");

        self.run_installer_command(cmd, server, "Forge").await?;
        let _ = tokio::fs::remove_file(installer_path).await;

        // For modern Forge, we don't rename anything. The run script will be used.
        // For older Forge, we might need to find the server jar.
        if !self.mod_loader_client.is_modern_forge(&instance.version) {
            // Try to find a forge jar for older versions
            let loader_jar = std::fs::read_dir(&instance.path)?
                .filter_map(|entry| entry.ok())
                .find(|entry| {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    name.contains("forge") && name.ends_with(".jar") && !name.contains("installer")
                });

            if let Some(jar) = loader_jar {
                let target_path = instance.path.join("server.jar");
                tokio::fs::rename(jar.path(), target_path).await?;
            }
        } else {
            // Check if run script was created
            let run_script = if cfg!(windows) { "run.bat" } else { "run.sh" };
            if !instance.path.join(run_script).exists() {
                return Err(anyhow!("Forge installation finished but no run script was found for modern version."));
            }
        }

        Ok(())
    }

    async fn install_neoforge(&self, server: Arc<ServerHandle>, instance: &InstanceMetadata) -> Result<()> {
        let loader_version = instance.loader_version.as_deref()
            .ok_or_else(|| anyhow!("NeoForge requires a loader version"))?;
        
        let installer_name = "neoforge-installer.jar";
        let installer_path = instance.path.join(installer_name);
        
        let server_clone = Arc::clone(&server);
        self.mod_loader_client.download_neoforge(
            loader_version,
            &installer_path,
            move |current, total| {
                server_clone.emit_progress(current, total, "Downloading NeoForge installer...".to_string());
            }
        ).await?;

        let mut cmd = tokio::process::Command::new("java");
        cmd.current_dir(&instance.path)
            .arg("-jar")
            .arg(&installer_path)
            .arg("--installServer");

        self.run_installer_command(cmd, server, "NeoForge").await?;
        let _ = tokio::fs::remove_file(installer_path).await;

        // NeoForge is always "modern", so it should have run scripts.
        Ok(())
    }

    async fn run_installer_command(&self, mut cmd: tokio::process::Command, server: Arc<ServerHandle>, loader_name: &str) -> Result<()> {
        let msg = format!("Running {} installer...", loader_name);
        info!("{}", msg);
        server.emit_log(msg);

        let mut child = cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
        let server_clone = Arc::clone(&server);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                server_clone.emit_log(line);
            }
        });

        let server_clone_err = Arc::clone(&server);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                server_clone_err.emit_log(format!("ERROR: {}", line));
            }
        });

        let status = child.wait().await?;
        if !status.success() {
            return Err(anyhow!("{} installer failed with status: {}", loader_name, status));
        }

        Ok(())
    }
}
