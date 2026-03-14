use reqwest::{Client, Error as ReqwestError};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error as MiddlewareError};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::time::Duration;
use crate::error::AppError;
use std::sync::Arc;

const USER_AGENT: &str = "WatchMark/2.0 (Windows; Desktop; +https://github.com/your-repo)";

#[derive(Clone)]
pub struct NetworkManager {
    pub external_client: ClientWithMiddleware,
    pub local_client: ClientWithMiddleware,
}

impl NetworkManager {
    pub fn new() -> Self {
        // External Client (TMDB)
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let ext_client = Client::builder()
            .user_agent(USER_AGENT)
            .https_only(true)
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build external reqwest client");

        let external_client = ClientBuilder::new(ext_client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Local Client (VLC Heartbeat)
        let loc_client = Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to build local reqwest client");

        let local_client = ClientBuilder::new(loc_client).build();

        Self {
            external_client,
            local_client,
        }
    }

    pub fn handle_error(err: MiddlewareError) -> AppError {
        match err {
            MiddlewareError::Reqwest(e) => Self::map_reqwest_error(e),
            MiddlewareError::Middleware(e) => AppError::Custom(format!("Middleware error: {}", e)),
        }
    }

    fn map_reqwest_error(e: ReqwestError) -> AppError {
        if e.is_timeout() {
            return AppError::NetworkTimeout;
        }

        if let Some(status) = e.status() {
            if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::UNAUTHORIZED {
                return AppError::NetworkBlocked;
            }
        }

        if e.is_connect() {
            // Can be os error 10013 (Permission Denied) or generic offline
            let msg = e.to_string().to_lowercase();
            if msg.contains("os error 10013") || msg.contains("permission denied") {
                return AppError::NetworkBlocked;
            }
            return AppError::NetworkOffline;
        }

        AppError::NetworkError(e)
    }
}

// Global instance to use across the app
lazy_static::lazy_static! {
    pub static ref NETWORK_MANAGER: Arc<NetworkManager> = Arc::new(NetworkManager::new());
}

#[cfg(test)]
#[path = "network_tests.rs"]
mod network_tests;
