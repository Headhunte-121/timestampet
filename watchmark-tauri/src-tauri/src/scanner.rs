use regex::Regex;
use rusqlite::{params, Connection, Result};
use std::path::Path;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm"];

pub fn parse_filename(filename: &str) -> (Option<String>, Option<i32>, Option<i32>) {
    let base_name = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    // Try TV show format first: S01E01
    let pattern_tv = Regex::new(r"^(.*?)[ \.\-_]*[sS](\d{1,2})[ \.\-_]*[eE](\d{1,3})").unwrap();
    if let Some(caps) = pattern_tv.captures(base_name) {
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
    if let Some(caps) = pattern_movie.captures(base_name) {
        let raw_movie = caps.get(1).map_or("", |m| m.as_str());
        let movie_name = raw_movie.replace(&['.', '_'][..], " ").trim().to_string();
        if !movie_name.is_empty() {
            return (Some(movie_name), None, None);
        }
    }

    (None, None, None)
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
    let mut new_unmatched_count = 0;
    let mut batch = Vec::new();

    let tx = conn.transaction()?;

    // Canonicalize path safely using dunce
    let sanitized_dir = match dunce::canonicalize(directory) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to canonicalize directory path: {}", e);
            return Err(rusqlite::Error::InvalidPath(std::path::PathBuf::from(directory)));
        }
    };

    // Windows Long Path Support: Ensure the root directory uses \\?\ prefix if absolute
    let root_path = std::path::PathBuf::from(sanitized_dir);
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
                        log::warn!("Scanner skipped path due to PermissionDenied: {}", err);
                    } else {
                        log::error!("Scanner encountered IO error: {}", err);
                    }
                }
                None
            }
        }
    }) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if VIDEO_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    let (series_name, season_num, episode_num) = parse_filename(&filename);
                    let str_path = path.to_string_lossy().to_string();

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
                            for show in shows {
                                if let Ok((id, title)) = show {
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
                            for movie in movies {
                                if let Ok((id, title)) = movie {
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
                        let _ = tx.execute(
                            "INSERT INTO Local_Files (episode_id, file_path) VALUES (?, ?) ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path",
                            params![ep_id, str_path],
                        );
                    } else {
                        let mut group_key = series_name.clone();
                        if group_key.is_none() {
                            let stem = path.file_stem().unwrap().to_string_lossy();
                            let fallback_match =
                                Regex::new(r"^(.+?)(?=\.[sS]\d\d|\.\d{4})").unwrap();
                            if let Some(caps) = fallback_match.captures(&stem) {
                                group_key = Some(caps.get(1).unwrap().as_str().to_string());
                            } else {
                                group_key = Some(stem.to_string());
                            }
                        }

                        let clean_group_key =
                            group_key.map(|k| k.replace(&['.', '_'][..], " ").trim().to_string());

                        let group_key_clone = clean_group_key.clone();
                        let res = tx.execute(
                            "INSERT INTO Unmatched_Files (file_path, filename, parsed_series, parsed_season, parsed_episode, group_key) VALUES (?, ?, ?, ?, ?, ?)",
                            params![
                                &str_path,
                                &filename,
                                &series_name,
                                &season_num,
                                &episode_num,
                                &clean_group_key
                            ],
                        );
                        if res.is_ok() {
                            new_unmatched_count += 1;
                            batch.push(serde_json::json!({
                                "file_path": str_path,
                                "filename": filename,
                                "parsed_series": series_name,
                                "parsed_season": season_num,
                                "parsed_episode": episode_num,
                                "group_key": group_key_clone
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

    if !batch.is_empty() {
        let _ = app_handle.emit("scan-match-batch", MatchBatchPayload { files: batch });
    }

    tx.commit()?;
    Ok(new_unmatched_count)
}
