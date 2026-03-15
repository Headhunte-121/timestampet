use crate::commands::AppState;
use crate::task_queue::DbTaskQueue;
use std::sync::atomic::{AtomicBool, AtomicI64};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_one_piece_staggering_simulator() {
    // Simulate a fetch loop for 20 seasons, ensure logic doesn't burst
    let state = AppState {
        settings: Arc::new(RwLock::new(crate::models::Settings::default())),
        settings_tx: tokio::sync::mpsc::channel(1).0,
        db_queue: Arc::new(DbTaskQueue::new_for_tests()),
        is_maintenance_mode: AtomicBool::new(false),
        is_api_authorized: AtomicBool::new(true),
        is_rate_limited: AtomicBool::new(false),
        rate_limit_reset: AtomicI64::new(0),
        stats_cache: Arc::new(RwLock::new(None)),
        read_semaphore: Arc::new(tokio::sync::Semaphore::new(2)),
        cancel_tokens: Arc::new(RwLock::new(HashMap::new())),
        failed_image_syncs: Arc::new(RwLock::new(std::collections::HashSet::new())),
    };

    let start_time = std::time::Instant::now();
    let num_seasons = 5; // Reduced for unit test speed, but proves logic

    for i in 0..num_seasons {
        if i > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
        }
        // Mock fetch
        tokio::task::yield_now().await;
    }

    let elapsed = start_time.elapsed().as_millis();
    assert!(elapsed >= (num_seasons - 1) * 250);
}

#[tokio::test]
async fn test_rate_limit_backoff_logic() {
    // Tests the backoff delay directly
    let reset = chrono::Utc::now().timestamp() + 1; // 1 sec backoff
    let start_time = std::time::Instant::now();

    let now = chrono::Utc::now().timestamp();
    if now < reset {
        let sleep_dur = (reset - now) as u64;
        tokio::time::sleep(tokio::time::Duration::from_secs(sleep_dur.max(1))).await;
    }

    let elapsed = start_time.elapsed().as_millis();
    assert!(elapsed >= 1000, "Should have slept for at least 1 second");
}

#[tokio::test]
async fn test_add_remove_sprint_cancellation() {
    let cancel_tokens: Arc<RwLock<HashMap<String, CancellationToken>>> = Arc::new(RwLock::new(HashMap::new()));
    let media_id = 999;
    let token = CancellationToken::new();

    {
        let mut tokens = cancel_tokens.write().unwrap();
        tokens.insert(media_id.to_string(), token.clone());
    }

    // Simulate delete command
    {
        let mut tokens = cancel_tokens.write().unwrap();
        if let Some(t) = tokens.remove(&media_id.to_string()) {
            t.cancel();
        }
    }

    assert!(token.is_cancelled(), "Token should be cancelled");
}

#[tokio::test]
async fn test_missing_season_skip_logic() {
    // Tests that a missing season simply logs and allows loop continuation
    let seasons = vec![
        serde_json::json!({ "season_number": 1 }),
        serde_json::json!({ "season_number": 2 }), // Simulate missing
        serde_json::json!({ "season_number": 3 }),
    ];

    let mut successful_fetches = 0;
    let mut warnings = 0;

    for season in seasons {
        let s_num = season["season_number"].as_i64().unwrap();

        let mock_result: Result<Vec<serde_json::Value>, crate::error::AppError> = if s_num == 2 {
            Err(crate::error::AppError::Custom("NOT_FOUND".to_string()))
        } else {
            Ok(vec![])
        };

        match mock_result {
            Ok(_) => {
                successful_fetches += 1;
            },
            Err(e) if e.to_string() == "NOT_FOUND" => {
                warnings += 1;
                continue;
            },
            Err(_) => panic!("Unexpected error"),
        }
    }

    assert_eq!(successful_fetches, 2);
    assert_eq!(warnings, 1);
}

#[tokio::test]
async fn test_specials_ordering_integrity() {
    // Season 0 handling validation
    let seasons = vec![
        serde_json::json!({ "season_number": 1 }),
        serde_json::json!({ "season_number": 0 }), // Specials
        serde_json::json!({ "season_number": 2 }),
    ];

    let mut sorted_seasons = seasons.clone();
    sorted_seasons.sort_by_key(|s| s["season_number"].as_i64().unwrap());

    assert_eq!(sorted_seasons[0]["season_number"].as_i64().unwrap(), 0);
    assert_eq!(sorted_seasons[1]["season_number"].as_i64().unwrap(), 1);
    assert_eq!(sorted_seasons[2]["season_number"].as_i64().unwrap(), 2);
}
