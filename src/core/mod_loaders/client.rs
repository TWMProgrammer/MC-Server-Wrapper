use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use crate::cache::CacheManager;

pub struct ModLoaderClient {
    pub(crate) client: reqwest::Client,
    pub(crate) cache_dir: Option<PathBuf>,
    pub(crate) cache: Arc<CacheManager>,
}

impl ModLoaderClient {
    pub fn new(cache_dir: Option<PathBuf>, cache: Arc<CacheManager>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache_dir,
            cache,
        }
    }
}
