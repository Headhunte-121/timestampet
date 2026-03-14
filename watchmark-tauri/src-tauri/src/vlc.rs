use reqwest::Client;
use rusqlite::params;
use serde_json::Value;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::process::{Child, Command};

use crate::db::get_db_connection;
use crate::error::AppError;

#[derive(Clone, serde::Serialize)]
struct RefreshPayload {
    message: String,
}

pub fn play_in_vlc(vlc_path: &str, file_path: &str, start_time: i32) -> Option<Child> {
    let mut cmd = Command::new(vlc_path);

    cmd.arg(file_path)
        .arg("--extraintf=http")
        .arg("--http-port=8080")
        .arg("--http-password=watchmark");

    if start_time > 0 {
        cmd.arg(format!("--start-time={}", start_time));
    }

    cmd.spawn().ok()
}

pub async fn get_vlc_status(client: &Client) -> Option<Value> {
    let res = client
        .get("http://127.0.0.1:8080/requests/status.json")
        .basic_auth("", Some("watchmark"))
        .send()
        .await
        .ok()?;

    if res.status().is_success() {
        if let Ok(json) = res.json::<Value>().await {
            return Some(json);
        }
    }
    None
}

pub async fn vlc_heartbeat(
    mut proc: Child,
    episode_id: i32,
    session_id: String,
    start_dt_str: String,
    app_handle: AppHandle,
) {
    let mut high_water_mark: f64 = 0.0;
    let mut last_time_seconds: f64 = 0.0;
    let mut pause_count: i32 = 0;
    let mut was_paused: bool = false;
    let mut consecutive_failures: i32 = 0;

    // Track state for throttling writes
    let mut last_written_time_seconds: f64 = 0.0;
    let mut last_flush_time = std::time::Instant::now();

    let mut interval = tokio::time::interval(Duration::from_secs(5));
    // The first tick completes immediately, we skip it so we wait 5s first.
    interval.tick().await;

    // Fetch the stored runtime in minutes
    let mut stored_runtime_minutes = 0.0;
    if let Ok(conn) = get_db_connection() {
        if let Ok(mut stmt) = conn.prepare("SELECT runtime FROM Episodes WHERE id=?") {
            if let Ok(mut rows) = stmt.query(params![episode_id]) {
                if let Ok(Some(row)) = rows.next() {
                    let runtime: i32 = row.get(0).unwrap_or(0);
                    stored_runtime_minutes = runtime as f64;
                }
            }
        }
    }

    let client = match Client::builder().timeout(Duration::from_secs(2)).build() {
        Ok(c) => c,
        Err(_) => return, // If we can't even build the client, abort heartbeat.
    };

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if let Some(status) = get_vlc_status(&client).await {
                    consecutive_failures = 0;

                    let length = status["length"].as_f64().unwrap_or(0.0);
                    let time = status["time"].as_f64().unwrap_or(0.0);

                    if length > 0.0 {
                        let pos = time / length;
                        if pos > high_water_mark {
                            high_water_mark = pos;
                        }

                        // Logical clamping based on stored runtime if available
                        let clamped_time = if stored_runtime_minutes > 0.0 {
                            let max_seconds = stored_runtime_minutes * 60.0;
                            if time > max_seconds {
                                max_seconds
                            } else {
                                time
                            }
                        } else {
                            time
                        };

                        let state = status["state"].as_str().unwrap_or("");
                        let is_paused = state == "paused";

                        if is_paused && !was_paused {
                            pause_count += 1;
                        }

                        let time_jumped = (clamped_time - last_written_time_seconds).abs() > 30.0;
                        let time_to_flush = last_flush_time.elapsed() >= Duration::from_secs(300);

                        // Trigger a write if paused, significant jump, or 5-min flush
                        let should_commit = (is_paused && !was_paused) || time_jumped || time_to_flush;
                        was_paused = is_paused;
                        last_time_seconds = clamped_time;

                        if should_commit {
                            last_written_time_seconds = clamped_time;
                            last_flush_time = std::time::Instant::now();

                            if let Ok(conn) = get_db_connection() {
                                let _ = conn.execute(
                                    "UPDATE Episodes SET last_position=? WHERE id=?",
                                    params![last_time_seconds as i32, episode_id],
                                );
                                let _ = conn.execute(
                                    "UPDATE History SET completion_ratio=?, pause_count=? WHERE episode_id=? AND timestamp=? AND session_id=?",
                                    params![high_water_mark, pause_count, episode_id, start_dt_str, session_id],
                                );
                            }
                        }
                    }
                } else {
                    consecutive_failures += 1;
                    if consecutive_failures >= 3 {
                        // Assume VLC has crashed or disconnected
                        break;
                    }
                }
            }
            status = proc.wait() => {
                // VLC process has exited
                let _ = status;
                break;
            }
        }
    }

    let end_dt_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Ok(conn) = get_db_connection() {
        // Retrieve final runtime in case it wasn't fetched earlier or was updated
        let mut final_runtime_minutes = stored_runtime_minutes;
        if final_runtime_minutes == 0.0 {
            if let Ok(mut stmt) = conn.prepare("SELECT runtime FROM Episodes WHERE id=?") {
                if let Ok(mut rows) = stmt.query(params![episode_id]) {
                    if let Ok(Some(row)) = rows.next() {
                        let runtime: i32 = row.get(0).unwrap_or(0);
                        final_runtime_minutes = runtime as f64;
                    }
                }
            }
        }

        let final_runtime_seconds = final_runtime_minutes * 60.0;
        let is_within_10s = final_runtime_seconds > 0.0 && (final_runtime_seconds - last_time_seconds) <= 10.0;

        if high_water_mark > 0.90 || is_within_10s {
            let _ = conn.execute(
                "UPDATE Episodes SET watch_count = watch_count + 1, status = 'Completed', last_position = 0 WHERE id = ?",
                params![episode_id],
            );
            let _ = conn.execute(
                "UPDATE History SET completion_ratio=1.0, end_time=? WHERE episode_id=? AND timestamp=? AND session_id=?",
                params![end_dt_str, episode_id, start_dt_str, session_id],
            );
        } else if high_water_mark > 0.05 {
            let _ = conn.execute(
                "UPDATE Episodes SET status = 'Watching', last_position = ? WHERE id = ? AND status != 'Completed'",
                params![last_time_seconds as i32, episode_id],
            );
            let _ = conn.execute(
                "UPDATE History SET completion_ratio=?, end_time=? WHERE episode_id=? AND timestamp=? AND session_id=?",
                params![high_water_mark, end_dt_str, episode_id, start_dt_str, session_id],
            );
        } else {
            let _ = conn.execute(
                "DELETE FROM History WHERE episode_id=? AND timestamp=? AND session_id=?",
                params![episode_id, start_dt_str, session_id],
            );
        }
    }

    // Emit event to frontend to refresh
    let _ = app_handle.emit(
        "vlc-closed",
        RefreshPayload {
            message: "VLC closed, refresh UI".into(),
        },
    );
}

#[tauri::command]
pub async fn play_episode_cmd(
    app_handle: AppHandle,
    episode_id: i32,
    file_path: String,
    last_position: i32,
) -> Result<(), AppError> {
    let settings = crate::settings::load_settings().unwrap_or_default();
    if settings.vlc_path.is_empty() {
        return Err(AppError::Custom(
            "VLC path not configured in Settings".to_string(),
        ));
    }

    // Sanitize path inputs to avoid injection or panics
    let canonical_path = dunce::canonicalize(&file_path).map_err(|_| {
        AppError::Custom(format!("Invalid or non-existent path: {}", file_path))
    })?;

    let canonical_vlc = dunce::canonicalize(&settings.vlc_path).map_err(|_| {
        AppError::Custom("Invalid VLC executable path configured in Settings".to_string())
    })?;

    let file_path = canonical_path.to_string_lossy().to_string();

    let start_sec = if last_position > 0 { last_position } else { 0 };

    if let Some(proc) = play_in_vlc(&canonical_vlc.to_string_lossy(), &file_path, start_sec) {
        let mut session_id = uuid::Uuid::new_v4().to_string();
        let start_dt_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        {
            let conn = get_db_connection()?;

            // Auto-binge detection logic
            let mut media_id = 0;
            let mut stmt = conn.prepare("SELECT media_id FROM Episodes WHERE id=?")?;
            let mut rows = stmt.query(params![episode_id])?;
            if let Some(row) = rows.next()? {
                media_id = row.get(0).unwrap_or(0);
            }

            if media_id > 0 {
                let mut hist_stmt = conn.prepare(
                    "SELECT session_id, timestamp FROM History
                        WHERE episode_id IN (SELECT id FROM Episodes WHERE media_id=?) AND is_legacy=0
                        ORDER BY timestamp DESC LIMIT 1"
                )?;

                let mut rows = hist_stmt.query(params![media_id])?;
                if let Some(row) = rows.next()? {
                    let last_session_id: String = row.get(0).unwrap_or_default();
                    let last_timestamp: String = row.get(1).unwrap_or_default();

                    // Parse timestamp and check if within 6 hours (21600 seconds)
                    if let Ok(last_dt) =
                        chrono::NaiveDateTime::parse_from_str(&last_timestamp, "%Y-%m-%d %H:%M:%S")
                    {
                        let last_dt_utc =
                            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                                last_dt,
                                chrono::Utc,
                            );
                        let now = chrono::Utc::now();
                        if (now - last_dt_utc).num_seconds() < 21600 {
                            session_id = last_session_id;
                        }
                    }
                }
            }

            let _ = conn.execute(
                "INSERT INTO History (episode_id, timestamp, session_id, is_legacy, start_time, pause_count, completion_ratio)
                    VALUES (?, ?, ?, 0, ?, 0, 0.0)",
                params![episode_id, start_dt_str, session_id, start_dt_str],
            )?;

            let mut status = "Unwatched".to_string();
            let mut stmt = conn.prepare("SELECT status FROM Episodes WHERE id=?")?;
            let mut rows = stmt.query(params![episode_id])?;
            if let Some(row) = rows.next()? {
                let current_status: String = row.get(0).unwrap_or_default();
                if current_status == "Unwatched" {
                    status = "Watching".to_string();
                } else {
                    status = current_status;
                }
            }

            if status == "Watching" {
                let _ = conn.execute(
                    "UPDATE Episodes SET status='Watching' WHERE id=?",
                    params![episode_id],
                )?;
            }
        } // `conn` dropped here

        tokio::spawn(async move {
            vlc_heartbeat(proc, episode_id, session_id, start_dt_str, app_handle).await;
        });

        Ok(())
    } else {
        Err(AppError::Custom("Failed to start VLC".to_string()))
    }
}
