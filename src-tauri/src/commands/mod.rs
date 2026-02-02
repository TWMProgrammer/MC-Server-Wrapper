pub mod instance;
pub mod server;
pub mod players;
pub mod config;
pub mod files;
pub mod backups;
pub mod scheduler;
pub mod plugins;

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub subscribed_servers: Arc<TokioMutex<HashSet<Uuid>>>,
}
