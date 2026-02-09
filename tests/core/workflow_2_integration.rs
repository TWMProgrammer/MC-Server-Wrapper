use anyhow::Result;
use mc_server_wrapper_core::cache::CacheManager;
use mc_server_wrapper_core::database::Database;
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::mods::{self, modrinth::ModrinthClient, types::SearchOptions};
use serde_json::json;
use std::io::Write;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_instance_manager(dir: &std::path::Path) -> Result<InstanceManager> {
    let db = Arc::new(Database::new(dir.join("test.db")).await?);
    InstanceManager::new(dir, db).await
}

async fn create_dummy_jar(path: &std::path::Path) {
    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        json!({
            "schemaVersion": 1,
            "id": "sodium",
            "version": "0.5.0",
            "name": "Sodium",
            "description": "A powerful optimization mod",
            "authors": ["jellysquid3"],
            "license": "LGPL-3.0-only"
        })
        .to_string()
        .as_bytes(),
    )
    .unwrap();
    zip.finish().unwrap();
}

#[tokio::test]
async fn test_workflow_2_marketplace_flow() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    fs::create_dir_all(&instances_dir).await?;

    let instance_manager = setup_instance_manager(&instances_dir).await?;
    let instance = instance_manager
        .create_instance("Modded Server", "1.20.1")
        .await?;
    let instance_path = instance.path.clone();

    // 1. User searches for a mod (e.g., "Sodium").
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = ModrinthClient::with_base_url(mock_server.uri(), cache.clone());

    let search_response = json!({
        "hits": [
            {
                "project_id": "A76uj67l",
                "slug": "sodium",
                "title": "Sodium",
                "description": "A powerful optimization mod",
                "downloads": 15000000,
                "icon_url": "https://cdn.modrinth.com/icon.png",
                "author": "jellysquid3",
                "categories": ["fabric"],
                "display_categories": ["fabric"],
                "client_side": "required",
                "server_side": "required",
                "project_type": "mod",
                "versions": ["v1"],
                "follows": 1000,
                "date_created": "2020-01-01T00:00:00Z",
                "date_modified": "2020-01-01T00:00:00Z",
                "latest_version": "1.0.0",
                "license": "MIT",
                "gallery": []
            }
        ],
        "offset": 0,
        "limit": 1,
        "total_hits": 1
    });

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("query", "sodium"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_response))
        .mount(&mock_server)
        .await;

    let options = SearchOptions {
        query: "sodium".to_string(),
        facets: None,
        sort: None,
        offset: Some(0),
        limit: Some(1),
        game_version: Some("1.20.1".to_string()),
        loader: Some("fabric".to_string()),
    };

    let search_results = client.search(&options).await?;
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].slug, "sodium");

    // 2. User installs it and verifies it appears in the "Installed Mods" list.
    // Create a dummy jar file to be "downloaded"
    let dummy_jar_path = dir.path().join("dummy-sodium.jar");
    create_dummy_jar(&dummy_jar_path).await;
    let dummy_jar_content = std::fs::read(&dummy_jar_path)?;

    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(&dummy_jar_content);
    let sha1_hash = hex::encode(hasher.finalize());

    let versions_response = json!([
        {
            "id": "v123",
            "project_id": "A76uj67l",
            "name": "Sodium 0.5.0",
            "version_number": "0.5.0",
            "version_type": "release",
            "featured": true,
            "author_id": "user123",
            "date_published": "2020-01-01T00:00:00Z",
            "downloads": 100,
            "files": [
                {
                    "url": format!("{}/download/sodium-0.5.0.jar", mock_server.uri()),
                    "filename": "sodium-0.5.0.jar",
                    "primary": true,
                    "size": dummy_jar_content.len(),
                    "hashes": {
                        "sha1": sha1_hash,
                        "sha512": "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                    }
                }
            ],
            "loaders": ["fabric"],
            "game_versions": ["1.20.1"],
            "dependencies": []
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/project/A76uj67l/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(versions_response))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/download/sodium-0.5.0.jar"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(dummy_jar_content, "application/java-archive"),
        )
        .mount(&mock_server)
        .await;

    // Get versions and download
    let versions = client.get_versions("A76uj67l", None, None).await?;
    let version = &versions[0];

    let mods_dir = instance_path.join("mods");
    client.download_version(version, &mods_dir).await?;

    // Verify it appears in the "Installed Mods" list
    let installed_mods = mods::list_installed_mods(&instance_path).await?;
    assert_eq!(installed_mods.len(), 1);
    assert_eq!(installed_mods[0].name, "Sodium");
    assert_eq!(installed_mods[0].version, Some("0.5.0".to_string()));

    Ok(())
}
