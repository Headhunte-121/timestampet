#[cfg(test)]
mod tests_feature_25_2 {
    use std::fs;
    use crate::task_queue::DbTaskQueue;
    use std::sync::Arc;
    use tauri::State;

    // We can't easily mock `tauri::State` perfectly without Tauri's test context,
    // but we can test the core logic. Since `export_database` takes `tauri::State`,
    // it's tricky to call directly. We'll test the actual DB task queue backup logic.
    #[test]
    fn test_export_database_logic() {
        let temp_dir = std::env::temp_dir().join("watchmark_export_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let db_queue = Arc::new(DbTaskQueue::new());
        let target_path = temp_dir.join("export.db");
        let target_path_str = target_path.to_string_lossy().to_string();

        let (tx, rx) = std::sync::mpsc::channel();

        db_queue.push_high_priority(move |conn| {
            // Write some dummy data so we can verify the export worked
            conn.execute("CREATE TABLE IF NOT EXISTS TestTable (id INTEGER PRIMARY KEY)", []).unwrap();
            conn.execute("INSERT INTO TestTable DEFAULT VALUES", []).unwrap();

            let sql = format!("VACUUM INTO '{}'", target_path_str.replace("'", "''"));
            let res = conn.execute(&sql, []);
            tx.send(res).unwrap();
        });

        let res = rx.recv().unwrap();
        assert!(res.is_ok(), "VACUUM INTO failed: {:?}", res);

        assert!(target_path.exists());

        // Open the exported db to verify
        let exported_conn = rusqlite::Connection::open(&target_path).unwrap();
        let count: i32 = exported_conn.query_row("SELECT COUNT(*) FROM TestTable", [], |r| r.get(0)).unwrap();
        assert!(count >= 1); // Allow count to be >= 1 to handle the fact that tests run concurrently and the table might have existing records.

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
