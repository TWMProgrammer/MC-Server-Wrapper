use super::super::ServerManager;
use crate::server::{ServerHandle, ServerStatus};
use crate::utils::fs::is_jar_valid;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use tracing::info;
use uuid::Uuid;

impl ServerManager {
    pub async fn prepare_server(&self, instance_id: Uuid) -> Result<Arc<ServerHandle>> {
        let server = self.get_or_create_server(instance_id).await?;
        let mut instance = self
            .instance_manager
            .get_instance(instance_id)
            .await?
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

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
        let mut is_installed = if let Some(jar) = &config.jar_path {
            if is_bedrock {
                jar.exists() && std::fs::metadata(jar).map(|m| m.len() > 0).unwrap_or(false)
            } else {
                is_jar_valid(jar)
            }
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

        // If we think it's installed but it's Fabric/Quilt, double check the specific loader jar
        // because build_server_config might have picked a non-existent jar if the loader jar is corrupt
        if is_installed {
            if let Some(loader) = &instance.mod_loader {
                let loader_lower = loader.to_lowercase();
                if loader_lower == "fabric" || loader_lower == "quilt" {
                    let loader_jar_name = format!("{}-server.jar", loader_lower);
                    let loader_jar_path = instance.path.join(loader_jar_name);
                    if !is_jar_valid(&loader_jar_path) {
                        is_installed = false;
                        // Also invalidate server.jar for Fabric/Quilt if the loader jar is corrupt
                        // as they need to be installed together
                        let _ = std::fs::remove_file(instance.path.join("server.jar"));
                    }
                }
            }
        }

        // Determine the target JAR path for download if missing
        let mut download_jar_path = config
            .jar_path
            .clone()
            .unwrap_or_else(|| instance.path.join("server.jar"));

        // For Fabric/Quilt, if we are not installed, the download_jar_path MUST be server.jar
        // because build_server_config might have pointed it to fabric-server.jar which we just invalidated
        if !is_installed {
            if let Some(loader) = &instance.mod_loader {
                let loader_lower = loader.to_lowercase();
                if loader_lower == "fabric" || loader_lower == "quilt" {
                    download_jar_path = instance.path.join("server.jar");
                }
            }
        }

        // Download jar/binary if missing or corrupt
        if !is_installed {
            // Set status to Installing
            {
                let mut status = server.status.lock().await;
                *status = ServerStatus::Installing;
            }
            // Delete potentially corrupt JAR if it exists
            let jar_to_delete = if let Some(loader) = &instance.mod_loader {
                let loader_lower = loader.to_lowercase();
                if loader_lower == "fabric" || loader_lower == "quilt" {
                    Some(instance.path.join(format!("{}-server.jar", loader_lower)))
                } else {
                    config.jar_path.clone()
                }
            } else {
                config.jar_path.clone()
            };

            if let Some(jar) = jar_to_delete {
                if jar.exists() {
                    let msg = format!(
                        "Existing JAR/binary at {:?} is invalid or corrupt. Redownloading...",
                        jar
                    );
                    info!("{}", msg);
                    server.emit_log(msg);
                    let _ = std::fs::remove_file(jar);
                }
            }

            if instance.version == "Imported" {
                return Err(anyhow!(
                    "Imported instance is missing its executable (jar or bat file). Please check the instance settings."
                ));
            }
            if let Some(loader) = &instance.mod_loader {
                let loader_lower = loader.to_lowercase();
                let is_fabric = loader_lower == "fabric";
                let is_quilt = loader_lower == "quilt";
                let is_forge = loader_lower == "forge";
                let is_neoforge = loader_lower == "neoforge";

                if is_fabric {
                    self.install_fabric(Arc::clone(&server), &instance).await?;
                    // Force update instance metadata after installation to ensure
                    // build_server_config sees the new fabric-server.jar
                    instance = self
                        .instance_manager
                        .get_instance(instance_id)
                        .await?
                        .unwrap();
                } else if is_quilt {
                    self.install_quilt(Arc::clone(&server), &instance).await?;
                    // Force update instance metadata after installation
                    instance = self
                        .instance_manager
                        .get_instance(instance_id)
                        .await?
                        .unwrap();
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
                    let final_size = Arc::new(AtomicU64::new(0));
                    let final_size_clone = Arc::clone(&final_size);
                    let display_name_clone = display_name.clone();
                    self.mod_loader_client
                        .download_loader(
                            loader,
                            &instance.version,
                            instance.loader_version.as_deref(),
                            &download_jar_path,
                            move |current, total| {
                                final_size_clone.store(current, Ordering::Relaxed);
                                server_clone.handle_download_progress(
                                    current,
                                    total,
                                    &format!("Downloading {}...", display_name_clone),
                                    &last_percent,
                                );
                            },
                        )
                        .await?;

                    let size_mb = final_size.load(Ordering::Relaxed) / (1024 * 1024);
                    server.emit_log(format!("Final size: {} MB", size_mb));

                    if loader.to_lowercase() == "bedrock" {
                        server.emit_log("Extracting Bedrock server...".to_string());
                    } else if loader.to_lowercase() == "paper"
                        || loader.to_lowercase() == "velocity"
                    {
                        server.emit_log("Verifying checksum...".to_string());
                    }

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
                let final_size = Arc::new(AtomicU64::new(0));
                let final_size_clone = Arc::clone(&final_size);
                self.downloader
                    .download_server(
                        &instance.version,
                        &download_jar_path,
                        move |current, total| {
                            final_size_clone.store(current, Ordering::Relaxed);
                            server_clone.handle_download_progress(
                                current,
                                total,
                                "Downloading vanilla server...",
                                &last_percent,
                            );
                        },
                    )
                    .await?;
                let size_mb = final_size.load(Ordering::Relaxed) / (1024 * 1024);
                server.emit_log(format!("Final size: {} MB", size_mb));
                server.emit_log("Download complete!".to_string());
            }

            // Also create eula.txt if it doesn't exist (Java only)
            if !is_bedrock {
                let eula_path = instance.path.join("eula.txt");
                if !eula_path.exists() {
                    tokio::fs::write(eula_path, "eula=true").await?;
                }
            }

            // Reset status back to Stopped after installation
            {
                let mut status = server.status.lock().await;
                *status = ServerStatus::Stopped;
            }
        }

        // Update server config after potential installation (in case jar path changed or was created)
        let new_config = self.build_server_config(&instance).await;
        server.update_config(new_config).await;
        Ok(server)
    }
}
