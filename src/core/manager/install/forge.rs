use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use anyhow::{Result, anyhow};
use super::ServerManager;
use crate::server::{ServerHandle, generate_ascii_bar};
use crate::instance::InstanceMetadata;

impl ServerManager {
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
}
