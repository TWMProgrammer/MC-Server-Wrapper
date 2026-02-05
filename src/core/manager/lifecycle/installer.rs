use super::super::ServerManager;
use crate::server::{ServerHandle, generate_ascii_bar};
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::info;
use uuid::Uuid;

impl ServerManager {
    pub async fn prepare_server(&self, instance_id: Uuid) -> Result<Arc<ServerHandle>> {
        let server = self.get_or_create_server(instance_id).await?;
        let instance = self
            .instance_manager
            .get_instance(instance_id)
            .await?
            .ok_or_else(|| anyhow!("Instance not found"))?;

        let is_bedrock = instance
            .mod_loader
            .as_deref()
            .map(|l| l.to_lowercase() == "bedrock")
            .unwrap_or(false);
        let bedrock_exe = if cfg!(windows) {
            "bedrock_server.exe"
        } else {
            "bedrock_server"
        };
        let bedrock_path = instance.path.join(bedrock_exe);

        // Robust check for existing installation using the built config
        let config = server.get_config().await;
        let exists = if let Some(jar) = &config.jar_path {
            jar.exists()
        } else if let Some(script) = &config.run_script {
            instance.path.join(script).exists()
        } else {
            // Fallback for Bedrock or edge cases where neither is set but directory isn't empty
            (is_bedrock && bedrock_path.exists())
                || (instance.path.exists()
                    && std::fs::read_dir(&instance.path)
                        .map(|mut d| d.next().is_some())
                        .unwrap_or(false))
        };

        // Determine the target JAR path for download if missing
        let download_jar_path = config
            .jar_path
            .unwrap_or_else(|| instance.path.join("server.jar"));

        // Download jar/binary if missing
        if !exists {
            if instance.version == "Imported" {
                return Err(anyhow!(
                    "Imported instance is missing its executable (jar or bat file). Please check the instance settings."
                ));
            }
            if let Some(loader) = &instance.mod_loader {
                let loader_lower = loader.to_lowercase();
                let is_fabric = loader_lower == "fabric";
                let is_forge = loader_lower == "forge";
                let is_neoforge = loader_lower == "neoforge";

                if is_fabric {
                    self.install_fabric(Arc::clone(&server), &instance).await?;
                } else if is_forge {
                    self.install_forge(Arc::clone(&server), &instance).await?;
                } else if is_neoforge {
                    self.install_neoforge(Arc::clone(&server), &instance)
                        .await?;
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

                    let msg = format!(
                        "Starting download of {} for version {}",
                        display_name, instance.version
                    );
                    info!("{}", msg);
                    server.emit_log(msg);

                    let last_percent = Arc::new(AtomicU32::new(0));
                    let display_name_clone = display_name.clone();
                    self.mod_loader_client
                        .download_loader(
                            loader,
                            &instance.version,
                            instance.loader_version.as_deref(),
                            &download_jar_path,
                            move |current, total| {
                                let percent = if total > 0 {
                                    (current as f64 / total as f64 * 100.0) as u32
                                } else {
                                    0
                                };
                                let prev = last_percent.load(Ordering::Relaxed);
                                if percent >= prev + 5 || percent == 100 {
                                    last_percent.store(percent, Ordering::Relaxed);
                                    let bar = generate_ascii_bar(current, total);
                                    server_clone.emit_log(format!(
                                        "Downloading {}... {}",
                                        display_name_clone, bar
                                    ));
                                }
                                server_clone.emit_progress(
                                    current,
                                    total,
                                    format!("Downloading {}...", display_name_clone),
                                );
                            },
                        )
                        .await?;
                    server.emit_log("Download complete!".to_string());
                }
            } else {
                let msg = format!(
                    "Starting download of vanilla server for version {}",
                    instance.version
                );
                info!("{}", msg);
                server.emit_log(msg);
                let server_clone = Arc::clone(&server);
                let last_percent = Arc::new(AtomicU32::new(0));
                self.downloader
                    .download_server(
                        &instance.version,
                        &download_jar_path,
                        move |current, total| {
                            let percent = if total > 0 {
                                (current as f64 / total as f64 * 100.0) as u32
                            } else {
                                0
                            };
                            let prev = last_percent.load(Ordering::Relaxed);
                            if percent >= prev + 5 || percent == 100 {
                                last_percent.store(percent, Ordering::Relaxed);
                                let bar = generate_ascii_bar(current, total);
                                server_clone
                                    .emit_log(format!("Downloading vanilla server... {}", bar));
                            }
                            server_clone.emit_progress(
                                current,
                                total,
                                "Downloading vanilla server...".to_string(),
                            );
                        },
                    )
                    .await?;
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

        // Update server config after potential installation (in case jar path changed or was created)
        let config = self.build_server_config(&instance).await;
        server.update_config(config).await;
        Ok(server)
    }
}
