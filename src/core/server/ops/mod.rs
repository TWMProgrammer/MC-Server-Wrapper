pub mod lifecycle;
pub mod commands;
pub mod monitor;

use super::handle::ServerHandle;
use super::super::config::ServerConfig;

impl ServerHandle {
    pub async fn update_config(&self, new_config: ServerConfig) {
        let mut config = self.config.lock().await;
        *config = new_config;
    }
}
