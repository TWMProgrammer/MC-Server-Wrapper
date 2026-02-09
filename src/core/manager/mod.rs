use super::app_config::GlobalConfigManager;
use super::artifacts::{ArtifactStore, HashAlgorithm};
use super::cache::CacheManager;
use super::downloader::VersionDownloader;
use super::instance::{InstanceManager, InstanceMetadata};
use super::mod_loaders::ModLoaderClient;
use super::server::ServerHandle;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

mod install;
mod lifecycle;

pub struct ServerManager {
    pub(crate) instance_manager: Arc<InstanceManager>,
    pub(crate) config_manager: Arc<GlobalConfigManager>,
    pub(crate) downloader: VersionDownloader,
    pub(crate) mod_loader_client: ModLoaderClient,
    pub(crate) cache: Arc<CacheManager>,
    pub(crate) artifact_store: Arc<ArtifactStore>,
    pub(crate) servers: Arc<Mutex<HashMap<Uuid, Arc<ServerHandle>>>>,
}

impl ServerManager {
    pub fn new(
        instance_manager: Arc<InstanceManager>,
        config_manager: Arc<GlobalConfigManager>,
    ) -> Self {
        let base_dir = instance_manager.get_base_dir();
        let cache_dir = base_dir.join("cache");
        let artifacts_dir = base_dir.join("resources").join("artifacts");

        let cache = Arc::new(CacheManager::new(
            1000,
            std::time::Duration::from_secs(3600),
            Some(cache_dir.clone()),
        ));
        let artifact_store = Arc::new(ArtifactStore::new(artifacts_dir));

        Self {
            instance_manager,
            config_manager,
            downloader: VersionDownloader::new(
                Some(cache_dir.clone()),
                Some(Arc::clone(&cache)),
                Some(Arc::clone(&artifact_store)),
            ),
            mod_loader_client: ModLoaderClient::new(Some(cache_dir), Arc::clone(&cache)),
            cache,
            artifact_store,
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_downloader(&self) -> &VersionDownloader {
        &self.downloader
    }

    pub fn get_mod_loader_client(&self) -> &ModLoaderClient {
        &self.mod_loader_client
    }

    pub fn get_instance_manager(&self) -> Arc<InstanceManager> {
        Arc::clone(&self.instance_manager)
    }

    pub fn get_cache(&self) -> Arc<CacheManager> {
        Arc::clone(&self.cache)
    }

    pub async fn get_server(&self, instance_id: Uuid) -> Option<Arc<ServerHandle>> {
        let servers = self.servers.lock().await;
        servers.get(&instance_id).cloned()
    }

    pub async fn create_instance_full(
        &self,
        name: &str,
        version: &str,
        mod_loader: Option<String>,
        loader_version: Option<String>,
    ) -> Result<InstanceMetadata> {
        let instance = self
            .instance_manager
            .create_instance_full(name, version, mod_loader, loader_version)
            .await?;
        Ok(instance)
    }

    pub async fn create_instance_from_modpack<F>(
        &self,
        name: &str,
        version: &crate::mods::types::ProjectVersion,
        on_progress: F,
    ) -> Result<InstanceMetadata>
    where
        F: Fn(crate::mods::modrinth::modpack::ModpackProgress) + Send + 'static,
    {
        let instance = self
            .instance_manager
            .create_instance_from_modpack(name, version, Arc::clone(&self.cache), on_progress)
            .await?;
        Ok(instance)
    }

    pub async fn get_bedrock_versions(&self) -> Result<crate::downloader::VersionManifest> {
        self.mod_loader_client.get_bedrock_versions().await
    }

    pub async fn get_velocity_versions(&self) -> Result<Vec<String>> {
        self.mod_loader_client.get_velocity_versions().await
    }

    /// Performs maintenance tasks: migrates existing JARs into the store and prunes unlinked artifacts.
    pub async fn perform_maintenance(&self) -> Result<()> {
        info!("Starting artifact store maintenance...");

        let instances = self.instance_manager.list_instances().await?;
        let mut active_hashes = HashSet::new();

        for instance in instances {
            let instance_path = instance.path.clone();

            // 1. Handle server.jar
            let server_jar = instance_path.join("server.jar");
            if server_jar.exists() {
                match self
                    .artifact_store
                    .calculate_hash(&server_jar, HashAlgorithm::Sha1)
                    .await
                {
                    Ok(hash) => {
                        debug!("Found server.jar in instance {}: {}", instance.name, hash);
                        if let Err(e) = self
                            .artifact_store
                            .add_artifact(&server_jar, &hash, HashAlgorithm::Sha1)
                            .await
                        {
                            warn!("Failed to add existing server.jar to store: {}", e);
                        } else {
                            active_hashes.insert(hash);
                        }
                    }
                    Err(e) => warn!(
                        "Failed to calculate hash for server.jar in {}: {}",
                        instance.name, e
                    ),
                }
            }

            // 2. Handle mods
            let mods_dir = instance_path.join("mods");
            if mods_dir.exists() {
                let mut entries = fs::read_dir(&mods_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar") {
                        match self
                            .artifact_store
                            .calculate_hash(&path, HashAlgorithm::Sha1)
                            .await
                        {
                            Ok(hash) => {
                                if let Err(e) = self
                                    .artifact_store
                                    .add_artifact(&path, &hash, HashAlgorithm::Sha1)
                                    .await
                                {
                                    warn!("Failed to add mod {:?} to store: {}", path, e);
                                } else {
                                    active_hashes.insert(hash);
                                }
                            }
                            Err(e) => warn!("Failed to calculate hash for mod {:?}: {}", path, e),
                        }
                    }
                }
            }
        }

        // 3. Prune unlinked artifacts
        let pruned = self
            .artifact_store
            .prune(&active_hashes, HashAlgorithm::Sha1)
            .await?;
        info!(
            "Maintenance complete. Pruned {} unlinked artifacts.",
            pruned
        );

        Ok(())
    }

    pub async fn get_velocity_builds(&self, version: &str) -> Result<Vec<String>> {
        self.mod_loader_client.get_velocity_builds(version).await
    }

    pub async fn get_bungeecord_versions(&self) -> Result<Vec<String>> {
        self.mod_loader_client.get_bungeecord_versions().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_server_manager_maintenance() {
        let dir = tempdir().unwrap();
        let base_dir = dir.path().to_path_buf();
        let db_path = base_dir.join("test.db");
        let db = Arc::new(Database::new(&db_path).await.unwrap());

        let instance_manager = Arc::new(InstanceManager::new(&base_dir, db).await.unwrap());
        let config_manager = Arc::new(GlobalConfigManager::new(base_dir.join("config")));

        let manager = ServerManager::new(instance_manager.clone(), config_manager);

        // Create a dummy instance with a server.jar
        let instance = instance_manager
            .create_instance_full("test_instance", "1.20.1", None, None)
            .await
            .unwrap();
        let server_jar_path = instance.path.join("server.jar");
        let jar_content = b"fake jar content";
        fs::write(&server_jar_path, jar_content).await.unwrap();

        let expected_hash = manager
            .artifact_store
            .calculate_hash(&server_jar_path, HashAlgorithm::Sha1)
            .await
            .unwrap();

        // Add an unlinked artifact to the store
        let unlinked_jar_path = base_dir.join("unlinked.jar");
        fs::write(&unlinked_jar_path, b"unlinked content")
            .await
            .unwrap();
        let unlinked_hash = manager
            .artifact_store
            .calculate_hash(&unlinked_jar_path, HashAlgorithm::Sha1)
            .await
            .unwrap();
        manager
            .artifact_store
            .add_artifact(&unlinked_jar_path, &unlinked_hash, HashAlgorithm::Sha1)
            .await
            .unwrap();

        // Perform maintenance
        manager.perform_maintenance().await.unwrap();

        // Verify: server.jar should be in store
        assert!(
            manager
                .artifact_store
                .exists(&expected_hash, HashAlgorithm::Sha1)
                .await
        );

        // Verify: unlinked artifact should be pruned
        assert!(
            !manager
                .artifact_store
                .exists(&unlinked_hash, HashAlgorithm::Sha1)
                .await
        );
    }
}
