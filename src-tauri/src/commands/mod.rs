pub mod assets;
pub mod backups;
pub mod config;
pub mod database;
pub mod files;
pub mod instance;
pub mod java;
pub mod mods;
pub mod players;
pub mod plugins;
pub mod scheduler;
pub mod server;

use mc_server_wrapper_core::errors::AppError;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

pub type CommandResult<T> = Result<T, AppError>;

#[derive(Clone)]
pub struct AppState {
    pub subscribed_servers: Arc<TokioMutex<HashSet<Uuid>>>,
}
