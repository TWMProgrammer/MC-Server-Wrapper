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
            client: reqwest::Client::builder()
                .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("Failed to create reqwest client"),
            cache,
        }
    }
}
