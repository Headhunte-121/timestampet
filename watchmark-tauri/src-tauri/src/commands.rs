use crate::db::get_db_connection;
use crate::error::{handle_panic, AppError};
use crate::models::{HistoryEntry, Media, Settings, UnmatchedFile};
use rusqlite::params;
use serde_json::{json, Value};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

pub struct AppState {
    pub settings: Arc<RwLock<Settings>>,
    pub settings_tx: mpsc::Sender<Settings>,
}

#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, AppError> {
    let settings = state.settings.read().unwrap();
    Ok(settings.clone())
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
                    "release_date": row.get::<_, Option<String>>(11).unwrap_or_default().unwrap_or_default(),
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
                let s_rows = s_stmt.query_map(params![media_id], |row| row.get::<_, i32>(0));
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
                ORDER BY e.season_num, e.ep_num
            ",
            )?;

            let mut episodes = vec![];
            if let Ok(ep_rows) = eps_stmt.query_map(params![media_id], |row| {
                Ok(json!({
                    "id": row.get::<_, i32>(0)?,
                    "media_id": row.get::<_, i32>(1)?,
                    "season_num": row.get::<_, i32>(2)?,
                    "ep_num": row.get::<_, i32>(3)?,
                    "title": row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    "runtime": row.get::<_, i32>(5)?,
                    "still_path": row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    "overview": row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                    "watch_count": row.get::<_, i32>(8)?,
                    "last_position": row.get::<_, i32>(9)?,
                    "status": row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                    "completed_date": row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                    "air_date": row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    "file_path": row.get::<_, Option<String>>(13)?
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

            if let Ok(details) = crate::tmdb::get_media_details(&settings.tmdb_api_key, &tmdb_id, &media_type) {

            if let Some(poster) = details["poster_path"].as_str() {
                crate::tmdb::download_image(poster, "w500");
            }
            if let Some(backdrop) = details["backdrop_path"].as_str() {
                crate::tmdb::download_image(backdrop, "w1280");
            }

            let mut all_eps = Vec::new();
            if media_type == "TV" {
                if let Some(seasons) = details["seasons"].as_array() {
                    for season in seasons {
                        if let Some(s_num) = season["season_number"].as_i64() {
                            if s_num > 0 {
                                if let Ok(eps) = crate::tmdb::get_tv_season_episodes(&settings.tmdb_api_key, &tmdb_id, s_num) {
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

                    if existing_id.is_none() {
                        let _ = tx.execute(
                            "INSERT INTO Media (tmdb_id, type, title, synopsis, poster_path, backdrop_path, total_episodes, status, vote_average, release_date)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                            params![
                                tmdb_id,
                                media_type,
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                details["synopsis"].as_str().unwrap_or(""),
                                details["poster_path"].as_str().unwrap_or(""),
                                details["backdrop_path"].as_str().unwrap_or(""),
                                details["total_episodes"].as_i64().unwrap_or(1) as i32,
                                details["status"].as_str().unwrap_or("Plan to Watch"),
                                details["vote_average"].as_f64().unwrap_or(0.0),
                                details["release_date"].as_str().unwrap_or("")
                            ]
                        );

                        let media_id = tx.last_insert_rowid() as i32;

                        let ep_status = if archive { "Completed" } else { "Unwatched" };
                        let ep_watch_count = if archive { 1 } else { 0 };

                        if media_type == "TV" {
                            for ep in all_eps {
                                let _ = tx.execute(
                                    "INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview, status, watch_count, air_date)
                                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                    params![
                                        media_id,
                                        ep["season_num"].as_i64().unwrap_or(1) as i32,
                                        ep["ep_num"].as_i64().unwrap_or(1) as i32,
                                        ep["title"].as_str().unwrap_or("Unknown Title"),
                                        ep["runtime"].as_i64().unwrap_or(0) as i32,
                                        ep["still_path"].as_str().unwrap_or(""),
                                        ep["overview"].as_str().unwrap_or(""),
                                        ep_status,
                                        ep_watch_count,
                                        ep["air_date"].as_str().unwrap_or("")
                                    ]
                                );
                            }
                        } else {
                            let _ = tx.execute(
                                "INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview, status, watch_count, air_date)
                                 VALUES (?, 1, 1, ?, ?, ?, ?, ?, ?, ?)",
                                params![
                                    media_id,
                                    details["title"].as_str().unwrap_or("Unknown Title"),
                                    details["runtime"].as_i64().unwrap_or(0) as i32,
                                    details["backdrop_path"].as_str().unwrap_or(""),
                                    details["synopsis"].as_str().unwrap_or(""),
                                    ep_status,
                                    ep_watch_count,
                                    details["release_date"].as_str().unwrap_or("")
                                ]
                            );
                        }
                    }

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
                        "season_num": ep_row.get::<_, i32>(2).unwrap_or(0),
                        "ep_num": ep_row.get::<_, i32>(3).unwrap_or(0),
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
                            "season_num": ep_row.get::<_, i32>(2).unwrap_or(0),
                            "ep_num": ep_row.get::<_, i32>(3).unwrap_or(0),
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
                release_date: row.get(11)?,
                completed_eps: row.get(12)?,
                last_watched: row.get(13)?,
                min_year: row.get(14)?,
                max_year: row.get(15)?,
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
                    release_date: row.get(11)?,
                    completed_eps: row.get(12)?,
                    last_watched: row.get(13)?,
                    min_year: row.get(14)?,
                    max_year: row.get(15)?,
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
pub fn fetch_unmatched_files() -> Result<Vec<UnmatchedFile>, AppError> {
    handle_panic(|| {
        let conn = get_db_connection()?;
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

        let mut files = Vec::new();
        for r in rows.flatten() {
            files.push(r);
        }
        Ok(files)
    })
}

#[tauri::command]
pub async fn fetch_history(page: Option<u32>, page_size: Option<u32>) -> Result<Vec<HistoryEntry>, AppError> {
    tokio::task::spawn_blocking(move || {
        handle_panic(|| {
            let conn = get_db_connection()?;
            let limit = page_size.unwrap_or(100);
            let offset = page.unwrap_or(0) * limit;

            let mut stmt = conn.prepare(
                "
                SELECT h.id as hist_id, h.timestamp, h.session_id, h.is_legacy, h.start_time, h.end_time, h.pause_count, h.completion_ratio,
                       e.id as episode_id, e.season_num, e.ep_num, e.title as ep_title, e.still_path, e.air_date,
                       m.id as media_id, m.title as show_title, m.poster_path, m.backdrop_path, m.type as media_type
                FROM History h
                JOIN Episodes e ON h.episode_id = e.id
                JOIN Media m ON e.media_id = m.id
                ORDER BY h.timestamp DESC
                LIMIT ? OFFSET ?
                "
            )?;

            let rows = stmt.query_map(params![limit, offset], |row| {
                Ok(HistoryEntry {
                    hist_id: row.get(0)?,
                    timestamp: row.get(1)?,
                    session_id: row.get(2)?,
                    is_legacy: row.get(3)?,
                    start_time: row.get(4)?,
                    end_time: row.get(5)?,
                    pause_count: row.get(6)?,
                    completion_ratio: row.get(7)?,
                    episode_id: row.get(8)?,
                    season_num: row.get(9)?,
                    ep_num: row.get(10)?,
                    ep_title: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                    still_path: row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    air_date: row.get::<_, Option<String>>(13)?.unwrap_or_default(),
                    media_id: row.get(14)?,
                    show_title: row.get::<_, Option<String>>(15)?.unwrap_or_default(),
                    poster_path: row.get::<_, Option<String>>(16)?.unwrap_or_default(),
                    backdrop_path: row.get::<_, Option<String>>(17)?.unwrap_or_default(),
                    media_type: row.get::<_, Option<String>>(18)?.unwrap_or_default(),
                })
            })?;

            let mut history = Vec::new();
            for r in rows.flatten() {
                history.push(r);
            }
            Ok(history)
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

            if let Ok(details) = crate::tmdb::get_media_details(&settings.tmdb_api_key, &tmdb_id, &media_type) {

            if let Some(poster) = details["poster_path"].as_str() {
                crate::tmdb::download_image(poster, "w500");
            }
            if let Some(backdrop) = details["backdrop_path"].as_str() {
                crate::tmdb::download_image(backdrop, "w1280");
            }

            let mut all_eps = Vec::new();
            if media_type == "TV" {
                if let Some(seasons) = details["seasons"].as_array() {
                    for season in seasons {
                        if let Some(s_num) = season["season_number"].as_i64() {
                            if s_num > 0 {
                                if let Ok(eps) = crate::tmdb::get_tv_season_episodes(&settings.tmdb_api_key, &tmdb_id, s_num) {
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
                        id
                    } else {
                        let _ = tx.execute(
                            "INSERT INTO Media (tmdb_id, type, title, synopsis, poster_path, backdrop_path, total_episodes, status, vote_average, release_date)
                             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                            params![
                                tmdb_id,
                                media_type,
                                details["title"].as_str().unwrap_or("Unknown Title"),
                                details["synopsis"].as_str().unwrap_or(""),
                                details["poster_path"].as_str().unwrap_or(""),
                                details["backdrop_path"].as_str().unwrap_or(""),
                                details["total_episodes"].as_i64().unwrap_or(1) as i32,
                                details["status"].as_str().unwrap_or("Plan to Watch"),
                                details["vote_average"].as_f64().unwrap_or(0.0),
                                details["release_date"].as_str().unwrap_or("")
                            ]
                        );

                        let new_media_id = tx.last_insert_rowid() as i32;

                        let ep_status = "Unwatched";
                        let ep_watch_count = 0;

                        if media_type == "TV" {
                            for ep in all_eps {
                                let _ = tx.execute(
                                    "INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview, status, watch_count, air_date)
                                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                    params![
                                        new_media_id,
                                        ep["season_num"].as_i64().unwrap_or(1) as i32,
                                        ep["ep_num"].as_i64().unwrap_or(1) as i32,
                                        ep["title"].as_str().unwrap_or("Unknown Title"),
                                        ep["runtime"].as_i64().unwrap_or(0) as i32,
                                        ep["still_path"].as_str().unwrap_or(""),
                                        ep["overview"].as_str().unwrap_or(""),
                                        ep_status,
                                        ep_watch_count,
                                        ep["air_date"].as_str().unwrap_or("")
                                    ]
                                );
                            }
                        } else {
                            let _ = tx.execute(
                                "INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview, status, watch_count, air_date)
                                 VALUES (?, 1, 1, ?, ?, ?, ?, ?, ?, ?)",
                                params![
                                    new_media_id,
                                    details["title"].as_str().unwrap_or("Unknown Title"),
                                    details["runtime"].as_i64().unwrap_or(0) as i32,
                                    details["backdrop_path"].as_str().unwrap_or(""),
                                    details["synopsis"].as_str().unwrap_or(""),
                                    ep_status,
                                    ep_watch_count,
                                    details["release_date"].as_str().unwrap_or("")
                                ]
                            );
                        }
                        new_media_id
                    };

                    // Now assign the unmatched files
                    for (file_path, parsed_season, parsed_ep) in &unmatched_files {
                        let mut matched_ep_id = None;
                        if media_type == "TV" {
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
