// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use regex::Regex;
use rusqlite::{params, Connection, Result};
use std::path::Path;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm"];

use std::sync::OnceLock;

static LEADING_BRACKET: OnceLock<Regex> = OnceLock::new();
static TRAILING_BRACKET: OnceLock<Regex> = OnceLock::new();
static METADATA_TAGS: OnceLock<Regex> = OnceLock::new();

pub fn clean_anime_release_tags(name: &str) -> String {
    let leading = LEADING_BRACKET.get_or_init(|| Regex::new(r"^[\[\(][^\]\)]+[\]\)]\s*").unwrap());

    // We remove the file extension from the match area to let trailing brackets apply correctly,
    // though the name passed here does not usually have an extension (from `file_stem()`).
    // But if it does, `$` won't match right before `.mkv`. The string might have an extension if not stripped.
    // In our test it has an extension. We can just use `\s*[\[\(][^\]\)]+[\]\)]` for trailing blocks.
    let trailing = TRAILING_BRACKET.get_or_init(|| Regex::new(r"\s*[\[\(][^\]\)]+[\]\)]").unwrap());
    let metadata = METADATA_TAGS.get_or_init(|| Regex::new(r"(?i)(_v2|_Final)").unwrap());

    let mut cleaned = name.to_string();
    cleaned = leading.replace(&cleaned, "").to_string();

    // In order to only replace trailing tags without taking out things in the middle
    // Since `file_stem` removes the extension usually, let's just replace all brackets
    // that look like typical anime noise at the end.
    cleaned = trailing.replace_all(&cleaned, "").to_string();

    cleaned = metadata.replace_all(&cleaned, "").to_string();
    cleaned.trim().to_string()
}

pub fn parse_filename(filename: &str) -> (Option<String>, Option<i32>, Option<i32>) {
    let base_name = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    let base_name_cleaned = clean_anime_release_tags(base_name);

    // Try TV show format first: S01E01
    let pattern_tv = Regex::new(r"^(.*?)[ \.\-_]*[sS](\d{1,2})[ \.\-_]*[eE](\d{1,3})").unwrap();
    if let Some(caps) = pattern_tv.captures(&base_name_cleaned) {
        let raw_series = caps.get(1).map_or("", |m| m.as_str());
        let season_num = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        let episode_num = caps.get(3).and_then(|m| m.as_str().parse::<i32>().ok());

        let mut series_name = raw_series.replace(&['.', '_'][..], " ").trim().to_string();
        let year_regex = Regex::new(r" (19|20)\d{2}$").unwrap();
        series_name = year_regex.replace(&series_name, "").trim().to_string();

        if !series_name.is_empty() {
            return (Some(series_name), season_num, episode_num);
        }
    }

    // Try Movie format: Title (Year) or just Title.Year
    let pattern_movie = Regex::new(r"^(.*?)[ \.\-_\[\(]*(19\d{2}|20\d{2})[\]\)]*").unwrap();
    if let Some(caps) = pattern_movie.captures(&base_name_cleaned) {
        let raw_movie = caps.get(1).map_or("", |m| m.as_str());
        let movie_name = raw_movie.replace(&['.', '_'][..], " ").trim().to_string();
        if !movie_name.is_empty() {
            return (Some(movie_name), None, None);
        }
    }

    // Anime Absolute Episode Number Fallback
    // Format: [Group] Show Name - 01 [Hash] -> Already cleaned to "Show Name - 01"
    let pattern_anime = Regex::new(r"^(.*?)[ \.\-_]+(\d{1,4})$").unwrap();
    if let Some(caps) = pattern_anime.captures(&base_name_cleaned) {
        let raw_series = caps.get(1).map_or("", |m| m.as_str());
        let episode_num = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        let series_name = raw_series.replace(&['.', '_'][..], " ").trim().to_string();

        if !series_name.is_empty() {
            return (Some(series_name), Some(1), episode_num);
        }
    }

    (None, None, None)
}

pub fn is_too_generic(name: &str) -> bool {
    let lower = name.to_lowercase();
    let lower_trim = lower.trim();

    // Remove all numbers to see if it's strictly one of the generic words
    let alpha_only: String = lower_trim.chars().filter(|c| c.is_alphabetic()).collect();

    if !lower.chars().any(|c| c.is_alphabetic()) {
        return true;
    }
    if ["episode", "ep", "part", "s"].contains(&alpha_only.as_str()) {
        return true;
    }
    false
}

pub fn get_parent_directory_name(path: &Path) -> Option<String> {
    if let Some(parent) = path.parent() {
        if let Some(file_name) = parent.file_name() {
            if let Some(name_str) = file_name.to_str() {
                let name = name_str.to_string();
                if is_too_generic(&name) {
                    return get_parent_directory_name(parent);
                }
                return Some(name);
            }
        }
    }
    None
}

#[derive(Clone, serde::Serialize)]
struct MatchBatchPayload {
    files: Vec<serde_json::Value>,
}

pub fn scan_directory(
    directory: &str,
    conn: &mut Connection,
    app_handle: &AppHandle,
) -> Result<i32> {
    tracing::info!("[BACKEND] 🔍 Scanning root directory... ");
    let mut new_unmatched_count = 0;
    let mut batch = Vec::new();
    let mut unmatched_insert_buffer = Vec::new();

    let tx = conn.transaction()?;

    // Canonicalize path safely using dunce
    let sanitized_dir = match dunce::canonicalize(directory) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to canonicalize directory path: {}", e);
            return Err(rusqlite::Error::InvalidPath(std::path::PathBuf::from(directory)));
        }
    };

    // Windows Long Path Support: Ensure the root directory uses \\?\ prefix if absolute
    let root_path = std::path::PathBuf::from(sanitized_dir);

    #[allow(unused_mut)]
    let mut scan_path = root_path.clone();

    #[cfg(windows)]
    {
        if root_path.is_absolute() {
            let path_str = root_path.to_string_lossy();
            if !path_str.starts_with(r"\\?\") {
                scan_path = std::path::PathBuf::from(format!(r"\\?\{}", path_str));
            }
        }
    }

    for entry in WalkDir::new(scan_path).into_iter().filter_map(|e| {
        match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                if let Some(io_err) = err.io_error() {
                    if io_err.kind() == std::io::ErrorKind::PermissionDenied {
                        tracing::warn!("Scanner skipped path due to PermissionDenied: {}", err);
                    } else {
                        tracing::error!("Scanner encountered IO error: {}", err);
                    }
                }
                None
            }
        }
    }) {
        let path = entry.path();

        // Timeout-wrapped metadata read (for network drives)
        let (is_file, file_size) = {
            let p = path.to_path_buf();
            let p_clone = p.clone();
            let (tx_meta, rx_meta) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let meta = std::fs::metadata(&p_clone);
                let _ = tx_meta.send(meta);
            });
            match rx_meta.recv_timeout(std::time::Duration::from_secs(2)) {
                Ok(Ok(m)) => (m.is_file(), m.len() as i64),
                Ok(Err(e)) => {
                    tracing::warn!("Failed to read metadata for {}: {}", p.display(), e);
                    continue; // Skip this file
                }
                Err(_) => {
                    return Err(rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_IOERR),
                        Some("Drive Disconnected or timed out during scan".to_string()),
                    ));
                }
            }
        };

        if is_file {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    let (series_name, season_num, episode_num) = parse_filename(&filename);
                    let str_path = path.to_string_lossy().to_string();

                    if file_size == 0 {
                        tracing::warn!("Bit-Rot or Empty File Detected: {}", str_path);
                    }

                    let mut matched_ep_id: Option<i32> = None;

                    if let Some(ref s_name) = series_name {
                        let safe_series: String =
                            s_name.chars().filter(|c| c.is_alphanumeric()).collect();
                        let safe_series = safe_series.to_lowercase();

                        if let (Some(s_num), Some(e_num)) = (season_num, episode_num) {
                            // TV Show Match
                            let mut stmt =
                                tx.prepare("SELECT id, title FROM Media WHERE type='TV'")?;
                            let shows = stmt.query_map([], |row| {
                                Ok((
                                    row.get::<_, i32>(0)?,
                                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                                ))
                            })?;

                            let mut matched_media_id = None;
                            for (id, title) in shows.flatten() {
                                let safe_db_name: String = title
                                    .chars()
                                    .filter(|c| c.is_alphanumeric())
                                    .collect::<String>()
                                    .to_lowercase();

                                if safe_series == safe_db_name
                                    || safe_db_name.contains(&safe_series)
                                    || safe_series.contains(&safe_db_name)
                                {
                                    matched_media_id = Some(id);
                                    break;
                                }
                            }

                            if let Some(m_id) = matched_media_id {
                                let mut ep_stmt = tx.prepare(
                                    "SELECT id FROM Episodes WHERE media_id = ? AND season_num = ? AND ep_num = ?"
                                )?;
                                let mut rows = ep_stmt.query(params![m_id, s_num, e_num])?;
                                if let Ok(Some(row)) = rows.next() {
                                    matched_ep_id = Some(row.get(0)?);
                                }
                            }
                        } else {
                            // Movie Match
                            let mut stmt =
                                tx.prepare("SELECT id, title FROM Media WHERE type='Movie'")?;
                            let movies = stmt.query_map([], |row| {
                                Ok((
                                    row.get::<_, i32>(0)?,
                                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                                ))
                            })?;

                            let mut matched_media_id = None;
                            for (id, title) in movies.flatten() {
                                let safe_db_name: String = title
                                    .chars()
                                    .filter(|c| c.is_alphanumeric())
                                    .collect::<String>()
                                    .to_lowercase();

                                if safe_series == safe_db_name {
                                    matched_media_id = Some(id);
                                    break;
                                }
                            }

                            if let Some(m_id) = matched_media_id {
                                let mut ep_stmt = tx.prepare(
                                    "SELECT id FROM Episodes WHERE media_id = ? AND season_num = 1 AND ep_num = 1"
                                )?;
                                let mut rows = ep_stmt.query(params![m_id])?;
                                if let Ok(Some(row)) = rows.next() {
                                    matched_ep_id = Some(row.get(0)?);
                                }
                            }
                        }
                    }

                    if let Some(ep_id) = matched_ep_id {
                        // Collision Detection: Automatic Heuristic (Larger Wins)
                        let mut existing_size: i64 = -1;
                        let mut update_needed = true;

                        if let Ok(mut stmt) = tx.prepare("SELECT file_size FROM Local_Files WHERE episode_id = ?") {
                            if let Ok(mut rows) = stmt.query(params![ep_id]) {
                                if let Ok(Some(row)) = rows.next() {
                                    existing_size = row.get(0).unwrap_or(0);
                                } else {
                                    // Row doesn't exist, we must insert
                                }
                            }
                        }

                        if existing_size != -1 {
                            if file_size <= existing_size {
                                update_needed = false;
                            } else {
                                tracing::info!("Auto-replaced episode_id {} with larger file: {} ({} bytes > {} bytes)", ep_id, str_path, file_size, existing_size);
                            }
                        }

                        if update_needed {
                            let _ = tx.execute(
                                "INSERT INTO Local_Files (episode_id, file_path, file_size) VALUES (?, ?, ?) ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path, file_size=excluded.file_size",
                                params![ep_id, &str_path, file_size],
                            );

                            // Auto-Migration: If it's matched mid-scan, ensure we wipe it from Unmatched_Files so it doesn't stay in Inbox
                            let _ = tx.execute("DELETE FROM Unmatched_Files WHERE file_path = ?", params![&str_path]);
                        }
                    } else {
                        // We must first ensure it doesn't already exist as a mapped path in Local_Files
                        // Wait, if it exists in Local_Files, it's already linked. We only insert to Unmatched if NOT in Local_Files.
                        let mut exists_in_local = false;
                        if let Ok(mut stmt) = tx.prepare("SELECT 1 FROM Local_Files WHERE file_path = ?") {
                            if let Ok(mut rows) = stmt.query(params![str_path]) {
                                if let Ok(Some(_)) = rows.next() {
                                    exists_in_local = true;
                                }
                            }
                        }

                        if !exists_in_local {
                            let mut group_key = series_name.clone();
                            if group_key.is_none() {
                                let stem = path.file_stem().unwrap().to_string_lossy();
                                // Fallback regex without unsupported lookaheads
                                let fallback_match =
                                    Regex::new(r"^(.+?)(\.[sS]\d\d|\.\d{4})").unwrap();
                                if let Some(caps) = fallback_match.captures(&stem) {
                                    group_key = Some(caps.get(1).unwrap().as_str().to_string());
                                } else {
                                    group_key = Some(stem.to_string());
                                }
                            }

                            // If group_key is too generic, fallback to parent directory
                            if let Some(ref gk) = group_key {
                                if is_too_generic(gk) {
                                    if let Some(parent_name) = get_parent_directory_name(path) {
                                        group_key = Some(parent_name);
                                    }
                                }
                            }

                            let clean_group_key =
                                group_key.map(|k| clean_anime_release_tags(&k).replace(&['.', '_'][..], " ").trim().to_lowercase());

                            let group_key_clone = clean_group_key.clone();
                            unmatched_insert_buffer.push((
                                str_path.clone(),
                                filename.clone(),
                                series_name.clone(),
                                season_num,
                                episode_num,
                                clean_group_key,
                                group_key_clone,
                            ));

                            // Process in chunks of 500
                            if unmatched_insert_buffer.len() >= 500 {
                                let mut insert_stmt = tx.prepare_cached(
                                    "INSERT OR IGNORE INTO Unmatched_Files (file_path, filename, parsed_series, parsed_season, parsed_episode, group_key) VALUES (?, ?, ?, ?, ?, ?)"
                                )?;

                                for (path, fname, series, s_num, e_num, clean_key, group_clone) in unmatched_insert_buffer.drain(..) {
                                    let res = insert_stmt.execute(params![
                                        &path,
                                        &fname,
                                        &series,
                                        &s_num,
                                        &e_num,
                                        &clean_key
                                    ]);

                                    if res.is_ok() && res.unwrap() > 0 {
                                        new_unmatched_count += 1;
                                        batch.push(serde_json::json!({
                                            "file_path": path,
                                            "filename": fname,
                                            "parsed_series": series,
                                            "parsed_season": s_num,
                                            "parsed_episode": e_num,
                                            "group_key": group_clone
                                        }));
                                    }
                                }

                                if batch.len() >= 50 {
                                    let _ = app_handle.emit(
                                        "scan-match-batch",
                                        MatchBatchPayload {
                                            files: batch.clone(),
                                        },
                                    );
                                    batch.clear();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Flush remaining buffer
    if !unmatched_insert_buffer.is_empty() {
        let mut insert_stmt = tx.prepare_cached(
            "INSERT OR IGNORE INTO Unmatched_Files (file_path, filename, parsed_series, parsed_season, parsed_episode, group_key) VALUES (?, ?, ?, ?, ?, ?)"
        )?;

        for (path, fname, series, s_num, e_num, clean_key, group_clone) in unmatched_insert_buffer.drain(..) {
            let res = insert_stmt.execute(params![
                &path,
                &fname,
                &series,
                &s_num,
                &e_num,
                &clean_key
            ]);

            if res.is_ok() && res.unwrap() > 0 {
                new_unmatched_count += 1;
                batch.push(serde_json::json!({
                    "file_path": path,
                    "filename": fname,
                    "parsed_series": series,
                    "parsed_season": s_num,
                    "parsed_episode": e_num,
                    "group_key": group_clone
                }));
            }
        }
    }

    if !batch.is_empty() {
        let _ = app_handle.emit("scan-match-batch", MatchBatchPayload { files: batch });
    }

    tx.commit()?;
    tracing::info!("[BACKEND] 🧠 Regex engine finished parsing. Found {} unmatched files.", new_unmatched_count);
    Ok(new_unmatched_count)
}
