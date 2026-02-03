use anyhow::{Result, anyhow};
use tokio::io::AsyncWriteExt;
use super::super::handle::ServerHandle;

impl ServerHandle {
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
