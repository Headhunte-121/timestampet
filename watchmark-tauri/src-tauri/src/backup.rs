// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use crate::db::{get_app_data_dir, get_db_connection};
use crate::error::AppError;
use chrono::Local;
use rusqlite::backup::Backup;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub fn ensure_backup_dir() -> std::io::Result<PathBuf> {
    let app_dir = get_app_data_dir();
    let backup_dir = app_dir.join("backups");
    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)?;
    }
    Ok(backup_dir)
}

pub fn get_sidecar_path(backup_path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.sha256", backup_path.to_string_lossy()))
}

pub fn find_sidecar_path(backup_path: &Path) -> Option<PathBuf> {
    let sidecar1 = get_sidecar_path(backup_path);
    if sidecar1.exists() {
        return Some(sidecar1);
    }
    let sidecar2 = backup_path.with_extension("sha256");
    if sidecar2.exists() && sidecar2 != backup_path {
        return Some(sidecar2);
    }
    None
}

pub fn compute_sha256(path: &Path) -> Result<String, AppError> {
    let mut file = fs::File::open(path)
        .map_err(|e| AppError::Custom(format!("Failed to open file for hashing {:?}: {}", path, e)))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| AppError::Custom(format!("Failed to read file for hashing {:?}: {}", path, e)))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

pub fn write_sha256_sidecar(backup_path: &Path, hash: &str) -> Result<PathBuf, AppError> {
    let sidecar_path = get_sidecar_path(backup_path);
    let filename = backup_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "backup.db".to_string());
    let content = format!("{}  {}\n", hash, filename);

    let mut file = fs::File::create(&sidecar_path)
        .map_err(|e| AppError::Custom(format!("Failed to create sidecar file {:?}: {}", sidecar_path, e)))?;
    file.write_all(content.as_bytes())
        .map_err(|e| AppError::Custom(format!("Failed to write sidecar file {:?}: {}", sidecar_path, e)))?;
    file.sync_all()
        .map_err(|e| AppError::Custom(format!("Failed to fsync sidecar file {:?}: {}", sidecar_path, e)))?;

    Ok(sidecar_path)
}

pub fn verify_sha256_sidecar(backup_path: &Path) -> Result<String, AppError> {
    let sidecar_path = find_sidecar_path(backup_path)
        .ok_or_else(|| AppError::Custom(format!("Missing SHA-256 sidecar file for {:?}", backup_path)))?;

    let sidecar_content = fs::read_to_string(&sidecar_path)
        .map_err(|e| AppError::Custom(format!("Failed to read SHA-256 sidecar {:?}: {}", sidecar_path, e)))?;

    let expected_hash = sidecar_content
        .split_whitespace()
        .next()
        .ok_or_else(|| AppError::Custom(format!("Empty or invalid SHA-256 sidecar {:?}", sidecar_path)))?
        .to_lowercase();

    if expected_hash.len() != 64 || !expected_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Custom(format!(
            "Invalid SHA-256 format in sidecar {:?}: {}",
            sidecar_path, expected_hash
        )));
    }

    let actual_hash = compute_sha256(backup_path)?;
    if expected_hash != actual_hash {
        return Err(AppError::Custom(format!(
            "SHA-256 checksum mismatch for {:?}: expected {}, found {}",
            backup_path, expected_hash, actual_hash
        )));
    }

    Ok(actual_hash)
}

#[tracing::instrument(level = "info")]
pub fn perform_backup() -> Result<PathBuf, AppError> {
    let limit = crate::settings::load_settings()
        .map(|s| s.backup_retention_count)
        .unwrap_or(5);
    perform_backup_with_retention(limit)
}

#[tracing::instrument(level = "info")]
pub fn perform_backup_with_retention(retention_limit: usize) -> Result<PathBuf, AppError> {
    let backup_dir = ensure_backup_dir()
        .map_err(|e| AppError::Custom(format!("Failed to create backups directory: {}", e)))?;

    // Check disk space (require at least 10MB free space)
    if let Ok(free_space) = fs3::available_space(&backup_dir) {
        if free_space < 10485760 {
            tracing::warn!("Backup aborted: Less than 10MB of free space available on drive.");
            return Err(AppError::Custom("Insufficient disk space".to_string()));
        }
    }

    // Create timestamped filename: watchmark_YYYY-MM-DD_HH-MM-SS.bak
    let now = Local::now();
    let filename = format!("watchmark_{}.bak", now.format("%Y-%m-%d_%H-%M-%S"));
    let backup_path = backup_dir.join(&filename);

    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path);
    }

    // SQLite Online Backup API
    let conn = get_db_connection()?;
    let mut dst = rusqlite::Connection::open(&backup_path)
        .map_err(|e| AppError::Custom(format!("Failed to open destination backup file: {}", e)))?;

    let backup_op = Backup::new(&conn, &mut dst)
        .map_err(|e| AppError::Custom(format!("Failed to initialize SQLite Online Backup: {}", e)))?;

    backup_op
        .run_to_completion(250, Duration::from_millis(5), None)
        .map_err(|e| AppError::Custom(format!("SQLite Online Backup execution failed: {}", e)))?;

    drop(backup_op);
    drop(dst);
    drop(conn);

    // Flush backup file to disk before hashing
    if let Ok(file) = fs::File::open(&backup_path) {
        let _ = file.sync_all();
    }

    // Compute SHA-256 digest
    let hash = match compute_sha256(&backup_path) {
        Ok(h) => h,
        Err(e) => {
            let _ = fs::remove_file(&backup_path);
            return Err(e);
        }
    };

    // Write .sha256 sidecar file with fsync
    if let Err(e) = write_sha256_sidecar(&backup_path, &hash) {
        let _ = fs::remove_file(&backup_path);
        let sidecar_path = get_sidecar_path(&backup_path);
        if sidecar_path.exists() {
            let _ = fs::remove_file(&sidecar_path);
        }
        return Err(e);
    }

    // Cryptographically verify digest immediately
    if let Err(e) = verify_sha256_sidecar(&backup_path) {
        let _ = fs::remove_file(&backup_path);
        let sidecar_path = get_sidecar_path(&backup_path);
        if sidecar_path.exists() {
            let _ = fs::remove_file(&sidecar_path);
        }
        return Err(e);
    }

    tracing::info!(
        action = "backup",
        path = ?backup_path,
        sha256 = %hash,
        "Backup created and cryptographically verified"
    );

    // Prune old backups using configurable retention
    prune_backups_with_limit(&backup_dir, retention_limit)?;

    Ok(backup_path)
}

#[allow(dead_code)]
pub fn prune_backups(backup_dir: &PathBuf) -> Result<(), AppError> {
    let limit = crate::settings::load_settings()
        .map(|s| s.backup_retention_count)
        .unwrap_or(5);
    prune_backups_with_limit(backup_dir, limit)
}

pub fn prune_backups_with_limit(backup_dir: &Path, limit: usize) -> Result<(), AppError> {
    let effective_limit = std::cmp::max(1, limit);
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

    // Sort by creation time, descending (newest first)
    backups.sort_by(|a, b| b.1.cmp(&a.1));

    // Keep only the most recent `effective_limit` backups
    if backups.len() > effective_limit {
        for (path, _) in backups.into_iter().skip(effective_limit) {
            if let Err(e) = fs::remove_file(&path) {
                tracing::error!("Failed to delete old backup {:?}: {}", path, e);
            }
            // Remove companion .sha256 sidecars
            let sidecar1 = get_sidecar_path(&path);
            if sidecar1.exists() {
                let _ = fs::remove_file(&sidecar1);
            }
            let sidecar2 = path.with_extension("sha256");
            if sidecar2.exists() && sidecar2 != path {
                let _ = fs::remove_file(&sidecar2);
            }
        }
    }

    // Orphaned sidecar cleanup: remove any .sha256 sidecars whose .bak no longer exists
    if let Ok(entries) = fs::read_dir(backup_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                if filename.ends_with(".bak.sha256") {
                    let bak_name = filename.trim_end_matches(".sha256");
                    let bak_path = path.with_file_name(bak_name);
                    if !bak_path.exists() {
                        let _ = fs::remove_file(&path);
                    }
                } else if path.extension().unwrap_or_default() == "sha256" {
                    let bak_path = path.with_extension("bak");
                    if !bak_path.exists() {
                        let _ = fs::remove_file(&path);
                    }
                }
            }
        }
    }

    Ok(())
}
