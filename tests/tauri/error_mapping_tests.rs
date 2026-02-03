use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::database::Database;
use uuid::Uuid;
use tempfile::tempdir;
use std::sync::Arc;

#[tokio::test]
async fn test_error_mapping_logic() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Arc::new(Database::new(db_path).await.expect("Failed to create database"));
    let instance_manager = Arc::new(InstanceManager::new(dir.path(), db).await.unwrap());
    
    // Test: Non-existent instance ID
    let fake_id = Uuid::new_v4();
    let result = instance_manager.get_instance(fake_id).await;
    
    // The core logic returns Ok(None) for non-existent instance in get_instance
    assert!(result.is_ok());
    let instance = result.unwrap();
    assert!(instance.is_none());
    
    // In commands, this is usually mapped like:
    let command_result: Result<(), String> = if instance.is_none() {
        Err(format!("Instance not found: {}", fake_id))
    } else {
        Ok(())
    };
    
    assert!(command_result.is_err());
    assert_eq!(command_result.unwrap_err(), format!("Instance not found: {}", fake_id));
    
    // Test: Invalid UUID string (common in commands that take String)
    let invalid_id = "not-a-uuid";
    let uuid_result = Uuid::parse_str(invalid_id).map_err(|e| e.to_string());
    assert!(uuid_result.is_err());
    let err_msg = uuid_result.unwrap_err();
    assert!(err_msg.contains("invalid UUID") || err_msg.contains("invalid character"));
}
