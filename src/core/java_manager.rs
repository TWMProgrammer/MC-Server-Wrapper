use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use crate::app_config::ManagedJavaVersion;

#[derive(Debug, Serialize, Deserialize)]
pub struct AdoptiumRelease {
    pub release_name: String,
    pub binaries: Vec<AdoptiumBinary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdoptiumBinary {
    pub package: AdoptiumPackage,
    pub architecture: String,
    pub os: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdoptiumPackage {
    pub link: String,
    pub name: String,
}

pub struct JavaManager {
    base_dir: PathBuf,
    client: reqwest::Client,
}

impl JavaManager {
    pub fn new() -> Result<Self> {
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;
        let base_dir = exe_path.parent()
            .context("Failed to get executable directory")?
            .join("java");
        
        let client = reqwest::Client::builder()
            .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self { base_dir, client })
    }

    /// Returns the path where Java versions are stored
    pub fn get_base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Fetches the latest available Java release for a given major version from Adoptium.
    pub async fn get_latest_release(&self, major_version: u32) -> Result<AdoptiumRelease> {
        let os = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "mac" } else { "linux" };
        let arch = if cfg!(target_arch = "x86_64") { "x64" } else if cfg!(target_arch = "aarch64") { "aarch64" } else { "x64" };

        let url = format!(
            "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&image_type=jdk&os={}&vendor=eclipse",
            major_version, arch, os
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch Java release: {}", response.status()));
        }

        let releases: Vec<AdoptiumRelease> = response.json().await?;
        releases.into_iter().next().ok_or_else(|| anyhow!("No releases found for Java {}", major_version))
    }

    /// Downloads and extracts a Java release.
    /// `progress_callback` is called with (downloaded_bytes, total_bytes).
    pub async fn download_and_install<F>(&self, release: AdoptiumRelease, mut progress_callback: F) -> Result<ManagedJavaVersion>
    where
        F: FnMut(u64, u64) + Send + 'static,
    {
        let binary = release.binaries.first().ok_or_else(|| anyhow!("No binaries in release"))?;
        let url = &binary.package.link;
        let filename = &binary.package.name;

        // 1. Download to temporary file
        let temp_dir = std::env::temp_dir().join("mc-server-wrapper-java");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).await?;
        }
        let temp_file_path = temp_dir.join(filename);

        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let mut file = fs::File::create(&temp_file_path).await?;

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }
        file.flush().await?;

        // 2. Extract
        let install_dir = self.base_dir.join(&release.release_name);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).await?;
        }
        fs::create_dir_all(&install_dir).await?;

        let temp_file_path_clone = temp_file_path.clone();
        let install_dir_clone = install_dir.clone();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&temp_file_path_clone)?;
            if temp_file_path_clone.extension().map_or(false, |ext| ext == "zip") {
                let mut archive = zip::ZipArchive::new(file)?;
                archive.extract(&install_dir_clone)?;
            } else {
                // Assume tar.gz for non-windows
                let tar = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(tar);
                archive.unpack(&install_dir_clone)?;
            }
            Ok::<(), anyhow::Error>(())
        }).await??;

        // 3. Cleanup temp file
        let _ = fs::remove_file(&temp_file_path).await;

        // 4. Identify the actual JDK root (Adoptium often nests inside a folder)
        let mut actual_root = install_dir.clone();
        let mut entries = fs::read_dir(&install_dir).await?;
        if let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() && path.join("bin").exists() {
                actual_root = path;
            }
        }

        // 5. Finalize version info
        let version_info = self.identify_java_version(&actual_root).await
            .ok_or_else(|| anyhow!("Failed to identify installed Java version"))?;
        
        Ok(version_info)
    }

    /// Deletes a managed Java version.
    pub async fn delete_version(&self, id: &str) -> Result<()> {
        let version_dir = self.base_dir.join(id);
        if version_dir.exists() {
            fs::remove_dir_all(&version_dir).await.context("Failed to delete Java version directory")?;
        }
        Ok(())
    }

    /// Scans the java/ folder for already installed versions.
    /// This is a basic implementation for Phase 1.
    pub async fn discover_installed_versions(&self) -> Result<Vec<ManagedJavaVersion>> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir).await.context("Failed to create java directory")?;
            return Ok(vec![]);
        }

        let mut installed_versions = Vec::new();
        let mut entries = fs::read_dir(&self.base_dir).await.context("Failed to read java directory")?;

        while let Some(entry) = entries.next_entry().await.context("Failed to read directory entry")? {
            let path = entry.path();
            if path.is_dir() {
                // For now, we just assume any directory in the java/ folder might be a JDK.
                // In Phase 2/3 we will add more robust validation (e.g. checking for bin/java).
                if let Some(version_info) = self.identify_java_version(&path).await {
                    installed_versions.push(version_info);
                }
            }
        }

        Ok(installed_versions)
    }

    /// Attempts to identify the Java version in a given directory.
    pub async fn identify_java_version(&self, path: &Path) -> Option<ManagedJavaVersion> {
        let java_exe = if cfg!(windows) {
            path.join("bin").join("java.exe")
        } else {
            path.join("bin").join("java")
        };

        if !java_exe.exists() {
            return None;
        }

        // Try to get version by running java -version
        let output = std::process::Command::new(&java_exe)
            .arg("-version")
            .output()
            .ok()?;
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Example output: openjdk version "17.0.7" 2023-04-18
        let version_regex = regex::Regex::new(r#"version "(\d+)\.(.*)""#).ok()?;
        let captures = version_regex.captures(&stderr)?;
        
        let major_version: u32 = captures.get(1)?.as_str().parse().ok()?;
        let full_version = captures.get(1)?.as_str().to_string() + "." + captures.get(2)?.as_str();

        let dir_name = path.file_name()?.to_string_lossy().to_string();
        
        Some(ManagedJavaVersion {
            id: dir_name,
            name: format!("Java {} (Adoptium)", major_version),
            path: java_exe,
            version: full_version,
            major_version,
        })
    }
}
