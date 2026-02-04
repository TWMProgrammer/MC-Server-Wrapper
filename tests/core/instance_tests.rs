use mc_server_wrapper_core::instance::{InstanceManager, InstanceMetadata};
use mc_server_wrapper_core::database::Database;
use std::sync::Arc;
use tempfile::tempdir;
use anyhow::Result;

async fn setup_manager(dir: &tempfile::TempDir) -> Result<InstanceManager> {
    let db_path = dir.path().join("test.db");
    let db = Arc::new(Database::new(db_path).await?);
    InstanceManager::new(dir.path(), db).await
}

#[tokio::test]
async fn test_instance_manager_init() -> Result<()> {
    let dir = tempdir()?;
    let manager = setup_manager(&dir).await?;
    assert!(dir.path().exists());
    assert_eq!(manager.list_instances().await?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_create_instance() -> Result<()> {
    let dir = tempdir()?;
    let manager = setup_manager(&dir).await?;
    
    let metadata = manager.create_instance("Test Server", "1.20.1").await?;
    assert_eq!(metadata.name, "Test Server");
    assert_eq!(metadata.version, "1.20.1");
    assert!(metadata.path.exists());
    
    let instances: Vec<InstanceMetadata> = manager.list_instances().await?;
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].id, metadata.id);
    Ok(())
}

#[tokio::test]
async fn test_delete_instance() -> Result<()> {
    let dir = tempdir()?;
    let manager = setup_manager(&dir).await?;
    
    let metadata = manager.create_instance("To Delete", "1.20.1").await?;
    assert!(metadata.path.exists());
    
    manager.delete_instance(metadata.id).await?;
    assert!(!metadata.path.exists());
    assert_eq!(manager.list_instances().await?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_clone_instance() -> Result<()> {
    let dir = tempdir()?;
    let manager = setup_manager(&dir).await?;
    
    let original = manager.create_instance("Original", "1.20.1").await?;
    let cloned = manager.clone_instance(original.id, "Cloned").await?;
    
    assert_eq!(cloned.name, "Cloned");
    assert_eq!(cloned.version, original.version);
    assert_ne!(cloned.id, original.id);
    assert!(cloned.path.exists());
    
    let instances: Vec<InstanceMetadata> = manager.list_instances().await?;
    assert_eq!(instances.len(), 2);
    Ok(())
}

#[tokio::test]
async fn test_update_settings() -> Result<()> {
    let dir = tempdir()?;
    let manager = setup_manager(&dir).await?;
    
    let metadata = manager.create_instance("Settings Test", "1.20.1").await?;
    let mut new_settings = metadata.settings.clone();
    new_settings.min_ram = 2;
    new_settings.max_ram = 4;
    new_settings.description = Some("Updated description".to_string());
    
    manager.update_settings(metadata.id, Some("New Name".to_string()), new_settings.clone()).await?;
    
    let updated: InstanceMetadata = manager.get_instance(metadata.id).await?.unwrap();
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.settings.min_ram, 2);
    assert_eq!(updated.settings.max_ram, 4);
    assert_eq!(updated.settings.description, Some("Updated description".to_string()));
    Ok(())
}

#[tokio::test]
async fn test_get_instance_by_name() -> Result<()> {
    let dir = tempdir()?;
    let manager = setup_manager(&dir).await?;
    
    manager.create_instance("Test Server", "1.20.1").await?;
    
    let found = manager.get_instance_by_name("Test Server").await?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Server");
    
    let not_found = manager.get_instance_by_name("Non Existent").await?;
    assert!(not_found.is_none());
    Ok(())
}

#[tokio::test]
async fn test_delete_instance_by_name() -> Result<()> {
    let dir = tempdir()?;
    let manager = setup_manager(&dir).await?;
    
    manager.create_instance("To Delete", "1.20.1").await?;
    manager.delete_instance_by_name("To Delete").await?;
    
    assert_eq!(manager.list_instances().await?.len(), 0);
    Ok(())
}
