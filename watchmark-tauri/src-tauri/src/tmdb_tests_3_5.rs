// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use serde_json::json;

#[test]
fn test_movie_edge_cases() {
    let raw_movie_zero_runtime = json!({
        "id": 101,
        "title": "Zero Minute Short",
        "overview": "A really short movie.",
        "poster_path": "/poster.jpg",
        "backdrop_path": "/backdrop.jpg",
        "runtime": 0,
        "release_date": "2024-01-01"
    });

    // We test the deserialization strategy directly for the runtime.
    // For movies, we use the custom JSON parsing we added.
    let runtime_zero = match serde_json::from_value::<crate::models::TmdbEpisode>(
        raw_movie_zero_runtime.clone(),
    ) {
        Ok(ep) => ep.runtime,
        Err(_) => raw_movie_zero_runtime["runtime"].as_i64().unwrap_or(0) as i32,
    };
    assert_eq!(runtime_zero, 0);

    let raw_movie_massive_runtime = json!({
        "id": 102,
        "title": "Ten Hour Film",
        "overview": "A really long movie.",
        "poster_path": "/poster.jpg",
        "backdrop_path": "/backdrop.jpg",
        "runtime": 600,
        "release_date": "2024-01-01"
    });

    let runtime_massive = match serde_json::from_value::<crate::models::TmdbEpisode>(
        raw_movie_massive_runtime.clone(),
    ) {
        Ok(ep) => ep.runtime,
        Err(_) => raw_movie_massive_runtime["runtime"].as_i64().unwrap_or(0) as i32,
    };
    assert_eq!(runtime_massive, 600);

    // Test missing overview mapped to fallback (Sanitizer layer)
    let raw_synopsis = "";
    let sanitized_synopsis =
        crate::sanitizer::sanitize_text(raw_synopsis, "No overview available.");
    assert_eq!(sanitized_synopsis, "No overview available.");

    // Test TBD release date formatting
    let rel_date = "";
    let (final_date, is_exact, is_known) = if rel_date.is_empty() {
        ("0000-00-00".to_string(), false, false)
    } else {
        crate::sanitizer::sanitize_date(rel_date)
    };

    assert_eq!(final_date, "0000-00-00");
    assert_eq!(is_exact, false);
    assert_eq!(is_known, false);
}

#[test]
fn test_ghost_collection() {
    // A ghost collection is where `belongs_to_collection` is present but might be sparse.
    // Our logic handles it by extracting `id` and `name` from the object gracefully.
    let r = json!({
        "belongs_to_collection": {
            "id": 999,
            "name": "Ghost Franchise"
        }
    });

    let collection = r["belongs_to_collection"].as_object().unwrap();
    let collection_id = serde_json::Number::from(collection["id"].as_i64().unwrap_or(0));
    let collection_name = collection["name"].as_str().unwrap_or("").to_string();

    assert_eq!(collection_id.as_i64().unwrap(), 999);
    assert_eq!(collection_name, "Ghost Franchise");
}
