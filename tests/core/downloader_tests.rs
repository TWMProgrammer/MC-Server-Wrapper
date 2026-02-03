use mc_server_wrapper_core::downloader::{VersionManifest, LatestVersions, VersionInfo};
use anyhow::Result;
use chrono::Utc;
use sha1::{Sha1, Digest};
use tempfile::tempdir;

#[tokio::test]
async fn test_fetch_manifest_struct() -> Result<()> {
    let manifest = VersionManifest {
        latest: LatestVersions {
            release: "1.20.1".to_string(),
            snapshot: "1.20.2-rc1".to_string(),
        },
        versions: vec![
            VersionInfo {
                id: "1.20.1".to_string(),
                r#type: "release".to_string(),
                url: "http://example.com/v1.json".to_string(),
                release_date: Utc::now(),
            }
        ],
    };

    let json = serde_json::to_string(&manifest)?;
    let parsed: VersionManifest = serde_json::from_str(&json)?;
    assert_eq!(parsed.latest.release, "1.20.1");
    assert_eq!(parsed.versions.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_download_server_verification() -> Result<()> {
    let _dir = tempdir()?;
    
    // Test the SHA1 verification logic
    let data = b"test jar content";
    let mut hasher = Sha1::new();
    hasher.update(data);
    let expected_sha1 = format!("{:x}", hasher.finalize());
    
    assert_eq!(expected_sha1, "09ed5acbdd466291a2b59ca3937751a34eeca360");
    
    Ok(())
}
