use super::super::handle::ServerHandle;
use crate::server::types::ServerStatus;
use anyhow::{Result, anyhow};
use tokio::io::AsyncWriteExt;

impl ServerHandle {
    pub async fn send_command(&self, command: &str) -> Result<()> {
        let cmd_trimmed = command.trim().to_lowercase();

        // Handle stop command detection for status transition
        {
            let config = self.config.lock().await;
            let is_bungeecord = config.server_type.as_deref() == Some("bungeecord");
            let is_stop_cmd = cmd_trimmed == "stop" || (is_bungeecord && cmd_trimmed == "end");

            if is_stop_cmd {
                let mut status = self.status.lock().await;
                if *status == ServerStatus::Running {
                    *status = ServerStatus::Stopping;
                }
            }
        }

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
