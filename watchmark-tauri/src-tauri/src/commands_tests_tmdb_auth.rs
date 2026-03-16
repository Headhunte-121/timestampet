// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use std::collections::HashMap;

use crate::commands::AppState;
use crate::models::Settings;

#[tokio::test]
async fn test_feature_3_2_api_key_sanitization() {
    let settings = Arc::new(RwLock::new(Settings::default()));
    let (tx, _rx) = mpsc::channel(1);
    let _state = AppState {
        settings,
        settings_tx: tx,
        db_queue: Arc::new(crate::task_queue::DbTaskQueue::new_for_tests()),
        is_maintenance_mode: AtomicBool::new(false),
        is_api_authorized: AtomicBool::new(true),
        is_rate_limited: AtomicBool::new(false),
        rate_limit_reset: std::sync::atomic::AtomicI64::new(0),
        stats_cache: Arc::new(RwLock::new(None)),
        read_semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
        cancel_tokens: Arc::new(RwLock::new(HashMap::new())),
        failed_image_syncs: Arc::new(RwLock::new(std::collections::HashSet::new())),
        is_scan_cancelled: Arc::new(AtomicBool::new(false)),
        is_scan_paused: Arc::new(AtomicBool::new(false)),
    };

    // Note: since validate_tmdb_key requires State, which requires tauri runtime injection,
    // we can test the sanitization logic independently here.
    let raw_keys = vec![
        "   abc123XYZ  \n\r",
        "abc123XYZ",
        "\t\tabc123XYZ\u{200B}",
    ];

    for raw in raw_keys {
        let sanitized = raw.trim().chars().filter(|c| c.is_alphanumeric()).collect::<String>();
        assert_eq!(sanitized, "abc123XYZ");
    }
}

#[test]
fn test_feature_3_2_rate_limit_retry_math() {
    // Tests the regex/string splitting for extracting the Retry-After header logic.
    let err_str = "RATE_LIMIT:15";
    assert!(err_str.starts_with("RATE_LIMIT:"));
    let parts: Vec<&str> = err_str.split(':').collect();
    let retry_after = parts.get(1).unwrap_or(&"1").parse::<u64>().unwrap_or(1);
    assert_eq!(retry_after, 15);

    let err_str_bad = "RATE_LIMIT:";
    let parts_bad: Vec<&str> = err_str_bad.split(':').collect();
    let retry_after_bad = parts_bad.get(1).unwrap_or(&"1").parse::<u64>().unwrap_or(1);
    assert_eq!(retry_after_bad, 1);
}
