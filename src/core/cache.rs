use std::time::Duration;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use anyhow::{Result, Context};
use chrono::{DateTime, Utc};

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

/// Centralized CacheManager providing a unified interface for get/set operations with TTL.
/// 
/// This manager uses an async-friendly in-memory cache (moka) to store
/// serialized JSON values, allowing for a flexible yet type-safe interface.
pub struct CacheManager {
    cache: Cache<String, String>,
}

impl CacheManager {
    /// Creates a new CacheManager with the specified capacity and default TTL.
    pub fn new(max_capacity: u64, default_ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(default_ttl)
            .build();
        Self { cache }
    }

    /// Retrieves a value from the cache and deserializes it.
    /// 
    /// # Errors
    /// Returns an error if the value exists but fails to deserialize into type `T`.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        if let Some(val_str) = self.cache.get(key).await {
            let entry: CacheEntry<T> = serde_json::from_str(&val_str)
                .with_context(|| format!("Failed to deserialize cached value for key: {}", key))?;
            Ok(Some(entry.data))
        } else {
            Ok(None)
        }
    }

    /// Stores a value in the cache with the default TTL.
    /// 
    /// # Errors
    /// Returns an error if the value fails to serialize into JSON.
    pub async fn set<T: Serialize>(&self, key: String, value: T) -> Result<()> {
        let entry = CacheEntry::new(value);
        let val_str = serde_json::to_string(&entry)
            .with_context(|| format!("Failed to serialize value for caching key: {}", key))?;
        self.cache.insert(key, val_str).await;
        Ok(())
    }

    /// Removes a value from the cache.
    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    /// Clears the entire cache.
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        // Default: 1000 entries, 1 hour TTL
        Self::new(1000, Duration::from_secs(3600))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestValue {
        name: String,
        count: i32,
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let manager = CacheManager::new(10, Duration::from_secs(60));
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
        let manager = CacheManager::new(10, Duration::from_millis(10));
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
}
