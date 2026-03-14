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
