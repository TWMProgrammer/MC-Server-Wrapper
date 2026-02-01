use tauri::State;
use uuid::Uuid;
use std::sync::Arc;
use mc_server_wrapper_core::scheduler::{SchedulerManager, ScheduledTask, ScheduleType};
use mc_server_wrapper_core::instance::InstanceManager;

#[tauri::command]
pub async fn add_scheduled_task(
    instance_id: Uuid,
    task_type: ScheduleType,
    cron: String,
    scheduler: State<'_, Arc<SchedulerManager>>,
    instance_manager: State<'_, Arc<InstanceManager>>,
) -> Result<ScheduledTask, String> {
    let task = ScheduledTask::new(instance_id, task_type, cron);
    
    // Save to instance metadata
    instance_manager.add_schedule(instance_id, task.clone()).await
        .map_err(|e| e.to_string())?;
    
    // Add to running scheduler
    scheduler.add_task(task.clone()).await
        .map_err(|e| e.to_string())?;
    
    Ok(task)
}

#[tauri::command]
pub async fn remove_scheduled_task(
    instance_id: Uuid,
    task_id: Uuid,
    scheduler: State<'_, Arc<SchedulerManager>>,
    instance_manager: State<'_, Arc<InstanceManager>>,
) -> Result<(), String> {
    // Remove from running scheduler
    scheduler.remove_task(task_id).await
        .map_err(|e| e.to_string())?;
    
    // Remove from instance metadata
    instance_manager.remove_schedule(instance_id, task_id).await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn list_scheduled_tasks(
    instance_id: Uuid,
    scheduler: State<'_, Arc<SchedulerManager>>,
) -> Result<Vec<ScheduledTask>, String> {
    Ok(scheduler.list_tasks(instance_id).await)
}
