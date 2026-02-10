use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// A utility to deduplicate concurrent executions of the same task.
/// 
/// If multiple callers request the same key simultaneously, only one will execute
/// the task, and the others will wait for it to complete.
pub struct SingleFlight {
    locks: Mutex<HashMap<String, Arc<Notify>>>,
}

impl SingleFlight {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// Executes the given future if no other task with the same key is currently running.
    /// Returns true if this call actually executed the task, false if it waited for another.
    pub async fn wait_or_execute<F, Fut>(&self, key: &str, task: F) -> anyhow::Result<bool>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<()>>,
    {
        let notify = {
            let mut locks = self.locks.lock().await;
            if let Some(notify) = locks.get(key) {
                // Someone else is already doing this, wait for them
                Arc::clone(notify)
            } else {
                // We are the first ones
                let notify = Arc::new(Notify::new());
                locks.insert(key.to_string(), Arc::clone(&notify));
                
                // Drop lock before executing task
                drop(locks);

                let result = task().await;

                // Task finished, remove lock and notify others
                let mut locks = self.locks.lock().await;
                locks.remove(key);
                notify.notify_waiters();
                
                return result.map(|_| true);
            }
        };

        // Wait for the active task to complete
        notify.notified().await;
        Ok(false)
    }
}
