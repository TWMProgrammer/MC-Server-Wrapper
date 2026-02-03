use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, broadcast};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{Pid, System, ProcessesToUpdate};
use regex::Regex;
use std::sync::OnceLock;
use std::collections::HashSet;

use super::super::handle::ServerHandle;
use super::super::types::{ServerStatus, ResourceUsage};

impl ServerHandle {
    pub(crate) async fn monitor_resources(pid: u32, usage_arc: Arc<Mutex<ResourceUsage>>) {
        let mut sys = System::new_all();
        let pid = Pid::from(pid as usize);
        loop {
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
            if let Some(process) = sys.process(pid) {
                let mut usage = usage_arc.lock().await;
                usage.cpu_usage = process.cpu_usage();
                usage.memory_usage = process.memory();
            } else {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub(crate) async fn process_stdout(
        stdout: tokio::process::ChildStdout, 
        log_sender: broadcast::Sender<String>, 
        status_arc: Arc<Mutex<ServerStatus>>, 
        players_arc: Arc<Mutex<HashSet<String>>>
    ) {
        static ANSI_REGEX: OnceLock<Regex> = OnceLock::new();
        let ansi_re = ANSI_REGEX.get_or_init(|| Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap());
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = log_sender.send(line.clone());
            let line_stripped = ansi_re.replace_all(&line, "");
            let line_lower = line_stripped.to_lowercase();
            
            if Self::is_ready_line(&line_lower) {
                let mut status = status_arc.lock().await;
                if *status == ServerStatus::Starting {
                    *status = ServerStatus::Running;
                }
            }

            if line_stripped.contains("joined the game") || line_stripped.contains("connected:") {
                let mut status = status_arc.lock().await;
                if *status == ServerStatus::Starting {
                    *status = ServerStatus::Running;
                }
                drop(status);

                if let Some(name) = Self::extract_username(&line_stripped, true) {
                    players_arc.lock().await.insert(name.to_string());
                }
            } else if line_stripped.contains("left the game") || line_stripped.contains("disconnected:") {
                if let Some(name) = Self::extract_username(&line_stripped, false) {
                    players_arc.lock().await.remove(name);
                }
            }
        }
    }

    pub(crate) async fn process_stderr(stderr: tokio::process::ChildStderr, log_sender: broadcast::Sender<String>) {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = log_sender.send(format!("ERROR: {}", line));
        }
    }

    fn is_ready_line(line: &str) -> bool {
        (line.contains("done") && line.contains("for help, type \"help\""))
            || line.contains("! for help, type \"help\"")
            || line.contains("server started.")
            || line.contains("rcon running on")
            || (line.contains("done") && line.contains("(") && line.contains("s)"))
            || line.contains("timings reset")
    }

    fn extract_username(line: &str, joined: bool) -> Option<&str> {
        let pattern = if joined { "joined the game" } else { "left the game" };
        let alt_pattern = if joined { "connected: " } else { "disconnected: " };

        if line.contains(pattern) {
            line.split(pattern).next()
                .and_then(|s| s.split("INFO]: ").last())
                .or_else(|| line.split(pattern).next().and_then(|s| s.split(": ").last()))
                .map(|s| s.trim())
        } else {
            line.split(alt_pattern).nth(1)
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim())
        }
    }
}
