use mc_server_wrapper_core::backup::BackupManager;
use tempfile::tempdir;
use uuid::Uuid;
use std::fs::File;
use std::io::Write;

#[tokio::test]
async fn test_create_and_list_backups() {
    let base_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let backup_mgr = BackupManager::new(base_dir.path());
    let instance_id = Uuid::new_v4();

    // Create a dummy file in source_dir
    let file_path = source_dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "test content").unwrap();

    // Create backup
    let backup_info = backup_mgr.create_backup(
        instance_id,
        source_dir.path(),
        "test_backup",
        |_, _| {}
    ).await.expect("Failed to create backup");

    assert!(backup_info.name.contains("test_backup"));
    assert!(backup_info.path.exists());

    // List backups
    let backups = backup_mgr.list_backups(instance_id).await.expect("Failed to list backups");
    assert_eq!(backups.len(), 1);
    assert_eq!(backups[0].name, backup_info.name);
}

#[tokio::test]
async fn test_delete_backup() {
    let base_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let backup_mgr = BackupManager::new(base_dir.path());
    let instance_id = Uuid::new_v4();

    // Create a dummy file
    let file_path = source_dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "test content").unwrap();

    // Create backup
    let backup_info = backup_mgr.create_backup(
        instance_id,
        source_dir.path(),
        "to_delete",
        |_, _| {}
    ).await.expect("Failed to create backup");

    assert!(backup_info.path.exists());

    // Delete backup
    backup_mgr.delete_backup(instance_id, &backup_info.name).await.expect("Failed to delete backup");

    assert!(!backup_info.path.exists());
    let backups = backup_mgr.list_backups(instance_id).await.expect("Failed to list backups");
    assert_eq!(backups.len(), 0);
}

#[tokio::test]
async fn test_restore_backup() {
    let base_dir = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let restore_dir = tempdir().unwrap();
    let backup_mgr = BackupManager::new(base_dir.path());
    let instance_id = Uuid::new_v4();

    // Create a dummy file in source_dir
    let file_path = source_dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "restore this").unwrap();

    // Create backup
    let backup_info = backup_mgr.create_backup(
        instance_id,
        source_dir.path(),
        "restore_test",
        |_, _| {}
    ).await.expect("Failed to create backup");

    // Restore backup
    backup_mgr.restore_backup(instance_id, &backup_info.name, restore_dir.path()).await.expect("Failed to restore backup");

    // Verify restored file
    let restored_file_path = restore_dir.path().join("test.txt");
    assert!(restored_file_path.exists());
    let content = std::fs::read_to_string(restored_file_path).unwrap();
    assert_eq!(content.trim(), "restore this");
}

#[tokio::test]
async fn test_disaster_recovery_workflow() {
    let base_dir = tempdir().unwrap();
    let instance_dir = tempdir().unwrap();
    let backup_mgr = BackupManager::new(base_dir.path());
    let instance_id = Uuid::new_v4();

    // 1. Setup: Create a config file
    let config_path = instance_dir.path().join("server.properties");
    let original_content = "difficulty=easy\nmax-players=10";
    std::fs::write(&config_path, original_content).unwrap();

    // 2. User creates a backup
    let backup_info = backup_mgr.create_backup(
        instance_id,
        instance_dir.path(),
        "disaster_recovery_pre",
        |_, _| {}
    ).await.expect("Failed to create backup");

    // 3. User modifies a config file incorrectly (the "disaster")
    let corrupted_content = "difficulty=HARDCORE_EXTREME\nmax-players=-1";
    std::fs::write(&config_path, corrupted_content).unwrap();
    
    // Verify it's corrupted
    let current_content = std::fs::read_to_string(&config_path).unwrap();
    assert_eq!(current_content, corrupted_content);

    // 4. User restores the backup
    backup_mgr.restore_backup(instance_id, &backup_info.name, instance_dir.path())
        .await
        .expect("Failed to restore backup");

    // 5. Verify the file is reverted
    let restored_content = std::fs::read_to_string(&config_path).expect("Config file should exist after restore");
    assert_eq!(restored_content.trim(), original_content.trim());
    
    // Verify only the original file exists (the restore should have wiped the corrupted one)
    assert!(config_path.exists());
}
