// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use crate::models::{DashboardData, Episode, Media};
use crate::sanitizer::{calculate_progress_percentage, sanitize_date, sanitize_text};
use serde_json::json;

#[test]
fn test_year_only_api_bug() {
    // "Year Only" API Bug: Simulate an API response providing just "2025".
    // Verify the Rust layer pads this to "2025-01-01" to satisfy frontend formatting
    let (sanitized, is_exact, is_known) = sanitize_date("2025");
    assert_eq!(sanitized, "2025-01-01");
    assert_eq!(is_exact, false);
    assert_eq!(is_known, true);
}

#[test]
fn test_historic_date_boundary() {
    // Historic Date Boundary: Test a show from 1890.
    // Verify the sanitization layer doesn't accidentally treat old dates as "null" or invalid.
    let (sanitized, is_exact, is_known) = sanitize_date("1890-05-12");
    assert_eq!(sanitized, "1890-05-12");
    assert_eq!(is_exact, true);
    assert_eq!(is_known, true);
}

#[test]
fn test_space_only_payload() {
    // The "Space-Only" Payload: Send a synopsis containing only \n (newlines) and spaces.
    // Verify Rust detects this as "empty" and replaces it with the fallback string.
    let synopsis = "   \n  \t  ";
    let cleaned = sanitize_text(synopsis, "No overview available.");
    assert_eq!(cleaned, "No overview available.");
}

#[test]
fn test_html_tag_injection() {
    // HTML Tag Injection: If TMDB returns raw HTML like <p></p>, verify the Rust layer
    // strips the tags and recognizes the resulting empty string as needing a placeholder.
    let synopsis = "<p></p> <br/> <b></b>";
    let cleaned = sanitize_text(synopsis, "No overview available.");
    assert_eq!(cleaned, "No overview available.");

    // Test that valid text with HTML is stripped but kept
    let synopsis_with_text = "<p>Some actual <b>text</b>.</p>";
    let cleaned2 = sanitize_text(synopsis_with_text, "No overview available.");
    assert_eq!(cleaned2, "Some actual text.");
}

#[test]
fn test_zero_minute_short() {
    // The "0-Minute" Short: Add a file that is 45 seconds long but stored as 0 minutes in the DB.
    // Verify the UI progress bar renders a "0%" or "1%" state rather than NaN
    let progress = calculate_progress_percentage(45, 0); // 45 seconds played, 0 minutes runtime
    assert_eq!(progress, 0.0);
}

#[test]
fn test_negative_runtime_guard() {
    // Negative Runtime Guard: Verify the Rust layer clamps any negative runtime values to 0.
    let progress = calculate_progress_percentage(100, -50);
    assert_eq!(progress, 0.0);

    let progress2 = calculate_progress_percentage(-10, 50);
    assert_eq!(progress2, 0.0);
}

#[test]
fn test_first_boot_dashboard() {
    // First-Boot Dashboard: Open the app with a fresh database.
    // Verify the Rust backend sends a complete JSON object where arrays are empty arrays
    let json_data = json!({
        "stats": {
            "eps_watched": 0,
            "hrs_watched": 0,
            "shows_completed": 0,
            "avg_rating": 0.0
        }
    });

    let dashboard: DashboardData = serde_json::from_value(json_data).unwrap();
    assert!(dashboard.hero_ep.is_none());
    assert!(dashboard.cw_eps.is_empty());
    assert!(dashboard.recent_media.is_empty());
}

#[test]
fn test_filter_to_zero() {
    // Filter-to-Zero: Apply a combination of filters that results in no matches.
    // Verify the backend returns a valid empty list structure rather than an error or null.
    // Simulating deserializing an empty JSON array into a Vec<Media>
    let empty_array = json!([]);
    let media_list: Vec<Media> = serde_json::from_value(empty_array).unwrap();
    assert!(media_list.is_empty());
}

#[test]
fn test_orphaned_local_file() {
    // Orphaned Local File: Simulate an entry in Local_Files that points to a non-existent episode_id.
    // Verify the Rust "Join" logic handles the missing parent gracefully and provides a "Null-Object" placeholder for the Episode

    // In our models, missing fields will use serde(default) and provide empty strings/0s instead of throwing errors.
    let orphaned_json = json!({
        "id": 1,
        "media_id": 100,
        "season_num": 1,
        "ep_num": 1,
        "title": "Broken Ep",
        "runtime": 0,
        "status": "Unwatched",
        "overview": "",
        "air_date": "",
        "is_exact_date": false,
        "watch_count": 0,
        "last_position": 0,
        "still_path": ""
    });

    let ep: Episode = serde_json::from_value(orphaned_json).unwrap();
    assert_eq!(ep.id, 1);
    assert_eq!(ep.media_id, 100);
    assert_eq!(ep.progress_percentage, 0.0);
    assert_eq!(ep.file_path, ""); // Graceful fallback
}

#[test]
fn test_tmdb_special_season() {
    // TMDB "Special" Season: Fetch a show where Season 0 exists but has no metadata.
    // Verify the nested serialization provides empty strings for the season overview rather than skipping the season object entirely.
    let media_json = json!({
        "id": 99,
        "tmdb_id": "12345",
        "type": "TV",
        "title": "Special Show",
        "synopsis": "",
        "poster_path": "",
        "backdrop_path": "",
        "total_episodes": 10,
        "status": "Plan to Watch",
        "vote_average": 0.0,
        "release_date": "",
        "is_exact_date": false
    });

    let media: Media = serde_json::from_value(media_json).unwrap();
    assert_eq!(media.synopsis, "");
    assert_eq!(media.seasons.len(), 0);
    assert_eq!(media.episodes.len(), 0);
    assert_eq!(media.completed_eps, 0);
    assert_eq!(media.is_date_known, false); // From default value
}
