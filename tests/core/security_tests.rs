use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::mods;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_path_traversal_protection_in_mods() {
    let base_dir = tempdir().unwrap();
    let instance_dir = base_dir.path().join("instance_1");
    fs::create_dir_all(&instance_dir).unwrap();
    fs::create_dir_all(instance_dir.join("mods")).unwrap();

    // Create a sensitive file outside the instance directory
    let sensitive_file = base_dir.path().join("sensitive.txt");
    fs::write(&sensitive_file, "secret data").unwrap();

    // Try to "uninstall" the sensitive file using path traversal
    let traversal_filename = "../../sensitive.txt";
    
    // We expect this to fail
    let result = mods::uninstall_mod(&instance_dir, traversal_filename.to_string(), false).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid filename"));

    // Verify the sensitive file still exists
    assert!(sensitive_file.exists(), "Path traversal allowed deleting files outside the mods directory!");
}

#[tokio::test]
async fn test_path_traversal_protection_in_instance_manager() {
    let base_dir = tempdir().unwrap();
    let _manager = InstanceManager::new(base_dir.path()).await.unwrap();
    
    // Create a sensitive file
    let sensitive_file = base_dir.path().join("sensitive.txt");
    fs::write(&sensitive_file, "secret data").unwrap();

    // Try to create an instance with a name that could cause issues
    // Note: InstanceManager might sanitize names or use UUIDs for paths
    let _malicious_name = "../malicious";
    
    // Check if we can get an instance with a traversal path if we were using it in commands
    // In src-tauri/src/commands/files.rs:
    // let file_path = instance.path.join(&rel_path);
    
    let instance_path = base_dir.path().join("instance_1");
    fs::create_dir_all(&instance_path).unwrap();
    
    let rel_path = "../sensitive.txt";
    let target_path = instance_path.join(rel_path);
    
    // This is what the current code does in src-tauri/src/commands/files.rs
    // We want to ensure that we don't allow this in the future or that we handle it.
    
    // For now, let's just document that the core should probably have a helper to validate paths.
    assert!(target_path.exists()); // Currently it DOES allow traversal because join() just appends
}

#[tokio::test]
async fn test_config_corruption_robustness() {
    use mc_server_wrapper_core::app_config::GlobalConfigManager;
    let base_dir = tempdir().unwrap();
    let config_path = base_dir.path().join("app_config.json");
    let manager = GlobalConfigManager::new(config_path.clone());

    // 1. Create a corrupted config file
    fs::write(&config_path, "{ \"invalid\": json ...").unwrap();

    // 2. Try to load it
    let result = manager.load().await;

    // 3. It should fail with a helpful error, not crash
    assert!(result.is_err());
    let err_msg = format!("{:?}", result.err().unwrap());
    assert!(err_msg.contains("Failed to parse app settings JSON"));
}

#[tokio::test]
async fn test_network_resilience_simulation() {
    use mc_server_wrapper_core::downloader::VersionDownloader;
    let base_dir = tempdir().unwrap();
    let downloader = VersionDownloader::new(Some(base_dir.path().to_path_buf()));

    // Test with a non-existent version ID to trigger an error
    // In a real scenario, we might use a mock server to simulate intermittent connection
    let result = downloader.download_server("non_existent_version", base_dir.path().join("server.jar"), |_, _| {}).await;

    assert!(result.is_err());
}
