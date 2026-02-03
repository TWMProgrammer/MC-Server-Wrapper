use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::database::Database;
use mc_server_wrapper_core::mods;
use mc_server_wrapper_core::plugins;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;
use std::io::Write;

async fn setup_instance_manager(dir: &std::path::Path) -> Arc<InstanceManager> {
    let db_path = dir.join("test.db");
    let db = Arc::new(Database::new(db_path).await.expect("Failed to create database"));
    Arc::new(InstanceManager::new(dir, db).await.unwrap())
}

async fn create_empty_jar(path: &std::path::Path) {
    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file("dummy.txt", zip::write::SimpleFileOptions::default()).unwrap();
    zip.write_all(b"dummy content").unwrap();
    zip.finish().unwrap();
}

#[tokio::test]
async fn test_mod_commands_logic() {
    let dir = tempdir().unwrap();
    let instance_manager = setup_instance_manager(dir.path()).await;
    
    let metadata = instance_manager.create_instance("Mod Test", "1.20.1").await.unwrap();
    let instance_id = metadata.id;
    let instance_path = dir.path().join(instance_id.to_string());
    
    // Create mods directory
    let mods_dir = instance_path.join("mods");
    fs::create_dir_all(&mods_dir).await.unwrap();
    
    // Create a dummy mod file (valid zip)
    let mod_file = mods_dir.join("test-mod.jar");
    create_empty_jar(&mod_file).await;
    
    // Test list_installed_mods logic
    let installed_mods = mods::list_installed_mods(&instance_path).await.unwrap();
    assert_eq!(installed_mods.len(), 1);
    assert_eq!(installed_mods[0].name, "test-mod");
    
    // Test toggle_mod logic (disable)
    mods::toggle_mod(&instance_path, "test-mod.jar".to_string(), false).await.unwrap();
    let disabled_mod = mods_dir.join("test-mod.jar.disabled");
    assert!(disabled_mod.exists());
    
    // Test toggle_mod logic (enable)
    mods::toggle_mod(&instance_path, "test-mod.jar.disabled".to_string(), true).await.unwrap();
    assert!(mod_file.exists());
    assert!(!disabled_mod.exists());
    
    // Test uninstall_mod logic
    mods::uninstall_mod(&instance_path, "test-mod.jar".to_string(), false).await.unwrap();
    assert!(!mod_file.exists());
}

#[tokio::test]
async fn test_plugin_commands_logic() {
    let dir = tempdir().unwrap();
    let instance_manager = setup_instance_manager(dir.path()).await;
    
    let metadata = instance_manager.create_instance("Plugin Test", "1.20.1").await.unwrap();
    let instance_id = metadata.id;
    let instance_path = dir.path().join(instance_id.to_string());
    
    // Create plugins directory
    let plugins_dir = instance_path.join("plugins");
    fs::create_dir_all(&plugins_dir).await.unwrap();
    
    // Create a dummy plugin file (valid zip)
    let plugin_file = plugins_dir.join("test-plugin.jar");
    create_empty_jar(&plugin_file).await;
    
    // Test list_installed_plugins logic
    let installed_plugins = plugins::list_installed_plugins(&instance_path).await.unwrap();
    assert_eq!(installed_plugins.len(), 1);
    assert_eq!(installed_plugins[0].name, "test-plugin");
    
    // Test toggle_plugin logic (disable)
    plugins::toggle_plugin(&instance_path, "test-plugin.jar".to_string(), false).await.unwrap();
    let disabled_plugin = plugins_dir.join("test-plugin.jar.disabled");
    assert!(disabled_plugin.exists());
    
    // Test uninstall_plugin logic (on disabled plugin)
    plugins::uninstall_plugin(&instance_path, "test-plugin.jar.disabled".to_string(), false).await.unwrap();
    assert!(!disabled_plugin.exists());
}
