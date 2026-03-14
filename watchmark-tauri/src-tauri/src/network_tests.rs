use crate::network::NetworkManager;
use crate::error::AppError;
use reqwest_middleware::Error as MiddlewareError;
use reqwest::{Client, Error as ReqwestError};

#[tokio::test]
async fn test_https_enforcement() {
    let network_manager = NetworkManager::new();

    // Attempting to make an HTTP request using the external client
    // should fail because of `https_only(true)`
    let res = network_manager.external_client.get("http://example.com").send().await;
    assert!(res.is_err(), "External client should reject HTTP requests");
}

#[cfg(test)]
mod tests_feature_3_3 {
    use crate::tmdb::search_media;

    // TMDB requires a valid key to actually hit the network in tests.
    // If TMDB_API_KEY is not set, we skip the test or mock it, but for integration
    // it's best to check for the env var.

    fn get_api_key() -> Option<String> {
        std::env::var("TMDB_API_KEY").ok()
    }

    #[tokio::test]
    async fn test_same_name_collision_scream() {
        if let Some(key) = get_api_key() {
            let results = search_media(&key, "Scream", 1).await.unwrap();
            let mut found_movie = false;
            let mut found_tv = false;

            for r in results {
                let r_type = r["type"].as_str().unwrap_or("");
                let r_title = r["title"].as_str().unwrap_or("");

                if r_type == "Movie" && r_title.contains("Scream") {
                    found_movie = true;
                }
                if r_type == "TV" && r_title.contains("Scream") {
                    found_tv = true;
                }
            }

            assert!(found_movie, "Expected to find Scream movie");
            assert!(found_tv, "Expected to find Scream TV series");
        } else {
            println!("Skipping TMDB network test: No API key.");
        }
    }

    #[tokio::test]
    async fn test_actor_name_search_brad_pitt() {
        if let Some(key) = get_api_key() {
            let results = search_media(&key, "Brad Pitt", 1).await.unwrap();
            // We should get a list of movies/shows Brad Pitt is known for
            // rather than failing or returning empty.
            assert!(!results.is_empty(), "Expected 'known_for' to be flattened into results");

            // Make sure none of them are type "Person"
            for r in results {
                let r_type = r["type"].as_str().unwrap_or("");
                assert!(r_type == "Movie" || r_type == "TV", "Expected only Movie or TV types, got {}", r_type);
            }
        } else {
            println!("Skipping TMDB network test: No API key.");
        }
    }

    #[tokio::test]
    async fn test_collection_result_filtered() {
        if let Some(key) = get_api_key() {
            let results = search_media(&key, "Star Wars Collection", 1).await.unwrap();

            // Collections should be filtered out. The results might still contain movies
            // with "Star Wars" or "Collection" in the title, but no raw TMDB collection types.
            // The function strictly allows "movie" and "tv".
            for r in results {
                let r_type = r["type"].as_str().unwrap_or("");
                assert!(r_type == "Movie" || r_type == "TV", "Expected only Movie or TV types");
            }
        } else {
            println!("Skipping TMDB network test: No API key.");
        }
    }

    #[tokio::test]
    async fn test_emoji_and_reserved_chars() {
        if let Some(key) = get_api_key() {
            // "🎥 Movie" tests emojis. "Batman & Robin" tests reserved chars.
            let res_emoji = search_media(&key, "🎥 Movie", 1).await;
            assert!(res_emoji.is_ok(), "Emoji search should not panic");

            let res_ampersand = search_media(&key, "Batman & Robin", 1).await.unwrap();
            let mut found = false;
            for r in res_ampersand {
                let title = r["title"].as_str().unwrap_or("");
                if title.contains("Batman") && title.contains("Robin") {
                    found = true;
                }
            }
            assert!(found, "Expected to find Batman & Robin correctly encoded");
        } else {
            println!("Skipping TMDB network test: No API key.");
        }
    }
}

#[tokio::test]
async fn test_error_mapping() {
    // Generate a timeout error to test the mapping
    let network_manager = NetworkManager::new();

    // We can simulate a timeout by setting a very short timeout and hitting a slow endpoint,
    // or just checking if AppError::NetworkTimeout serializes properly as required by React.
    let timeout_err = AppError::NetworkTimeout;
    let serialized = serde_json::to_string(&timeout_err).unwrap();
    assert!(serialized.contains(r#""code":"TIMEOUT""#));
    assert!(serialized.contains(r#""type":"NetworkError""#));

    let blocked_err = AppError::NetworkBlocked;
    let serialized_blocked = serde_json::to_string(&blocked_err).unwrap();
    assert!(serialized_blocked.contains(r#""code":"BLOCKED""#));

    let offline_err = AppError::NetworkOffline;
    let serialized_offline = serde_json::to_string(&offline_err).unwrap();
    assert!(serialized_offline.contains(r#""code":"OFFLINE""#));
}

#[tokio::test]
async fn test_local_client_allows_http() {
    let network_manager = NetworkManager::new();

    // Local client should NOT fail strictly because of HTTP, it might fail because nothing is on port 8080
    // but we can check if it attempts the request instead of rejecting the schema.
    let res = network_manager.local_client.get("http://127.0.0.1:8080/dummy").send().await;

    // It should be a connection refused error, NOT a scheme error.
    if let Err(e) = res {
        if let MiddlewareError::Reqwest(reqwest_err) = e {
            assert!(reqwest_err.is_connect() || reqwest_err.is_timeout(), "Expected connect or timeout error, got: {}", reqwest_err);
        } else {
            panic!("Expected Reqwest error type inside middleware");
        }
    }
}
