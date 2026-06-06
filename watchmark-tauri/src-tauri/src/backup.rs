// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use crate::db::{get_app_data_dir, get_db_connection};
use crate::error::AppError;
use chrono::Local;
use std::fs;
use std::path::PathBuf;

pub fn ensure_backup_dir() -> std::io::Result<PathBuf> {
    let app_dir = get_app_data_dir();
    let backup_dir = app_dir.join("backups");
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)?;
    }
    Ok(backup_dir)
}

pub fn perform_backup() -> Result<(), AppError> {
    let backup_dir = ensure_backup_dir()
        .map_err(|e| AppError::Custom(format!("Failed to create backups directory: {}", e)))?;

    // Check disk space
    if let Ok(free_space) = fs3::available_space(&backup_dir) {
        // 10 MB = 10 * 1024 * 1024 = 10485760 bytes
        if free_space < 10485760 {
            tracing::warn!("Backup aborted: Less than 10MB of free space available on drive.");
            return Err(AppError::Custom("Insufficient disk space".to_string()));
        }
    }

    // Create timestamped filename: watchmark_YYYY-MM-DD_HH-MM.bak
    let now = Local::now();
    let filename = format!("watchmark_{}.bak", now.format("%Y-%m-%d_%H-%M"));
    let backup_path = backup_dir.join(&filename);

    let conn = get_db_connection()?;

    // SQLite VACUUM INTO
    // We must pass the path as a string literal or bound parameter.
    // Wait, VACUUM INTO syntax is `VACUUM INTO 'filename';`
    let sql = format!(
        "VACUUM INTO '{}'",
        backup_path.to_string_lossy().replace("'", "''")
    );
    conn.execute(&sql, [])?;

    // Prune old backups (keep only the 3 most recent)
    prune_backups(&backup_dir)?;

    Ok(())
}

pub fn prune_backups(backup_dir: &PathBuf) -> Result<(), AppError> {
    let mut backups = Vec::new();

    if let Ok(entries) = fs::read_dir(backup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "bak" {
                if let Ok(metadata) = entry.metadata() {
                    let time = metadata.created().unwrap_or_else(|_| {
                        metadata
                            .modified()
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                    });
                    backups.push((path, time));
                }
            }
        }
    }

    // Sort by creation time, descending
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    // Keep only the first 3
    if backups.len() > 3 {
        for (path, _) in backups.into_iter().skip(3) {
            if let Err(e) = fs::remove_file(&path) {
                tracing::error!("Failed to delete old backup {:?}: {}", path, e);
            }
        }
    }

    Ok(())
}
