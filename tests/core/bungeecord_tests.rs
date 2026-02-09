use anyhow::Result;
use mc_server_wrapper_core::app_config::GlobalConfigManager;
use mc_server_wrapper_core::database::Database;
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use std::sync::Arc;
use tempfile::tempdir;

async fn setup_instance_manager(dir: &std::path::Path) -> Result<InstanceManager> {
    let db = Arc::new(Database::new(dir.join("test.db")).await?);
    InstanceManager::new(dir, db).await
}

#[tokio::test]
async fn test_bungeecord_stop_command() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");

    std::fs::create_dir_all(&instances_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let instance_manager = setup_instance_manager(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));

    let manager = ServerManager::new(Arc::new(instance_manager), Arc::new(config_manager));

    // Create a BungeeCord instance
    let instance = manager
        .get_instance_manager()
        .create_instance_full(
            "Bungee Test",
            "1.20.1",
            Some("bungeecord".to_string()), // Mod loader name
            Some("latest".to_string()),
        )
        .await?;

    let server = manager.get_or_create_server(instance.id).await?;
    let config = server.get_config().await;

    assert_eq!(config.stop_command, "exit");

    // Create a standard instance
    let instance_std = manager
        .get_instance_manager()
        .create_instance_full("Standard Test", "1.20.1", None, None)
        .await?;

    let server_std = manager.get_or_create_server(instance_std.id).await?;
    let config_std = server_std.get_config().await;

    assert_eq!(config_std.stop_command, "stop");

    Ok(())
}
