use std::sync::Arc;
use crate::cache::CacheManager;

pub mod search;
pub mod versions;
pub mod download;

pub struct HangarClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) cache: Arc<CacheManager>,
}

impl HangarClient {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("Failed to create reqwest client"),
            base_url: "https://hangar.papermc.io/api/v1".to_string(),
            cache,
        }
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
