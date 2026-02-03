use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use mc_server_wrapper_core::mods::modrinth::ModrinthClient;
use mc_server_wrapper_core::mods::types::SearchOptions;
use serde_json::json;

#[tokio::test]
async fn test_modrinth_search_parsing() {
    let mock_server = MockServer::start().await;
    let client = ModrinthClient::with_base_url(mock_server.uri());

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
                "categories": ["fabric"]
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
    let client = ModrinthClient::with_base_url(mock_server.uri());

    let project_response = json!({
        "id": "A76uj67l",
        "slug": "fabric-api",
        "title": "Fabric API",
        "description": "Core API library for the Fabric mod toolchain.",
        "downloads": 150000000,
        "icon_url": "https://cdn.modrinth.com/data/A76uj67l/icon.png",
        "categories": ["fabric"]
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
    let client = ModrinthClient::with_base_url(mock_server.uri());

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
    let client = ModrinthClient::with_base_url(mock_server.uri());

    // Drop the server to simulate a network failure
    drop(mock_server);

    let result = client.get_project("A76uj67l").await;
    assert!(result.is_err());
}
