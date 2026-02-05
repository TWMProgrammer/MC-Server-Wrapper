use std::process::Stdio;
use tokio::process::{Command, Child, ChildStdin};
use tokio::sync::{Mutex, broadcast};
use tracing::{info, error};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashSet;

use crate::server::handle::ServerHandle;
use crate::server::types::{ServerStatus, ResourceUsage, ProgressPayload};
use crate::config::ServerConfig;
use crate::instance::CrashHandlingMode;

impl ServerHandle {
    pub(crate) async fn lifecycle_loop(
        config_arc: Arc<Mutex<ServerConfig>>,
        status_arc: Arc<Mutex<ServerStatus>>,
        child_arc: Arc<Mutex<Option<Child>>>,
        stdin_arc: Arc<Mutex<Option<ChildStdin>>>,
        usage_arc: Arc<Mutex<ResourceUsage>>,
        online_players_arc: Arc<Mutex<HashSet<String>>>,
        log_sender: broadcast::Sender<String>,
        _progress_sender: broadcast::Sender<ProgressPayload>,
        start_time_arc: Arc<Mutex<Option<std::time::Instant>>>,
    ) {
        loop {
            let config = config_arc.lock().await.clone();
            info!("Starting server: {}", config.name);

            let mut cmd = Self::build_command(&config);
            cmd.current_dir(&config.working_dir);
            for arg in &config.args {
                cmd.arg(arg);
            }

            cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to spawn Minecraft server process: {}", e);
                    let _ = log_sender.send(format!("ERROR: Failed to spawn process: {}", e));
                    *status_arc.lock().await = ServerStatus::Crashed;
                    break;
                }
            };

            let pid = child.id().unwrap_or(0);
            let stdout = child.stdout.take().expect("Failed to open stdout");
            let stderr = child.stderr.take().expect("Failed to open stderr");
            let stdin = child.stdin.take().expect("Failed to open stdin");

            {
                *child_arc.lock().await = Some(child);
                *stdin_arc.lock().await = Some(stdin);
                *start_time_arc.lock().await = Some(std::time::Instant::now());
            }

            let monitor_handle = tokio::spawn(Self::monitor_resources(pid, Arc::clone(&usage_arc), Arc::clone(&start_time_arc), Arc::clone(&online_players_arc)));
            let stdout_handle = tokio::spawn(Self::process_stdout(stdout, log_sender.clone(), Arc::clone(&status_arc), Arc::clone(&online_players_arc)));
            let stderr_handle = tokio::spawn(Self::process_stderr(stderr, log_sender.clone()));

            let mut child = child_arc.lock().await.take().expect("Child disappeared");
            let exit_status = child.wait().await;
            
            let _ = tokio::time::timeout(Duration::from_millis(500), stdout_handle).await;
            let _ = tokio::time::timeout(Duration::from_millis(500), stderr_handle).await;
            monitor_handle.abort();

            {
                *start_time_arc.lock().await = None;
                let mut usage = usage_arc.lock().await;
                usage.cpu_usage = 0.0;
                usage.memory_usage = 0;
                usage.uptime = 0;
                usage.player_count = 0;
            }

            let mut status = status_arc.lock().await;
            let exited_cleanly = exit_status.as_ref().map(|s| s.success()).unwrap_or(false);

            if *status == ServerStatus::Stopping || *status == ServerStatus::Stopped || exited_cleanly {
                info!("Server stopped gracefully.");
                *status = ServerStatus::Stopped;
                *stdin_arc.lock().await = None;
                online_players_arc.lock().await.clear();
                break;
            } else {
                let exit_msg = match &exit_status {
                    Ok(s) => format!("Server process exited unexpectedly with status: {}", s),
                    Err(e) => format!("Error waiting for server process: {}", e),
                };
                error!("{}", exit_msg);
                let _ = log_sender.send(format!("CRASH: {}", exit_msg));
                *status = ServerStatus::Crashed;
                *stdin_arc.lock().await = None;
                online_players_arc.lock().await.clear();

                let should_restart = match config_arc.lock().await.crash_handling {
                    CrashHandlingMode::Nothing => false,
                    CrashHandlingMode::Elevated => exit_status.as_ref().map(|s| !s.success()).unwrap_or(true),
                    CrashHandlingMode::Aggressive => true,
                };

                if should_restart {
                    info!("Crash handling mode active. Restarting in 5 seconds...");
                    let _ = log_sender.send("Crash handling mode active. Restarting in 5 seconds...".to_string());
                    drop(status);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    *status_arc.lock().await = ServerStatus::Starting;
                    continue;
                } else {
                    break;
                }
            }
        }
    }

    pub(crate) fn build_command(config: &ServerConfig) -> Command {
        if let Some(script) = &config.run_script {
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
            let is_jar = config.jar_path.as_ref()
                .map(|p| p.to_string_lossy().to_lowercase().ends_with(".jar"))
                .unwrap_or(true);

            if is_jar {
                let java_cmd = config.java_path.as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "java".to_string());

                let mut c = Command::new(java_cmd);
                c.arg(format!("-Xmx{}", config.max_memory))
                 .arg(format!("-Xms{}", config.min_memory))
                 .arg("-Dterminal.jline=false")
                 .arg("-Dterminal.ansi=true")
                 .arg("-Dlog4j.skipJansi=false");

                if let Some(jar_path) = &config.jar_path {
                    c.arg("-jar").arg(jar_path);
                }
                c
            } else {
                Command::new(config.jar_path.as_ref().unwrap())
            }
        }
    }
}
