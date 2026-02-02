use anyhow::Result;
use super::ModLoaderClient;
use std::io::Cursor;
use zip::ZipArchive;
use std::path::Path;
use tokio::fs;

impl ModLoaderClient {
    pub async fn get_bedrock_versions(&self) -> Result<Vec<String>> {
        // In a real scenario, we might want to scrape the download page or use a community API
        Ok(vec![
            "1.21.60.10".to_string(),
            "1.21.50.07".to_string(),
            "1.21.44.01".to_string(),
            "1.21.40.01".to_string(),
            "1.21.30.03".to_string(),
            "1.21.20.03".to_string(),
            "1.21.2.02".to_string(),
            "1.21.1.03".to_string(),
        ])
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
