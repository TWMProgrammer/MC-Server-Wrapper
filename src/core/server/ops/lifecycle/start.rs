use std::sync::Arc;
use anyhow::Result;
use crate::server::types::ServerStatus;
use crate::server::handle::ServerHandle;

impl ServerHandle {
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
        let start_time = Arc::clone(&self.start_time);

        tokio::spawn(async move {
            Self::lifecycle_loop(
                config, status, child, stdin, usage, online_players, log_sender, progress_sender, start_time
            ).await;
        });

        Ok(())
    }
}
