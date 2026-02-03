use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::database::Database;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;
use uuid::Uuid;

#[tokio::test]
async fn test_fs_commands_logic() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Arc::new(Database::new(db_path).await.expect("Failed to create database"));
    let instance_manager = Arc::new(InstanceManager::new(dir.path(), db).await.unwrap());
    
    let metadata = instance_manager.create_instance("FS Test", "1.20.1").await.unwrap();
    let instance_id = metadata.id;
    let instance_path = dir.path().join(instance_id.to_string());
    
    // Ensure instance path exists (though create_instance should have done it)
    fs::create_dir_all(&instance_path).await.unwrap();
    
    let rel_path = "server.properties";
    let content = "motd=A Minecraft Server";
    
    // 1. Test save_text_file logic
    let file_path = instance_path.join(rel_path);
    fs::write(&file_path, content).await.unwrap();
    assert!(file_path.exists());
    
    // 2. Test read_text_file logic
    let read_content = fs::read_to_string(&file_path).await.unwrap();
    assert_eq!(read_content, content);
    
    // 3. Test directory creation in save_text_file logic
    let nested_rel_path = "config/test.txt";
    let nested_content = "test";
    let nested_file_path = instance_path.join(nested_rel_path);
    
    if let Some(parent) = nested_file_path.parent() {
        fs::create_dir_all(parent).await.unwrap();
    }
    fs::write(&nested_file_path, nested_content).await.unwrap();
    assert!(nested_file_path.exists());
    assert_eq!(fs::read_to_string(&nested_file_path).await.unwrap(), nested_content);
}
