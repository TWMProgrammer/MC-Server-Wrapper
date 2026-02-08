use super::SpigetClient;
use anyhow::Result;
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use std::path::Path;
use tokio::fs;

use crate::utils::{DownloadOptions, download_with_resumption};

impl SpigetClient {
    pub async fn download_resource(
        &self,
        resource_id: &str,
        target_dir: impl AsRef<Path>,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<String> {
        // Fetch project info first to check for external downloads and get the slug
        let url = format!("{}/resources/{}", self.base_url, resource_id);
        let info_response = self.client.get(&url).send().await?.error_for_status()?;
        let info_json = info_response.json::<serde_json::Value>().await?;

        let title = info_json["name"].as_str().unwrap_or("Unknown Plugin");
        let slug = info_json["name"]
            .as_str()
            .unwrap_or("plugin")
            .to_lowercase()
            .replace(' ', "-");

        if info_json["file"]["type"].as_str() == Some("external") {
            let external_url = info_json["file"]["external_url"]
                .as_str()
                .or_else(|| info_json["file"]["externalUrl"].as_str())
                .unwrap_or("unknown");

            if external_url.contains("github.com") {
                return self
                    .download_from_github(external_url, target_dir, title, game_version, loader)
                    .await;
            }

            return Err(anyhow::anyhow!(
                "Plugin '{}' (ID: {}) has an external download URL: {}. \
                Spiget cannot automatically download external resources. \
                Please download it manually and place it in the plugins folder.",
                title,
                resource_id,
                external_url
            ));
        }

        let download_url = format!("{}/resources/{}/download", self.base_url, resource_id);

        // Use a HEAD request to get the filename and content type
        let head_response = self.client.head(&download_url).send().await?;

        // Check content type to ensure we're not downloading an error page or JSON
        let content_type = head_response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.contains("text/html") || content_type.contains("application/json") {
            return Err(anyhow::anyhow!(
                "Spiget returned an unexpected content type for '{}': {}. \
                This usually means the resource is blocked by Cloudflare, requires a manual download, or redirected to an external site. \
                Try downloading it manually from SpigotMC.",
                title,
                content_type
            ));
        }

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        // Try to extract filename from Content-Disposition header
        let mut filename = None;
        if let Some(cd) = head_response.headers().get(CONTENT_DISPOSITION) {
            if let Ok(cd_str) = cd.to_str() {
                // Look for filename="name.jar" or filename=name.jar
                if let Some(f) = cd_str
                    .split(';')
                    .find(|p| p.trim().starts_with("filename="))
                    .map(|p| p.trim().trim_start_matches("filename=").trim_matches('"'))
                {
                    if !f.is_empty() {
                        filename = Some(f.to_string());
                    }
                }
            }
        }

        // Fallback to slug-based filename if header is missing or invalid
        let filename = filename.unwrap_or_else(|| {
            let ext = info_json["file"]["type"]
                .as_str()
                .unwrap_or(".jar")
                .trim_start_matches('.');
            format!("{}.{}", slug, ext)
        });

        let target_path = target_dir.as_ref().join(&filename);

        download_with_resumption(
            &self.client,
            DownloadOptions {
                url: &download_url,
                target_path: &target_path,
                expected_hash: None, // Spiget doesn't provide hashes in a standard way
                total_size: head_response.content_length(),
            },
            |_, _| {},
        )
        .await?;

        Ok(filename)
    }
}
