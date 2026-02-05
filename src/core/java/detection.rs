use std::path::Path;
use anyhow::{Result, Context};
use tokio::fs;
use crate::app_config::ManagedJavaVersion;
use super::JavaManager;

impl JavaManager {
    /// Scans the java/ folder for already installed versions.
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
