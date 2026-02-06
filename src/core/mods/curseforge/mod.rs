use std::sync::Arc;
use crate::cache::CacheManager;

pub mod search;
pub mod download;
pub mod versions;

pub struct CurseForgeClient {
    pub(crate) client: reqwest::Client,
    pub(crate) api_key: Option<String>,
    pub(crate) cache: Arc<CacheManager>,
}

impl CurseForgeClient {
    pub fn new(api_key: Option<String>, cache: Arc<CacheManager>) -> Self {
        Self {
            client: cache.get_client().clone(),
            api_key,
            cache,
        }
    }
}
