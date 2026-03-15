use crate::db::{get_db_connection, init_db};
use crate::commands::get_media_details_db;
use std::sync::Once;
use rusqlite::params;

static INIT: Once = Once::new();

fn setup_test_db() {
    INIT.call_once(|| {
        let _ = std::fs::remove_dir_all(crate::db::get_app_data_dir());
        init_db().unwrap();
    });
}

#[test]
fn test_get_media_details_db_handles_null_episode_data_safely_no_panics() {
    setup_test_db();

    let media_id = {
        let mut conn = get_db_connection().unwrap();
        let tx = conn.transaction().unwrap();

        tx.execute(
            "INSERT INTO Media (tmdb_id, type, title, total_episodes) VALUES ('test_null_episode', 'TV', 'Test Show', 1)",
            [],
        ).unwrap();

        let media_id = tx.last_insert_rowid() as i32;

        // Insert an episode with mostly NULL/default values simulating unpopulated schema columns
        // note: last_position has a NOT NULL constraint, so we supply a value for it, but test others with NULL
        tx.execute(
            "INSERT INTO Episodes (
                media_id, season_num, ep_num, title,
                runtime, last_position, watch_count,
                status, overview, season_overview,
                air_date, completed_date
            ) VALUES (
                ?, 1, 1, 'Null Ep',
                NULL, 0, NULL,
                NULL, NULL, NULL,
                NULL, NULL
            )",
            params![media_id],
        ).unwrap();

        tx.commit().unwrap();
        media_id
    };

    // The explicit query selection and the Option-wrapped mappings should prevent rusqlite panics
    // here when it processes the mostly-NULL row.
    let details = get_media_details_db(media_id);
    assert!(details.is_ok(), "Expected OK, but got error: {:?}", details.err());

    let data = details.unwrap();
    let episodes = data["episodes"].as_array().expect("Expected episodes array");
    assert_eq!(episodes.len(), 1);

    let ep = &episodes[0];

    // Check fallback behavior to ensure Option::unwrap_or logic triggered correctly
    assert_eq!(ep["runtime"].as_i64(), Some(0));
    assert_eq!(ep["last_position"].as_i64(), Some(0));
    assert_eq!(ep["watch_count"].as_i64(), Some(0));
    assert_eq!(ep["air_date"].as_str(), Some("0000-00-00")); // Sanitized fallback for missing date
    assert_eq!(ep["status"].as_str(), Some(""));
}
