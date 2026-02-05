use std::sync::Arc;
use crate::cache::CacheManager;

pub mod search;
pub mod download;
pub mod github;

pub struct SpigetClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) cache: Arc<CacheManager>,
}

impl SpigetClient {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self::with_base_url("https://api.spiget.org/v2".to_string(), cache)
    }

    pub fn with_base_url(base_url: String, cache: Arc<CacheManager>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("Failed to create reqwest client"),
            base_url,
            cache,
        }
    }
}
