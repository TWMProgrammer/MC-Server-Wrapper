use anyhow::{Context, Result, anyhow};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, info};
use uuid::Uuid;

/// Supported hash algorithms for artifact verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
}

/// A centralized store for large binary artifacts (JARs, mods, etc.).
/// Files are stored in a content-addressable structure based on their hashes.
pub struct ArtifactStore {
    base_dir: PathBuf,
}

impl ArtifactStore {
    /// Creates a new ArtifactStore at the specified base directory.
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Returns the path where an artifact with the given hash would be stored.
    /// Structure: base_dir/sha256/aa/bb/aabbcc...
    pub fn get_artifact_path(&self, hash: &str, algorithm: HashAlgorithm) -> PathBuf {
        let algo_dir = match algorithm {
            HashAlgorithm::Sha1 => "sha1",
            HashAlgorithm::Sha256 => "sha256",
        };

        // Use a 2-level nested directory structure to avoid too many files in one directory
        let prefix1 = &hash[0..2];
        let prefix2 = &hash[2..4];

        self.base_dir
            .join(algo_dir)
            .join(prefix1)
            .join(prefix2)
            .join(hash)
    }

    /// Checks if an artifact with the given hash exists in the store.
    pub async fn exists(&self, hash: &str, algorithm: HashAlgorithm) -> bool {
        let path = self.get_artifact_path(hash, algorithm);
        path.exists()
    }

    /// Adds an artifact to the store from an existing file.
    /// This will verify the hash of the source file before moving it to the store.
    /// If the artifact already exists, it will not be overwritten.
    pub async fn add_artifact(
        &self,
        source_path: impl AsRef<Path>,
        expected_hash: &str,
        algorithm: HashAlgorithm,
    ) -> Result<PathBuf> {
        let source_path = source_path.as_ref();
        let target_path = self.get_artifact_path(expected_hash, algorithm);

        if target_path.exists() {
            debug!("Artifact {} already exists in store", expected_hash);
            return Ok(target_path);
        }

        // Verify hash before adding
        let actual_hash = self.calculate_hash(source_path, algorithm).await?;
        if actual_hash != expected_hash {
            return Err(anyhow!(
                "Hash mismatch for artifact! Expected: {}, Got: {}",
                expected_hash,
                actual_hash
            ));
        }

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        // Safe concurrent write: write to a unique temporary file and then rename
        let temp_path = target_path.with_extension(format!("{}.tmp", Uuid::new_v4()));
        fs::copy(source_path, &temp_path)
            .await
            .with_context(|| format!("Failed to copy {:?} to {:?}", source_path, temp_path))?;

        // Rename is atomic on most filesystems. If the target already exists, 
        // it might have been added by another process while we were copying.
        if let Err(e) = fs::rename(&temp_path, &target_path).await {
            if target_path.exists() {
                debug!("Artifact {} was already added by another process", expected_hash);
                let _ = fs::remove_file(&temp_path).await;
            } else {
                return Err(e).with_context(|| {
                    format!("Failed to rename {:?} to {:?}", temp_path, target_path)
                });
            }
        }

        info!("Added artifact {} to store", expected_hash);
        Ok(target_path)
    }

    /// Provisions an artifact from the store to a target path (e.g., an instance folder).
    /// Currently uses simple file copying. Hard-links could be used in the future for optimization.
    pub async fn provision(
        &self,
        hash: &str,
        algorithm: HashAlgorithm,
        target_path: impl AsRef<Path>,
    ) -> Result<()> {
        let artifact_path = self.get_artifact_path(hash, algorithm);
        if !artifact_path.exists() {
            return Err(anyhow!("Artifact {} not found in store", hash));
        }

        let target_path = target_path.as_ref();
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Use temp-and-rename for safe concurrent provisioning
        let temp_path = target_path.with_extension(format!("{}.tmp", Uuid::new_v4()));
        fs::copy(&artifact_path, &temp_path)
            .await
            .with_context(|| format!("Failed to copy artifact to temporary path {:?}", temp_path))?;

        if let Err(e) = fs::rename(&temp_path, target_path).await {
            if target_path.exists() {
                // Someone else already provisioned it
                let _ = fs::remove_file(&temp_path).await;
            } else {
                return Err(e).with_context(|| {
                    format!("Failed to rename {:?} to {:?}", temp_path, target_path)
                });
            }
        }

        Ok(())
    }

    /// Prunes artifacts that are not in the provided set of active hashes.
    /// Returns the number of files deleted.
    pub async fn prune(
        &self,
        active_hashes: &HashSet<String>,
        algorithm: HashAlgorithm,
    ) -> Result<u64> {
        let algo_dir = match algorithm {
            HashAlgorithm::Sha1 => "sha1",
            HashAlgorithm::Sha256 => "sha256",
        };
        let base = self.base_dir.join(algo_dir);
        if !base.exists() {
            return Ok(0);
        }

        let mut deleted_count = 0;
        let mut entries = fs::read_dir(&base).await?;

        // Walk level 1: prefix1
        while let Some(entry1) = entries.next_entry().await? {
            let path1 = entry1.path();
            if !path1.is_dir() {
                continue;
            }

            let mut entries2 = fs::read_dir(&path1).await?;
            // Walk level 2: prefix2
            while let Some(entry2) = entries2.next_entry().await? {
                let path2 = entry2.path();
                if !path2.is_dir() {
                    continue;
                }

                let mut entries3 = fs::read_dir(&path2).await?;
                // Walk level 3: the actual hash files
                while let Some(entry3) = entries3.next_entry().await? {
                    let path3 = entry3.path();
                    if !path3.is_file() {
                        continue;
                    }

                    if let Some(hash) = path3.file_name().and_then(|n| n.to_str()) {
                        // Temporary files might have .tmp extension, ignore them or clean them up
                        if hash.ends_with(".tmp") {
                            let _ = fs::remove_file(&path3).await;
                            continue;
                        }

                        if !active_hashes.contains(hash) {
                            debug!("Pruning unlinked artifact: {}", hash);
                            fs::remove_file(&path3).await.with_context(|| {
                                format!("Failed to delete unlinked artifact: {:?}", path3)
                            })?;
                            deleted_count += 1;
                        }
                    }
                }

                // Try to remove empty directory
                if fs::read_dir(&path2).await?.next_entry().await?.is_none() {
                    let _ = fs::remove_dir(&path2).await;
                }
            }

            // Try to remove empty directory
            if fs::read_dir(&path1).await?.next_entry().await?.is_none() {
                let _ = fs::remove_dir(&path1).await;
            }
        }

        if deleted_count > 0 {
            info!(
                "Pruned {} artifacts from the store ({:?})",
                deleted_count, algorithm
            );
        }
        Ok(deleted_count)
    }

    /// Calculates the hash of a file.
    pub async fn calculate_hash(&self, path: &Path, algorithm: HashAlgorithm) -> Result<String> {
        let mut file = fs::File::open(path)
            .await
            .with_context(|| format!("Failed to open file for hashing: {:?}", path))?;

        let mut buffer = vec![0u8; 8192];

        match algorithm {
            HashAlgorithm::Sha1 => {
                let mut hasher = Sha1::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                loop {
                    let n = file.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
                Ok(format!("{:x}", hasher.finalize()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_artifact_store_add_and_exists() {
        let dir = tempdir().unwrap();
        let store = ArtifactStore::new(dir.path().to_path_buf());

        let test_file = dir.path().join("test.txt");
        let content = b"hello world";
        fs::write(&test_file, content).await.unwrap();

        // sha1 of "hello world"
        let expected_sha1 = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";

        let path = store
            .add_artifact(&test_file, expected_sha1, HashAlgorithm::Sha1)
            .await
            .unwrap();
        assert!(path.exists());
        assert!(store.exists(expected_sha1, HashAlgorithm::Sha1).await);

        // Verify nested structure
        assert!(path.to_string_lossy().contains("sha1"));
        assert!(path.to_string_lossy().contains("2a"));
        assert!(path.to_string_lossy().contains("ae"));
    }

    #[tokio::test]
    async fn test_artifact_store_mismatch() {
        let dir = tempdir().unwrap();
        let store = ArtifactStore::new(dir.path().to_path_buf());

        let test_file = dir.path().join("test.txt");
        fs::write(&test_file, b"wrong content").await.unwrap();

        let result = store
            .add_artifact(&test_file, "somehash", HashAlgorithm::Sha1)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_artifact_store_prune() {
        let dir = tempdir().unwrap();
        let store = ArtifactStore::new(dir.path().to_path_buf());

        let test_file = dir.path().join("test.txt");
        let content = b"hello world";
        fs::write(&test_file, content).await.unwrap();
        let hash = "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed";

        store
            .add_artifact(&test_file, hash, HashAlgorithm::Sha1)
            .await
            .unwrap();
        assert!(store.exists(hash, HashAlgorithm::Sha1).await);

        // Prune with empty set - should delete the artifact
        let active = HashSet::new();
        let deleted = store.prune(&active, HashAlgorithm::Sha1).await.unwrap();
        assert_eq!(deleted, 1);
        assert!(!store.exists(hash, HashAlgorithm::Sha1).await);

        // Add it back
        store
            .add_artifact(&test_file, hash, HashAlgorithm::Sha1)
            .await
            .unwrap();

        // Prune with hash in set - should keep it
        let mut active = HashSet::new();
        active.insert(hash.to_string());
        let deleted = store.prune(&active, HashAlgorithm::Sha1).await.unwrap();
        assert_eq!(deleted, 0);
        assert!(store.exists(hash, HashAlgorithm::Sha1).await);
    }
}
