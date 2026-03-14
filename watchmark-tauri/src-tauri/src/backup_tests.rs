#[cfg(test)]
mod tests_feature_25_1 {
    use std::fs;
    use std::path::PathBuf;
    use crate::backup::prune_backups;

    #[test]
    fn test_backup_pruning_logic() {
        // Create a temporary directory manually
        let temp_dir = std::env::temp_dir().join("watchmark_backup_test");
        let _ = fs::remove_dir_all(&temp_dir); // clean up any old test
        fs::create_dir_all(&temp_dir).unwrap();

        // Add 10 dummy files with slightly different names
        for i in 0..10 {
            let filename = format!("watchmark_2024-01-{:02}_12-00.bak", i + 1);
            let path = temp_dir.join(&filename);
            fs::write(&path, b"dummy data").unwrap();

            // Artificial delay to ensure creation times differ (if supported by FS).
            // Actually, `prune_backups` sorts by `metadata.created()`. Some filesystems (ext4)
            // don't always track this reliably depending on config, but Rust usually falls back to mtime if btime isn't available.
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Verify there are 10 files
        let count_before = fs::read_dir(&temp_dir).unwrap().count();
        assert_eq!(count_before, 10);

        // Run pruning logic
        prune_backups(&temp_dir).unwrap();

        // Verify there are exactly 3 files left
        let mut count_after = 0;
        for entry in fs::read_dir(&temp_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() && entry.path().extension().unwrap_or_default() == "bak" {
                count_after += 1;
            }
        }
        assert_eq!(count_after, 3);

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
