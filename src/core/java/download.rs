use super::JavaManager;
use super::types::AdoptiumRelease;
use crate::app_config::ManagedJavaVersion;
use anyhow::{Result, anyhow};
use tokio::fs;

use crate::artifacts::HashAlgorithm;
use crate::utils::{DownloadOptions, download_with_resumption};

impl JavaManager {
    /// Fetches the latest available Java release for a given major version from Adoptium.
    pub async fn get_latest_release(&self, major_version: u32) -> Result<AdoptiumRelease> {
        let os = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "mac"
        } else {
            "linux"
        };
        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x64"
        };

        let url = format!(
            "https://api.adoptium.net/v3/assets/latest/{}/hotspot?architecture={}&image_type=jdk&os={}&vendor=eclipse",
            major_version, arch, os
        );

        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch Java release: {}",
                response.status()
            ));
        }

        let releases: Vec<AdoptiumRelease> = response.json().await?;
        releases
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No releases found for Java {}", major_version))
    }

    /// Downloads and extracts a Java release.
    pub async fn download_and_install<F>(
        &self,
        release: AdoptiumRelease,
        progress_callback: F,
    ) -> Result<ManagedJavaVersion>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let binary = release
            .binaries
            .first()
            .ok_or_else(|| anyhow!("No binaries in release"))?;
        let url = &binary.package.link;
        let filename = &binary.package.name;
        let expected_sha256 = &binary.package.checksum;

        // 1. Download to temporary file
        let temp_dir = std::env::temp_dir().join("mc-server-wrapper-java");
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).await?;
        }
        let temp_file_path = temp_dir.join(filename);

        download_with_resumption(
            &self.client,
            DownloadOptions {
                url,
                target_path: &temp_file_path,
                expected_hash: Some((expected_sha256, HashAlgorithm::Sha256)),
                total_size: Some(binary.package.size),
            },
            progress_callback,
        )
        .await?;

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
            if temp_file_path_clone
                .extension()
                .map_or(false, |ext| ext == "zip")
            {
                let mut archive = zip::ZipArchive::new(file)?;
                archive.extract(&install_dir_clone)?;
            } else {
                // Assume tar.gz for non-windows
                let tar = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(tar);
                archive.unpack(&install_dir_clone)?;
            }
            Ok::<(), anyhow::Error>(())
        })
        .await??;

        // 3. Cleanup temp file
        let _ = fs::remove_file(&temp_file_path).await;

        // 4. Identify the actual JDK root
        let mut actual_root = install_dir.clone();
        let mut entries = fs::read_dir(&install_dir).await?;
        if let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() && path.join("bin").exists() {
                actual_root = path;
            }
        }

        // 5. Finalize version info
        let version_info = self
            .identify_java_version(&actual_root)
            .await
            .ok_or_else(|| anyhow!("Failed to identify installed Java version"))?;

        Ok(version_info)
    }
}
