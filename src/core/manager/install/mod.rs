use super::ServerManager;
use crate::server::ServerHandle;
use anyhow::{Result, anyhow};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tracing::info;

pub mod fabric;
pub mod forge;
pub mod neoforge;
pub mod quilt;

impl ServerManager {
    pub(crate) async fn run_installer_command(
        &self,
        mut cmd: tokio::process::Command,
        server: Arc<ServerHandle>,
        loader_name: &str,
    ) -> Result<()> {
        let msg = format!("Running {} installer...", loader_name);
        info!("{}", msg);
        server.emit_log(msg);

        let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let server_clone = Arc::clone(&server);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                server_clone.emit_log(line);
            }
        });

        let server_clone_err = Arc::clone(&server);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                server_clone_err.emit_log(format!("ERROR: {}", line));
            }
        });

        let status = child.wait().await?;
        if !status.success() {
            return Err(anyhow!(
                "{} installer failed with status: {}",
                loader_name,
                status
            ));
        }

        Ok(())
    }
}
