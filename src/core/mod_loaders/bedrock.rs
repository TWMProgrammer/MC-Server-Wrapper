use anyhow::Result;
use super::ModLoaderClient;
use std::io::Cursor;
use zip::ZipArchive;
use std::path::Path;
use tokio::fs;

impl ModLoaderClient {
    pub async fn get_bedrock_versions(&self) -> Result<Vec<String>> {
        // Fetch versions from Bedrock-OSS/BDS-Versions repository
        let url = "https://api.github.com/repos/Bedrock-OSS/BDS-Versions/contents/linux";
        
        #[derive(Debug, serde::Deserialize)]
        struct GitHubContent {
            name: String,
            r#type: String,
        }

        let response = self.client.get(url)
            .header("User-Agent", "Minecraft-Server-Wrapper")
            .send()
            .await?;

        if !response.status().is_success() {
            // Fallback to some hardcoded versions if API fails
            return Ok(vec![
                "1.21.60.10".to_string(),
                "1.21.50.07".to_string(),
                "1.21.44.01".to_string(),
                "1.21.40.01".to_string(),
            ]);
        }

        let contents: Vec<GitHubContent> = response.json().await?;
        let mut versions: Vec<String> = contents.into_iter()
            .filter(|c| c.r#type == "file" && c.name.ends_with(".json") && c.name != "versions.json")
            .map(|c| c.name.replace(".json", ""))
            .collect();

        // Sort versions descending
        versions.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
            let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
            b_parts.cmp(&a_parts)
        });

        Ok(versions)
    }

    pub async fn download_bedrock<F>(&self, version: &str, target_dir: impl AsRef<Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        let os = if cfg!(windows) { "win" } else { "linux" };
        let url = format!("https://minecraft.azureedge.net/bin-{}/bedrock-server-{}.zip", os, version);
        
        let temp_zip = target_dir.as_ref().join("bedrock-server.zip");
        self.download_with_progress(&url, &temp_zip, on_progress).await?;

        // Extract ZIP
        let zip_content = fs::read(&temp_zip).await?;
        let mut archive = ZipArchive::new(Cursor::new(zip_content))?;
        
        for i in 0..archive.len() {
            let (_name, is_dir, outpath) = {
                let file = archive.by_index(i)?;
                let name = file.name().to_string();
                let is_dir = name.ends_with('/');
                let outpath = match file.enclosed_name() {
                    Some(path) => target_dir.as_ref().join(path),
                    None => continue,
                };
                (name, is_dir, outpath)
            };

            if is_dir {
                fs::create_dir_all(&outpath).await?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p).await?;
                    }
                }
                
                // Extract file content in a separate block to ensure ZipFile is dropped
                let mut content = Vec::new();
                {
                    let mut file = archive.by_index(i)?;
                    std::io::copy(&mut file, &mut content)?;
                }
                
                fs::write(&outpath, content).await?;
            }
        }

        // Clean up ZIP
        fs::remove_file(temp_zip).await?;

        Ok(())
    }
}
