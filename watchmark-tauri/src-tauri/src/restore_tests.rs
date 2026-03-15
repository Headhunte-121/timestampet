// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

#[cfg(test)]
mod tests_feature_25_3 {
    use std::fs;
    use std::path::PathBuf;
    use crate::commands::validate_and_stage_restore;
    use crate::execute_cold_swap;

    fn setup_test_env(test_name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join(test_name);
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    #[test]
    fn test_fake_extension_rejection() {
        let temp_dir = setup_test_env("test_fake_extension_rejection");
        let fake_db_path = temp_dir.join("fake.db");

        // Write non-sqlite data
        fs::write(&fake_db_path, "This is not a sqlite database").unwrap();

        let result = validate_and_stage_restore(&fake_db_path, &temp_dir);
        assert!(result.is_err(), "Should have failed integrity check");
        let err_msg = match result.unwrap_err() {
            crate::error::AppError::Custom(msg) => msg,
            _ => panic!("Expected AppError::Custom"),
        };
        assert!(err_msg.contains("not a database"), "Error message should mention invalid SQLite database: {}", err_msg);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_valid_sqlite_staging() {
        let temp_dir = setup_test_env("test_valid_sqlite_staging");
        let valid_db_path = temp_dir.join("valid.db");

        // Create a valid, empty SQLite DB
        let conn = rusqlite::Connection::open(&valid_db_path).unwrap();
        conn.execute("CREATE TABLE Test (id INTEGER)", []).unwrap();
        drop(conn);

        let result = validate_and_stage_restore(&valid_db_path, &temp_dir);
        assert!(result.is_ok(), "Should have passed integrity check");

        assert!(temp_dir.join(".restore_pending").exists(), "Trigger file should exist");
        assert!(temp_dir.join("db").join("watchmark.db.pending").exists(), "Pending DB should exist");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cold_swap_execution() {
        let temp_dir = setup_test_env("test_cold_swap_execution");
        let db_dir = temp_dir.join("db");
        fs::create_dir_all(&db_dir).unwrap();

        let active_db = db_dir.join("watchmark.db");
        let pending_db = db_dir.join("watchmark.db.pending");
        let trigger_file = temp_dir.join(".restore_pending");

        fs::write(&active_db, "active").unwrap();
        fs::write(&pending_db, "pending").unwrap();
        fs::write(&trigger_file, "trigger").unwrap();

        execute_cold_swap(&temp_dir);

        assert!(!trigger_file.exists(), "Trigger file should be deleted");
        assert!(!pending_db.exists(), "Pending DB should be moved");
        assert!(active_db.exists(), "Active DB should exist");

        // Active should now contain "pending"
        let active_content = fs::read_to_string(&active_db).unwrap();
        assert_eq!(active_content, "pending");

        // Old should contain "active"
        let old_db = db_dir.join("watchmark.db.old");
        assert!(old_db.exists(), "Old DB should exist");
        let old_content = fs::read_to_string(&old_db).unwrap();
        assert_eq!(old_content, "active");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cold_swap_missing_pending_cleanup() {
        let temp_dir = setup_test_env("test_cold_swap_missing_pending_cleanup");
        let db_dir = temp_dir.join("db");
        fs::create_dir_all(&db_dir).unwrap();

        let active_db = db_dir.join("watchmark.db");
        let trigger_file = temp_dir.join(".restore_pending");

        fs::write(&active_db, "active").unwrap();
        fs::write(&trigger_file, "trigger").unwrap();

        // Deliberately DO NOT create pending_db

        execute_cold_swap(&temp_dir);

        // Should clean up the trigger but not touch active
        assert!(!trigger_file.exists(), "Trigger file should be deleted");

        let active_content = fs::read_to_string(&active_db).unwrap();
        assert_eq!(active_content, "active");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
