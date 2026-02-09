use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use mc_server_wrapper_core::plugins::spiget::SpigetClient;
use mc_server_wrapper_core::plugins::types::{SearchOptions, PluginProvider};
use mc_server_wrapper_core::cache::CacheManager;
use std::sync::Arc;
use serde_json::json;

#[tokio::test]
async fn test_spiget_search_parsing() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = SpigetClient::with_base_url(mock_server.uri(), cache);

    let search_response = json!([
        {
            "id": 12345,
            "name": "EssentialsX",
            "tag": "The modern Essentials suite for Spigot and Paper.",
            "downloads": 5000000,
            "icon": {
                "url": "https://www.spigotmc.org/data/resource_icons/12/12345.jpg"
            },
            "author": {
                "id": 101
            }
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/search/resources/essentials"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_response))
        .mount(&mock_server)
        .await;

    let options = SearchOptions {
        query: "essentials".to_string(),
        facets: None,
        sort: None,
        offset: Some(0),
        limit: Some(1),
        game_version: None,
        loader: None,
    };

    let results = client.search(&options).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "12345");
    assert_eq!(results[0].title, "EssentialsX");
    assert_eq!(results[0].provider, PluginProvider::Spiget);
}

#[tokio::test]
async fn test_spiget_get_project_parsing() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = SpigetClient::with_base_url(mock_server.uri(), cache);

    let project_response = json!({
        "id": 12345,
        "name": "EssentialsX",
        "tag": "The modern Essentials suite for Spigot and Paper.",
        "downloads": 5000000,
        "icon": {
            "url": "https://www.spigotmc.org/data/resource_icons/12/12345.jpg"
        },
        "author": {
            "id": 101
        }
    });

    Mock::given(method("GET"))
        .and(path("/resources/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_response))
        .mount(&mock_server)
        .await;

    let project = client.get_project("12345").await.unwrap();
    assert_eq!(project.id, "12345");
    assert_eq!(project.title, "EssentialsX");
}

#[tokio::test]
async fn test_spiget_rate_limit() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = SpigetClient::with_base_url(mock_server.uri(), cache);

    Mock::given(method("GET"))
        .and(path("/resources/12345"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_server)
        .await;

    let result = client.get_project("12345").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_spiget_download_resource() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = SpigetClient::with_base_url(mock_server.uri(), cache);
    let temp_dir = tempfile::tempdir().unwrap();

    let project_response = json!({
        "id": 12345,
        "name": "My Plugin",
        "file": {
            "type": ".jar",
            "size": 100,
            "sizeUnit": "B",
            "url": "resources/12345/download"
        }
    });

    Mock::given(method("GET"))
        .and(path("/resources/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_response))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/resources/12345/download"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_raw(vec![0u8; 100], "application/java-archive")
            .insert_header("Content-Disposition", "attachment; filename=\"my-actual-plugin.jar\""))
        .mount(&mock_server)
        .await;

    Mock::given(method("HEAD"))
        .and(path("/resources/12345/download"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("Content-Type", "application/java-archive")
            .insert_header("Content-Length", "100")
            .insert_header("Content-Disposition", "attachment; filename=\"my-actual-plugin.jar\""))
        .mount(&mock_server)
        .await;

    let filename = client
        .download_resource("12345", temp_dir.path(), None, None)
        .await
        .unwrap();
    assert_eq!(filename, "my-actual-plugin.jar");
    assert!(temp_dir.path().join("my-actual-plugin.jar").exists());
}

#[tokio::test]
async fn test_spiget_download_external_fails() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = SpigetClient::with_base_url(mock_server.uri(), cache);
    let temp_dir = tempfile::tempdir().unwrap();

    let project_response = json!({
        "id": 12345,
        "name": "External Plugin",
        "file": {
            "type": "external",
            "externalUrl": "https://example.com/download"
        }
    });

    Mock::given(method("GET"))
        .and(path("/resources/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_response))
        .mount(&mock_server)
        .await;

    let result = client
        .download_resource("12345", temp_dir.path(), None, None)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("external download URL"));
}

#[tokio::test]
async fn test_spiget_network_failure() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = SpigetClient::with_base_url(mock_server.uri(), cache);

    // Drop the server to simulate a network failure
    drop(mock_server);

    let result = client.get_project("12345").await;
    assert!(result.is_err());
}
