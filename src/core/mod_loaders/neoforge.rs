use anyhow::Result;
use super::ModLoaderClient;

impl ModLoaderClient {
    pub async fn get_neoforge_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        // NeoForge uses Maven metadata.
        let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let text = response.text().await?;
        let mut versions = Vec::new();

        let mc_parts: Vec<&str> = mc_version.split('.').collect();
        if mc_parts.len() < 2 {
            return Ok(vec![]);
        }
        
        let major = mc_parts[1]; // "20" from "1.20.1"
        
        let prefix = if mc_parts.len() > 2 {
            format!("{}.{}", major, mc_parts[2])
        } else {
            format!("{}.0", major)
        };

        let re = regex::Regex::new(r"<version>([^<]+)</version>")?;
        for cap in re.captures_iter(&text) {
            let ver = &cap[1];
            if ver.starts_with(&prefix) {
                versions.push(ver.to_string());
            }
        }

        // Sort versions numerically (descending)
        versions.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
            let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
            b_parts.cmp(&a_parts)
        });

        Ok(versions)
    }

    pub async fn download_neoforge<F>(&self, neoforge_version: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        // NeoForge download URL pattern: https://maven.neoforged.net/releases/net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar
        let url = format!("https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar", neoforge_version, neoforge_version);
        self.download_with_progress(&url, target_path, on_progress).await
    }
}
