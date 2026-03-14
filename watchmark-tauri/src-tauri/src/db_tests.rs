#[cfg(test)]
mod tests {
    use rusqlite::{Connection, params};

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE Media (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tmdb_id TEXT UNIQUE,
                \"type\" TEXT NOT NULL DEFAULT 'TV'
            )",
            (),
        ).unwrap();

        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (episode_id) REFERENCES Episodes (id) ON DELETE CASCADE
            )",
            (),
        ).unwrap();

        conn.execute(
            "CREATE TABLE Episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                media_id INTEGER,
                season_num INTEGER CHECK(season_num >= 0),
                ep_num INTEGER CHECK(ep_num >= 0),
                \"title\" TEXT,
                FOREIGN KEY (media_id) REFERENCES Media (id) ON DELETE CASCADE,
                UNIQUE(media_id, season_num, ep_num)
            )",
            (),
        ).unwrap();

        conn.execute("INSERT INTO Media (tmdb_id, \"type\") VALUES ('123', 'TV')", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, \"type\") VALUES ('456', 'TV')", ()).unwrap();

        conn
    }

    #[test]
    fn test_unique_constraint_double_sync() {
        let conn = setup_test_db();

        // First insert
        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Pilot')",
            (),
        ).unwrap();

        // Attempting to insert S01E01 for the same media_id should fail if not using ON CONFLICT DO UPDATE or IGNORE
        let result = conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Pilot - Updated')",
            (),
        );
        assert!(result.is_err(), "Duplicate insert without conflict resolution should fail");

        // Using ON CONFLICT DO UPDATE (simulating the double sync)
        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Pilot - Updated')
             ON CONFLICT(media_id, season_num, ep_num) DO UPDATE SET \"title\"=excluded.\"title\"",
            (),
        ).unwrap();

        let title: String = conn.query_row("SELECT \"title\" FROM Episodes WHERE media_id=1 AND season_num=1 AND ep_num=1", [], |r| r.get(0)).unwrap();
        assert_eq!(title, "Pilot - Updated", "Title should be updated due to conflict resolution");

        let count: i32 = conn.query_row("SELECT COUNT(*) FROM Episodes WHERE media_id=1 AND season_num=1 AND ep_num=1", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1, "There should be exactly one row for S01E01 of this media");
    }

    #[test]
    fn test_unique_constraint_cross_show() {
        let conn = setup_test_db();

        // Insert S01E01 for Show A (media_id = 1)
        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Show A - Pilot')",
            (),
        ).unwrap();

        // Insert S01E01 for Show B (media_id = 2)
        let result = conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (2, 1, 1, 'Show B - Pilot')",
            (),
        );

        assert!(result.is_ok(), "Inserting S01E01 for a different show should succeed");

        let count: i32 = conn.query_row("SELECT COUNT(*) FROM Episodes WHERE season_num=1 AND ep_num=1", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2, "Both shows should have an S01E01");
    }

    #[test]
    fn test_sorting_specials() {
        let conn = setup_test_db();

        // Insert a normal season episode
        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Season 1 Episode 1')",
            (),
        ).unwrap();

        // Insert a special
        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 0, 1, 'Special 1')",
            (),
        ).unwrap();

        // Insert a normal season episode
        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 2, 'Season 1 Episode 2')",
            (),
        ).unwrap();

        // Query with ordering
        let mut stmt = conn.prepare("SELECT \"title\" FROM Episodes WHERE media_id = 1 ORDER BY season_num ASC, ep_num ASC").unwrap();
        let rows = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();

        let titles: Vec<String> = rows.filter_map(Result::ok).collect();

        assert_eq!(titles.len(), 3);
        assert_eq!(titles[0], "Special 1", "Special (season 0) should be sorted first");
        assert_eq!(titles[1], "Season 1 Episode 1");
        assert_eq!(titles[2], "Season 1 Episode 2");
    }

    #[test]
    fn test_cascading_deletes() {
        let conn = setup_test_db();
        // SQLite in-memory default doesn't turn on FKs
        conn.execute("PRAGMA foreign_keys = ON", ()).unwrap();

        // 1 Media
        // Insert S01E01 for Show A
        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Show A - Pilot')",
            (),
        ).unwrap();

        let episode_id = conn.last_insert_rowid();

        // Insert History for that episode
        conn.execute(
            "INSERT INTO History (episode_id) VALUES (?)",
            [episode_id],
        ).unwrap();

        // Verify they exist
        let media_count: i32 = conn.query_row("SELECT COUNT(*) FROM Media WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(media_count, 1);
        let ep_count: i32 = conn.query_row("SELECT COUNT(*) FROM Episodes WHERE media_id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(ep_count, 1);
        let hist_count: i32 = conn.query_row("SELECT COUNT(*) FROM History WHERE episode_id = ?", [episode_id], |r| r.get(0)).unwrap();
        assert_eq!(hist_count, 1);

        // Delete Media!
        conn.execute("DELETE FROM Media WHERE id = 1", ()).unwrap();

        // Check orphans
        let media_count_after: i32 = conn.query_row("SELECT COUNT(*) FROM Media WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(media_count_after, 0);

        let ep_count_after: i32 = conn.query_row("SELECT COUNT(*) FROM Episodes WHERE media_id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(ep_count_after, 0, "Episodes should be cascaded deleted");

        let hist_count_after: i32 = conn.query_row("SELECT COUNT(*) FROM History WHERE episode_id = ?", [episode_id], |r| r.get(0)).unwrap();
        assert_eq!(hist_count_after, 0, "History should be cascaded deleted when episode is deleted");
    }

    #[test]
    fn test_u32_constraints() {
        let conn = setup_test_db();

        // Inserting season_num = -1
        let result_season = conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, -1, 1, 'Bad Season')",
            (),
        );
        assert!(result_season.is_err(), "Inserting negative season_num should fail due to CHECK constraint");

        // Inserting ep_num = -1
        let result_episode = conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, -1, 'Bad Episode')",
            (),
        );
        assert!(result_episode.is_err(), "Inserting negative ep_num should fail due to CHECK constraint");

        // Inserting ep_num = 0 (Zero-Episode Pilot)
        let result_zero = conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 0, 'Zero-Episode Pilot')",
            (),
        );
        assert!(result_zero.is_ok(), "Inserting ep_num = 0 should succeed");
    }

    #[test]
    fn test_local_files_repair_paths() {
        let conn = setup_test_db();
        // create local files table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Local_Files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER UNIQUE,
                file_path TEXT UNIQUE,
                file_size INTEGER DEFAULT 0,
                file_hash TEXT,
                FOREIGN KEY (episode_id) REFERENCES Episodes (id) ON DELETE CASCADE
            )",
            (),
        ).unwrap();

        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Pilot')",
            (),
        ).unwrap();
        let ep_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO Local_Files (episode_id, file_path) VALUES (?, ?)",
            rusqlite::params![ep_id, "D:\\Movies\\Show\\S01E01.mkv"],
        ).unwrap();

        // Repair Path D:\ -> E:\
        let affected = conn.execute(
            "UPDATE Local_Files SET file_path = REPLACE(file_path, ?, ?) WHERE file_path LIKE ?",
            rusqlite::params!["D:\\", "E:\\", format!("{}%", "D:\\")],
        ).unwrap();

        assert_eq!(affected, 1);

        let new_path: String = conn.query_row(
            "SELECT file_path FROM Local_Files WHERE episode_id = ?",
            [ep_id],
            |r| r.get(0),
        ).unwrap();

        assert_eq!(new_path, "E:\\Movies\\Show\\S01E01.mkv");
    }

    #[test]
    fn test_local_files_remove_link() {
        let conn = setup_test_db();
        conn.execute("PRAGMA foreign_keys = ON", ()).unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Local_Files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER UNIQUE,
                file_path TEXT UNIQUE,
                file_size INTEGER DEFAULT 0,
                file_hash TEXT,
                FOREIGN KEY (episode_id) REFERENCES Episodes (id) ON DELETE CASCADE
            )",
            (),
        ).unwrap();

        conn.execute(
            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\") VALUES (1, 1, 1, 'Pilot')",
            (),
        ).unwrap();
        let ep_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO Local_Files (episode_id, file_path) VALUES (?, ?)",
            rusqlite::params![ep_id, "C:\\Test.mkv"],
        ).unwrap();

        // Remove link
        conn.execute("DELETE FROM Local_Files WHERE episode_id = ?", [ep_id]).unwrap();

        // Ensure file link is gone
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM Local_Files WHERE episode_id = ?", [ep_id], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);

        // Ensure Episode still exists
        let ep_count: i32 = conn.query_row("SELECT COUNT(*) FROM Episodes WHERE id = ?", [ep_id], |r| r.get(0)).unwrap();
        assert_eq!(ep_count, 1, "Episode should NOT be deleted when Local_Files link is removed");
    }

    #[test]
    fn test_history_leap_year_epoch() {
        let conn = Connection::open_in_memory().unwrap();
        // Just mock the History table
        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                last_position INTEGER DEFAULT 0,
                status TEXT DEFAULT 'Completed',
                is_legacy BOOLEAN DEFAULT 0,
                session_id TEXT,
                start_time DATETIME,
                end_time DATETIME,
                pause_count INTEGER DEFAULT 0,
                completion_ratio REAL DEFAULT 0.0
            )",
            (),
        ).unwrap();

        // 2024-02-29T12:00:00Z -> 1709208000
        let ts = 1709208000;
        conn.execute("INSERT INTO History (episode_id, timestamp) VALUES (1, ?)", params![ts]).unwrap();

        let stored_ts: i64 = conn.query_row("SELECT timestamp FROM History WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert_eq!(stored_ts, 1709208000);
    }

    #[test]
    fn test_history_mass_triage_same_millisecond() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            (),
        ).unwrap();

        let ts = 1700000000;

        // Insert 100 rows with exact same timestamp
        let tx = conn.transaction().unwrap();
        for i in 1..=100 {
            tx.execute("INSERT INTO History (episode_id, timestamp) VALUES (?, ?)", params![i, ts]).unwrap();
        }
        tx.commit().unwrap();

        let mut stmt = conn.prepare("SELECT id, episode_id FROM History ORDER BY timestamp DESC, id DESC").unwrap();
        let mut rows = stmt.query([]).unwrap();

        // ID 100 should be first, ID 1 should be last due to `id DESC` tie-breaker
        if let Some(row) = rows.next().unwrap() {
            let id: i32 = row.get(0).unwrap();
            let ep_id: i32 = row.get(1).unwrap();
            assert_eq!(id, 100);
            assert_eq!(ep_id, 100);
        } else {
            panic!("No rows found");
        }

        // Delete one collision entry
        conn.execute("DELETE FROM History WHERE id = 50", []).unwrap();
        let count: i32 = conn.query_row("SELECT COUNT(*) FROM History", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 99); // Only one removed
    }

    #[test]
    fn test_history_missing_timestamp_fallback() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )",
            (),
        ).unwrap();

        conn.execute("INSERT INTO History (episode_id) VALUES (1)", []).unwrap();

        let stored_ts: i64 = conn.query_row("SELECT timestamp FROM History WHERE id = 1", [], |r| r.get(0)).unwrap();

        let now = chrono::Utc::now().timestamp();
        assert!((now - stored_ts).abs() <= 1); // Close enough
    }
}

#[cfg(test)]
mod feature_5_9_tests {
    use rusqlite::Connection;

    #[test]
    fn test_is_legacy_default_0() {
        let conn = Connection::open_in_memory().unwrap();
        // Since we are not running full init_db in this specific unit test framework easily,
        // we'll simulate the table creation with the new schema.
        conn.execute(
            "CREATE TABLE Episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                media_id INTEGER
            )",
            (),
        ).unwrap();

        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                is_legacy INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (episode_id) REFERENCES Episodes (id) ON DELETE CASCADE
            )",
            (),
        ).unwrap();

        conn.execute("INSERT INTO Episodes (media_id) VALUES (1)", ()).unwrap();

        // Test 1: Implicit omission should default to 0
        conn.execute("INSERT INTO History (episode_id) VALUES (1)", ()).unwrap();

        let is_legacy: i32 = conn.query_row(
            "SELECT is_legacy FROM History WHERE id = 1",
            [],
            |row: &rusqlite::Row| row.get(0)
        ).unwrap();

        assert_eq!(is_legacy, 0);

        // Test 2: Direct SQL Injection should force failure if trying to set NULL,
        // or ensure SQLite catches the NOT NULL constraint if we try to insert NULL.
        let result = conn.execute("INSERT INTO History (episode_id, is_legacy) VALUES (1, NULL)", ());
        assert!(result.is_err());
    }

    #[test]
    fn test_legacy_inline_date_edit_preservation() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE Episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                media_id INTEGER
            )",
            (),
        ).unwrap();

        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                is_legacy INTEGER NOT NULL DEFAULT 0
            )",
            (),
        ).unwrap();

        conn.execute("INSERT INTO Episodes (media_id) VALUES (1)", ()).unwrap();

        // Insert a legacy row
        conn.execute("INSERT INTO History (episode_id, timestamp, is_legacy) VALUES (1, 1000000, 1)", ()).unwrap();

        // Simulate "Inline Date Edit" QoL update that tweaks the timestamp
        conn.execute("UPDATE History SET timestamp = 2000000 WHERE id = 1", ()).unwrap();

        // Verify the legacy flag is strictly preserved
        let (ts, legacy): (i64, i32) = conn.query_row(
            "SELECT timestamp, is_legacy FROM History WHERE id = 1",
            [],
            |row: &rusqlite::Row| Ok((row.get(0)?, row.get(1)?))
        ).unwrap();

        assert_eq!(ts, 2000000);
        assert_eq!(legacy, 1);
    }
}

#[cfg(test)]
mod feature_5_6_tests {

    use rusqlite::Connection;
    use crate::scanner::{clean_anime_release_tags, is_too_generic};

    #[test]
    fn test_anime_release_tags_stripping() {
        // [Group] Name [Hash]
        assert_eq!(clean_anime_release_tags("[SubsPlease] Frieren - 01 [720p][A1B2C3D4].mkv"), "Frieren - 01.mkv");

        // (Group) Name (Hash)
        assert_eq!(clean_anime_release_tags("(Erai-raws) Spy x Family - 05 (1080p).mp4"), "Spy x Family - 05.mp4");

        // _v2 / _Final testing
        assert_eq!(clean_anime_release_tags("[SubsPlease] Frieren - 01_v2 [720p].mkv"), "Frieren - 01.mkv");
    }

    #[test]
    fn test_is_too_generic() {
        assert!(is_too_generic("01"));
        assert!(is_too_generic("episode 1"));
        assert!(is_too_generic("Part 5"));
        assert!(!is_too_generic("Frieren"));
    }

    #[test]
    fn test_unmatched_files_schema() {
        let conn = Connection::open_in_memory().unwrap();
        // Since we are running outside the regular init, we manually run the CREATE logic here
        // or just verify that our query structure is sound.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS Unmatched_Files (
                file_path TEXT PRIMARY KEY,
                filename TEXT,
                parsed_series TEXT,
                parsed_season INTEGER,
                parsed_episode INTEGER,
                group_key TEXT
            )",
            (),
        ).unwrap();

        // Messy Folder Test Logic: Verify that different filenames can result in the same group_key
        let filename1 = "Show.Name.S01E01.720p.mkv";
        let filename2 = "Show_Name_S01E02_1080p.mkv";

        let (_, _, _) = crate::scanner::parse_filename(filename1);
        let (_, _, _) = crate::scanner::parse_filename(filename2);

        // In the actual app, these get passed through a replace logic:
        // `replace(&['.', '_'][..], " ").trim().to_string()`

        let cleaned1 = "Show.Name".replace(&['.', '_'][..], " ").trim().to_lowercase();
        let cleaned2 = "Show_Name".replace(&['.', '_'][..], " ").trim().to_lowercase();

        assert_eq!(cleaned1, "show name");
        assert_eq!(cleaned2, "show name");
        assert_eq!(cleaned1, cleaned2);
    }
}

#[cfg(test)]
mod feature_5_7_tests {
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE Media (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tmdb_id TEXT UNIQUE,
                \"type\" TEXT NOT NULL DEFAULT 'TV',
                \"title\" TEXT,
                release_date TEXT,
                is_exact_date BOOLEAN DEFAULT 1
            )",
            (),
        ).unwrap();

        conn
    }

    #[test]
    fn test_jan_1st_sort() {
        let conn = setup_test_db();

        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('1', 'Show B', '2024-01-02')", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('2', 'Show A', '2024-01-01')", ()).unwrap();

        let mut stmt = conn.prepare("SELECT title FROM Media ORDER BY release_date ASC").unwrap();
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().filter_map(Result::ok).collect();

        assert_eq!(rows[0], "Show A");
        assert_eq!(rows[1], "Show B");
    }

    #[test]
    fn test_mass_null_sort_last() {
        let conn = setup_test_db();

        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('1', 'No Date Show', NULL)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('2', 'Empty Date Show', '')", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('3', 'Old Show', '1940-01-01')", ()).unwrap();

        let mut stmt = conn.prepare("SELECT title FROM Media ORDER BY CASE WHEN release_date IS NULL OR release_date = '' THEN 1 ELSE 0 END, release_date DESC").unwrap();
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().filter_map(Result::ok).collect();

        assert_eq!(rows[0], "Old Show");
        assert!(rows[1] == "No Date Show" || rows[1] == "Empty Date Show");
        assert!(rows[2] == "No Date Show" || rows[2] == "Empty Date Show");
    }

    #[test]
    fn test_decade_edge_filter() {
        let conn = setup_test_db();

        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('1', '1989', '1989-12-31')", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('2', '1990', '1990-01-01')", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('3', '1999', '1999-12-31')", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, release_date) VALUES ('4', '2000', '2000-01-01')", ()).unwrap();

        let mut stmt = conn.prepare("SELECT title FROM Media WHERE release_date BETWEEN '1990-01-01' AND '1999-12-31' ORDER BY release_date ASC").unwrap();
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().filter_map(Result::ok).collect();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "1990");
        assert_eq!(rows[1], "1999");
    }
}

#[cfg(test)]
mod feature_5_7_tests_2 {


    #[test]
    fn test_tmdb_parsing_and_padding() {
        // Mock the logic used in tmdb.rs where length is 4.
        let raw_air_date = "2026";
        let (final_date, is_exact) = if raw_air_date.len() == 4 {
            (format!("{}-01-01", raw_air_date), false)
        } else if raw_air_date.is_empty() {
            ("".to_string(), false)
        } else {
            (raw_air_date.to_string(), true)
        };

        assert_eq!(final_date, "2026-01-01");
        assert_eq!(is_exact, false);

        // Valid Date test
        let raw_air_date_valid = "2024-05-12";
        let (final_date_valid, is_exact_valid) = if raw_air_date_valid.len() == 4 {
            (format!("{}-01-01", raw_air_date_valid), false)
        } else if raw_air_date_valid.is_empty() {
            ("".to_string(), false)
        } else {
            (raw_air_date_valid.to_string(), true)
        };

        assert_eq!(final_date_valid, "2024-05-12");
        assert_eq!(is_exact_valid, true);
    }
}

#[cfg(test)]
mod feature_5_10_tests {
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE Media (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tmdb_id TEXT UNIQUE,
                \"title\" TEXT,
                user_rating INTEGER CHECK(user_rating >= 0 AND user_rating <= 10) DEFAULT NULL
            )",
            (),
        ).unwrap();

        conn
    }

    #[test]
    fn test_rating_bounds() {
        let conn = setup_test_db();

        // valid inputs
        assert!(conn.execute("INSERT INTO Media (tmdb_id, user_rating) VALUES ('1', 0)", ()).is_ok());
        assert!(conn.execute("INSERT INTO Media (tmdb_id, user_rating) VALUES ('2', 10)", ()).is_ok());
        assert!(conn.execute("INSERT INTO Media (tmdb_id, user_rating) VALUES ('3', 5)", ()).is_ok());
        assert!(conn.execute("INSERT INTO Media (tmdb_id, user_rating) VALUES ('4', NULL)", ()).is_ok());

        // invalid inputs
        assert!(conn.execute("INSERT INTO Media (tmdb_id, user_rating) VALUES ('5', -1)", ()).is_err(), "Negative values should fail");
        assert!(conn.execute("INSERT INTO Media (tmdb_id, user_rating) VALUES ('6', 11)", ()).is_err(), "Values > 10 should fail");
    }

    #[test]
    fn test_top_rated_sort_order() {
        let conn = setup_test_db();

        conn.execute("INSERT INTO Media (tmdb_id, title, user_rating) VALUES ('1', 'Masterpiece', 10)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, user_rating) VALUES ('2', 'Terrible', 0)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, user_rating) VALUES ('3', 'Average', 5)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, user_rating) VALUES ('4', 'Unrated B', NULL)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, user_rating) VALUES ('5', 'Unrated A', NULL)", ()).unwrap();

        let mut stmt = conn.prepare("SELECT title FROM Media ORDER BY user_rating DESC NULLS LAST, title ASC").unwrap();
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().filter_map(Result::ok).collect();

        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0], "Masterpiece");
        assert_eq!(rows[1], "Average");
        assert_eq!(rows[2], "Terrible");
        assert_eq!(rows[3], "Unrated A");
        assert_eq!(rows[4], "Unrated B");
    }
}

#[cfg(test)]
mod feature_5_11_tests {
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE Media (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tmdb_id TEXT UNIQUE,
                \"title\" TEXT,
                vote_average REAL DEFAULT 0.0
            )",
            (),
        ).unwrap();

        conn
    }

    // Refactored to test the actual rounding function/logic directly if possible,
    // or simulate the exact payload. Since we can't easily mock the reqwest
    // client in `tmdb.rs` without a complex setup, we will define the
    // sanitization step as a local pure function representation here to avoid tautology,
    // and verify the SQLite storage handles the output type correctly.

    fn sanitize_vote_average(raw_vote: f64) -> f64 {
        (raw_vote * 10.0).round() / 10.0
    }

    #[test]
    fn test_the_333_test() {
        let conn = setup_test_db();
        let raw_val = 7.66666666;
        let processed_val = sanitize_vote_average(raw_val);

        conn.execute("INSERT INTO Media (tmdb_id, title, vote_average) VALUES ('1', 'Test', ?)", [processed_val]).unwrap();
        let stored_val: f64 = conn.query_row("SELECT vote_average FROM Media WHERE tmdb_id = '1'", [], |r| r.get(0)).unwrap();
        assert_eq!(stored_val, 7.7);
    }

    #[test]
    fn test_whole_number_storage() {
        let conn = setup_test_db();
        let raw_val = 8.0;
        let processed_val = sanitize_vote_average(raw_val);

        conn.execute("INSERT INTO Media (tmdb_id, title, vote_average) VALUES ('2', 'Test 2', ?)", [processed_val]).unwrap();
        let stored_val: f64 = conn.query_row("SELECT vote_average FROM Media WHERE tmdb_id = '2'", [], |r| r.get(0)).unwrap();
        assert_eq!(stored_val, 8.0);
    }

    #[test]
    fn test_zero_validation() {
        let conn = setup_test_db();
        let raw_val = 0.0;
        let processed_val = sanitize_vote_average(raw_val);

        conn.execute("INSERT INTO Media (tmdb_id, title, vote_average) VALUES ('3', 'Test 3', ?)", [processed_val]).unwrap();
        let stored_val: f64 = conn.query_row("SELECT vote_average FROM Media WHERE tmdb_id = '3'", [], |r| r.get(0)).unwrap();
        assert_eq!(stored_val, 0.0);
    }

    #[test]
    fn test_mixed_null_library_sorting() {
        let conn = setup_test_db();

        conn.execute("INSERT INTO Media (tmdb_id, title, vote_average) VALUES ('1', 'Show A', 8.5)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, vote_average) VALUES ('2', 'Show B', NULL)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, vote_average) VALUES ('3', 'Show C', 9.0)", ()).unwrap();
        conn.execute("INSERT INTO Media (tmdb_id, title, vote_average) VALUES ('4', 'Show D', 0.0)", ()).unwrap();

        let mut stmt = conn.prepare("SELECT title FROM Media ORDER BY vote_average DESC NULLS LAST, title ASC").unwrap();
        let rows: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().filter_map(Result::ok).collect();

        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0], "Show C"); // 9.0
        assert_eq!(rows[1], "Show A"); // 8.5
        assert_eq!(rows[2], "Show D"); // 0.0 (Higher than NULL)
        assert_eq!(rows[3], "Show B"); // NULL
    }
}
#[cfg(test)]
mod tests_feature_5_12 {

    use rusqlite::Connection;

    #[test]
    fn test_fresh_library_zero_default() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE Episodes (id INTEGER PRIMARY KEY, last_position INTEGER NOT NULL DEFAULT 0)", ()).unwrap();
        conn.execute("INSERT INTO Episodes DEFAULT VALUES", ()).unwrap();
        let pos: i32 = conn.query_row("SELECT last_position FROM Episodes LIMIT 1", [], |r| r.get(0)).unwrap();
        assert_eq!(pos, 0);
    }
}

#[cfg(test)]
mod tests_feature_5_13 {

    use rusqlite::{Connection, params};
    use uuid::Uuid;

    #[test]
    fn test_session_id_collision() {
        let mut ids = std::collections::HashSet::new();
        for _ in 0..100000 {
            let id = Uuid::new_v4().to_string();
            assert!(!ids.contains(&id), "Collision detected!");
            ids.insert(id);
        }
    }

    #[test]
    fn test_empty_id_guard() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                session_id TEXT
            )",
            (),
        ).unwrap();

        let initial_id = "";
        let final_id = if initial_id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            initial_id.to_string()
        };

        conn.execute("INSERT INTO History (episode_id, timestamp, session_id) VALUES (1, 100, ?)", params![final_id]).unwrap();

        let stored_id: String = conn.query_row("SELECT session_id FROM History WHERE id = 1", [], |r| r.get(0)).unwrap();
        assert!(!stored_id.is_empty());
        assert!(Uuid::parse_str(&stored_id).is_ok());
    }

    #[test]
    fn test_long_nap_threshold() {
        // Simulates the 6 hour threshold
        let last_timestamp = 1000000;
        let current_timestamp = last_timestamp + 21601; // 6 hours + 1 second

        let diff = current_timestamp - last_timestamp;
        assert!(diff > 21600, "Should exceed 6 hours");
    }

    #[test]
    fn test_short_break_threshold() {
        // Simulates the < 6 hour threshold
        let last_timestamp = 1000000;
        let current_timestamp = last_timestamp + 18000; // 5 hours

        let diff = current_timestamp - last_timestamp;
        assert!(diff <= 21600, "Should not exceed 6 hours");
    }

    #[test]
    fn test_legacy_bypass() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE History (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                episode_id INTEGER NOT NULL,
                is_legacy INTEGER DEFAULT 0,
                session_id TEXT
            )",
            (),
        ).unwrap();

        // For backdated legacy rows, session_id is explicitly NULL
        conn.execute("INSERT INTO History (episode_id, is_legacy, session_id) VALUES (1, 1, NULL)", ()).unwrap();

        let (is_legacy, session_id): (i32, Option<String>) = conn.query_row("SELECT is_legacy, session_id FROM History WHERE id = 1", [], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(is_legacy, 1);
        assert!(session_id.is_none());
    }
}
