use super::super::config::ServerConfig;
use super::types::{ProgressPayload, ResourceUsage, ServerStatus};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::process::{Child, ChildStdin};
use tokio::sync::{Mutex, broadcast};

use std::time::Instant;

#[allow(dead_code)]
#[derive(Clone)]
pub struct ServerHandle {
    pub(crate) config: Arc<Mutex<ServerConfig>>,
    pub(crate) child: Arc<Mutex<Option<Child>>>,
    pub(crate) stdin: Arc<Mutex<Option<ChildStdin>>>,
    pub(crate) status: Arc<Mutex<ServerStatus>>,
    pub(crate) usage: Arc<Mutex<ResourceUsage>>,
    pub(crate) online_players: Arc<Mutex<HashSet<String>>>,
    pub(crate) log_sender: broadcast::Sender<String>,
    pub(crate) progress_sender: broadcast::Sender<ProgressPayload>,
    pub(crate) start_time: Arc<Mutex<Option<Instant>>>,
}

impl ServerHandle {
    pub fn new(config: ServerConfig) -> Self {
        let (log_sender, _) = broadcast::channel(100);
        let (progress_sender, _) = broadcast::channel(10);
        Self {
            config: Arc::new(Mutex::new(config)),
            child: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(ServerStatus::Stopped)),
            usage: Arc::new(Mutex::new(ResourceUsage::default())),
            online_players: Arc::new(Mutex::new(HashSet::new())),
            log_sender,
            progress_sender,
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_status(&self) -> ServerStatus {
        *self.status.lock().await
    }

    pub async fn get_stop_timeout(&self) -> u64 {
        self.config.lock().await.stop_timeout
    }

    pub async fn get_usage(&self) -> ResourceUsage {
        self.usage.lock().await.clone()
    }

    pub async fn get_online_players(&self) -> Vec<String> {
        let players = self.online_players.lock().await;
        players.iter().cloned().collect()
    }

    pub async fn get_config(&self) -> ServerConfig {
        self.config.lock().await.clone()
    }

    pub fn subscribe_logs(&self) -> broadcast::Receiver<String> {
        self.log_sender.subscribe()
    }

    pub fn subscribe_progress(&self) -> broadcast::Receiver<ProgressPayload> {
        self.progress_sender.subscribe()
    }

    pub fn emit_log(&self, line: String) {
        let _ = self.log_sender.send(line);
    }

    pub fn emit_progress(&self, current: u64, total: u64, message: String) {
        let _ = self.progress_sender.send(ProgressPayload {
            current,
            total,
            message,
        });
    }

    /// Handles download progress by emitting logs with an ASCII bar and progress events.
    /// Logs are throttled to every 5% or when 100% is reached.
    pub fn handle_download_progress(
        &self,
        current: u64,
        total: u64,
        message: &str,
        last_percent: &AtomicU32,
    ) {
        let percent = if total > 0 {
            (current as f64 / total as f64 * 100.0) as u32
        } else {
            0
        };

        let prev = last_percent.load(Ordering::Relaxed);

        // Initial log or progress update
        let should_log = if prev == 0 && current == 0 {
            true // Log the very first 0% call
        } else if total > 0 {
            percent >= prev + 5 || percent == 100
        } else {
            // For chunked downloads (total == 0), log every 5MB
            let current_mb = (current / (5 * 1024 * 1024)) as u32;
            current_mb > prev
        };

        if should_log {
            if total > 0 {
                last_percent.store(percent, Ordering::Relaxed);
                let bar = generate_ascii_bar(current, total);
                self.emit_log(bar);
            } else {
                let current_mb = (current / (5 * 1024 * 1024)) as u32;
                last_percent.store(current_mb, Ordering::Relaxed);
                self.emit_log(format!("{} MB", current / (1024 * 1024)));
            }
        }
        self.emit_progress(current, total, message.to_string());
    }
}

pub fn generate_ascii_bar(current: u64, total: u64) -> String {
    if total == 0 {
        return "[--------------------] 0%".to_string();
    }
    let width = 20;
    let percentage = (current as f64 / total as f64 * 100.0) as u32;
    let progress = (current as f64 / total as f64 * width as f64).round() as usize;
    let progress = progress.min(width);
    let bar = "#".repeat(progress) + &"-".repeat(width - progress);
    format!("[{}] {}%", bar, percentage)
}
