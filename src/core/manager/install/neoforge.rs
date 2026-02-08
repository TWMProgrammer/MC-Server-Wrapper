use super::ServerManager;
use crate::instance::InstanceMetadata;
use crate::server::ServerHandle;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

impl ServerManager {
    pub(crate) async fn install_neoforge(
        &self,
        server: Arc<ServerHandle>,
        instance: &InstanceMetadata,
    ) -> Result<()> {
        let loader_version = instance
            .loader_version
            .as_deref()
            .ok_or_else(|| anyhow!("NeoForge requires a loader version"))?;

        let installer_name = "neoforge-installer.jar";
        let installer_path = instance.path.join(installer_name);

        let server_clone = Arc::clone(&server);
        let last_percent = Arc::new(AtomicU32::new(0));
        let final_size = Arc::new(AtomicU64::new(0));
        let final_size_clone = Arc::clone(&final_size);
        server.emit_log("Starting download of NeoForge installer...".to_string());
        self.mod_loader_client
            .download_neoforge(loader_version, &installer_path, move |current, total| {
                final_size_clone.store(current, Ordering::Relaxed);
                server_clone.handle_download_progress(
                    current,
                    total,
                    "Downloading NeoForge installer...",
                    &last_percent,
                );
            })
            .await?;
        let size_mb = final_size.load(Ordering::Relaxed) / (1024 * 1024);
        server.emit_log(format!("Final size: {} MB", size_mb));
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
