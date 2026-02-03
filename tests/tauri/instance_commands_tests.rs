use tempfile::tempdir;
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::database::Database;
use std::sync::Arc;

#[tokio::test]
async fn test_instance_commands_integration() {
    // Note: This is an integration test for the Tauri command logic
    // In a real scenario, we might want to mock State<T> or use tauri::test helpers
    
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Arc::new(Database::new(db_path).await.expect("Failed to create database"));
    let manager = InstanceManager::new(dir.path(), db).await.unwrap();
    
    // Test logic directly (simulating what the command does)
    let instances = manager.list_instances().await.unwrap();
    assert_eq!(instances.len(), 0);
    
    let metadata = manager.create_instance("Command Test", "1.20.1").await.unwrap();
    assert_eq!(metadata.name, "Command Test");
    
    let instances = manager.list_instances().await.unwrap();
    assert_eq!(instances.len(), 1);
}
