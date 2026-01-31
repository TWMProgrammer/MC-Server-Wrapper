mod core;

use crate::core::config::ConfigManager;
use crate::core::server::ServerHandle;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Minecraft Server Wrapper Starting...");
    
    // 0. Initialize Directories
    let current_exe = std::env::current_exe()?;
    let exe_dir = current_exe.parent().expect("Failed to get exe directory");
    let app_dirs = crate::core::init::init_directories(exe_dir).await?;

    // 1. Handle Configuration
    let config_manager = ConfigManager::new(app_dirs.resources.join("config.toml"));
    let config = config_manager.load().await?;
    info!("Configuration loaded: {:?}", config);

    // 2. Initialize Server Handle
    let server = ServerHandle::new(config);

    // 3. Basic CLI Interaction (Simulated for Phase 1)
    info!("Server Status: {}", server.get_status().await);
    
    // Note: To actually test this, you'd need a server.jar in the current directory.
    // For now, we just demonstrate the logic.
    info!("To start the server, place a server.jar in the directory and run.");
    
    // Keep the main thread alive for a bit if we were actually running a server
    // For MVP demonstration, we just exit after initialization
    info!("MVP Core initialized successfully.");

    Ok(())
}
