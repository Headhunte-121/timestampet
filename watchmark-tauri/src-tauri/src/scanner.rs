// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use regex::Regex;
use rusqlite::{params, Connection, Result};
use std::path::Path;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

use std::sync::OnceLock;

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION};

static LEADING_BRACKET: OnceLock<Regex> = OnceLock::new();
static TRAILING_BRACKET: OnceLock<Regex> = OnceLock::new();
static METADATA_TAGS: OnceLock<Regex> = OnceLock::new();
static JUNK_DICTIONARY: OnceLock<Regex> = OnceLock::new();
static PROTECTED_DOTS: OnceLock<Regex> = OnceLock::new();
static TRAILING_VACUUM: OnceLock<Regex> = OnceLock::new();
static MULTI_SPACE: OnceLock<Regex> = OnceLock::new();

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

pub fn clean_string(name: &str) -> String {
    let mut cleaned = name.to_string();

    // 1. Technical Token Stripping (Codecs & Resolutions)
    let junk_regex = JUNK_DICTIONARY.get_or_init(|| {
        Regex::new(r"(?i)\b(x264|x265|hevc|aac|dts|bluray|webrip|720p|1080p|4k|2160p|1080i|2160i)\b").unwrap()
    });
    cleaned = junk_regex.replace_all(&cleaned, "").to_string();

    // 2. Delimiter Normalization (Expansion Protection)
    // Protect dots surrounded by letters (e.g. "Mr. Robot" or "S.H.I.E.L.D.")
    let protected_dots_regex = PROTECTED_DOTS.get_or_init(|| {
        Regex::new(r"([a-zA-Z])\.([a-zA-Z ]|$)").unwrap()
    });

    let mut old = String::new();
    while old != cleaned {
        old = cleaned.clone();
        cleaned = protected_dots_regex.replace_all(&cleaned, "${1}%%DOT%%${2}").to_string();
    }

    cleaned = cleaned.replace('.', " ").replace('_', " ");
    cleaned = cleaned.replace("%%DOT%%", ".");

    // 3. Geometry Trimming
    // Vacuum: strips any trailing hyphens, brackets, or multiple consecutive spaces.
    let trailing_vacuum = TRAILING_VACUUM.get_or_init(|| {
        Regex::new(r"[\-\[\]\(\)\s]+$").unwrap()
    });
    cleaned = trailing_vacuum.replace_all(&cleaned, "").to_string();

    let multi_space = MULTI_SPACE.get_or_init(|| {
        Regex::new(r"\s{2,}").unwrap()
    });
    cleaned = multi_space.replace_all(&cleaned, " ").to_string();

    cleaned.trim().to_string()
}

pub fn parse_filename(filename: &str) -> (Option<String>, Option<i32>, Vec<i32>) {
    let base_name = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    let base_name_cleaned = clean_anime_release_tags(base_name);

    // Try standard TV format first: S01E01, S1E5, S01E01-E02
    let pattern_tv = Regex::new(r"^(.*?)[ \.\-_]+(?i:s)(\d{1,2})[ \.\-_]*(?i:e)(\d{1,3})(?:[ \.\-_]*(?:-|(?i:e)|(?i:ep))(\d{1,3}))?(?:[ \.\-_\[\(].*)?$").unwrap();
    if let Some(caps) = pattern_tv.captures(&base_name_cleaned) {
        let raw_series = caps.get(1).map_or("", |m| m.as_str());
        let season_num = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        let episode_num = caps.get(3).and_then(|m| m.as_str().parse::<i32>().ok());
        let episode_range_end = caps.get(4).and_then(|m| m.as_str().parse::<i32>().ok());

        let mut episodes = Vec::new();
        if let Some(start) = episode_num {
            episodes.push(start);
            if let Some(end) = episode_range_end {
                if end > start && end <= start + 100 {
                    for e in (start + 1)..=end {
                        episodes.push(e);
                    }
                }
            }
        }

        let series_name = clean_string(raw_series);
        if !series_name.is_empty() {
            return (Some(series_name), season_num, episodes);
        }
    }

    // Try Anime bracketed format: [01][105] or [S01E05]
    let pattern_bracketed = Regex::new(r"^(.*?)\[(?i:s)?(\d{1,2})\]\[(?i:e)?(\d{1,3})\](?:[ \.\-_\[\(].*)?$").unwrap();
    if let Some(caps) = pattern_bracketed.captures(&base_name_cleaned) {
        let raw_series = caps.get(1).map_or("", |m| m.as_str());
        let season_num = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        let episode_num = caps.get(3).and_then(|m| m.as_str().parse::<i32>().ok());
        let mut episodes = Vec::new();
        if let Some(start) = episode_num {
            episodes.push(start);
        }

        let series_name = clean_string(raw_series);
        if !series_name.is_empty() {
            return (Some(series_name), season_num, episodes);
        }
    }

    // Natural Language: Season X Episode Y
    let pattern_natural = Regex::new(r"(?i)^(.*?)[ \.\-_]*season[ \.\-_]*(\d{1,2})[ \.\-_]*episode[ \.\-_]*(\d{1,3})(?:[ \.\-_\[\(].*)?$").unwrap();
    if let Some(caps) = pattern_natural.captures(&base_name_cleaned) {
        let raw_series = caps.get(1).map_or("", |m| m.as_str());
        let season_num = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        let episode_num = caps.get(3).and_then(|m| m.as_str().parse::<i32>().ok());
        let mut episodes = Vec::new();
        if let Some(start) = episode_num {
            episodes.push(start);
        }

        let series_name = clean_string(raw_series);
        if !series_name.is_empty() {
            return (Some(series_name), season_num, episodes);
        }
    }

    // Try Movie format: Title (Year) or just Title.Year
    // Match greedy so we get the LAST 4-digit year.
    // It captures group 1 as the title and group 2 as the year (from 1888 to ~2029).
    let pattern_movie = Regex::new(r"^(.*)[\.\s\(\[]+(18[8-9]\d|19\d{2}|20[0-2]\d)[\)\]]*.*$").unwrap();
    if let Some(caps) = pattern_movie.captures(&base_name_cleaned) {
        let raw_movie = caps.get(1).map_or("", |m| m.as_str());
        // Do string cleaning on the extracted title to remove any remaining resolution tags, dots, etc.
        let movie_name = clean_string(raw_movie);
        if !movie_name.is_empty() {
            return (Some(movie_name), None, vec![]);
        }
    }

    // Year-based fallback (2023.10.05) or (2023-10-05) - Date-Pattern Fallback
    // Return empty episodes. Mapped to Inbox Triage.
    let pattern_date = Regex::new(r"^(.*?)[ \.\-_\[\(]+(\d{4})[ \.\-_](\d{2})[ \.\-_](\d{2})(?:[ \.\-_\[\(].*)?$").unwrap();
    if let Some(caps) = pattern_date.captures(&base_name_cleaned) {
        let raw_series = caps.get(1).map_or("", |m| m.as_str());
        let series_name = clean_string(raw_series);
        if !series_name.is_empty() {
            return (Some(series_name), None, vec![]);
        }
    }

    // Absolute episode numbering
    let pattern_absolute = Regex::new(r"^(.*?)[ \.\-_]+(\d{3})(?:[ \.\-_\[\(].*)?$").unwrap();
    if let Some(caps) = pattern_absolute.captures(&base_name_cleaned) {
        let raw_series = caps.get(1).map_or("", |m| m.as_str());
        let episode_num = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        let mut episodes = Vec::new();
        if let Some(start) = episode_num {
            episodes.push(start);
        }

        let series_name = clean_string(raw_series);
        if !series_name.is_empty() {
            // Flag for Inbox Triage by returning None for Season
            return (Some(series_name), None, episodes);
        }
    }

    (None, None, vec![])
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

pub fn normalize_path(path: &Path) -> String {
    #[allow(unused_mut)]
    let mut s = path.to_string_lossy().to_string();
    #[cfg(windows)]
    {
        if s.starts_with(r"\\?\") {
            s = s[4..].to_string();
        }
        s = s.to_lowercase();
    }
    s
}

pub fn scan_directory(
    directory: &str,
    conn: &mut Connection,
    app_handle: &AppHandle,
    cancel_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pause_flag: std::sync::Arc<std::sync::atomic::AtomicBool>,
    supported_extensions: &[String],
) -> Result<i32> {
    tracing::info!("[BACKEND] 🔍 Scanning root directory... ");
    let mut new_unmatched_count = 0;
    let mut batch = Vec::new();
    let mut unmatched_insert_buffer = Vec::new();

    let tx = conn.transaction()?;

    // Pre-check cache for path collision detection (O(1) skipping)
    let mut existing_paths = std::collections::HashSet::new();
    if let Ok(mut stmt) = tx.prepare("SELECT file_path FROM Local_Files") {
        if let Ok(mut rows) = stmt.query([]) {
            while let Ok(Some(row)) = rows.next() {
                let p: String = row.get(0).unwrap_or_default();
                existing_paths.insert(p);
            }
        }
    }
    if let Ok(mut stmt) = tx.prepare("SELECT file_path FROM Unmatched_Files") {
        if let Ok(mut rows) = stmt.query([]) {
            while let Ok(Some(row)) = rows.next() {
                let p: String = row.get(0).unwrap_or_default();
                existing_paths.insert(p);
            }
        }
    }

    // Migration cache for file renames
    let mut migration_cache: std::collections::HashMap<(i64, String, Option<String>), i32> = std::collections::HashMap::new();
    if let Ok(mut stmt) = tx.prepare("SELECT id, file_size, file_path FROM Local_Files") {
        if let Ok(mut rows) = stmt.query([]) {
            while let Ok(Some(row)) = rows.next() {
                let id: i32 = row.get(0).unwrap_or(0);
                let size: i64 = row.get(1).unwrap_or(0);
                let path_str: String = row.get(2).unwrap_or_default();
                let p = Path::new(&path_str);
                if !p.exists() {
                    let filename = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                    let parent = p.parent().map(|p| normalize_path(p));
                    migration_cache.insert((size, filename, parent), id);
                }
            }
        }
    }

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

    let mut visited_inodes = std::collections::HashSet::new();

    for entry in WalkDir::new(scan_path).max_depth(15).follow_links(true).into_iter().filter_map(|e| {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            tracing::info!("[BACKEND] 🛑 Scanner gracefully halted via Cancel signal.");
            return None;
        }

        while pause_flag.load(std::sync::atomic::Ordering::SeqCst) {
            if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                tracing::info!("[BACKEND] 🛑 Scanner gracefully halted via Cancel signal during Pause.");
                return None;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

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
        let mut path = entry.path().to_path_buf();

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.to_lowercase() == "lnk" {
                if let Ok(shortcut) = parselnk::Lnk::try_from(path.as_path()) {
                    if let Some(target) = shortcut.relative_path() {
                        let target_path = path.parent().unwrap_or(Path::new("")).join(target);
                        if target_path.exists() {
                            path = target_path;
                        } else {
                            // If relative path fails or doesn't exist, we might try to fall back
                            // but for simplicity, if it's broken, canonicalize below will catch it.
                            path = target_path;
                        }
                    } else {
                        let link_info = &shortcut.link_info;
                        if let Some(local_base) = &link_info.local_base_path {
                            let target_path = Path::new(local_base).to_path_buf();
                            if target_path.exists() {
                                path = target_path;
                            }
                        }
                    }
                } else {
                    continue;
                }
            }
        }

        // Final canonicalization to get terminal file path for DB storage
        let path = match std::fs::canonicalize(&path) {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!("Broken link or missing target, skipping: {}", path.display());
                continue;
            }
        };

        let normalized_path = normalize_path(&path);

        if existing_paths.contains(&normalized_path) {
            continue; // O(1) skipping for existing items
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = std::fs::metadata(&path) {
                let id = (meta.dev(), meta.ino());
                if !visited_inodes.insert(id) {
                    tracing::warn!("Cyclic Link Abort detected, skipping: {}", path.display());
                    continue;
                }
            }
        }
        #[cfg(windows)]
        {
            if let Ok(file) = std::fs::File::open(&path) {
                let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
                if unsafe { GetFileInformationByHandle(file.as_raw_handle() as isize, &mut info) } != 0 {
                    let file_index = (info.nFileIndexHigh as u64) << 32 | (info.nFileIndexLow as u64);
                    let id = (info.dwVolumeSerialNumber, file_index);
                    if !visited_inodes.insert(id) {
                        tracing::warn!("Cyclic Link Abort detected, skipping: {}", path.display());
                        continue;
                    }
                }
            }
        }

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
            let file_name_str = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let is_hidden = file_name_str.starts_with('.') ||
                ["thumbs.db", "desktop.ini", ".ds_store"].contains(&file_name_str.to_lowercase().as_str());

            #[cfg(windows)]
            let is_hidden = is_hidden || {
                use std::os::windows::fs::MetadataExt;
                if let Ok(meta) = std::fs::metadata(&path) {
                    (meta.file_attributes() & 2) != 0 // FILE_ATTRIBUTE_HIDDEN
                } else {
                    false
                }
            };

            if is_hidden {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();

                // Exclude common double extensions / archive files
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let stem_lower = stem.to_lowercase();
                    if stem_lower.ends_with(".tar") || stem_lower.ends_with(".part") {
                        continue;
                    }
                }

                if supported_extensions.contains(&ext_lower) {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    let (series_name, season_num, episode_num) = parse_filename(&filename);
                    let str_path = normalized_path.clone();

                    if file_size == 0 {
                        tracing::warn!("Bit-Rot or Empty File Detected: {}", str_path);
                        continue;
                    }

                    // Migration logic
                    if let Some(old_id) = migration_cache.remove(&(file_size, filename.clone(), path.parent().map(|p| normalize_path(p)))) {
                        let _ = tx.execute(
                            "UPDATE Local_Files SET file_path = ? WHERE id = ?",
                            params![&str_path, old_id],
                        );
                        tracing::info!("Migrated missing file to new path: {}", str_path);
                        continue;
                    }

                    // Fast Metadata Read (4KB check)
                    let mut is_corrupt = false;
                    if let Ok(mut file) = std::fs::File::open(&path) {
                        use std::io::Read;
                        let mut buffer = [0; 4096];
                        if let Err(e) = file.read(&mut buffer) {
                            tracing::warn!("Corrupt File Detected (Failed fast read): {} ({})", str_path, e);
                            is_corrupt = true;
                        }
                    } else {
                        is_corrupt = true;
                    }

                    if is_corrupt {
                        continue;
                    }


                    let mut matched_ep_id: Option<i32> = None;

                    if let Some(ref s_name) = series_name {
                        let safe_series: String =
                            s_name.chars().filter(|c| c.is_alphanumeric()).collect();
                        let safe_series = safe_series.to_lowercase();

                        if let (Some(s_num), episodes) = (season_num, &episode_num) {
                            if !episodes.is_empty() {
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
                                    let mut rows = ep_stmt.query(params![m_id, s_num, episodes[0]])?;
                                    if let Ok(Some(row)) = rows.next() {
                                        matched_ep_id = Some(row.get(0)?);
                                    }
                                }
                            } else {
                                // Fallback for episode list empty but season present (should not happen with our parser)
                                // We can just fall through to the movie matching block.
                            }
                        }

                        if matched_ep_id.is_none() && season_num.is_none() {
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
                        // For ranges: If we matched S01E01-E02, `episode_num` contains `[1, 2]`.
                        // We should map ALL episodes in `episode_num` to this same file.
                        // First we need to find the `media_id` which matched `ep_id`.
                        let mut m_id: Option<i32> = None;
                        if let Ok(mut stmt) = tx.prepare("SELECT media_id FROM Episodes WHERE id = ?") {
                            if let Ok(mut rows) = stmt.query(params![ep_id]) {
                                if let Ok(Some(row)) = rows.next() {
                                    m_id = Some(row.get(0).unwrap_or(0));
                                }
                            }
                        }

                        let mut matched_ep_ids = Vec::new();
                        matched_ep_ids.push(ep_id);

                        if let Some(m_id) = m_id {
                            if let Some(s_num) = season_num {
                                if episode_num.len() > 1 {
                                    if let Ok(mut ep_stmt) = tx.prepare(
                                        "SELECT id, ep_num FROM Episodes WHERE media_id = ? AND season_num = ?"
                                    ) {
                                        if let Ok(mut rows) = ep_stmt.query(params![m_id, s_num]) {
                                            while let Ok(Some(row)) = rows.next() {
                                                let eid: i32 = row.get(0).unwrap_or(0);
                                                let enum_val: i32 = row.get(1).unwrap_or(0);
                                                if episode_num.contains(&enum_val) && eid != ep_id {
                                                    matched_ep_ids.push(eid);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        for e_id in matched_ep_ids {
                            // Collision Detection: Automatic Heuristic (Larger Wins)
                            let mut existing_size: i64 = -1;
                            let mut update_needed = true;

                            if let Ok(mut stmt) = tx.prepare("SELECT file_size FROM Local_Files WHERE episode_id = ?") {
                                if let Ok(mut rows) = stmt.query(params![e_id]) {
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
                                    tracing::info!("Auto-replaced episode_id {} with larger file: {} ({} bytes > {} bytes)", e_id, str_path, file_size, existing_size);
                                }
                            }

                            if update_needed {
                                let _ = tx.execute(
                                    "INSERT INTO Local_Files (episode_id, file_path, file_size) VALUES (?, ?, ?) ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path, file_size=excluded.file_size",
                                    params![e_id, &str_path, file_size],
                                );

                                // Auto-Migration: If it's matched mid-scan, ensure we wipe it from Unmatched_Files so it doesn't stay in Inbox
                                let _ = tx.execute("DELETE FROM Unmatched_Files WHERE file_path = ?", params![&str_path]);
                            }
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
                                    if let Some(parent_name) = get_parent_directory_name(&path) {
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
                                episode_num.get(0).copied(), // we store the first matched episode number in the inbox
                                clean_group_key,
                                group_key_clone,
                            ));

                            // Process in chunks of 100 for 10,000+ files low memory footprint
                            if unmatched_insert_buffer.len() >= 100 {
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
