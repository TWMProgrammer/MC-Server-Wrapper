use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use anyhow::{Context, Result, anyhow};
use tracing::{info, error, warn};
use std::sync::Arc;
use std::time::Duration;
use std::path::PathBuf;
use sysinfo::{Pid, System, ProcessesToUpdate};

use super::handle::ServerHandle;
use super::types::{ServerStatus, ResourceUsage};
use super::super::config::ServerConfig;

impl ServerHandle {
    pub async fn update_config(&self, new_config: ServerConfig) {
        let mut config = self.config.lock().await;
        *config = new_config;
    }

    #[allow(dead_code)]
    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        if matches!(*status, ServerStatus::Running | ServerStatus::Starting) {
            return Ok(());
        }

        *status = ServerStatus::Starting;
        let config = self.config.lock().await;
        info!("Starting server: {}", config.name);

        let mut cmd = if let Some(script) = &config.run_script {
            #[cfg(target_os = "windows")]
            {
                let mut c = Command::new("cmd");
                c.arg("/c").arg(script);
                c
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut c = Command::new("sh");
                c.arg(script);
                c
            }
        } else {
            let mut is_jar = true;
            if let Some(jar_path) = &config.jar_path {
                let path_str = jar_path.to_string_lossy().to_lowercase();
                if !path_str.ends_with(".jar") {
                    is_jar = false;
                }
            }

            if is_jar {
                let java_cmd = config.java_path.as_ref()
                    .map(|p: &PathBuf| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "java".to_string());

                let mut c = Command::new(java_cmd);
                c.arg(format!("-Xmx{}", config.max_memory))
                 .arg(format!("-Xms{}", config.min_memory));

                if let Some(jar_path) = &config.jar_path {
                    c.arg("-jar").arg(jar_path);
                }
                c
            } else {
                // Non-jar executable (e.g. Bedrock)
                let exe_path = config.jar_path.as_ref().unwrap();
                Command::new(exe_path)
            }
        };

        cmd.current_dir(&config.working_dir);

        for arg in &config.args {
            cmd.arg(arg);
        }

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn Minecraft server process")?;
        let pid = child.id().context("Failed to get child PID")?;
        
        let stdout = child.stdout.take().expect("Failed to open stdout");
        let stderr = child.stderr.take().expect("Failed to open stderr");
        let stdin = child.stdin.take().expect("Failed to open stdin");

        let status_clone = Arc::clone(&self.status);
        let usage_clone = Arc::clone(&self.usage);
        let status_clone_for_monitor = Arc::clone(&self.status);
        
        // Monitoring task
        tokio::spawn(async move {
            let mut sys = System::new_all();
            let pid = Pid::from(pid as usize);
            
            loop {
                {
                    let status = status_clone_for_monitor.lock().await;
                    if *status == ServerStatus::Stopped {
                        break;
                    }
                }

                sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                if let Some(process) = sys.process(pid) {
                    let mut usage = usage_clone.lock().await;
                    usage.cpu_usage = process.cpu_usage();
                    usage.memory_usage = process.memory();
                } else {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            
            let mut usage = usage_clone.lock().await;
            *usage = ResourceUsage::default();
        });

        // Output capture tasks
        let log_sender_stdout = self.log_sender.clone();
        let online_players_clone = Arc::clone(&self.online_players);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                info!("[Server Output] {}", line);
                let _ = log_sender_stdout.send(line.clone());
                
                if line.contains("Done") && line.contains("For help, type \"help\"") {
                    let mut status = status_clone.lock().await;
                    if *status == ServerStatus::Starting {
                        *status = ServerStatus::Running;
                        info!("Server is now Running");
                    }
                }

                if line.contains("joined the game") {
                    if let Some(username) = line.split("INFO]: ").nth(1).and_then(|s| s.split(' ').next()) {
                        let mut players = online_players_clone.lock().await;
                        players.insert(username.to_string());
                    }
                } else if line.contains("left the game") {
                    if let Some(username) = line.split("INFO]: ").nth(1).and_then(|s| s.split(' ').next()) {
                        let mut players = online_players_clone.lock().await;
                        players.remove(username);
                    }
                }
            }
        });

        let log_sender_stderr = self.log_sender.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                error!("[Server Error] {}", line);
                let _ = log_sender_stderr.send(format!("ERROR: {}", line));
            }
        });

        let mut child_lock = self.child.lock().await;
        let mut stdin_lock = self.stdin.lock().await;
        *child_lock = Some(child);
        *stdin_lock = Some(stdin);

        Ok(())
    }

    pub async fn send_command(&self, command: &str) -> Result<()> {
        let mut stdin_lock = self.stdin.lock().await;
        if let Some(stdin) = stdin_lock.as_mut() {
            let cmd = format!("{}\n", command);
            stdin.write_all(cmd.as_bytes()).await?;
            stdin.flush().await?;
            Ok(())
        } else {
            Err(anyhow!("Server is not running or stdin is unavailable"))
        }
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        
        if *status == ServerStatus::Stopped {
            return Ok(());
        }

        *status = ServerStatus::Stopping;
        {
            let config = self.config.lock().await;
            info!("Stopping server: {}", config.name);
        }

        if let Err(e) = self.send_command("stop").await {
            warn!("Failed to send stop command: {}. Falling back to kill.", e);
        }

        let child_arc = Arc::clone(&self.child);
        let status_arc = Arc::clone(&self.status);
        let stdin_arc = Arc::clone(&self.stdin);
        let online_players_arc = Arc::clone(&self.online_players);

        tokio::spawn(async move {
            let mut child_lock = child_arc.lock().await;
            if let Some(mut child) = child_lock.take() {
                let wait_timeout = Duration::from_secs(30);
                match tokio::time::timeout(wait_timeout, child.wait()).await {
                    Ok(Ok(exit_status)) => {
                        info!("Server exited gracefully with status: {}", exit_status);
                    }
                    _ => {
                        warn!("Server failed to exit gracefully. Killing process.");
                        let _ = child.kill().await;
                    }
                }
            }
            let mut status = status_arc.lock().await;
            *status = ServerStatus::Stopped;
            let mut stdin_lock = stdin_arc.lock().await;
            *stdin_lock = None;
            
            let mut players = online_players_arc.lock().await;
            players.clear();
        });

        Ok(())
    }
}
