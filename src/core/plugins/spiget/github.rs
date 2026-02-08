use super::SpigetClient;
use crate::utils::{DownloadOptions, download_with_resumption};
use anyhow::Result;
use regex::Regex;
use reqwest::header::USER_AGENT;
use std::path::Path;
use tracing::info;

impl SpigetClient {
    pub async fn download_from_github(
        &self,
        url: &str,
        target_dir: impl AsRef<Path>,
        title: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<String> {
        let re = Regex::new(r"github\.com/([^/]+)/([^/]+)")?;
        let caps = re
            .captures(url)
            .ok_or_else(|| anyhow::anyhow!("Invalid GitHub URL: {}", url))?;

        let owner = &caps[1];
        let repo = &caps[2].trim_end_matches(".git");

        // Try to find a tag in the URL (e.g., /releases/tag/v1.0.0 or /releases/download/v1.0.0/...)
        let tag_re = Regex::new(r"/releases/(?:tag|download)/([^/]+)")?;
        let tag = tag_re.captures(url).map(|c| c[1].to_string());

        let api_url = match tag {
            Some(t) => format!(
                "https://api.github.com/repos/{}/{}/releases/tags/{}",
                owner, repo, t
            ),
            None => format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                owner, repo
            ),
        };

        let response = self
            .client
            .get(&api_url)
            .header(
                USER_AGENT,
                concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch release info from GitHub for '{}' ({}): {}. \
                The plugin might not have a public release or the URL is invalid.",
                title,
                api_url,
                response.status()
            ));
        }

        let release: serde_json::Value = response.json().await?;
        let assets = release["assets"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No assets found in GitHub release for '{}'", title))?;

        // Find the best asset:
        let title_l = title.to_lowercase();
        let repo_l = repo.to_lowercase();
        let loader_l = loader.map(|l| l.to_lowercase());
        let gv_l = game_version.map(|gv| gv.to_lowercase());

        let other_loaders = [
            "fabric",
            "forge",
            "neoforge",
            "quilt",
            "bungeecord",
            "velocity",
        ];
        let exclusion_list: Vec<&str> = other_loaders
            .iter()
            .filter(|&&l| loader_l.as_ref().map_or(true, |curr| curr != l))
            .copied()
            .collect();

        let candidates: Vec<&serde_json::Value> = assets
            .iter()
            .filter(|a| {
                let name = a["name"].as_str().unwrap_or_default().to_lowercase();
                if !name.ends_with(".jar") {
                    return false;
                }

                for excluded in &exclusion_list {
                    if name.contains(excluded) {
                        return false;
                    }
                }

                if loader_l
                    .as_ref()
                    .map_or(true, |l| l == "paper" || l == "spigot")
                {
                    if name.contains("-mod-") || name.contains(".mod.") {
                        return false;
                    }
                }

                true
            })
            .collect();

        let asset = candidates
            .iter()
            .max_by_key(|a| {
                let name = a["name"].as_str().unwrap_or_default().to_lowercase();
                let mut score = 0;
                if name.contains(&title_l) || name.contains(&repo_l) {
                    score += 10;
                }
                if let Some(ref l) = loader_l {
                    if name.contains(l) {
                        score += 5;
                    }
                }
                if let Some(ref gv) = gv_l {
                    if name.contains(gv) {
                        score += 3;
                    }
                }
                score
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No suitable .jar asset found in GitHub release for '{}'",
                    title
                )
            })?;

        let download_url = asset["browser_download_url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing download URL for asset"))?;
        let filename = asset["name"].as_str().unwrap_or("plugin.jar");
        let size = asset["size"].as_u64();

        let target_path = target_dir.as_ref().join(filename);
        info!("Downloading plugin from GitHub: {}", download_url);

        download_with_resumption(
            &self.client,
            DownloadOptions {
                url: download_url,
                target_path: &target_path,
                expected_hash: None,
                total_size: size,
            },
            |_, _| {},
        )
        .await?;

        Ok(filename.to_string())
    }
}
