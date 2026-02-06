use mc_server_wrapper_core::assets::AssetManager;
use mc_server_wrapper_core::cache::CacheManager;
use sha2::{Digest, Sha256};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_asset_caching() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().to_path_buf();
    let cache_manager = Arc::new(CacheManager::new(100, Duration::from_secs(60), None));
    let asset_manager = AssetManager::new(cache_dir.clone(), cache_manager);

    // We can't easily test actual downloads without mocking reqwest or using a local server.
    // However, AssetManager::get_asset checks for existence first.
    // If it exists and is fresh, it returns the path without downloading.

    let url = "https://example.com/test.png";

    // Calculate hash exactly as AssetManager does
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let url_hash = hex::encode(hasher.finalize());

    let expected_path = cache_dir.join(format!("{}.png", url_hash));

    // Pre-seed the cache
    fs::write(&expected_path, "fake_data").unwrap();

    // This should return the pre-seeded path instead of attempting a download
    let path = asset_manager
        .get_asset(url)
        .await
        .expect("Should return cached asset path");
    assert_eq!(path, expected_path);
    assert!(path.exists());
}

#[tokio::test]
async fn test_asset_cleanup() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().to_path_buf();
    let cache_manager = Arc::new(CacheManager::new(100, Duration::from_secs(60), None));
    let asset_manager = AssetManager::new(cache_dir.clone(), cache_manager);

    let old_file = cache_dir.join("old.png");
    let new_file = cache_dir.join("new.png");

    fs::write(&old_file, "old").unwrap();
    fs::write(&new_file, "new").unwrap();

    // Returns 0 because files were just created and aren't older than 1 hour
    let count = asset_manager
        .cleanup_assets(Duration::from_secs(3600))
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_asset_stats() {
    let dir = tempdir().unwrap();
    let cache_dir = dir.path().to_path_buf();
    let cache_manager = Arc::new(CacheManager::new(100, Duration::from_secs(60), None));
    let asset_manager = AssetManager::new(cache_dir.clone(), cache_manager);

    fs::write(cache_dir.join("1.png"), "data").unwrap();
    fs::write(cache_dir.join("2.png"), "more_data").unwrap();

    let stats = asset_manager.get_stats().await.unwrap();
    assert_eq!(stats.count, 2);
    assert_eq!(stats.total_size, 4 + 9);
}
