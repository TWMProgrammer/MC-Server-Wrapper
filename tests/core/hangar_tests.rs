use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use mc_server_wrapper_core::plugins::hangar::HangarClient;
use mc_server_wrapper_core::plugins::types::SearchOptions;
use serde_json::json;

#[tokio::test]
async fn test_hangar_search_parsing() {
    let mock_server = MockServer::start().await;
    let client = HangarClient::with_base_url(mock_server.uri());

    let search_response = json!({
        "result": [
            {
                "name": "ProtocolLib",
                "description": "Provides read/write access to the Minecraft protocol.",
                "namespace": {
                    "owner": "dmulloy2",
                    "slug": "ProtocolLib"
                },
                "stats": {
                    "downloads": 1000000,
                    "stars": 500,
                    "views": 2000000,
                    "recent_views": 1000,
                    "recent_downloads": 500
                },
                "avatarUrl": "https://hangar.papermc.io/api/v1/projects/dmulloy2/ProtocolLib/avatar"
            }
        ],
        "pagination": {
            "offset": 0,
            "limit": 1,
            "count": 1
        }
    });

    Mock::given(method("GET"))
        .and(path("/projects"))
        .and(query_param("q", "ProtocolLib"))
        .respond_with(ResponseTemplate::new(200).set_body_json(search_response))
        .mount(&mock_server)
        .await;

    let options = SearchOptions {
        query: "ProtocolLib".to_string(),
        facets: None,
        sort: None,
        offset: Some(0),
        limit: Some(1),
        game_version: None,
        loader: None,
    };

    let results = client.search(&options).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "dmulloy2/ProtocolLib");
    assert_eq!(results[0].title, "ProtocolLib");
    assert_eq!(results[0].author, "dmulloy2");
}

#[tokio::test]
async fn test_hangar_get_dependencies() {
    let mock_server = MockServer::start().await;
    let client = HangarClient::with_base_url(mock_server.uri());

    // Mock for version response
    let version_response = json!({
        "result": [
            {
                "name": "1.0.0",
                "pluginDependencies": {
                    "PAPER": [
                        {
                            "name": "Vault",
                            "required": true,
                            "namespace": {
                                "owner": "MilkBowl",
                                "slug": "Vault"
                            }
                        },
                        {
                            "name": "PlaceholderAPI",
                            "required": false,
                            "externalUrl": "https://example.com/papi"
                        }
                    ]
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/projects/test-project/versions"))
        .and(query_param("limit", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(version_response))
        .mount(&mock_server)
        .await;

    // Mock for Vault project fetch
    let vault_project_response = json!({
        "name": "Vault",
        "namespace": {
            "owner": "MilkBowl",
            "slug": "Vault"
        },
        "description": "Vault is a Permissions, Chat, and Economy API",
        "stats": {
            "downloads": 5000000
        },
        "avatarUrl": null
    });

    Mock::given(method("GET"))
        .and(path("/projects/MilkBowl/Vault"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vault_project_response))
        .mount(&mock_server)
        .await;

    let deps = client.get_dependencies("test-project").await.unwrap();
    assert_eq!(deps.len(), 2);

    // Check Vault (internal dependency)
    assert_eq!(deps[0].project.title, "Vault");
    assert_eq!(deps[0].project.id, "MilkBowl/Vault");
    assert_eq!(deps[0].dependency_type, "required");

    // Check PlaceholderAPI (external dependency)
    assert_eq!(deps[1].project.title, "PlaceholderAPI");
    assert_eq!(deps[1].project.id, "https://example.com/papi");
    assert_eq!(deps[1].dependency_type, "optional");
}

#[tokio::test]
async fn test_hangar_get_project() {
    let mock_server = MockServer::start().await;
    let client = HangarClient::with_base_url(mock_server.uri());

    let project_response = json!({
        "name": "ProtocolLib",
        "namespace": {
            "owner": "dmulloy2",
            "slug": "ProtocolLib"
        },
        "description": "Provides read/write access to the Minecraft protocol.",
        "stats": {
            "downloads": 1000000
        },
        "avatarUrl": "https://hangar.papermc.io/api/v1/projects/dmulloy2/ProtocolLib/avatar"
    });

    Mock::given(method("GET"))
        .and(path("/projects/dmulloy2/ProtocolLib"))
        .respond_with(ResponseTemplate::new(200).set_body_json(project_response))
        .mount(&mock_server)
        .await;

    let project = client.get_project("dmulloy2/ProtocolLib").await.unwrap();
    assert_eq!(project.id, "dmulloy2/ProtocolLib");
    assert_eq!(project.slug, "ProtocolLib");
    assert_eq!(project.title, "ProtocolLib");
    assert_eq!(project.author, "dmulloy2");
}
