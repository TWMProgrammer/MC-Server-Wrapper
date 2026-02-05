use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use anyhow::{Result, anyhow};
use super::ServerManager;
use crate::server::{ServerHandle, generate_ascii_bar};
use crate::instance::InstanceMetadata;

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
}
