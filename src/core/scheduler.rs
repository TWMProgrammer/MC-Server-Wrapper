use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, error};
use super::manager::ServerManager;
use super::backup::BackupManager;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ScheduleType {
    Backup,
    Restart,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub task_type: ScheduleType,
    pub cron: String,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
}

impl ScheduledTask {
    pub fn new(instance_id: Uuid, task_type: ScheduleType, cron: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            instance_id,
            task_type,
            cron,
            enabled: true,
            last_run: None,
            next_run: None,
        }
    }
}

pub struct SchedulerManager {
    job_scheduler: JobScheduler,
    server_manager: Arc<ServerManager>,
    backup_manager: Arc<BackupManager>,
    tasks: Arc<Mutex<HashMap<Uuid, ScheduledTask>>>,
    job_ids: Arc<Mutex<HashMap<Uuid, Uuid>>>, // Task ID -> Job ID
}

impl SchedulerManager {
    pub async fn new(server_manager: Arc<ServerManager>, backup_manager: Arc<BackupManager>) -> Result<Self> {
        let job_scheduler = JobScheduler::new().await?;
        job_scheduler.start().await?;

        Ok(Self {
            job_scheduler,
            server_manager,
            backup_manager,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            job_ids: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn add_task(&self, mut task: ScheduledTask) -> Result<()> {
        let task_id = task.id;
        let instance_id = task.instance_id;
        let task_type = task.task_type.clone();
        
        let server_manager = Arc::clone(&server_manager_ptr(self));
        let backup_manager = Arc::clone(&backup_manager_ptr(self));
        let tasks = Arc::clone(&self.tasks);

        let job = Job::new_async(task.cron.as_str(), move |_uuid, _l| {
            let server_manager = Arc::clone(&server_manager);
            let backup_manager = Arc::clone(&backup_manager);
            let tasks = Arc::clone(&tasks);
            let task_type = task_type.clone();

            Box::pin(async move {
                info!("Executing scheduled task {:?} for instance {}", task_type, instance_id);
                
                let result: Result<()> = match task_type {
                    ScheduleType::Backup => {
                        let instance_manager = &server_manager.instance_manager;
                        if let Some(instance) = instance_manager.get_instance(instance_id).await.unwrap_or(None) {
                            backup_manager.create_backup(
                                instance_id, 
                                instance.path, 
                                "scheduled_backup", 
                                |_, _| {}
                            ).await.map(|_| ())
                        } else {
                            Err(anyhow::anyhow!("Instance not found"))
                        }
                    }
                    ScheduleType::Restart => {
                        server_manager.restart_server(instance_id).await
                    }
                };

                if let Err(e) = result {
                    error!("Failed to execute scheduled task: {:?}", e);
                } else {
                    let mut tasks_lock = tasks.lock().await;
                    if let Some(t) = tasks_lock.get_mut(&task_id) {
                        t.last_run = Some(Utc::now());
                    }
                }
            })
        })?;

        let job_id = self.job_scheduler.add(job).await?;
        
        let mut tasks_lock = self.tasks.lock().await;
        task.next_run = None; // Can be updated later if needed
        tasks_lock.insert(task_id, task);

        let mut job_ids_lock = self.job_ids.lock().await;
        job_ids_lock.insert(task_id, job_id);

        Ok(())
    }

    pub async fn remove_task(&self, task_id: Uuid) -> Result<()> {
        let mut job_ids_lock = self.job_ids.lock().await;
        if let Some(job_id) = job_ids_lock.remove(&task_id) {
            self.job_scheduler.remove(&job_id).await?;
        }

        let mut tasks_lock = self.tasks.lock().await;
        tasks_lock.remove(&task_id);

        Ok(())
    }

    pub async fn list_tasks(&self, instance_id: Uuid) -> Vec<ScheduledTask> {
        let tasks_lock = self.tasks.lock().await;
        tasks_lock.values()
            .filter(|t| t.instance_id == instance_id)
            .cloned()
            .collect()
    }
}

fn server_manager_ptr(mgr: &SchedulerManager) -> &Arc<ServerManager> {
    &mgr.server_manager
}

fn backup_manager_ptr(mgr: &SchedulerManager) -> &Arc<BackupManager> {
    &mgr.backup_manager
}
