use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use mc_server_wrapper_core::plugins::spiget::SpigetClient;
use mc_server_wrapper_core::plugins::types::{SearchOptions, PluginProvider};
use serde_json::json;

#[tokio::test]
async fn test_spiget_search_parsing() {
    let mock_server = MockServer::start().await;
    let client = SpigetClient::with_base_url(mock_server.uri());

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
    let client = SpigetClient::with_base_url(mock_server.uri());

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
    let client = SpigetClient::with_base_url(mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/resources/12345"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_server)
        .await;

    let result = client.get_project("12345").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_spiget_network_failure() {
    let mock_server = MockServer::start().await;
    let client = SpigetClient::with_base_url(mock_server.uri());

    // Drop the server to simulate a network failure
    drop(mock_server);

    let result = client.get_project("12345").await;
    assert!(result.is_err());
}
