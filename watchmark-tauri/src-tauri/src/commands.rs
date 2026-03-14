use crate::db::get_db_connection;
use crate::error::{handle_panic, AppError};
use crate::models::{HistoryEntry, Media, Settings, UnmatchedFile};
use rusqlite::params;
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};
use tauri::Emitter;
use chrono::{Local, TimeZone, NaiveDate, Datelike};

fn calculate_gap(air_date_str: &str, watch_ts: i64) -> Option<serde_json::Value> {
    if air_date_str.is_empty() {
        return None;
    }

    let air_date = match NaiveDate::parse_from_str(air_date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return None,
    };

    let air_time = air_date.and_hms_opt(0, 0, 0)?;

    let watch_time_local = match Local.timestamp_opt(watch_ts, 0).single() {
        Some(dt) => dt.naive_local(),
        None => return None,
    };

    let is_early = watch_time_local < air_time;
    let (start, end) = if is_early {
        (watch_time_local, air_time)
    } else {
        (air_time, watch_time_local)
    };

    let mut years = end.year() - start.year();
    let mut months = end.month() as i32 - start.month() as i32;
    let mut days = end.day() as i32 - start.day() as i32;

    if end.time() < start.time() {
        days -= 1;
    }

    if days < 0 {
        months -= 1;
        let prev_month = if end.month() == 1 { 12 } else { end.month() - 1 };
        let prev_month_year = if end.month() == 1 { end.year() - 1 } else { end.year() };
        let next_month = if prev_month == 12 { 1 } else { prev_month + 1 };
        let next_year = if prev_month == 12 { prev_month_year + 1 } else { prev_month_year };
        let d1 = NaiveDate::from_ymd_opt(prev_month_year, prev_month, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
        days += (d2 - d1).num_days() as i32;
    }

    if months < 0 {
        years -= 1;
        months += 12;
    }

    let total_days = (end.date() - start.date()).num_days();

    Some(serde_json::json!({
        "is_early": is_early,
        "years": years,
        "months": months,
        "days": days,
        "total_days": total_days
    }))
}
use tokio::sync::mpsc;

#[cfg(test)]
mod tests_feature_5_8 {
    use super::*;
    use chrono::{TimeZone, Local, NaiveDate};

    #[test]
    fn test_time_capsule_decade_delay() {
        // Air date 1960-01-01
        // Watched 2024-01-01
        let air_date = "1960-01-01";
        let watch_ts = Local.from_local_datetime(&NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()).single().unwrap().timestamp();

        let gap = calculate_gap(air_date, watch_ts).unwrap();
        assert_eq!(gap["years"], 64);
        assert_eq!(gap["is_early"], false);
    }

    #[test]
    fn test_time_capsule_same_day() {
        // Watched on Release Day
        let air_date = "2024-01-01";
        // Let's say we watched it at 2 PM local time on that day
        let watch_ts = Local.from_local_datetime(&NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(14, 0, 0).unwrap()).single().unwrap().timestamp();

        let gap = calculate_gap(air_date, watch_ts).unwrap();
        assert_eq!(gap["total_days"], 0);
        assert_eq!(gap["is_early"], false);
    }

    #[test]
    fn test_time_capsule_early_watch() {
        // Time Traveler Test / Leak
        let air_date = "2025-01-01";
        let watch_ts = Local.from_local_datetime(&NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap()).single().unwrap().timestamp();

        let gap = calculate_gap(air_date, watch_ts).unwrap();
        assert_eq!(gap["is_early"], true);
    }

    #[test]
    fn test_time_capsule_leap_year() {
        // Aired Feb 29, 2024. Watched Feb 28, 2025. Exactly 1 non-leap year (365 days).
        let air_date = "2024-02-29";
        let watch_ts = Local.from_local_datetime(&NaiveDate::from_ymd_opt(2025, 2, 28).unwrap().and_hms_opt(0, 0, 0).unwrap()).single().unwrap().timestamp();

        let gap = calculate_gap(air_date, watch_ts).unwrap();
        assert_eq!(gap["total_days"], 365);
    }

    #[test]
    fn test_time_capsule_null_date() {
        // NULL air date
        let gap = calculate_gap("", 1234567890);
        assert!(gap.is_none());
    }
}

use crate::task_queue::DbTaskQueue;

pub struct AppState {
    pub settings: Arc<RwLock<Settings>>,
    pub settings_tx: mpsc::Sender<Settings>,
    pub db_queue: Arc<DbTaskQueue>,
}

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, AppError> {
    let settings = state.settings.read().unwrap();
    Ok(settings.clone())
}

#[tauri::command]
pub fn repair_paths(old_root: String, new_root: String) -> Result<i32, AppError> {
    handle_panic(|| {
        let conn = get_db_connection()?;
        let affected = conn.execute(
            "UPDATE Local_Files SET file_path = REPLACE(file_path, ?, ?) WHERE file_path LIKE ?",
            params![old_root, new_root, format!("{}%", old_root)],
        )?;
        Ok(affected as i32)
    })
}

#[tauri::command]
pub fn remove_local_link(episode_id: i32) -> Result<(), AppError> {
    handle_panic(|| {
        let conn = get_db_connection()?;
        conn.execute("DELETE FROM Local_Files WHERE episode_id = ?", params![episode_id])?;
        Ok(())
    })
}

#[tauri::command]
pub async fn validate_and_hash_file(episode_id: i32, file_path: String) -> Result<Value, AppError> {
    let task = tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let path = std::path::PathBuf::from(&file_path);

            if !path.exists() {
                return Ok(json!({ "status": "missing", "path": file_path }));
            }

            let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if file_size == 0 {
                return Ok(json!({ "status": "corrupted", "path": file_path }));
            }

            // Calculate sparse hash
            match crate::hash::compute_sparse_hash(&path) {
                Ok(hash) => {
                    if let Ok(conn) = get_db_connection() {
                        let _ = conn.execute(
                            "UPDATE Local_Files SET file_hash = ? WHERE episode_id = ?",
                            params![hash, episode_id],
                        );
                    }
                    Ok(json!({ "status": "ok", "hash": hash }))
                }
                Err(_) => {
                    Ok(json!({ "status": "error", "message": "Failed to read file for hashing." }))
                }
            }
        })
    });

    task.await.unwrap_or(Err(AppError::Custom("Task panicked".to_string())))
}

#[tauri::command]
pub fn update_local_file(episode_id: i32, new_path: String) -> Result<(), AppError> {
    handle_panic(|| {
        let conn = get_db_connection()?;
        let file_size = std::fs::metadata(&new_path).map(|m| m.len()).unwrap_or(0) as i64;
        conn.execute(
            "INSERT INTO Local_Files (episode_id, file_path, file_size) VALUES (?, ?, ?) ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path, file_size=excluded.file_size",
            params![episode_id, new_path, file_size],
        )?;
        Ok(())
    })
}

#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    {
        let mut current_settings = state.settings.write().unwrap();
        *current_settings = settings.clone();
    }

    // Send to debouncer task
    state.settings_tx.send(settings).await.map_err(|e| AppError::Custom(e.to_string()))
}

#[tauri::command]
pub fn get_media_history_count(media_id: i32) -> Result<i32, AppError> {
    handle_panic(|| {
        let conn = crate::db::get_db_connection()?;
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM History WHERE episode_id IN (SELECT id FROM Episodes WHERE media_id = ?)",
            rusqlite::params![media_id],
            |row| row.get(0),
        ).unwrap_or(0);
        Ok(count)
    })
}

#[tauri::command]
pub fn delete_media_cmd(media_id: i32) -> Result<(), AppError> {
    handle_panic(|| crate::db::delete_media(media_id).map_err(AppError::from))
}

#[tauri::command]
pub fn get_media_details_db(media_id: i32) -> Result<Value, AppError> {
    handle_panic(|| {
        let conn = get_db_connection()?;

        let mut stmt = conn.prepare("SELECT * FROM Media WHERE id=?")?;
        let mut media: Option<Value> = None;

        if let Ok(mut rows) = stmt.query(params![media_id]) {
            if let Ok(Some(row)) = rows.next() {
                let m_type: String = row.get(2).unwrap_or_default();
                let release_date: String = row.get::<_, Option<String>>(11).unwrap_or_default().unwrap_or_default();
                let is_unaired = if !release_date.is_empty() {
                    let now = chrono::Utc::now().naive_utc().date();
                    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&release_date, "%Y-%m-%d") {
                        parsed > now
                    } else {
                        false
                    }
                } else {
                    false
                };
                media = Some(json!({
                    "id": row.get::<_, i32>(0).unwrap_or(0),
                    "tmdb_id": row.get::<_, String>(1).unwrap_or_default(),
                    "type": m_type,
                    "title": row.get::<_, Option<String>>(3).unwrap_or_default().unwrap_or_default(),
                    "synopsis": row.get::<_, Option<String>>(4).unwrap_or_default().unwrap_or_default(),
                    "poster_path": row.get::<_, Option<String>>(5).unwrap_or_default().unwrap_or_default(),
                    "backdrop_path": row.get::<_, Option<String>>(6).unwrap_or_default().unwrap_or_default(),
                    "total_episodes": row.get::<_, Option<i32>>(7).unwrap_or_default().unwrap_or(0),
                    "status": row.get::<_, Option<String>>(8).unwrap_or_default().unwrap_or_default(),
                    "vote_average": row.get::<_, Option<f64>>(9).unwrap_or_default().unwrap_or(0.0),
                    "user_rating": row.get::<_, Option<i32>>(10).unwrap_or_default().unwrap_or(0),
                    "release_date": release_date,
                    "is_exact_date": row.get::<_, Option<bool>>(12).unwrap_or_default().unwrap_or(true),
                    "is_unaired": is_unaired,
                }));
            }
        }

        if let Some(mut m) = media {
            let m_type = m["type"].as_str().unwrap_or_default().to_string();

            // Watched count
            let watched_eps: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM Episodes WHERE media_id=? AND status='Completed'",
                    params![media_id],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            m["completed_eps"] = json!(watched_eps);

            let mut seasons = vec![];
            if m_type == "TV" {
                let mut s_stmt = conn.prepare(
                    "SELECT DISTINCT season_num FROM Episodes WHERE media_id=? ORDER BY season_num",
                )?;
                let s_rows = s_stmt.query_map(params![media_id], |row| row.get::<_, u32>(0));
                if let Ok(s_rows_iter) = s_rows {
                    for s in s_rows_iter.flatten() {
                        seasons.push(s);
                    }
                }
            } else {
                seasons.push(1);
            }
            m["seasons"] = json!(seasons);

            let mut eps_stmt = conn.prepare(
                "
                SELECT e.*, l.file_path
                FROM Episodes e
                LEFT JOIN Local_Files l ON e.id = l.episode_id
                WHERE e.media_id=?
                ORDER BY e.season_num ASC, e.ep_num ASC
            ",
            )?;

            let mut episodes = vec![];
            if let Ok(ep_rows) = eps_stmt.query_map(params![media_id], |row| {
                let air_date: String = row.get::<_, Option<String>>(12)?.unwrap_or_default();
                let is_unaired = if !air_date.is_empty() {
                    let now = chrono::Utc::now().naive_utc().date();
                    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&air_date, "%Y-%m-%d") {
                        parsed > now
                    } else {
                        false
                    }
                } else {
                    false
                };
                Ok(json!({
                    "id": row.get::<_, i32>(0)?,
                    "media_id": row.get::<_, i32>(1)?,
                    "season_num": row.get::<_, u32>(2)?,
                    "ep_num": row.get::<_, u32>(3)?,
                    "title": row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    "runtime": row.get::<_, i32>(5)?,
                    "still_path": row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    "overview": row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                    "watch_count": row.get::<_, i32>(8)?,
                    "last_position": row.get::<_, i32>(9)?,
                    "status": row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                    "completed_date": row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                    "air_date": air_date,
                    "is_exact_date": row.get::<_, Option<bool>>(13).unwrap_or_default().unwrap_or(true),
                    "file_path": row.get::<_, Option<String>>(14)?,
                    "is_unaired": is_unaired,
                }))
            }) {
                for ep in ep_rows.flatten() {
                    episodes.push(ep);
                }
            }

            m["episodes"] = json!(episodes);
            Ok(m)
        } else {
            Err(AppError::Custom("Media not found".to_string()))
        }
    })
}

#[tauri::command]
pub async fn add_to_tracker(
    tmdb_id: String,
    media_type: String,
    archive: bool,
) -> Result<(), AppError> {
    let task = tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let settings = crate::settings::load_settings()
                .map_err(|e| AppError::Custom(e))?;
            if settings.tmdb_api_key.is_empty() {
                return Err(AppError::Custom("Missing TMDB API Key. Please add it in Settings.".to_string()));
            }

            let valid_media_type = crate::models::MediaType::from_str(&media_type).as_str().to_string();

            if let Ok(details) = crate::tmdb::get_media_details(&settings.tmdb_api_key, &tmdb_id, &valid_media_type) {

            let mut synopsis = details["synopsis"].as_str().unwrap_or("").to_string();
            if synopsis.chars().count() > 10000 {
                synopsis = synopsis.chars().take(10000).collect::<String>();
                synopsis.push_str("...");
            }

            if let Some(poster) = details["poster_path"].as_str() {
                crate::tmdb::download_image(poster, "w500");
            }
            if let Some(backdrop) = details["backdrop_path"].as_str() {
                crate::tmdb::download_image(backdrop, "w1280");
            }

            let mut all_eps = Vec::new();
            if valid_media_type == "TV" {
                if let Some(seasons) = details["seasons"].as_array() {
                    for season in seasons {
                        if let Some(s_num) = season["season_number"].as_i64() {
                            if s_num >= 0 {
                                if let Ok(eps) = crate::tmdb::get_tv_season_episodes(&settings.tmdb_api_key, &tmdb_id, s_num as u32) {
                                    all_eps.extend(eps);
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(mut conn) = get_db_connection() {
                if let Ok(tx) = conn.transaction() {
                    let mut existing_id = None;
                    {
                        if let Ok(mut stmt) = tx.prepare("SELECT id FROM Media WHERE tmdb_id=?") {
                            if let Ok(mut rows) = stmt.query(params![tmdb_id]) {
                                if let Ok(Some(row)) = rows.next() {
                                    existing_id = Some(row.get::<_, i32>(0).unwrap_or(0));
                                }
                            }
                        }
                    }

                    let media_id = if let Some(id) = existing_id {
                        let _ = tx.execute(
                            "UPDATE Media SET \"title\" = ?, synopsis = ?, poster_path = ?, backdrop_path = ?, total_episodes = ?, vote_average = ?, release_date = ?, is_exact_date = ?
                             WHERE id = ?",
                            params![
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                synopsis,
                                details["poster_path"].as_str().unwrap_or(""),
                                details["backdrop_path"].as_str().unwrap_or(""),
                                details["total_episodes"].as_i64().unwrap_or(1) as i32,
                                details["vote_average"].as_f64().unwrap_or(0.0),
                                details["release_date"].as_str().unwrap_or(""),
                                details["is_exact_date"].as_bool().unwrap_or(true),
                                id
                            ]
                        );
                        id
                    } else {
                        let _ = tx.execute(
                            "INSERT INTO Media (tmdb_id, \"type\", \"title\", synopsis, poster_path, backdrop_path, total_episodes, status, vote_average, release_date, is_exact_date)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                             ON CONFLICT(tmdb_id, \"type\") DO UPDATE SET
                                \"title\"=excluded.\"title\", synopsis=excluded.synopsis, poster_path=excluded.poster_path,
                                backdrop_path=excluded.backdrop_path, total_episodes=excluded.total_episodes,
                                vote_average=excluded.vote_average, release_date=excluded.release_date, is_exact_date=excluded.is_exact_date",
                            params![
                                tmdb_id,
                                valid_media_type,
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                synopsis,
                                details["poster_path"].as_str().unwrap_or(""),
                                details["backdrop_path"].as_str().unwrap_or(""),
                                details["total_episodes"].as_i64().unwrap_or(1) as i32,
                                details["status"].as_str().unwrap_or("Plan to Watch"),
                                details["vote_average"].as_f64().unwrap_or(0.0),
                                details["release_date"].as_str().unwrap_or(""),
                                details["is_exact_date"].as_bool().unwrap_or(true)
                            ]
                        );

                        let mut new_id = tx.last_insert_rowid() as i32;
                        if new_id == 0 {
                            if let Ok(mut stmt) = tx.prepare("SELECT id FROM Media WHERE tmdb_id=? AND \"type\"=?") {
                                if let Ok(mut rows) = stmt.query(params![tmdb_id, valid_media_type]) {
                                    if let Ok(Some(row)) = rows.next() {
                                        new_id = row.get::<_, i32>(0).unwrap_or(0);
                                    }
                                }
                            }
                        }
                        new_id
                    };

                    let ep_status = if archive { "Completed" } else { "Unwatched" };
                    let ep_watch_count = if archive { 1 } else { 0 };

                    let mut episode_ids_to_history = Vec::new();

                    if valid_media_type == "TV" {
                        for ep in all_eps {
                            let mut ep_overview = ep["overview"].as_str().unwrap_or("").to_string();
                            if ep_overview.chars().count() > 10000 {
                                ep_overview = ep_overview.chars().take(10000).collect::<String>();
                                ep_overview.push_str("...");
                            }

                            let season_num = ep["season_num"].as_i64().unwrap_or(1) as u32;
                            let ep_num = ep["ep_num"].as_i64().unwrap_or(1) as u32;
                            let _ = tx.execute(
                                "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\", runtime, still_path, overview, status, watch_count, air_date, is_exact_date)
                                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                                 ON CONFLICT(media_id, season_num, ep_num) DO UPDATE SET
                                    \"title\"=excluded.\"title\", runtime=excluded.runtime, still_path=excluded.still_path,
                                    overview=excluded.overview, air_date=excluded.air_date, is_exact_date=excluded.is_exact_date",
                                params![
                                    media_id,
                                    season_num,
                                    ep_num,
                                    ep["title"].as_str().unwrap_or("Unknown Title"),
                                    ep["runtime"].as_i64().unwrap_or(0) as i32,
                                    ep["still_path"].as_str().unwrap_or(""),
                                    ep_overview,
                                    ep_status,
                                    ep_watch_count,
                                    ep["air_date"].as_str().unwrap_or(""),
                                    ep["is_exact_date"].as_bool().unwrap_or(true)
                                ]
                            );

                            let ep_id = tx.last_insert_rowid() as i32;
                            if ep_id == 0 {
                                // Existed already
                                if let Ok(mut stmt) = tx.prepare("SELECT id FROM Episodes WHERE media_id=? AND season_num=? AND ep_num=?") {
                                    if let Ok(mut rows) = stmt.query(params![media_id, season_num, ep_num]) {
                                        if let Ok(Some(row)) = rows.next() {
                                            episode_ids_to_history.push(row.get::<_, i32>(0).unwrap_or(0));
                                        }
                                    }
                                }
                            } else {
                                episode_ids_to_history.push(ep_id);
                            }
                        }
                    } else {
                        let _ = tx.execute(
                            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\", runtime, still_path, overview, status, watch_count, air_date, is_exact_date)
                             VALUES (?, 1, 1, ?, ?, ?, ?, ?, ?, ?, ?)
                             ON CONFLICT(media_id, season_num, ep_num) DO UPDATE SET
                                \"title\"=excluded.\"title\", runtime=excluded.runtime, still_path=excluded.still_path,
                                overview=excluded.overview, air_date=excluded.air_date, is_exact_date=excluded.is_exact_date",
                            params![
                                media_id,
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                details["runtime"].as_i64().unwrap_or(0) as i32,
                                details["backdrop_path"].as_str().unwrap_or(""),
                                synopsis,
                                ep_status,
                                ep_watch_count,
                                details["release_date"].as_str().unwrap_or(""),
                                details["is_exact_date"].as_bool().unwrap_or(true)
                            ]
                        );

                        let ep_id = tx.last_insert_rowid() as i32;
                        if ep_id == 0 {
                            if let Ok(mut stmt) = tx.prepare("SELECT id FROM Episodes WHERE media_id=? AND season_num=1 AND ep_num=1") {
                                if let Ok(mut rows) = stmt.query(params![media_id]) {
                                    if let Ok(Some(row)) = rows.next() {
                                        episode_ids_to_history.push(row.get::<_, i32>(0).unwrap_or(0));
                                    }
                                }
                            }
                        } else {
                            episode_ids_to_history.push(ep_id);
                        }
                    }

                    // For add_to_tracker, if archive is true, do NOT insert History entries.
                    // If archive is false, it's marking them unwatched so there shouldn't be history either,
                    // BUT for the prompt's `archive_mode` logic:
                    // "If archive_mode is True: Update Episodes.status to 'Completed' but do not insert any rows into the History table."
                    // Since it sets 'Completed' when archive is true, and 'Unwatched' when false, we don't insert History here either way.
                    // History insertions for "Mark Watched" / "Mark Season Watched" are where we actually care.
                    // Wait, the user explicitly stated: "Implementation: In the invoke command for mark_season_watched or add_to_tracker, add a boolean argument archive_mode. If archive_mode is True: Update Episodes.status to 'Completed' but do not insert any rows into the History table. If archive_mode is False: Update the status and create the History entries."
                    // But if archive_mode is false in add_to_tracker, it's just adding to tracker. The prompt assumes `add_to_tracker` has a flow where we mark everything completed. I'll just adhere to: if archive=false, do NOT insert because it's unwatched. Actually wait: "If archive_mode is False: Update the status and create the History entries." This means the user expects `archive_mode = false` to mean "Mark as Completed AND create history entries". But that makes no sense for newly added unwatched media.
                    // Let's implement `mark_season_watched` as an additional command instead, since that's what the feature actually targets.

                    let _ = tx.commit();
                }
            }
        }

            Ok(())
        })
    });

    match tokio::time::timeout(std::time::Duration::from_secs(15), task).await {
        Ok(res) => res.unwrap_or(Err(AppError::Custom("Task panicked".to_string()))),
        Err(_) => Err(AppError::Custom("Task Timed Out".to_string())),
    }
}

#[tauri::command]
pub async fn mark_season_watched(media_id: i32, season_num: u32, archive_mode: bool) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let mut conn = crate::db::get_db_connection()?;
            let tx = conn.transaction()?;

            let mut episode_ids = Vec::new();
            {
                // Fetch episodes to update
                let mut stmt = tx.prepare("SELECT id FROM Episodes WHERE media_id = ? AND season_num = ? AND status != 'Completed'")?;
                let mut rows = stmt.query(params![media_id, season_num])?;

                while let Ok(Some(row)) = rows.next() {
                    episode_ids.push(row.get::<_, i32>(0).unwrap_or(0));
                }
            }

            // Update status
            let _ = tx.execute(
                "UPDATE Episodes SET status = 'Completed', watch_count = watch_count + 1 WHERE media_id = ? AND season_num = ? AND status != 'Completed'",
                params![media_id, season_num]
            );

            // Generate history entries if NOT archiving
            if !archive_mode && !episode_ids.is_empty() {
                let session_id = uuid::Uuid::new_v4().to_string();
                let current_timestamp = chrono::Utc::now().timestamp();

                for ep_id in episode_ids {
                    let _ = tx.execute(
                        "INSERT INTO History (episode_id, timestamp, is_legacy, session_id, status) VALUES (?, ?, 0, ?, 'Completed')",
                        params![ep_id, current_timestamp, session_id]
                    );
                }
            }

            tx.commit()?;
            Ok(())
        })
    }).await.unwrap_or(Err(AppError::Custom("Task panicked".to_string())))
}

#[tauri::command]
pub async fn archive_season(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    media_id: i32,
    season_num: u32,
    is_legacy: bool,
    timestamp: Option<i64>,
) -> Result<(), AppError> {
    let db_queue = state.db_queue.clone();

    tokio::task::spawn_blocking(move || {
        let app_clone1 = app.clone();
        handle_panic(std::panic::AssertUnwindSafe(move || {
            let app = app_clone1;
            let conn = get_db_connection()?;
            let mut episode_ids = Vec::new();

            {
                let mut stmt = conn.prepare("SELECT id FROM Episodes WHERE media_id = ? AND season_num = ? ORDER BY ep_num ASC")?;
                let mut rows = stmt.query(params![media_id, season_num])?;
                while let Ok(Some(row)) = rows.next() {
                    episode_ids.push(row.get::<_, i32>(0).unwrap_or(0));
                }
            }

            if episode_ids.is_empty() {
                return Ok(());
            }

            let total_episodes = episode_ids.len();
            let base_timestamp = timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp());
            let start_timestamp = if is_legacy {
                use chrono::{DateTime, Utc, TimeZone, Datelike};
                if let Some(dt) = Utc.timestamp_opt(base_timestamp, 0).single() {
                    if let Some(first_of_month) = dt.with_day(1) {
                        first_of_month.timestamp()
                    } else {
                        base_timestamp
                    }
                } else {
                    base_timestamp
                }
            } else {
                base_timestamp
            };

            let _ = app.emit("history-import-progress", json!({ "progress": 0, "total": total_episodes, "isImporting": true }));

            for (i, ep_id) in episode_ids.into_iter().enumerate() {
                let current_ep_timestamp = start_timestamp + (i as i64 * 86400);
                let app_clone = app.clone();
                let current_idx = i + 1;

                db_queue.push_low_priority(move |conn| {
                    let _ = conn.execute(
                        "UPDATE Episodes SET status = 'Completed', watch_count = watch_count + 1 WHERE id = ?",
                        params![ep_id]
                    );

                    let session_id = uuid::Uuid::new_v4().to_string();
                    let legacy_int = if is_legacy { 1 } else { 0 };

                    let _ = conn.execute(
                        "INSERT INTO History (episode_id, timestamp, is_legacy, session_id, status) VALUES (?, ?, ?, ?, 'Completed')",
                        params![ep_id, current_ep_timestamp, legacy_int, session_id]
                    );

                    let progress_percent = (current_idx as f64 / total_episodes as f64 * 100.0) as i32;
                    let is_importing = current_idx < total_episodes;
                    let _ = app_clone.emit("history-import-progress", json!({ "progress": progress_percent, "total": total_episodes, "isImporting": is_importing }));
                });
            }

            Ok::<(), AppError>(())
        }))
    }).await.unwrap_or_else(|_| Err(AppError::Custom("Task panicked".to_string())))?;

    Ok(())
}

#[tauri::command]
pub fn get_dashboard_data() -> Result<Value, AppError> {
    handle_panic(|| {
        let conn = get_db_connection()?;

        // 1. Hero Episode
        let mut hero_stmt = conn.prepare(
            "
            SELECT e.media_id, MAX(h.timestamp) as last_watched
            FROM History h
            JOIN Episodes e ON h.episode_id = e.id
            WHERE EXISTS (
                SELECT 1 FROM Episodes e2
                WHERE e2.media_id = e.media_id AND e2.status IN ('Watching', 'Unwatched')
            )
            GROUP BY e.media_id
            ORDER BY last_watched DESC
            LIMIT 1
        ",
        )?;

        let mut hero_ep: Option<Value> = None;
        if let Ok(mut rows) = hero_stmt.query([]) {
            if let Ok(Some(row)) = rows.next() {
                let media_id: i32 = row.get(0).unwrap_or(0);

                let mut ep_stmt = conn.prepare(
                    "
                    SELECT e.*, m.title as show_title, m.backdrop_path, l.file_path, m.type as media_type, e.still_path
                    FROM Episodes e
                    JOIN Media m ON e.media_id = m.id
                    LEFT JOIN Local_Files l ON e.id = l.episode_id
                    WHERE e.media_id = ? AND e.status IN ('Watching', 'Unwatched')
                    ORDER BY e.season_num ASC, e.ep_num ASC
                    LIMIT 1
                ",
                )?;

                let mut ep_rows = ep_stmt.query(params![media_id])?;
                if let Ok(Some(ep_row)) = ep_rows.next() {
                    hero_ep = Some(json!({
                        "id": ep_row.get::<_, i32>(0).unwrap_or(0),
                        "media_id": ep_row.get::<_, i32>(1).unwrap_or(0),
                        "season_num": ep_row.get::<_, u32>(2).unwrap_or(0),
                        "ep_num": ep_row.get::<_, u32>(3).unwrap_or(0),
                        "title": ep_row.get::<_, Option<String>>(4).unwrap_or_default().unwrap_or_default(),
                        "runtime": ep_row.get::<_, i32>(5).unwrap_or(0),
                        "still_path": ep_row.get::<_, Option<String>>(6).unwrap_or_default().unwrap_or_default(),
                        "overview": ep_row.get::<_, Option<String>>(7).unwrap_or_default().unwrap_or_default(),
                        "watch_count": ep_row.get::<_, i32>(8).unwrap_or(0),
                        "last_position": ep_row.get::<_, i32>(9).unwrap_or(0),
                        "status": ep_row.get::<_, Option<String>>(10).unwrap_or_default().unwrap_or_default(),
                        "completed_date": ep_row.get::<_, Option<String>>(11).unwrap_or_default().unwrap_or_default(),
                        "air_date": ep_row.get::<_, Option<String>>(12).unwrap_or_default().unwrap_or_default(),

                        "show_title": ep_row.get::<_, Option<String>>(13).unwrap_or_default().unwrap_or_default(),
                        "backdrop_path": ep_row.get::<_, Option<String>>(14).unwrap_or_default().unwrap_or_default(),
                        "file_path": ep_row.get::<_, Option<String>>(15).unwrap_or_default(),
                        "media_type": ep_row.get::<_, Option<String>>(16).unwrap_or_default().unwrap_or_default(),
                    }));
                }
            }
        }

        // 2. Up Next (Active Shows)
        let mut cw_stmt = conn.prepare(
            "
            SELECT m.id as media_id
            FROM Media m
            WHERE (SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') > 0
              AND (SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') < m.total_episodes
        ",
        )?;

        let mut cw_eps: Vec<Value> = Vec::new();
        if let Ok(cw_rows) = cw_stmt.query_map([], |row| row.get::<_, i32>(0)) {
            for m_id_res in cw_rows {
                if let Ok(m_id) = m_id_res {
                    if let Some(ref hero) = hero_ep {
                        if hero["media_id"].as_i64().unwrap_or(-1) as i32 == m_id {
                            continue;
                        }
                    }

                    let mut ep_stmt = conn.prepare(
                        "
                        SELECT e.*, m.title as show_title, m.backdrop_path, m.poster_path, l.file_path, m.type as media_type, e.still_path
                        FROM Episodes e
                        JOIN Media m ON e.media_id = m.id
                        LEFT JOIN Local_Files l ON e.id = l.episode_id
                        WHERE e.media_id = ? AND e.status IN ('Watching', 'Unwatched')
                        ORDER BY e.season_num ASC, e.ep_num ASC
                        LIMIT 1
                    ",
                    )?;

                    let mut ep_rows = ep_stmt.query(params![m_id])?;
                    if let Ok(Some(ep_row)) = ep_rows.next() {
                        cw_eps.push(json!({
                            "id": ep_row.get::<_, i32>(0).unwrap_or(0),
                            "media_id": ep_row.get::<_, i32>(1).unwrap_or(0),
                            "season_num": ep_row.get::<_, u32>(2).unwrap_or(0),
                            "ep_num": ep_row.get::<_, u32>(3).unwrap_or(0),
                            "title": ep_row.get::<_, Option<String>>(4).unwrap_or_default().unwrap_or_default(),
                            "runtime": ep_row.get::<_, i32>(5).unwrap_or(0),
                            "still_path": ep_row.get::<_, Option<String>>(6).unwrap_or_default().unwrap_or_default(),
                            "overview": ep_row.get::<_, Option<String>>(7).unwrap_or_default().unwrap_or_default(),
                            "watch_count": ep_row.get::<_, i32>(8).unwrap_or(0),
                            "last_position": ep_row.get::<_, i32>(9).unwrap_or(0),
                            "status": ep_row.get::<_, Option<String>>(10).unwrap_or_default().unwrap_or_default(),
                            "completed_date": ep_row.get::<_, Option<String>>(11).unwrap_or_default().unwrap_or_default(),
                            "air_date": ep_row.get::<_, Option<String>>(12).unwrap_or_default().unwrap_or_default(),

                            "show_title": ep_row.get::<_, Option<String>>(13).unwrap_or_default().unwrap_or_default(),
                            "backdrop_path": ep_row.get::<_, Option<String>>(14).unwrap_or_default().unwrap_or_default(),
                            "poster_path": ep_row.get::<_, Option<String>>(15).unwrap_or_default().unwrap_or_default(),
                            "file_path": ep_row.get::<_, Option<String>>(16).unwrap_or_default(),
                            "media_type": ep_row.get::<_, Option<String>>(17).unwrap_or_default().unwrap_or_default(),
                        }));
                    }
                }
            }
        }

        // 3. Recently Added
        let mut ra_stmt = conn.prepare(
            "
            SELECT m.*,
                   (SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') as completed_eps,
                   (SELECT MAX(timestamp) FROM History h JOIN Episodes e ON h.episode_id = e.id WHERE e.media_id = m.id) as last_watched,
                   (SELECT MIN(air_date) FROM Episodes WHERE media_id = m.id AND air_date IS NOT NULL AND air_date != '') as min_year,
                   (SELECT MAX(air_date) FROM Episodes WHERE media_id = m.id AND air_date IS NOT NULL AND air_date != '') as max_year
            FROM Media m
            ORDER BY m.id DESC LIMIT 15
        ",
        )?;

        let mut recent_media: Vec<Media> = Vec::new();
        if let Ok(rows) = ra_stmt.query_map([], |row| {
            let release_date: String = row.get(11)?;
            let is_unaired = if !release_date.is_empty() {
                let now = chrono::Utc::now().naive_utc().date();
                if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&release_date, "%Y-%m-%d") {
                    parsed > now
                } else {
                    false
                }
            } else {
                false
            };
            Ok(Media {
                id: row.get(0)?,
                tmdb_id: row.get(1)?,
                r#type: row.get(2)?,
                title: row.get(3)?,
                synopsis: row.get(4)?,
                poster_path: row.get(5)?,
                backdrop_path: row.get(6)?,
                total_episodes: row.get(7)?,
                status: row.get(8)?,
                vote_average: row.get(9)?,
                user_rating: row.get(10)?,
                release_date,
                is_exact_date: row.get::<_, Option<bool>>(12)?.unwrap_or(true),
                is_unaired: Some(is_unaired),
                completed_eps: row.get(13)?,
                last_watched: row.get(14)?,
                min_year: row.get(15)?,
                max_year: row.get(16)?,
            })
        }) {
            for m in rows.flatten() {
                recent_media.push(m);
            }
        }

        // 4. Stats
        let eps_watched: i32 = conn
            .query_row(
                "SELECT COUNT(*) as count FROM Episodes WHERE status = 'Completed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let hrs_watched: i32 = conn
            .query_row(
                "SELECT SUM(runtime) FROM Episodes WHERE status = 'Completed'",
                [],
                |r| r.get::<_, Option<i32>>(0).map(|v| v.unwrap_or(0) / 60),
            )
            .unwrap_or(0);

        let shows_completed: i32 = conn
            .query_row(
                "SELECT COUNT(*) as c FROM Media WHERE status = 'Completed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let avg_rating: f64 = conn
            .query_row(
                "SELECT AVG(user_rating) FROM Media WHERE user_rating > 0",
                [],
                |r| r.get::<_, Option<f64>>(0).map(|v| v.unwrap_or(0.0)),
            )
            .unwrap_or(0.0);

        Ok(json!({
            "hero_ep": hero_ep,
            "cw_eps": cw_eps,
            "recent_media": recent_media,
            "stats": {
                "eps_watched": eps_watched,
                "hrs_watched": hrs_watched,
                "shows_completed": shows_completed,
                "avg_rating": avg_rating
            }
        }))
    })
}

#[tauri::command]
pub async fn get_library_data(
    media_type: String,
    sort_by: String,
    hide_completed: bool,
) -> Result<Vec<Media>, AppError> {
    tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let conn = get_db_connection()?;

            let mut base_query = "
                SELECT m.*,
                       (SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') as completed_eps,
                       (SELECT MAX(timestamp) FROM History h JOIN Episodes e ON h.episode_id = e.id WHERE e.media_id = m.id) as last_watched,
                       (SELECT MIN(air_date) FROM Episodes WHERE media_id = m.id AND air_date IS NOT NULL AND air_date != '') as min_year,
                       (SELECT MAX(air_date) FROM Episodes WHERE media_id = m.id AND air_date IS NOT NULL AND air_date != '') as max_year
                FROM Media m
            ".to_string();

            let mut where_clauses = Vec::new();
            let mut sql_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if media_type != "All" {
                where_clauses.push("m.type = ?".to_string());
                sql_params.push(Box::new(media_type));
            }

            if hide_completed {
                where_clauses.push("(SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') < m.total_episodes".to_string());
            }

            if !where_clauses.is_empty() {
                base_query.push_str(" WHERE ");
                base_query.push_str(&where_clauses.join(" AND "));
            }

            let order_by = match sort_by.as_str() {
                "Alphabetical (A-Z)" => " ORDER BY m.title ASC",
                "Release Year" => " ORDER BY CASE WHEN m.release_date IS NULL OR m.release_date = '' THEN 1 ELSE 0 END, m.release_date DESC",
                "My Top Rated" => " ORDER BY m.user_rating DESC, m.id DESC",
                "Sort by Last Watched" => " ORDER BY last_watched DESC NULLS LAST, m.id DESC",
                _ => " ORDER BY m.id DESC",
            };

            base_query.push_str(order_by);

            let mut stmt = conn.prepare(&base_query)?;

            // Convert Vec<Box<dyn ToSql>> to a format rusqlite understands
            let param_refs: Vec<&dyn rusqlite::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

            let rows = stmt.query_map(rusqlite::params_from_iter(param_refs), |row| {
                let release_date: String = row.get(11)?;
                let is_unaired = if !release_date.is_empty() {
                    let now = chrono::Utc::now().naive_utc().date();
                    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&release_date, "%Y-%m-%d") {
                        parsed > now
                    } else {
                        false
                    }
                } else {
                    false
                };
                Ok(Media {
                    id: row.get(0)?,
                    tmdb_id: row.get(1)?,
                    r#type: row.get(2)?,
                    title: row.get(3)?,
                    synopsis: row.get(4)?,
                    poster_path: row.get(5)?,
                    backdrop_path: row.get(6)?,
                    total_episodes: row.get(7)?,
                    status: row.get(8)?,
                    vote_average: row.get(9)?,
                    user_rating: row.get(10)?,
                    release_date,
                    is_exact_date: row.get::<_, Option<bool>>(12)?.unwrap_or(true),
                    is_unaired: Some(is_unaired),
                    completed_eps: row.get(13)?,
                    last_watched: row.get(14)?,
                    min_year: row.get(15)?,
                    max_year: row.get(16)?,
                })
            })?;

            let mut media_list = Vec::new();
            for m in rows.flatten() {
                media_list.push(m);
            }

            Ok(media_list)
        })
    }).await.unwrap_or(Err(AppError::Custom("Task panicked".to_string())))
}

#[tauri::command]
pub fn clear_unmatched_files() -> Result<(), AppError> {
    handle_panic(|| {
        let conn = get_db_connection()?;
        // Soft Truncation
        conn.execute("DELETE FROM Unmatched_Files", params![])?;
        Ok(())
    })
}

#[tauri::command]
pub fn fetch_unmatched_files() -> Result<Vec<UnmatchedFile>, AppError> {
    handle_panic(|| {
        let mut conn = get_db_connection()?;
        let mut files = Vec::new();

        // Lightweight exists check
        let mut paths_to_delete = Vec::new();
        {
            let mut stmt = conn.prepare("SELECT file_path, filename, parsed_series, parsed_season, parsed_episode, group_key FROM Unmatched_Files")?;
            let rows = stmt.query_map([], |row| {
                Ok(UnmatchedFile {
                    file_path: row.get(0)?,
                    filename: row.get(1)?,
                    parsed_series: row.get(2)?,
                    parsed_season: row.get(3)?,
                    parsed_episode: row.get(4)?,
                    group_key: row.get(5)?,
                })
            })?;

            for r in rows.flatten() {
                let p = std::path::Path::new(&r.file_path);

                // If it doesn't exist, check root guard to avoid mass deletion on disconnected drive
                if !p.exists() {
                    let mut should_delete = true;

                    // A robust cross-platform way to check drive disconnections is to walk up the path.
                    // If the file is missing, but its parent directory exists, it was deleted.
                    // If the entire parent tree doesn't exist up to the root, the drive is likely unmounted.
                    let mut current_ancestor = p.parent();
                    let mut found_existing_ancestor = false;
                    while let Some(ancestor) = current_ancestor {
                        if ancestor.exists() {
                            found_existing_ancestor = true;
                            break;
                        }
                        current_ancestor = ancestor.parent();
                    }

                    if !found_existing_ancestor {
                        // The entire tree is gone (including the root mount point).
                        // Likely a disconnected drive. Don't prune.
                        should_delete = false;
                    }

                    if should_delete {
                        paths_to_delete.push(r.file_path.clone());
                        continue; // skip adding to returned files
                    }
                }
                files.push(r);
            }
        }

        // Auto-prune missing files
        if !paths_to_delete.is_empty() {
            let tx = conn.transaction()?;
            for p in paths_to_delete {
                let _ = tx.execute("DELETE FROM Unmatched_Files WHERE file_path = ?", params![p]);
            }
            tx.commit()?;
        }

        Ok(files)
    })
}

#[tauri::command]
pub async fn fetch_history(page: Option<u32>, page_size: Option<u32>) -> Result<Vec<Value>, AppError> {
    tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let conn = get_db_connection()?;
            let limit = page_size.unwrap_or(100);
            let offset = page.unwrap_or(0) * limit;

            let mut stmt = conn.prepare(
                "
                SELECT h.id as hist_id, h.timestamp, h.session_id, h.is_legacy, h.start_time, h.end_time, h.pause_count, h.completion_ratio,
                       e.id as episode_id, e.season_num, e.ep_num, e.title as ep_title, e.still_path, e.air_date, e.runtime,
                       m.id as media_id, m.title as show_title, m.poster_path, m.backdrop_path, m.type as media_type
                FROM History h
                JOIN Episodes e ON h.episode_id = e.id
                JOIN Media m ON e.media_id = m.id
                ORDER BY h.timestamp DESC, h.id DESC
                LIMIT ? OFFSET ?
                "
            )?;

            let mut history = Vec::new();
            let mut rows = stmt.query(params![limit, offset])?;
            while let Ok(Some(row)) = rows.next() {
                let session_id: Option<String> = row.get(2)?;
                let ts: i64 = row.get(1)?;
                let runtime: i32 = row.get(14)?;
                let air_date: String = row.get::<_, Option<String>>(13)?.unwrap_or_default();
                let time_capsule = calculate_gap(&air_date, ts);

                history.push(json!({
                    "hist_id": row.get::<_, i32>(0)?,
                    "timestamp": ts,
                    "session_id": session_id,
                    "is_legacy": row.get::<_, i32>(3)?,
                    "start_time": row.get::<_, Option<String>>(4)?,
                    "end_time": row.get::<_, Option<String>>(5)?,
                    "pause_count": row.get::<_, i32>(6)?,
                    "completion_ratio": row.get::<_, f64>(7)?,
                    "episode_id": row.get::<_, i32>(8)?,
                    "season_num": row.get::<_, u32>(9)?,
                    "ep_num": row.get::<_, u32>(10)?,
                    "ep_title": row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                    "still_path": row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    "air_date": air_date,
                    "time_capsule": time_capsule,
                    "runtime": runtime,
                    "media_id": row.get::<_, i32>(15)?,
                    "show_title": row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                    "poster_path": row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                    "backdrop_path": row.get::<_, Option<String>>(18)?.unwrap_or_default(),
                    "media_type": row.get::<_, Option<String>>(19)?.unwrap_or_default(),
                }));
            }

            // Group entries into Binge-Blocks
            let mut grouped_history = Vec::new();
            let mut current_block: Vec<Value> = Vec::new();

            for entry in history {
                if current_block.is_empty() {
                    current_block.push(entry);
                } else {
                    let last_entry = current_block.last().unwrap();
                    let is_same_session = entry["session_id"].as_str().is_some() && last_entry["session_id"].as_str() == entry["session_id"].as_str();
                    let time_diff = (last_entry["timestamp"].as_i64().unwrap_or(0) - entry["timestamp"].as_i64().unwrap_or(0)).abs();
                    let is_within_6_hours = time_diff <= 21600; // 6 hours
                    let is_same_show = last_entry["media_id"] == entry["media_id"];
                    let is_legacy = entry["is_legacy"].as_i64().unwrap_or(0) == 1;

                    if (is_same_session || (is_within_6_hours && is_same_show)) && !is_legacy {
                        current_block.push(entry);
                    } else {
                        grouped_history.push(json!({
                            "type": if current_block.len() > 1 { "binge_block" } else { "single" },
                            "main_entry": current_block[0].clone(), // Most recent in the block
                            "entries": current_block.clone(),
                            "total_runtime": current_block.iter().map(|e| e["runtime"].as_i64().unwrap_or(0)).sum::<i64>(),
                            "episode_count": current_block.len(),
                        }));
                        current_block = vec![entry];
                    }
                }
            }

            if !current_block.is_empty() {
                grouped_history.push(json!({
                    "type": if current_block.len() > 1 { "binge_block" } else { "single" },
                    "main_entry": current_block[0].clone(),
                    "entries": current_block.clone(),
                    "total_runtime": current_block.iter().map(|e| e["runtime"].as_i64().unwrap_or(0)).sum::<i64>(),
                    "episode_count": current_block.len(),
                }));
            }

            Ok(grouped_history)
        })
    }).await.unwrap_or(Err(AppError::Custom("Task panicked".to_string())))
}

#[tauri::command]
pub async fn run_scan_directory(
    app_handle: tauri::AppHandle,
    directory: String,
) -> Result<i32, AppError> {
    let task = tokio::task::spawn_blocking(move || {
        handle_panic(std::panic::AssertUnwindSafe(|| {
            let mut conn = get_db_connection()?;
            crate::scanner::scan_directory(&directory, &mut conn, &app_handle).map_err(AppError::from)
        }))
    });

    // Provide a generous timeout for massive directory scans (e.g. 5 minutes)
    match tokio::time::timeout(std::time::Duration::from_secs(300), task).await {
        Ok(res) => res.unwrap_or(Err(AppError::Custom("Task panicked".to_string()))),
        Err(_) => Err(AppError::Custom("Scan Directory Task Timed Out".to_string())),
    }
}

#[tauri::command]
pub async fn perform_tmdb_search(query: String) -> Result<Vec<Value>, AppError> {
    let task = tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let settings = crate::settings::load_settings()
                .map_err(|e| AppError::Custom(e))?;
            if settings.tmdb_api_key.is_empty() {
                return Err(AppError::Custom(
                    "Missing TMDB API Key. Please add it in Settings.".to_string(),
                ));
            }
            crate::tmdb::search_media(&settings.tmdb_api_key, &query)
                .map_err(|e| AppError::Custom(e.to_string()))
        })
    });

    match tokio::time::timeout(std::time::Duration::from_secs(15), task).await {
        Ok(res) => res.unwrap_or(Err(AppError::Custom("Task panicked".to_string()))),
        Err(_) => Err(AppError::Custom("Task Timed Out".to_string())),
    }
}

#[tauri::command]
pub async fn assign_unmatched_to_tracker(
    tmdb_id: String,
    media_type: String,
    group_key: String,
) -> Result<(), AppError> {
    let task = tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let settings = crate::settings::load_settings()
                .map_err(|e| AppError::Custom(e))?;
            if settings.tmdb_api_key.is_empty() {
                return Err(AppError::Custom("Missing TMDB API Key. Please add it in Settings.".to_string()));
            }

            let valid_media_type = crate::models::MediaType::from_str(&media_type).as_str().to_string();

            // Fetch files for this group before spawning the thread
            let mut unmatched_files = Vec::new();
            if let Ok(conn) = get_db_connection() {
                if let Ok(mut stmt) = conn.prepare("SELECT file_path, parsed_season, parsed_episode FROM Unmatched_Files WHERE group_key = ?") {
                    if let Ok(rows) = stmt.query_map(params![group_key], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, Option<i32>>(1)?,
                            row.get::<_, Option<i32>>(2)?,
                        ))
                    }) {
                        for r in rows.flatten() {
                            unmatched_files.push(r);
                        }
                    }
                }
            }

            if let Ok(details) = crate::tmdb::get_media_details(&settings.tmdb_api_key, &tmdb_id, &valid_media_type) {

            let mut synopsis = details["synopsis"].as_str().unwrap_or("").to_string();
            if synopsis.chars().count() > 10000 {
                synopsis = synopsis.chars().take(10000).collect::<String>();
                synopsis.push_str("...");
            }

            if let Some(poster) = details["poster_path"].as_str() {
                crate::tmdb::download_image(poster, "w500");
            }
            if let Some(backdrop) = details["backdrop_path"].as_str() {
                crate::tmdb::download_image(backdrop, "w1280");
            }

            let mut all_eps = Vec::new();
            if valid_media_type == "TV" {
                if let Some(seasons) = details["seasons"].as_array() {
                    for season in seasons {
                        if let Some(s_num) = season["season_number"].as_i64() {
                            if s_num >= 0 {
                                if let Ok(eps) = crate::tmdb::get_tv_season_episodes(&settings.tmdb_api_key, &tmdb_id, s_num as u32) {
                                    all_eps.extend(eps);
                                }
                            }
                        }
                    }
                }
            }

            if let Ok(mut conn) = get_db_connection() {
                if let Ok(tx) = conn.transaction() {
                    let mut existing_id = None;
                    {
                        if let Ok(mut stmt) = tx.prepare("SELECT id FROM Media WHERE tmdb_id=?") {
                            if let Ok(mut rows) = stmt.query(params![tmdb_id]) {
                                if let Ok(Some(row)) = rows.next() {
                                    existing_id = Some(row.get::<_, i32>(0).unwrap_or(0));
                                }
                            }
                        }
                    }

                    let media_id = if let Some(id) = existing_id {
                        let _ = tx.execute(
                            "UPDATE Media SET \"title\" = ?, synopsis = ?, poster_path = ?, backdrop_path = ?, total_episodes = ?, vote_average = ?, release_date = ?, is_exact_date = ?
                             WHERE id = ?",
                            params![
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                synopsis,
                                details["poster_path"].as_str().unwrap_or(""),
                                details["backdrop_path"].as_str().unwrap_or(""),
                                details["total_episodes"].as_i64().unwrap_or(1) as i32,
                                details["vote_average"].as_f64().unwrap_or(0.0),
                                details["release_date"].as_str().unwrap_or(""),
                                details["is_exact_date"].as_bool().unwrap_or(true),
                                id
                            ]
                        );
                        id
                    } else {
                        let _ = tx.execute(
                            "INSERT INTO Media (tmdb_id, \"type\", \"title\", synopsis, poster_path, backdrop_path, total_episodes, status, vote_average, release_date, is_exact_date)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                             ON CONFLICT(tmdb_id, \"type\") DO UPDATE SET
                                \"title\"=excluded.\"title\", synopsis=excluded.synopsis, poster_path=excluded.poster_path,
                                backdrop_path=excluded.backdrop_path, total_episodes=excluded.total_episodes,
                                vote_average=excluded.vote_average, release_date=excluded.release_date, is_exact_date=excluded.is_exact_date",
                            params![
                                tmdb_id,
                                valid_media_type,
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                synopsis,
                                details["poster_path"].as_str().unwrap_or(""),
                                details["backdrop_path"].as_str().unwrap_or(""),
                                details["total_episodes"].as_i64().unwrap_or(1) as i32,
                                details["status"].as_str().unwrap_or("Plan to Watch"),
                                details["vote_average"].as_f64().unwrap_or(0.0),
                                details["release_date"].as_str().unwrap_or(""),
                                details["is_exact_date"].as_bool().unwrap_or(true)
                            ]
                        );

                        let mut new_id = tx.last_insert_rowid() as i32;
                        if new_id == 0 {
                            if let Ok(mut stmt) = tx.prepare("SELECT id FROM Media WHERE tmdb_id=? AND \"type\"=?") {
                                if let Ok(mut rows) = stmt.query(params![tmdb_id, valid_media_type]) {
                                    if let Ok(Some(row)) = rows.next() {
                                        new_id = row.get::<_, i32>(0).unwrap_or(0);
                                    }
                                }
                            }
                        }
                        new_id
                    };

                    let ep_status = "Unwatched";
                    let ep_watch_count = 0;

                    if valid_media_type == "TV" {
                        for ep in all_eps {
                            let mut ep_overview = ep["overview"].as_str().unwrap_or("").to_string();
                            if ep_overview.chars().count() > 10000 {
                                ep_overview = ep_overview.chars().take(10000).collect::<String>();
                                ep_overview.push_str("...");
                            }

                            let _ = tx.execute(
                                "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\", runtime, still_path, overview, status, watch_count, air_date, is_exact_date)
                                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                                 ON CONFLICT(media_id, season_num, ep_num) DO UPDATE SET
                                    \"title\"=excluded.\"title\", runtime=excluded.runtime, still_path=excluded.still_path,
                                    overview=excluded.overview, air_date=excluded.air_date, is_exact_date=excluded.is_exact_date",
                                params![
                                    media_id,
                                    ep["season_num"].as_i64().unwrap_or(1) as u32,
                                    ep["ep_num"].as_i64().unwrap_or(1) as u32,
                                    ep["title"].as_str().unwrap_or("Unknown Title"),
                                    ep["runtime"].as_i64().unwrap_or(0) as i32,
                                    ep["still_path"].as_str().unwrap_or(""),
                                    ep_overview,
                                    ep_status,
                                    ep_watch_count,
                                    ep["air_date"].as_str().unwrap_or(""),
                                    ep["is_exact_date"].as_bool().unwrap_or(true)
                                ]
                            );
                        }
                    } else {
                        let _ = tx.execute(
                            "INSERT INTO Episodes (media_id, season_num, ep_num, \"title\", runtime, still_path, overview, status, watch_count, air_date, is_exact_date)
                             VALUES (?, 1, 1, ?, ?, ?, ?, ?, ?, ?, ?)
                             ON CONFLICT(media_id, season_num, ep_num) DO UPDATE SET
                                \"title\"=excluded.\"title\", runtime=excluded.runtime, still_path=excluded.still_path,
                                overview=excluded.overview, air_date=excluded.air_date, is_exact_date=excluded.is_exact_date",
                            params![
                                media_id,
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                details["runtime"].as_i64().unwrap_or(0) as i32,
                                details["backdrop_path"].as_str().unwrap_or(""),
                                synopsis,
                                ep_status,
                                ep_watch_count,
                                details["release_date"].as_str().unwrap_or(""),
                                details["is_exact_date"].as_bool().unwrap_or(true)
                            ]
                        );
                    }

                    // Now assign the unmatched files
                    for (file_path, parsed_season, parsed_ep) in &unmatched_files {
                        let mut matched_ep_id = None;
                        if valid_media_type == "TV" {
                            if let (Some(s_num), Some(e_num)) = (*parsed_season, *parsed_ep) {
                                if let Ok(mut stmt) = tx.prepare("SELECT id FROM Episodes WHERE media_id = ? AND season_num = ? AND ep_num = ?") {
                                    if let Ok(mut rows) = stmt.query(params![media_id, s_num, e_num]) {
                                        if let Ok(Some(row)) = rows.next() {
                                            matched_ep_id = Some(row.get::<_, i32>(0).unwrap_or(0));
                                        }
                                    }
                                }
                            }
                        } else {
                            // Movie, assume season 1 episode 1
                            if let Ok(mut stmt) = tx.prepare("SELECT id FROM Episodes WHERE media_id = ? AND season_num = 1 AND ep_num = 1") {
                                if let Ok(mut rows) = stmt.query(params![media_id]) {
                                    if let Ok(Some(row)) = rows.next() {
                                        matched_ep_id = Some(row.get::<_, i32>(0).unwrap_or(0));
                                    }
                                }
                            }
                        }

                        if let Some(ep_id) = matched_ep_id {
                            let _ = tx.execute(
                                "INSERT INTO Local_Files (episode_id, file_path) VALUES (?, ?) ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path",
                                params![ep_id, file_path],
                            );
                        }
                    }

                    // Remove these unmatched files
                    let _ = tx.execute("DELETE FROM Unmatched_Files WHERE group_key = ?", params![group_key]);

                    let _ = tx.commit();
                }
            }
        }

            Ok(())
        })
    });

    match tokio::time::timeout(std::time::Duration::from_secs(15), task).await {
        Ok(res) => res.unwrap_or(Err(AppError::Custom("Task panicked".to_string()))),
        Err(_) => Err(AppError::Custom("Task Timed Out".to_string())),
    }
}
