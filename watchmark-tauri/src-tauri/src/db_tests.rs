#[cfg(test)]
mod tests {
    use rusqlite::Connection;

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
}
