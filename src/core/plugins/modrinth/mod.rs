use std::sync::Arc;
use crate::cache::CacheManager;

pub mod search;
pub mod versions;
pub mod download;

pub struct ModrinthClient {
    pub(crate) client: reqwest::Client,
    pub(crate) cache: Arc<CacheManager>,
}

impl ModrinthClient {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self {
            client: cache.get_client().clone(),
            cache,
        }
    }

    pub fn with_base_url(_base_url: String, cache: Arc<CacheManager>) -> Self {
        // base_url is ignored for now as it's not used in search/versions/download
        Self::new(cache)
    }
}
