use mc_server_wrapper_core::scheduler::{SchedulerManager, ScheduledTask, ScheduleType};
use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::backup::BackupManager;
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::app_config::GlobalConfigManager;
use mc_server_wrapper_core::database::Database;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn test_scheduler_add_remove_task() {
    let base_dir = tempdir().unwrap();
    let config_dir = tempdir().unwrap();
    
    let db_path = base_dir.path().join("test.db");
    let db = Arc::new(Database::new(db_path).await.expect("Failed to create database"));
    let instance_manager = Arc::new(InstanceManager::new(base_dir.path(), db).await.expect("Failed to create instance manager"));
    let config_manager = Arc::new(GlobalConfigManager::new(config_dir.path().to_path_buf()));
    let server_manager = Arc::new(ServerManager::new(instance_manager, config_manager));
    let backup_manager = Arc::new(BackupManager::new(base_dir.path().join("backups")));
    
    let scheduler = SchedulerManager::new(server_manager, backup_manager).await.expect("Failed to create scheduler");
    
    let instance_id = Uuid::new_v4();
    let task = ScheduledTask::new(
        instance_id,
        ScheduleType::Backup,
        "0 0 * * * *".to_string(), // Every hour
    );
    let task_id = task.id;
    
    // Add task
    scheduler.add_task(task).await.expect("Failed to add task");
    
    // List tasks
    let tasks: Vec<ScheduledTask> = scheduler.list_tasks(instance_id).await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, task_id);
    
    // Remove task
    scheduler.remove_task(task_id).await.expect("Failed to remove task");
    
    let tasks: Vec<ScheduledTask> = scheduler.list_tasks(instance_id).await;
    assert_eq!(tasks.len(), 0);
}
