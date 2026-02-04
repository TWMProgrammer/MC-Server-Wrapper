use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::mods::{self, modrinth::ModrinthClient, types::SearchOptions};
use mc_server_wrapper_core::database::Database;
use tempfile::tempdir;
use anyhow::Result;
use tokio::fs;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use serde_json::json;
use std::io::Write;
use std::sync::Arc;

async fn setup_instance_manager(dir: &std::path::Path) -> Result<InstanceManager> {
    let db = Arc::new(Database::new(dir.join("test.db")).await?);
    InstanceManager::new(dir, db).await
}

async fn create_dummy_jar(path: &std::path::Path) {
    let file = std::fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    zip.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default()).unwrap();
    zip.write_all(json!({
        "schemaVersion": 1,
        "id": "sodium",
        "version": "0.5.0",
        "name": "Sodium",
        "description": "A powerful optimization mod",
        "authors": ["jellysquid3"],
        "license": "LGPL-3.0-only"
    }).to_string().as_bytes()).unwrap();
    zip.finish().unwrap();
}

#[tokio::test]
async fn test_workflow_2_marketplace_flow() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    fs::create_dir_all(&instances_dir).await?;

    let instance_manager = setup_instance_manager(&instances_dir).await?;
    let instance = instance_manager.create_instance("Modded Server", "1.20.1").await?;
    let instance_path = instance.path.clone();

    // 1. User searches for a mod (e.g., "Sodium").
    let mock_server = MockServer::start().await;
    let client = ModrinthClient::with_base_url(mock_server.uri());

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
                "categories": ["fabric"]
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
    let versions_response = json!([
        {
            "id": "v123",
            "project_id": "A76uj67l",
            "version_number": "0.5.0",
            "files": [
                {
                    "url": format!("{}/download/sodium-0.5.0.jar", mock_server.uri()),
                    "filename": "sodium-0.5.0.jar",
                    "primary": true,
                    "size": 1234
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

    // Create a dummy jar file to be "downloaded"
    let dummy_jar_path = dir.path().join("dummy-sodium.jar");
    create_dummy_jar(&dummy_jar_path).await;
    let dummy_jar_content = std::fs::read(&dummy_jar_path)?;

    Mock::given(method("GET"))
        .and(path("/download/sodium-0.5.0.jar"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(dummy_jar_content, "application/java-archive"))
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
