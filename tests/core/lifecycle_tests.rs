use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::app_config::GlobalConfigManager;
use tempfile::tempdir;
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_get_or_create_server() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");
    
    std::fs::create_dir_all(&instances_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let instance_manager = InstanceManager::new(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));
    
    let manager = ServerManager::new(
        Arc::new(instance_manager),
        Arc::new(config_manager),
    );

    // Create an instance
    let instance = manager.get_instance_manager().create_instance("Test Server", "1.20.1").await?;
    
    // Get server handle
    let server = manager.get_or_create_server(instance.id).await?;
    assert_eq!(server.get_config().await.name, "Test Server");
    
    // Getting it again should return the same handle
    let server2 = manager.get_or_create_server(instance.id).await?;
    assert!(Arc::ptr_eq(&server, &server2));

    Ok(())
}

#[tokio::test]
async fn test_prepare_server_fabric_no_vanilla_jar() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");
    
    std::fs::create_dir_all(&instances_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let instance_manager = InstanceManager::new(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));
    
    let manager = ServerManager::new(
        Arc::new(instance_manager),
        Arc::new(config_manager),
    );

    // Create a Fabric instance
    let instance = manager.get_instance_manager().create_instance_full("Fabric Test", "1.20.1", Some("fabric".to_string()), Some("0.14.22".to_string())).await?;
    
    // Create fabric-server.jar but NOT server.jar
    let fabric_jar_path = instance.path.join("fabric-server.jar");
    std::fs::write(&fabric_jar_path, b"dummy fabric content")?;
    
    // Prepare the server - this should NOT trigger a download
    // Since we can't easily check if download was skipped without mocking, 
    // we check if server.jar was NOT created.
    let server = manager.prepare_server(instance.id).await?;
    
    let jar_path = instance.path.join("server.jar");
    assert!(!jar_path.exists(), "server.jar should not have been created for Fabric if fabric-server.jar exists");
    
    let config = server.get_config().await;
    assert_eq!(config.jar_path, Some(fabric_jar_path));

    Ok(())
}

#[tokio::test]
async fn test_prepare_server_vanilla_missing() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");
    
    std::fs::create_dir_all(&instances_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let instance_manager = InstanceManager::new(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));
    
    let manager = ServerManager::new(
        Arc::new(instance_manager),
        Arc::new(config_manager),
    );

    let instance = manager.get_instance_manager().create_instance("Vanilla Test", "1.20.1").await?;
    
    let jar_path = instance.path.join("server.jar");
    std::fs::write(&jar_path, b"dummy jar content")?;
    
    let server = manager.prepare_server(instance.id).await?;
    assert_eq!(server.get_config().await.jar_path, Some(jar_path));

    Ok(())
}

#[tokio::test]
async fn test_prepare_server_with_run_script() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");
    
    std::fs::create_dir_all(&instances_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let instance_manager = InstanceManager::new(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));
    
    let manager = ServerManager::new(
        Arc::new(instance_manager),
        Arc::new(config_manager),
    );

    let instance = manager.get_instance_manager().create_instance("Script Test", "1.20.1").await?;
    
    let run_script = if cfg!(windows) { "run.bat" } else { "run.sh" };
    let script_path = instance.path.join(run_script);
    std::fs::write(&script_path, b"echo test")?;
    
    let server = manager.prepare_server(instance.id).await?;
    let config = server.get_config().await;
    assert_eq!(config.run_script, Some(run_script.to_string()));
    assert_eq!(config.jar_path, None);

    Ok(())
}

#[tokio::test]
async fn test_prepare_server_paper_mock() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");
    
    std::fs::create_dir_all(&instances_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let instance_manager = InstanceManager::new(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));
    
    let manager = ServerManager::new(
        Arc::new(instance_manager),
        Arc::new(config_manager),
    );

    let instance = manager.get_instance_manager().create_instance_full(
        "Paper Test", 
        "1.20.1", 
        Some("Paper".to_string()), 
        Some("123".to_string())
    ).await?;
    
    let jar_path = instance.path.join("server.jar");
    std::fs::write(&jar_path, b"dummy paper jar")?;
    
    let server = manager.prepare_server(instance.id).await?;
    let config = server.get_config().await;
    assert_eq!(config.jar_path, Some(jar_path));

    Ok(())
}

#[tokio::test]
async fn test_prepare_server_invalid_instance() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");
    
    std::fs::create_dir_all(&instances_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let instance_manager = InstanceManager::new(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));
    
    let manager = ServerManager::new(
        Arc::new(instance_manager),
        Arc::new(config_manager),
    );

    let result = manager.prepare_server(Uuid::new_v4()).await;
    assert!(result.is_err());
    
    if let Err(e) = result {
        assert!(e.to_string().contains("Instance not found"));
    }

    Ok(())
}
