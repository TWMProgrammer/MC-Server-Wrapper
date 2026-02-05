use super::ServerManager;
use crate::instance::InstanceMetadata;
use crate::server::{ServerHandle, generate_ascii_bar};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

impl ServerManager {
    pub(crate) async fn install_quilt(
        &self,
        server: Arc<ServerHandle>,
        instance: &InstanceMetadata,
    ) -> Result<()> {
        let installer_name = "quilt-installer.jar";
        let installer_path = instance.path.join(installer_name);

        // Get latest quilt installer version
        let installer_versions = self
            .mod_loader_client
            .get_quilt_installer_versions()
            .await?;
        let latest_installer = installer_versions
            .first()
            .ok_or_else(|| anyhow!("No Quilt installer versions found"))?;

        let server_clone = Arc::clone(&server);
        let last_percent = Arc::new(AtomicU32::new(0));
        server.emit_log("Starting download of Quilt installer...".to_string());
        self.mod_loader_client
            .download_quilt_installer(latest_installer, &installer_path, move |current, total| {
                let percent = if total > 0 {
                    (current as f64 / total as f64 * 100.0) as u32
                } else {
                    0
                };
                let prev = last_percent.load(Ordering::Relaxed);
                if percent >= prev + 5 || percent == 100 {
                    last_percent.store(percent, Ordering::Relaxed);
                    let bar = generate_ascii_bar(current, total);
                    server_clone.emit_log(format!("Downloading Quilt installer... {}", bar));
                }
                server_clone.emit_progress(
                    current,
                    total,
                    "Downloading Quilt installer...".to_string(),
                );
            })
            .await?;
        server.emit_log("Quilt installer download complete!".to_string());

        let mut cmd = tokio::process::Command::new("java");
        cmd.current_dir(&instance.path)
            .arg("-jar")
            .arg(&installer_path)
            .arg("install")
            .arg("server")
            .arg(&instance.version);

        if let Some(loader_ver) = &instance.loader_version {
            cmd.arg(loader_ver);
        }

        cmd.arg("--download-server");
        cmd.arg("--install-dir=.");

        self.run_installer_command(cmd, Arc::clone(&server), "Quilt")
            .await?;
        let _ = tokio::fs::remove_file(installer_path).await;

        // Quilt installer creates a 'quilt-server-launch.jar' which requires 'server.jar' and 'libraries' folder
        // We should rename 'quilt-server-launch.jar' to 'quilt-server.jar' to match our config detection
        let mut loader_jar = None;
        if let Ok(mut entries) = tokio::fs::read_dir(&instance.path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("quilt-server-launch") && name.ends_with(".jar") {
                    loader_jar = Some(entry.path());
                    break;
                }
            }
        }

        if let Some(jar_path) = loader_jar {
            let target_path = instance.path.join("quilt-server.jar");
            server.emit_log(format!(
                "Renaming {} to quilt-server.jar",
                jar_path.file_name().unwrap().to_string_lossy()
            ));
            tokio::fs::rename(jar_path, target_path).await?;
        } else {
            server.emit_log("Warning: Could not find quilt-server-launch.jar. Checking for other loader jars...".to_string());
            // Fallback: check for any quilt-loader jar
            if let Ok(mut entries) = tokio::fs::read_dir(&instance.path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name.contains("quilt-loader") && name.ends_with(".jar") {
                        let target_path = instance.path.join("quilt-server.jar");
                        server.emit_log(format!("Found {}, renaming to quilt-server.jar", name));
                        tokio::fs::rename(entry.path(), target_path).await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
