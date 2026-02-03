use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::server::{ServerStatus, ResourceUsage};
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::app_config::GlobalConfigManager;
use mc_server_wrapper_core::database::Database;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_server_commands_logic() {
    // Setup
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("app_settings.json");
    let config_manager = Arc::new(GlobalConfigManager::new(config_path));
    let db_path = dir.path().join("test.db");
    let db = Arc::new(Database::new(db_path).await.expect("Failed to create database"));
    let instance_manager = Arc::new(InstanceManager::new(dir.path(), db).await.unwrap());
    let server_manager = Arc::new(ServerManager::new(instance_manager.clone(), config_manager.clone()));
    
    // Create a dummy instance
    let metadata = instance_manager.create_instance("Server Command Test", "1.20.1").await.unwrap();
    let instance_id = metadata.id;

    // 1. Test get_server_status (should be Stopped initially)
    let status = if let Some(server) = server_manager.get_server(instance_id).await {
        server.get_status().await.to_string()
    } else {
        ServerStatus::Stopped.to_string()
    };
    assert_eq!(status, "Stopped");

    // 2. Test get_server_usage (should be default)
    let usage = if let Some(server) = server_manager.get_server(instance_id).await {
        server.get_usage().await
    } else {
        ResourceUsage::default()
    };
    assert_eq!(usage.cpu_usage, 0.0);

    // 3. Test start_server logic
    let server = server_manager.get_or_create_server(instance_id).await.unwrap();
    assert_eq!(server.get_status().await, ServerStatus::Stopped);
    
    // 4. Test stop_server logic
    let result = server_manager.stop_server(instance_id).await;
    assert!(result.is_ok());

    // 5. Test send_command logic
    if let Some(server) = server_manager.get_server(instance_id).await {
        let result = server.send_command("say hello").await;
        assert!(result.is_err()); // Server is stopped
    }
}
