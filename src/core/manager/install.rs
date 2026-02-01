use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::process::Stdio;
use anyhow::{Result, anyhow};
use tracing::info;
use tokio::io::AsyncBufReadExt;
use super::ServerManager;
use super::super::server::{ServerHandle, generate_ascii_bar};
use super::super::instance::InstanceMetadata;

impl ServerManager {
    pub(crate) async fn install_fabric(&self, server: Arc<ServerHandle>, instance: &InstanceMetadata) -> Result<()> {
        let installer_name = "fabric-installer.jar";
        let installer_path = instance.path.join(installer_name);
        
        // Get latest fabric installer version
        let installer_versions = self.mod_loader_client.get_fabric_installer_versions().await?;
        let latest_installer = installer_versions.first()
            .ok_or_else(|| anyhow!("No Fabric installer versions found"))?;
        
        let server_clone = Arc::clone(&server);
        let last_percent = Arc::new(AtomicU32::new(0));
        server.emit_log("Starting download of Fabric installer...".to_string());
        self.mod_loader_client.download_fabric_installer(
            latest_installer,
            &installer_path,
            move |current, total| {
                let percent = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };
                let prev = last_percent.load(Ordering::Relaxed);
                if percent >= prev + 5 || percent == 100 {
                    last_percent.store(percent, Ordering::Relaxed);
                    let bar = generate_ascii_bar(current, total);
                    server_clone.emit_log(format!("Downloading Fabric installer... {}", bar));
                }
                server_clone.emit_progress(current, total, "Downloading Fabric installer...".to_string());
            }
        ).await?;
        server.emit_log("Fabric installer download complete!".to_string());

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

    pub(crate) async fn install_forge(&self, server: Arc<ServerHandle>, instance: &InstanceMetadata) -> Result<()> {
        let loader_version = instance.loader_version.as_deref()
            .ok_or_else(|| anyhow!("Forge requires a loader version"))?;
        
        let installer_name = "forge-installer.jar";
        let installer_path = instance.path.join(installer_name);
        
        let server_clone = Arc::clone(&server);
        let last_percent = Arc::new(AtomicU32::new(0));
        server.emit_log("Starting download of Forge installer...".to_string());
        self.mod_loader_client.download_forge(
            &instance.version,
            loader_version,
            &installer_path,
            move |current, total| {
                let percent = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };
                let prev = last_percent.load(Ordering::Relaxed);
                if percent >= prev + 5 || percent == 100 {
                    last_percent.store(percent, Ordering::Relaxed);
                    let bar = generate_ascii_bar(current, total);
                    server_clone.emit_log(format!("Downloading Forge installer... {}", bar));
                }
                server_clone.emit_progress(current, total, "Downloading Forge installer...".to_string());
            }
        ).await?;
        server.emit_log("Forge installer download complete!".to_string());

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

    pub(crate) async fn install_neoforge(&self, server: Arc<ServerHandle>, instance: &InstanceMetadata) -> Result<()> {
        let loader_version = instance.loader_version.as_deref()
            .ok_or_else(|| anyhow!("NeoForge requires a loader version"))?;
        
        let installer_name = "neoforge-installer.jar";
        let installer_path = instance.path.join(installer_name);
        
        let server_clone = Arc::clone(&server);
        let last_percent = Arc::new(AtomicU32::new(0));
        server.emit_log("Starting download of NeoForge installer...".to_string());
        self.mod_loader_client.download_neoforge(
            loader_version,
            &installer_path,
            move |current, total| {
                let percent = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };
                let prev = last_percent.load(Ordering::Relaxed);
                if percent >= prev + 5 || percent == 100 {
                    last_percent.store(percent, Ordering::Relaxed);
                    let bar = generate_ascii_bar(current, total);
                    server_clone.emit_log(format!("Downloading NeoForge installer... {}", bar));
                }
                server_clone.emit_progress(current, total, "Downloading NeoForge installer...".to_string());
            }
        ).await?;
        server.emit_log("NeoForge installer download complete!".to_string());

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

    pub(crate) async fn run_installer_command(&self, mut cmd: tokio::process::Command, server: Arc<ServerHandle>, loader_name: &str) -> Result<()> {
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
