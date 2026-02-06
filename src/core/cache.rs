use std::time::Duration;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs;

/// A wrapper for cached data that includes a timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub data: T,
    pub timestamp: DateTime<Utc>,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            timestamp: Utc::now(),
        }
    }
}

/// A persistent cache entry that can be saved to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentCacheEntry {
    pub data: String, // JSON serialized data
    pub expiry: DateTime<Utc>,
    pub etag: Option<String>,
}

pub enum CacheStatus<T> {
    Hit(T),
    Stale(T),
    Miss,
}

/// Centralized CacheManager providing a unified interface for get/set operations with TTL and disk persistence.
/// 
/// This manager uses an async-friendly in-memory cache (moka) to store
/// serialized JSON values, and can optionally back them up to disk.
pub struct CacheManager {
    cache: Cache<String, PersistentCacheEntry>,
    cache_dir: Option<PathBuf>,
    dirty_keys: Arc<Mutex<HashSet<String>>>,
    default_ttl: Duration,
    background_task_started: Arc<std::sync::atomic::AtomicBool>,
    client: reqwest::Client,
}

impl CacheManager {
    /// Creates a new CacheManager with the specified capacity and default TTL.
    pub fn new(max_capacity: u64, default_ttl: Duration, cache_dir: Option<PathBuf>) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            // Keep entries in memory for twice the default TTL to support SWR
            .time_to_live(default_ttl * 2)
            .build();
        
        let client = reqwest::Client::builder()
            .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        let manager = Self {
            cache,
            cache_dir,
            dirty_keys: Arc::new(Mutex::new(HashSet::new())),
            default_ttl,
            background_task_started: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            client,
        };

        // Try to start background flush task if cache_dir is provided
        manager.ensure_background_tasks();

        manager
    }

    /// Gets the shared reqwest client.
    pub fn get_client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Ensures the background flush task is running if a cache directory is present.
    fn ensure_background_tasks(&self) {
        if self.cache_dir.is_none() {
            return;
        }

        if self.background_task_started.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let handle = match tokio::runtime::Handle::try_current() {
            Ok(h) => h,
            Err(_) => return, // No runtime available yet
        };

        if self.background_task_started.compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        ).is_ok() {
            let cache_clone = self.cache.clone();
            let cache_dir_clone = self.cache_dir.clone().unwrap();
            let dirty_keys_clone = Arc::clone(&self.dirty_keys);

            handle.spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60)); // Flush every minute
                loop {
                    interval.tick().await;
                    if let Err(e) = Self::flush_to_disk(&cache_clone, &cache_dir_clone, &dirty_keys_clone).await {
                        tracing::error!("Failed to flush cache to disk: {}", e);
                    }
                }
            });
        }
    }

    /// Flushes dirty entries to disk.
    async fn flush_to_disk(
        cache: &Cache<String, PersistentCacheEntry>,
        cache_dir: &Path,
        dirty_keys: &Arc<Mutex<HashSet<String>>>,
    ) -> Result<()> {
        let keys_to_flush = {
            let mut dirty = dirty_keys.lock().await;
            if dirty.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *dirty)
        };

        let metadata_dir = cache_dir.join("metadata");
        if !metadata_dir.exists() {
            fs::create_dir_all(&metadata_dir).await?;
        }

        for key in keys_to_flush {
            if let Some(entry) = cache.get(&key).await {
                // Skip if very old (more than 2x TTL)
                // In disk we can keep it even longer if we want, but let's be consistent
                let file_path = metadata_dir.join(format!("{}.json", urlencoding::encode(&key)));
                let content = serde_json::to_string(&entry)?;
                fs::write(file_path, content).await?;
            }
        }

        Ok(())
    }

    /// Retrieves a value from the cache and deserializes it.
    /// 
    /// # Errors
    /// Returns an error if the value exists but fails to deserialize into type `T`.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.get_with_status::<T>(key).await? {
            CacheStatus::Hit(data) => Ok(Some(data)),
            CacheStatus::Stale(data) => Ok(Some(data)),
            CacheStatus::Miss => Ok(None),
        }
    }

    /// Retrieves a value from the cache with its status (Hit, Stale, or Miss).
    pub async fn get_with_status<T: DeserializeOwned>(&self, key: &str) -> Result<CacheStatus<T>> {
        self.ensure_background_tasks();
        // 1. Try memory cache
        if let Some(entry) = self.cache.get(key).await {
            let data: T = serde_json::from_str(&entry.data)
                .with_context(|| format!("Failed to deserialize cached value for key: {}", key))?;
            
            if entry.expiry > Utc::now() {
                return Ok(CacheStatus::Hit(data));
            } else {
                return Ok(CacheStatus::Stale(data));
            }
        }

        // 2. Try disk cache if memory missed
        if let Some(ref cache_dir) = self.cache_dir {
            let file_path = cache_dir.join("metadata").join(format!("{}.json", urlencoding::encode(key)));
            if file_path.exists() {
                let content = fs::read_to_string(&file_path).await?;
                let entry: PersistentCacheEntry = serde_json::from_str(&content)?;
                
                // Populate memory cache
                self.cache.insert(key.to_string(), entry.clone()).await;
                
                let data: T = serde_json::from_str(&entry.data)?;
                if entry.expiry > Utc::now() {
                    return Ok(CacheStatus::Hit(data));
                } else {
                    return Ok(CacheStatus::Stale(data));
                }
            }
        }

        Ok(CacheStatus::Miss)
    }

    /// Retrieves a value from the cache or fetches it using the provided function (SWR).
    pub async fn fetch_with_cache<T, F, Fut>(
        self: &Arc<Self>,
        key: String,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        self.fetch_with_options(key, ttl, true, fetch_fn).await
    }

    /// Retrieves a value from the cache or fetches it using the provided function (SWR) with options.
    pub async fn fetch_with_options<T, F, Fut>(
        self: &Arc<Self>,
        key: String,
        ttl: Duration,
        persistent: bool,
        fetch_fn: F,
    ) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        match self.get_with_status::<T>(&key).await? {
            CacheStatus::Hit(data) => Ok(data),
            CacheStatus::Stale(data) => {
                // Return stale data and refresh in background
                let self_clone = Arc::clone(self);
                let key_clone = key.clone();
                tokio::spawn(async move {
                    if let Ok(fresh_data) = fetch_fn().await {
                        let _ = self_clone.set_with_ttl(key_clone, fresh_data, ttl, persistent, None).await;
                    }
                });
                Ok(data)
            }
            CacheStatus::Miss => {
                // Fetch fresh and wait
                let data = fetch_fn().await?;
                self.set_with_ttl(key, data.clone(), ttl, persistent, None).await?;
                Ok(data)
            }
        }
    }

    /// Stores a value in the cache with the default TTL.
    pub async fn set<T: Serialize>(&self, key: String, value: T) -> Result<()> {
        self.set_with_ttl(key, value, self.default_ttl, true, None).await
    }

    /// Stores a value in the cache with a specific TTL, persistence option and optional ETag.
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: String,
        value: T,
        ttl: Duration,
        persistent: bool,
        etag: Option<String>,
    ) -> Result<()> {
        self.ensure_background_tasks();
        let data = serde_json::to_string(&value)
            .with_context(|| format!("Failed to serialize value for caching key: {}", key))?;
        
        let entry = PersistentCacheEntry {
            data,
            expiry: Utc::now() + chrono::Duration::from_std(ttl)?,
            etag,
        };

        self.cache.insert(key.clone(), entry).await;

        if persistent && self.cache_dir.is_some() {
            self.dirty_keys.lock().await.insert(key);
        }

        Ok(())
    }

    /// Removes a value from the cache.
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
        if let Some(ref cache_dir) = self.cache_dir {
            let file_path = cache_dir.join("metadata").join(format!("{}.json", urlencoding::encode(key)));
            let _ = fs::remove_file(file_path).await;
        }
    }

    /// Clears the entire cache.
    pub async fn clear(&self) {
        self.cache.invalidate_all();
        if let Some(ref cache_dir) = self.cache_dir {
            let metadata_dir = cache_dir.join("metadata");
            let _ = fs::remove_dir_all(metadata_dir).await;
        }
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        // Default: 1000 entries, 1 hour TTL, no disk persistence
        Self::new(1000, Duration::from_secs(3600), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestValue {
        name: String,
        count: i32,
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let manager = CacheManager::new(10, Duration::from_secs(60), None);
        let key = "test_key".to_string();
        let value = TestValue {
            name: "test".to_string(),
            count: 42,
        };

        manager.set(key.clone(), value.name.clone()).await.unwrap();
        let retrieved: Option<String> = manager.get(&key).await.unwrap();

        assert_eq!(retrieved, Some(value.name));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        // Short TTL for testing
        let manager = CacheManager::new(10, Duration::from_millis(10), None);
        let key = "expire_key".to_string();
        let value = "some_value".to_string();

        manager.set(key.clone(), value.clone()).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Moka might need a bit of time or a get to process expiration
        let retrieved: Option<String> = manager.get(&key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let manager = CacheManager::default();
        let key = "invalidate_key".to_string();
        let value = "some_value".to_string();

        manager.set(key.clone(), value.clone()).await.unwrap();
        manager.invalidate(&key).await;
        
        let retrieved: Option<String> = manager.get(&key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_disk_persistence() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().to_path_buf();
        let manager = CacheManager::new(10, Duration::from_secs(60), Some(cache_dir.clone()));
        
        let key = "disk_key".to_string();
        let value = "disk_value".to_string();

        manager.set(key.clone(), value.clone()).await.unwrap();
        
        // Manually trigger flush for testing
        CacheManager::flush_to_disk(&manager.cache, &cache_dir, &manager.dirty_keys).await.unwrap();

        // Create a new manager with the same disk path
        let manager2 = CacheManager::new(10, Duration::from_secs(60), Some(cache_dir));
        let retrieved: Option<String> = manager2.get(&key).await.unwrap();
        
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_swr_logic() {
        let manager = Arc::new(CacheManager::new(10, Duration::from_millis(50), None));
        let key = "swr_key".to_string();
        
        // Initial fetch
        let val = manager.fetch_with_cache(key.clone(), Duration::from_millis(50), || async {
            Ok("fresh".to_string())
        }).await.unwrap();
        assert_eq!(val, "fresh");

        // Wait for it to become stale
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Fetch again - should get stale "fresh" and trigger background update
        let val2 = manager.fetch_with_cache(key.clone(), Duration::from_millis(50), || async {
            Ok("updated".to_string())
        }).await.unwrap();
        assert_eq!(val2, "fresh");

        // Wait a bit for background update
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Fetch again - should get "updated"
        let val3 = manager.fetch_with_cache(key.clone(), Duration::from_millis(50), || async {
            Ok("never_called".to_string())
        }).await.unwrap();
        assert_eq!(val3, "updated");
    }
}
