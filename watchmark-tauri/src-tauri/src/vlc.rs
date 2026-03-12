use reqwest::blocking::Client;
use rusqlite::params;
use serde_json::Value;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::db::get_db_connection;

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

pub fn get_vlc_status() -> Option<Value> {
    let client = Client::builder().timeout(Duration::from_secs(2)).build().ok()?;
    let res = client
        .get("http://127.0.0.1:8080/requests/status.json")
        .basic_auth("", Some("watchmark"))
        .send()
        .ok()?;

    if res.status().is_success() {
        if let Ok(json) = res.json::<Value>() {
            return Some(json);
        }
    }
    None
}

pub fn vlc_heartbeat(
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

    let is_running = Arc::new(AtomicBool::new(true));

    while is_running.load(Ordering::SeqCst) {
        if let Ok(Some(_)) = proc.try_wait() {
            is_running.store(false, Ordering::SeqCst);
            break;
        }

        std::thread::sleep(Duration::from_secs(5));

        if let Some(status) = get_vlc_status() {
            let length = status["length"].as_f64().unwrap_or(0.0);
            let time = status["time"].as_f64().unwrap_or(0.0);

            if length > 0.0 {
                let pos = time / length;
                if pos > high_water_mark {
                    high_water_mark = pos;
                }

                let state = status["state"].as_str().unwrap_or("");
                let is_paused = state == "paused";

                if is_paused && !was_paused {
                    pause_count += 1;
                }
                was_paused = is_paused;

                last_time_seconds = time;

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
    }

    let end_dt_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    if let Ok(conn) = get_db_connection() {
        if high_water_mark > 0.90 {
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
pub fn play_episode_cmd(
    app_handle: AppHandle,
    episode_id: i32,
    file_path: &str,
    last_position: i32,
) -> Result<(), String> {
    let settings = crate::settings::load_settings();
    if settings.vlc_path.is_empty() {
        return Err("VLC path not configured in Settings".to_string());
    }

    if !std::path::Path::new(file_path).exists() {
        return Err("File does not exist".to_string());
    }

    let start_sec = if last_position > 0 { last_position } else { 0 };

    if let Some(proc) = play_in_vlc(&settings.vlc_path, file_path, start_sec) {
        let mut session_id = uuid::Uuid::new_v4().to_string();
        let start_dt_str = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        if let Ok(conn) = get_db_connection() {
            // Auto-binge detection logic
            let mut media_id = 0;
            if let Ok(mut stmt) = conn.prepare("SELECT media_id FROM Episodes WHERE id=?") {
                if let Ok(mut rows) = stmt.query(params![episode_id]) {
                    if let Ok(Some(row)) = rows.next() {
                        media_id = row.get(0).unwrap_or(0);
                    }
                }
            }

            if media_id > 0 {
                let mut hist_stmt = conn.prepare(
                    "SELECT session_id, timestamp FROM History
                     WHERE episode_id IN (SELECT id FROM Episodes WHERE media_id=?) AND is_legacy=0
                     ORDER BY timestamp DESC LIMIT 1"
                ).unwrap();

                let mut rows = hist_stmt.query(params![media_id]).unwrap();
                if let Ok(Some(row)) = rows.next() {
                    let last_session_id: String = row.get(0).unwrap_or_default();
                    let last_timestamp: String = row.get(1).unwrap_or_default();

                    // Parse timestamp and check if within 6 hours (21600 seconds)
                    if let Ok(last_dt) = chrono::NaiveDateTime::parse_from_str(&last_timestamp, "%Y-%m-%d %H:%M:%S") {
                        let last_dt_utc = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(last_dt, chrono::Utc);
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
            );

            let mut status = "Unwatched".to_string();
            if let Ok(mut stmt) = conn.prepare("SELECT status FROM Episodes WHERE id=?") {
                 if let Ok(mut rows) = stmt.query(params![episode_id]) {
                     if let Ok(Some(row)) = rows.next() {
                         let current_status: String = row.get(0).unwrap_or_default();
                         if current_status == "Unwatched" {
                             status = "Watching".to_string();
                         } else {
                             status = current_status;
                         }
                     }
                 }
            }
            if status == "Watching" {
                let _ = conn.execute("UPDATE Episodes SET status='Watching' WHERE id=?", params![episode_id]);
            }
        }

        std::thread::spawn(move || {
            vlc_heartbeat(
                proc,
                episode_id,
                session_id,
                start_dt_str,
                app_handle,
            );
        });

        Ok(())
    } else {
        Err("Failed to start VLC".to_string())
    }
}
