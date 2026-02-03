use std::process::Stdio;
use tokio::process::{Command, Child, ChildStdin};
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::sync::{Mutex, broadcast};
use anyhow::{Result, anyhow};
use tracing::{info, error, warn};
use std::sync::Arc;
use std::time::Duration;
use std::path::PathBuf;
use sysinfo::{Pid, System, ProcessesToUpdate};
use regex::Regex;
use std::sync::OnceLock;
use std::collections::HashSet;

use super::handle::ServerHandle;
use super::types::{ServerStatus, ResourceUsage};
use super::super::config::ServerConfig;

impl ServerHandle {
    pub async fn update_config(&self, new_config: ServerConfig) {
        let mut config = self.config.lock().await;
        *config = new_config;
    }

    pub async fn start(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        if matches!(*status, ServerStatus::Running | ServerStatus::Starting) {
            return Ok(());
        }

        *status = ServerStatus::Starting;
        
        let config = Arc::clone(&self.config);
        let status = Arc::clone(&self.status);
        let child = Arc::clone(&self.child);
        let stdin = Arc::clone(&self.stdin);
        let usage = Arc::clone(&self.usage);
        let online_players = Arc::clone(&self.online_players);
        let log_sender = self.log_sender.clone();
        let progress_sender = self.progress_sender.clone();

        tokio::spawn(async move {
            Self::lifecycle_loop(
                config, status, child, stdin, usage, online_players, log_sender, progress_sender
            ).await;
        });

        Ok(())
    }

    async fn lifecycle_loop(
        config_arc: Arc<Mutex<ServerConfig>>,
        status_arc: Arc<Mutex<ServerStatus>>,
        child_arc: Arc<Mutex<Option<Child>>>,
        stdin_arc: Arc<Mutex<Option<ChildStdin>>>,
        usage_arc: Arc<Mutex<ResourceUsage>>,
        online_players_arc: Arc<Mutex<HashSet<String>>>,
        log_sender: broadcast::Sender<String>,
        _progress_sender: broadcast::Sender<super::types::ProgressPayload>,
    ) {
        loop {
            let config: ServerConfig = config_arc.lock().await.clone();
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
                let is_jar = config.jar_path.as_ref()
                    .map(|p| p.to_string_lossy().to_lowercase().ends_with(".jar"))
                    .unwrap_or(true);

                if is_jar {
                    let java_cmd = config.java_path.as_ref()
                        .map(|p: &PathBuf| p.to_string_lossy().to_string())
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

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to spawn Minecraft server process: {}", e);
                    let _ = log_sender.send(format!("ERROR: Failed to spawn process: {}", e));
                    let mut status = status_arc.lock().await;
                    *status = ServerStatus::Crashed;
                    break;
                }
            };

            let pid = child.id().unwrap_or(0);
            let stdout = child.stdout.take().expect("Failed to open stdout");
            let stderr = child.stderr.take().expect("Failed to open stderr");
            let stdin = child.stdin.take().expect("Failed to open stdin");

            {
                let mut child_lock = child_arc.lock().await;
                *child_lock = Some(child);
                let mut stdin_lock = stdin_arc.lock().await;
                *stdin_lock = Some(stdin);
            }

            // Monitor and Log tasks
            let status_clone: Arc<Mutex<ServerStatus>> = Arc::clone(&status_arc);
            let usage_clone: Arc<Mutex<ResourceUsage>> = Arc::clone(&usage_arc);
            let _online_players_clone: Arc<Mutex<HashSet<String>>> = Arc::clone(&online_players_arc);
            let _log_sender_clone = log_sender.clone();
            
            let monitor_handle = tokio::spawn(async move {
                let mut sys = System::new_all();
                let pid = Pid::from(pid as usize);
                loop {
                    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                    if let Some(process) = sys.process(pid) {
                        let mut usage: tokio::sync::MutexGuard<ResourceUsage> = usage_clone.lock().await;
                        usage.cpu_usage = process.cpu_usage();
                        usage.memory_usage = process.memory();
                    } else {
                        // Process gone, but we wait for child.wait() in the main loop
                        break;
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });

            let log_sender_stdout = log_sender.clone();
            let status_log: Arc<Mutex<ServerStatus>> = Arc::clone(&status_clone);
            let players_log: Arc<Mutex<HashSet<String>>> = Arc::clone(&online_players_arc);
            let stdout_handle = tokio::spawn(async move {
                static ANSI_REGEX: OnceLock<Regex> = OnceLock::new();
                let ansi_re = ANSI_REGEX.get_or_init(|| Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap());
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = log_sender_stdout.send(line.clone());
                    let line_stripped = ansi_re.replace_all(&line, "");
                    let line_lower = line_stripped.to_lowercase();
                    
                    let is_ready = (line_lower.contains("done") && line_lower.contains("for help, type \"help\""))
                        || line_lower.contains("! for help, type \"help\"")
                        || line_lower.contains("server started.")
                        || line_lower.contains("rcon running on")
                        || (line_lower.contains("done") && line_lower.contains("(") && line_lower.contains("s)"))
                        || line_lower.contains("timings reset");

                    if is_ready {
                        let mut status: tokio::sync::MutexGuard<ServerStatus> = status_log.lock().await;
                        if *status == ServerStatus::Starting {
                            *status = ServerStatus::Running;
                        }
                    }

                    if line_stripped.contains("joined the game") || line_stripped.contains("connected:") {
                        let mut status: tokio::sync::MutexGuard<ServerStatus> = status_log.lock().await;
                        if *status == ServerStatus::Starting {
                            *status = ServerStatus::Running;
                        }
                        drop(status);

                        let username = if line_stripped.contains("joined the game") {
                            line_stripped.split("joined the game").next().and_then(|s| s.split("INFO]: ").last()).or_else(|| line_stripped.split("joined the game").next().and_then(|s| s.split(": ").last())).map(|s| s.trim())
                        } else {
                            line_stripped.split("connected: ").nth(1).and_then(|s| s.split(',').next()).map(|s| s.trim())
                        };

                        if let Some(name) = username {
                            if !name.is_empty() {
                                let mut players: tokio::sync::MutexGuard<HashSet<String>> = players_log.lock().await;
                                players.insert(name.to_string());
                            }
                        }
                    } else if line_stripped.contains("left the game") || line_stripped.contains("disconnected:") {
                        let username = if line_stripped.contains("left the game") {
                            line_stripped.split("left the game").next().and_then(|s| s.split("INFO]: ").last()).or_else(|| line_stripped.split("left the game").next().and_then(|s| s.split(": ").last())).map(|s| s.trim())
                        } else {
                            line_stripped.split("disconnected: ").nth(1).and_then(|s| s.split(',').next()).map(|s| s.trim())
                        };
                        if let Some(name) = username {
                            if !name.is_empty() {
                                let mut players: tokio::sync::MutexGuard<HashSet<String>> = players_log.lock().await;
                                players.remove(name);
                            }
                        }
                    }
                }
            });

            let log_sender_stderr = log_sender.clone();
            let stderr_handle = tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = log_sender_stderr.send(format!("ERROR: {}", line));
                }
            });

            // Wait for process to exit
            let mut child: Child = {
                let mut child_lock: tokio::sync::MutexGuard<Option<Child>> = child_arc.lock().await;
                child_lock.take().expect("Child disappeared")
            };
            let exit_status = child.wait().await;
            
            // Give log handles a moment to finish processing remaining buffer
            let _ = tokio::time::timeout(Duration::from_millis(500), stdout_handle).await;
            let _ = tokio::time::timeout(Duration::from_millis(500), stderr_handle).await;
            monitor_handle.abort();

            let mut status: tokio::sync::MutexGuard<ServerStatus> = status_arc.lock().await;
            let current_status = *status;

            if current_status == ServerStatus::Stopping {
                info!("Server stopped gracefully.");
                *status = ServerStatus::Stopped;
                let mut stdin: tokio::sync::MutexGuard<Option<ChildStdin>> = stdin_arc.lock().await;
                *stdin = None;
                let mut players: tokio::sync::MutexGuard<HashSet<String>> = online_players_arc.lock().await;
                players.clear();
                break;
            } else {
                let exit_msg = match &exit_status {
                    Ok(s) => format!("Server process exited unexpectedly with status: {}", s),
                    Err(e) => format!("Error waiting for server process: {}", e),
                };
                error!("{}", exit_msg);
                let _ = log_sender.send(format!("CRASH: {}", exit_msg));
                *status = ServerStatus::Crashed;
                
                let mut stdin: tokio::sync::MutexGuard<Option<ChildStdin>> = stdin_arc.lock().await;
                *stdin = None;
                let mut players: tokio::sync::MutexGuard<HashSet<String>> = online_players_arc.lock().await;
                players.clear();

                let crash_handling = {
                    let config = config_arc.lock().await;
                    config.crash_handling.clone()
                };

                let should_restart = match crash_handling {
                    super::super::instance::CrashHandlingMode::Nothing => false,
                    super::super::instance::CrashHandlingMode::Elevated => {
                        match &exit_status {
                            Ok(s) => !s.success(),
                            Err(_) => true,
                        }
                    },
                    super::super::instance::CrashHandlingMode::Aggressive => true,
                };

                if should_restart {
                    info!("Crash handling mode {:?} active. Restarting in 5 seconds...", crash_handling);
                    let _ = log_sender.send(format!("Crash handling mode {:?} active. Restarting in 5 seconds...", crash_handling));
                    drop(status);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let mut status = status_arc.lock().await;
                    *status = ServerStatus::Starting;
                    continue;
                } else {
                    break;
                }
            }
        }
    }

    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        
        if *status == ServerStatus::Stopped {
            return Ok(());
        }

        if *status == ServerStatus::Stopping {
            return Ok(());
        }

        *status = ServerStatus::Stopping;
        let stop_timeout = self.config.lock().await.stop_timeout;
        drop(status);

        if let Err(e) = self.send_command("stop").await {
            warn!("Failed to send stop command: {}. Falling back to kill.", e);
        }

        // Wait for it to stop with timeout
        let start_wait = std::time::Instant::now();
        let wait_limit = Duration::from_secs(stop_timeout);
        
        while start_wait.elapsed() < wait_limit {
            if *self.status.lock().await == ServerStatus::Stopped {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Kill if still not stopped
        warn!("Server failed to exit gracefully. Killing process.");
        let mut child_lock = self.child.lock().await;
        if let Some(mut child) = child_lock.take() {
            #[cfg(target_os = "windows")]
            {
                if let Some(pid) = child.id() {
                    let _ = Command::new("taskkill").arg("/F").arg("/T").arg("/PID").arg(pid.to_string()).output().await;
                }
            }
            let _ = child.kill().await;
        }
        
        let mut status = self.status.lock().await;
        *status = ServerStatus::Stopped;
        self.stdin.lock().await.take();
        self.online_players.lock().await.clear();

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
}
