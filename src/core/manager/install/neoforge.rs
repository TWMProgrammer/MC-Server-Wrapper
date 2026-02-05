use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use anyhow::{Result, anyhow};
use super::ServerManager;
use crate::server::{ServerHandle, generate_ascii_bar};
use crate::instance::InstanceMetadata;

impl ServerManager {
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
}
