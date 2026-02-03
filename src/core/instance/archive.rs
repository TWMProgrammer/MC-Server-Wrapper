use std::path::Path;
use anyhow::Result;
use tokio::fs;

pub async fn extract_zip<F>(zip_path: &Path, dst: &Path, root_within_zip: Option<String>, on_progress: F) -> Result<()> 
where F: Fn(u64, u64, String) + Send + Sync + 'static
{
    let zip_path = zip_path.to_path_buf();
    let dst = dst.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let total = archive.len() as u64;

        let root = root_within_zip.map(|r| {
            if r.ends_with('/') { r } else { format!("{}/", r) }
        });

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();
            
            on_progress(i as u64, total, format!("Extracting {}...", name));

            // If a root is specified, only extract files within that root
            if let Some(ref root_path) = root {
                if !name.starts_with(root_path) {
                    continue;
                }
            }

            let relative_name = if let Some(ref root_path) = root {
                name.strip_prefix(root_path).unwrap_or(&name)
            } else {
                &name
            };

            if relative_name.is_empty() {
                continue;
            }

            let outpath = dst.join(relative_name);

            if name.ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok::<(), anyhow::Error>(())
    }).await?
}

pub async fn extract_7z<F>(sz_path: &Path, dst: &Path, root_within_zip: Option<String>, on_progress: F) -> Result<()> 
where F: Fn(u64, u64, String) + Send + Sync + 'static
{
    let sz_path = sz_path.to_path_buf();
    let dst = dst.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let root = root_within_zip.map(|r| {
            if r.ends_with('/') { r } else { format!("{}/", r) }
        });

        // For 7z we need to count entries first to have a total
        let total = {
            let mut file = std::fs::File::open(&sz_path)?;
            let len = file.metadata()?.len();
            let archive = sevenz_rust::Archive::read(&mut file, len, &[])
                .map_err(|e| anyhow::anyhow!("7z read error: {}", e))?;
            archive.files.len() as u64
        };

        let mut current = 0;
        sevenz_rust::decompress_file_with_extract_fn(&sz_path, &dst, |entry, reader, out_dir| {
            let name = entry.name().to_string();
            current += 1;
            on_progress(current, total, format!("Extracting {}...", name));

            // If a root is specified, only extract files within that root
            if let Some(ref root_path) = root {
                if !name.starts_with(root_path) {
                    return Ok(true); // Skip this entry but continue
                }
            }

            let relative_name = if let Some(ref root_path) = root {
                name.strip_prefix(root_path).unwrap_or(&name)
            } else {
                &name
            };

            if relative_name.is_empty() {
                return Ok(true);
            }

            let outpath = out_dir.join(relative_name);

            if entry.is_directory() {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(reader, &mut outfile)?;
            }
            Ok(true)
        }).map_err(|e| anyhow::anyhow!("7z decompression error: {}", e))?;

        Ok::<(), anyhow::Error>(())
    }).await?
}

pub async fn copy_dir_all<F>(src: impl AsRef<Path>, dst: impl AsRef<Path>, on_progress: F) -> Result<()> 
where F: Fn(u64, u64, String) + Send + Sync + 'static
{
    let src = src.as_ref().to_path_buf();
    let dst = dst.as_ref().to_path_buf();
    
    if !dst.exists() {
        fs::create_dir_all(&dst).await?;
    }

    let entries: Vec<_> = walkdir::WalkDir::new(&src).into_iter().filter_map(|e| e.ok()).collect();
    let total = entries.len() as u64;

    for (i, entry) in entries.into_iter().enumerate() {
        let relative_path = entry.path().strip_prefix(&src)?;
        let target_path = dst.join(relative_path);

        on_progress(i as u64, total, format!("Copying {}...", relative_path.display()));

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path).await?;
        } else {
            fs::copy(entry.path(), &target_path).await?;
        }
    }
    Ok(())
}
