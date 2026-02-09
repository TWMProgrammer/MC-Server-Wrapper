use crate::cache::CacheManager;
use crate::modrinth::ModrinthClient as CommonClient;
use std::sync::Arc;

pub mod download;
pub mod modpack;
pub mod search;
pub mod versions;

pub struct ModrinthClient {
    pub(crate) inner: CommonClient,
}

impl ModrinthClient {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self {
            inner: CommonClient::new(cache),
        }
    }

    pub fn with_base_url(base_url: String, cache: Arc<CacheManager>) -> Self {
        Self {
            inner: CommonClient::with_base_url(base_url, cache),
        }
    }
}
