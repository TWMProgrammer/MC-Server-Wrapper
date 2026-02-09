use crate::artifacts::HashAlgorithm;
use anyhow::{Context, Result, anyhow};
use futures_util::StreamExt;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::{debug, warn};

pub struct DownloadOptions<'a> {
    pub url: &'a str,
    pub target_path: &'a Path,
    pub expected_hash: Option<(&'a str, HashAlgorithm)>,
    pub total_size: Option<u64>,
}

pub async fn download_with_resumption<F>(
    client: &reqwest::Client,
    options: DownloadOptions<'_>,
    on_progress: F,
) -> Result<()>
where
    F: Fn(u64, u64) + Send + Sync + 'static,
{
    let mut attempt = 0;
    let max_retries = 5;
    let mut delay = Duration::from_secs(2);

    loop {
        attempt += 1;
        match perform_download(client, &options, &on_progress).await {
            Ok(_) => {
                // Verify hash if provided
                if let Some((expected_hash, algo)) = options.expected_hash {
                    debug!(
                        "Verifying hash for downloaded file: {}",
                        options.target_path.display()
                    );
                    let actual_hash = calculate_hash(options.target_path, algo).await?;
                    if actual_hash != expected_hash {
                        warn!(
                            "Hash mismatch for {}. Expected: {}, Got: {}. Retrying from scratch...",
                            options.target_path.display(),
                            expected_hash,
                            actual_hash
                        );
                        // If hash mismatch, we should probably delete the file and start over
                        let _ = fs::remove_file(options.target_path).await;
                        if attempt < max_retries {
                            tokio::time::sleep(delay).await;
                            delay *= 2;
                            continue;
                        }
                        return Err(anyhow!("Hash mismatch after {} attempts", max_retries));
                    }
                }
                return Ok(());
            }
            Err(e) if attempt < max_retries => {
                warn!(
                    "Download failed (attempt {}/{}): {}. Retrying in {:?}...",
                    attempt, max_retries, e, delay
                );
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => {
                return Err(e).context(format!("Download failed after {} attempts", max_retries));
            }
        }
    }
}

async fn perform_download<F>(
    client: &reqwest::Client,
    options: &DownloadOptions<'_>,
    on_progress: &F,
) -> Result<()>
where
    F: Fn(u64, u64) + Send + Sync + 'static,
{
    let target_path = options.target_path;
    let mut downloaded = 0;

    if target_path.exists() {
        downloaded = fs::metadata(target_path).await?.len();
    }

    // If we have an expected size and the file is already that size, we might be done
    // but we'll still send a Range request to be sure (or just return if we trust the size)
    // Actually, it's safer to let the server decide or verify hash later.
    if let Some(total) = options.total_size {
        if downloaded >= total {
            debug!(
                "File {} already exists with size {} (expected {}). Checking...",
                target_path.display(),
                downloaded,
                total
            );
            // We don't return here because the file might be corrupted.
            // We'll let the hash verification handle it if provided.
            // If no hash provided, we might want to return Ok if size matches.
            if options.expected_hash.is_none() {
                // If total is 0, we only skip if the file actually exists.
                // This prevents skipping downloads when the server doesn't provide a size (or returns 0).
                if total > 0 || target_path.exists() {
                    on_progress(total, total);
                    return Ok(());
                }
            }
        }
    }

    let mut request = client.get(options.url);
    if downloaded > 0 {
        request = request.header("Range", format!("bytes={}-", downloaded));
        debug!(
            "Requesting resumption from byte {} for {}",
            downloaded, options.url
        );
    }

    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(anyhow!("Failed to start download: HTTP {}", status));
    }

    let (mut file, current_pos) = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        let file = fs::OpenOptions::new()
            .append(true)
            .open(target_path)
            .await?;
        (file, downloaded)
    } else {
        // Either the server doesn't support Range, or we started from 0,
        // or the server returned 200 OK because the range was invalid/ignored.

        // Ensure parent directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let file = fs::File::create(target_path).await?;
        (file, 0)
    };

    let total_size = options
        .total_size
        .or_else(|| response.content_length().map(|len| len + current_pos))
        .unwrap_or(0);

    on_progress(current_pos, total_size);

    let mut stream = response.bytes_stream();
    let mut last_progress_update = std::time::Instant::now();
    let mut current_downloaded = current_pos;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        file.write_all(&chunk).await?;
        current_downloaded += chunk.len() as u64;

        // Throttle progress updates to avoid overwhelming the UI/logs
        if last_progress_update.elapsed() > Duration::from_millis(100)
            || current_downloaded == total_size
        {
            on_progress(current_downloaded, total_size);
            last_progress_update = std::time::Instant::now();
        }
    }

    file.flush().await?;
    Ok(())
}

async fn calculate_hash(path: &Path, algorithm: HashAlgorithm) -> Result<String> {
    let mut file = fs::File::open(path).await?;
    let mut buffer = [0u8; 8192];

    match algorithm {
        HashAlgorithm::Sha1 => {
            let mut hasher = Sha1::new();
            loop {
                let n = file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            Ok(hex::encode(hasher.finalize()))
        }
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            loop {
                let n = file.read(&mut buffer).await?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            Ok(hex::encode(hasher.finalize()))
        }
    }
}
