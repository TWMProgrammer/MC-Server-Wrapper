use mc_server_wrapper_core::cache::CacheManager;
use mc_server_wrapper_core::mods::modrinth::ModrinthClient;
use mc_server_wrapper_core::mods::types::SearchOptions;
use serde_json::json;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_modrinth_search_parsing() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = ModrinthClient::with_base_url(mock_server.uri(), cache);

    let search_response = json!({
        "hits": [
            {
                "project_id": "A76uj67l",
                "slug": "fabric-api",
                "title": "Fabric API",
                "description": "Core API library for the Fabric mod toolchain.",
                "downloads": 150000000,
                "icon_url": "https://cdn.modrinth.com/data/A76uj67l/icon.png",
                "author": "FabricMC",
                "categories": ["fabric"],
                "display_categories": ["fabric"],
                "client_side": "required",
                "server_side": "required",
                "project_type": "mod",
                "versions": ["1.0.0"],
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
        .and(query_param("query", "fabric"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_response))
        .mount(&mock_server)
        .await;

    let options = SearchOptions {
        query: "fabric".to_string(),
        facets: None,
        sort: None,
        offset: Some(0),
        limit: Some(1),
        game_version: None,
        loader: None,
        project_type: Some("mod".to_string()),
    };

    let results = client.search(&options).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "A76uj67l");
    assert_eq!(results[0].title, "Fabric API");
    assert_eq!(results[0].author, "FabricMC");
}

#[tokio::test]
async fn test_modrinth_get_project_parsing() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = ModrinthClient::with_base_url(mock_server.uri(), cache);

    let project_response = json!({
        "id": "A76uj67l",
        "slug": "fabric-api",
        "title": "Fabric API",
        "description": "Core API library for the Fabric mod toolchain.",
        "downloads": 150000000,
        "icon_url": "https://cdn.modrinth.com/data/A76uj67l/icon.png",
        "categories": ["fabric"],
        "additional_categories": [],
        "client_side": "required",
        "server_side": "required",
        "project_type": "mod",
        "body": "Detailed description",
        "team": "team-id",
        "status": "approved",
        "published": "2020-01-01T00:00:00Z",
        "updated": "2020-01-01T00:00:00Z",
        "followers": 100,
        "license": {
            "id": "MIT",
            "name": "MIT",
            "url": "https://opensource.org/licenses/MIT"
        },
        "game_versions": ["1.20.1"],
        "loaders": ["fabric"],
        "issues_url": null,
        "source_url": null,
        "wiki_url": null,
        "discord_url": null,
        "donation_urls": [],
        "gallery": [],
        "thread_id": "thread-id",
        "monetization_status": "monetized",
        "versions": ["v1"]
    });

    Mock::given(method("GET"))
        .and(path("/project/A76uj67l"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_response))
        .mount(&mock_server)
        .await;

    let project = client.get_project("A76uj67l").await.unwrap();
    assert_eq!(project.id, "A76uj67l");
    assert_eq!(project.slug, "fabric-api");
    assert_eq!(project.title, "Fabric API");
}

#[tokio::test]
async fn test_modrinth_rate_limit() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = ModrinthClient::with_base_url(mock_server.uri(), cache);

    Mock::given(method("GET"))
        .and(path("/project/A76uj67l"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_server)
        .await;

    let result = client.get_project("A76uj67l").await;
    assert!(result.is_err());
    // reqwest's error_for_status() is not explicitly called in get_project,
    // but the json parsing will fail if it's not JSON.
    // Actually, ModrinthClient::get_project calls .json() directly.
}

#[tokio::test]
async fn test_modrinth_network_failure() {
    let mock_server = MockServer::start().await;
    let cache = Arc::new(CacheManager::default());
    let client = ModrinthClient::with_base_url(mock_server.uri(), cache);

    // Drop the server to simulate a network failure
    drop(mock_server);

    let result = client.get_project("A76uj67l").await;
    assert!(result.is_err());
}
