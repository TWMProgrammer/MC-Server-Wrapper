use anyhow::Result;
use tracing::warn;
use std::time::Duration;
use tokio::process::Command;
use crate::server::types::ServerStatus;
use crate::server::handle::ServerHandle;

impl ServerHandle {
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        if matches!(*status, ServerStatus::Stopped | ServerStatus::Stopping) {
            return Ok(());
        }

        *status = ServerStatus::Stopping;
        let stop_timeout = self.config.lock().await.stop_timeout;
        drop(status);

        if let Err(e) = self.send_command("stop").await {
            warn!("Failed to send stop command: {}. Falling back to kill.", e);
        }

        let start_wait = std::time::Instant::now();
        let wait_limit = Duration::from_secs(stop_timeout);
        
        while start_wait.elapsed() < wait_limit {
            if *self.status.lock().await == ServerStatus::Stopped {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        warn!("Server failed to exit gracefully. Killing process.");
        let mut child_lock = self.child.lock().await;
        if let Some(mut child) = child_lock.take() {
            #[cfg(target_os = "windows")]
            if let Some(pid) = child.id() {
                let _ = Command::new("taskkill").arg("/F").arg("/T").arg("/PID").arg(pid.to_string()).output().await;
            }
            let _ = child.kill().await;
        }
        
        let mut status = self.status.lock().await;
        *status = ServerStatus::Stopped;
        *self.stdin.lock().await = None;
        self.online_players.lock().await.clear();
        Ok(())
    }

    pub async fn kill(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        if *status == ServerStatus::Stopped {
            return Ok(());
        }

        let mut child_lock = self.child.lock().await;
        if let Some(mut child) = child_lock.take() {
            #[cfg(target_os = "windows")]
            if let Some(pid) = child.id() {
                let _ = Command::new("taskkill").arg("/F").arg("/T").arg("/PID").arg(pid.to_string()).output().await;
            }
            let _ = child.kill().await;
        }

        *status = ServerStatus::Stopped;
        *self.stdin.lock().await = None;
        self.online_players.lock().await.clear();
        Ok(())
    }
}
