// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use rusqlite::params;
use serde_json::json;
use std::time::Duration;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::process::{Child, Command};
use std::process::Stdio;

use crate::db::get_db_connection;
use crate::error::AppError;
use crate::network::NETWORK_MANAGER;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::net::TcpListener;
use tokio::io::AsyncReadExt;

lazy_static::lazy_static! {
    static ref ACTIVE_VLC: Mutex<Option<u32>> = Mutex::new(None);
    static ref VLC_PORT: Mutex<u16> = Mutex::new(8080);
    static ref VLC_PASSWORD: Mutex<String> = Mutex::new(String::new());
}

#[derive(Clone, serde::Serialize)]
struct RefreshPayload {
    message: String,
}

pub async fn kill_active_vlc() {
    let pid_to_kill = {
        let mut lock = ACTIVE_VLC.lock().unwrap();
        lock.take()
    };

    if let Some(pid) = pid_to_kill {
        tracing::info!("[VLC] 🛑 Killing previous VLC instance (PID {})...", pid);
        #[cfg(unix)]
        {
            let _ = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .status()
                .await;
        }
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .arg("/F")
                .arg("/PID")
                .arg(pid.to_string())
                .status()
                .await;
        }
    }
}

pub fn play_in_vlc(vlc_path: &str, file_path: &str, start_time: i32) -> Result<Child, AppError> {
    // Generate a secure 16-character alphanumeric password
    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // Perform a socket probe to find a free port between 8080 and 8090
    let mut port = 8080;
    loop {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            break;
        }
        port += 1;
        if port > 8090 {
            return Err(AppError::Custom("Could not find a free port for VLC HTTP interface.".to_string()));
        }
    }

    {
        let mut p_lock = VLC_PORT.lock().unwrap();
        *p_lock = port;
        let mut pw_lock = VLC_PASSWORD.lock().unwrap();
        *pw_lock = password.clone();
    }

    // Determine path based on OS logic (Verbatim for Windows)
    #[cfg(windows)]
    let safe_file_path = if file_path.starts_with(r"\\?\") {
        // The ? gets URI-encoded by VLC and breaks playback.
        // Strip the verbatim prefix for VLC compatibility.
        file_path[4..].to_string()
    } else {
        file_path.to_string()
    };

    #[cfg(unix)]
    let safe_file_path = file_path.to_string();

    // Verify existence immediately before spawning to prevent ghost spawns
    if !std::path::Path::new(file_path).exists() {
        tracing::error!("[VLC] 🚨 File not found right before spawn: {}", file_path);
        return Err(AppError::Custom("FILE_NOT_FOUND".to_string()));
    }

    #[allow(unused_mut)]
    let mut std_cmd = std::process::Command::new(vlc_path);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW = 0x08000000
        std_cmd.creation_flags(0x08000000);
    }

    let mut cmd = Command::from(std_cmd);

    // Capture stderr to check for binding/auth errors, while ignoring stdout
    cmd.stdout(Stdio::null())
       .stderr(Stdio::piped());

    cmd.arg(&safe_file_path)
        .arg("--extraintf=http")
        .arg(format!("--http-port={}", port))
        .arg("--http-host=127.0.0.1");

    cmd.arg(format!("--http-password={}", password));

    if start_time > 0 {
        cmd.arg(format!("--start-time={}", start_time));
    }

    cmd.spawn().map_err(|e| AppError::Custom(format!("Failed to start VLC: {}", e)))
}

#[derive(serde::Deserialize)]
pub struct VlcStatus {
    pub length: Option<f64>,
    pub time: Option<f64>,
    pub state: Option<String>,
}

pub async fn get_vlc_status() -> Option<VlcStatus> {
    let (port, password) = {
        let p_lock = VLC_PORT.lock().unwrap();
        let pw_lock = VLC_PASSWORD.lock().unwrap();
        (*p_lock, pw_lock.clone())
    };

    if password.is_empty() {
        return None;
    }

    if let Ok(res) = NETWORK_MANAGER.local_client
        .get(&format!("http://127.0.0.1:{}/requests/status.json", port))
        .timeout(Duration::from_secs(1))
        .basic_auth("", Some(password))
        .send()
        .await
    {
        if let Ok(json) = res.json::<VlcStatus>().await {
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
    tracing::info!("[VLC] 💓 Establishing VLC Heartbeat connection...");
    let mut high_water_mark: f64 = 0.0;
    let mut last_time_seconds: f64 = 0.0;
    let mut pause_count: i32 = 0;
    let mut was_paused: bool = false;
    let mut consecutive_failures: i32 = 0;

    // Session Snapshot for frontend sync
    // Not actually used by frontend yet but maintains state
    let mut session_snapshot_time: f64 = 0.0;

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

    let mut has_overridden_runtime = false;

    let mut is_initial_probe = true;
    let probe_start = std::time::Instant::now();
    let mut in_high_res_mode = false;
    let mut high_res_end_time = std::time::Instant::now();

    let mut length_averaging_buffer: Vec<f64> = Vec::new();
    let mut stable_state_time = std::time::Instant::now();

    loop {
        let tick_duration = if is_initial_probe {
            Duration::from_millis(200)
        } else if in_high_res_mode {
            Duration::from_millis(500)
        } else {
            Duration::from_secs(5)
        };

        if in_high_res_mode && std::time::Instant::now() > high_res_end_time {
            in_high_res_mode = false;
        }

        tokio::select! {
            _ = tokio::time::sleep(tick_duration) => {
                if is_initial_probe && probe_start.elapsed() > Duration::from_secs(3) {
                    is_initial_probe = false;
                }

                if let Some(status) = get_vlc_status().await {
                    is_initial_probe = false;
                    consecutive_failures = 0;

                    let length = status.length.unwrap_or(0.0);
                    let raw_time = status.time;

                    // Negative time and garbage data filtration
                    let time = match raw_time {
                        Some(t) if t >= 0.0 => {
                            if length > 0.0 && t > length {
                                session_snapshot_time // Overflow, discard and keep previous
                            } else {
                                t
                            }
                        },
                        _ => session_snapshot_time // Negative or missing node, discard and keep previous
                    };

                    // High-frequency capture during rapid seeking
                    if (time - session_snapshot_time).abs() > 10.0 && !is_initial_probe {
                        in_high_res_mode = true;
                        high_res_end_time = std::time::Instant::now() + Duration::from_secs(2);
                    }

                    session_snapshot_time = time;

                    // Sync with frontend state
                    if let Ok(mut live_time) = app_handle.state::<crate::commands::AppState>().live_playback_time.write() {
                        *live_time = session_snapshot_time;
                    }

                    if length > 0.0 && !has_overridden_runtime {
                        length_averaging_buffer.push(length);

                        if length_averaging_buffer.len() >= 3 {
                            // Check if consistent
                            let mut consistent = true;
                            let first_len = length_averaging_buffer[0];
                            for l in &length_averaging_buffer {
                                if (*l - first_len).abs() > 1.0 {
                                    consistent = false;
                                    break;
                                }
                            }

                            if consistent {
                                let length_minutes = (first_len / 60.0).round();
                                if (length_minutes - stored_runtime_minutes).abs() > 2.0 {
                                    tracing::info!("[VLC] 🔄 Local file length ({:?}m) differs from TMDB ({:?}m). Overriding.", length_minutes, stored_runtime_minutes);
                                    if let Ok(conn) = get_db_connection() {
                                        let _ = conn.execute(
                                            "UPDATE Episodes SET runtime=? WHERE id=?",
                                            params![length_minutes as i32, episode_id],
                                        );
                                    }
                                    stored_runtime_minutes = length_minutes;
                                }
                                has_overridden_runtime = true;
                            } else {
                                // If inconsistent, clear buffer and try again
                                length_averaging_buffer.clear();
                            }
                        }
                    } else if length == 0.0 {
                        // Handle Live Stream or corrupted index
                        tracing::warn!("[VLC] ⚠️ VLC reported length 0. Activating fallback.");
                        let _ = app_handle.emit("vlc-livestream-fallback", json!({ "episode_id": episode_id }));
                    }

                    if length > 0.0 {
                        // Math logic uses f64 clamping.
                        let mut pos = time / length;
                        if pos < 0.0 { pos = 0.0; }
                        if pos > 1.0 { pos = 1.0; }

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

                        let current_state = status.state.unwrap_or_default().to_lowercase();

                        let is_paused = current_state == "paused";
                        let is_stopped = current_state == "stopped";
                        let mut newly_paused = false;

                        let time_jumped = (clamped_time - last_written_time_seconds).abs() > 30.0;
                        let time_to_flush = last_flush_time.elapsed() >= Duration::from_secs(300);

                        let state_changed = is_paused != was_paused;
                        let lockout_active = stable_state_time.elapsed() < std::time::Duration::from_millis(1000);

                        // Rapid "Spam-Click" State Debouncing
                        if state_changed && !lockout_active {
                            stable_state_time = std::time::Instant::now();
                            was_paused = is_paused;
                            if is_paused {
                                pause_count += 1;
                                newly_paused = true;
                            }
                        }

                        let should_commit = newly_paused || time_jumped || time_to_flush || is_stopped;

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

                        if is_stopped {
                            // "Session Closure" protocol
                            tracing::info!("[VLC] 🛑 VLC reported 'stopped' state. Committing and ending session.");
                            break;
                        }
                    }
                } else {
                    if !is_initial_probe {
                        consecutive_failures += 1;
                        if consecutive_failures >= 3 {
                            // Assume VLC has crashed or disconnected
                            tracing::warn!("[VLC] ⚠️ Heartbeat missed 3 times. Assuming disconnect.");
                            break;
                        }
                    }
                }
            }
            status = proc.wait() => {
                // VLC process has exited
                tracing::info!("[VLC] 🛑 VLC Process terminated with status: {:?}", status);
                if let Ok(exit_status) = status {
                    if !exit_status.success() {
                        let _ = app_handle.emit("vlc-crashed", serde_json::json!({
                            "message": "Playback Interrupted"
                        }));
                    }
                }
                break;
            }
        }
    }

    // Clear password from memory when session ends
    {
        let mut pw_lock = VLC_PASSWORD.lock().unwrap();
        *pw_lock = String::new();
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

        let mut completion_threshold = 0.90;
        // Handling short-form media (clips) < 5 minutes
        if final_runtime_seconds > 0.0 && final_runtime_seconds < 300.0 {
            completion_threshold = (final_runtime_seconds - 30.0) / final_runtime_seconds;
        }

        // High-precision float math safety
        let is_within_10s = final_runtime_seconds > 0.0 && (final_runtime_seconds - last_time_seconds) <= 10.0;

        if high_water_mark > completion_threshold || is_within_10s {
            tracing::info!("[BACKEND] 🧠 Math evaluated > threshold watched. Marking episode 'Completed'.");
            let _ = conn.execute(
                "UPDATE Episodes SET watch_count = watch_count + 1, status = 'Completed', last_position = 0 WHERE id = ?",
                params![episode_id],
            );
            let _ = conn.execute(
                "UPDATE History SET completion_ratio=1.0, end_time=? WHERE episode_id=? AND timestamp=? AND session_id=?",
                params![end_dt_str, episode_id, start_dt_str, session_id],
            );
        } else if high_water_mark > 0.05 {
            tracing::info!("[BACKEND] 🧠 Math evaluated <90% watched. Saving pause state as 'Watching'.");
            let _ = conn.execute(
                "UPDATE Episodes SET status = 'Watching', last_position = ? WHERE id = ? AND status != 'Completed'",
                params![last_time_seconds as i32, episode_id],
            );
            let _ = conn.execute(
                "UPDATE History SET completion_ratio=?, end_time=? WHERE episode_id=? AND timestamp=? AND session_id=?",
                params![high_water_mark, end_dt_str, episode_id, start_dt_str, session_id],
            );
        } else {
            tracing::info!("[BACKEND] 🧠 Math evaluated <5% watched. Abandoning session tracking.");
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
    state: tauri::State<'_, crate::commands::AppState>,
) -> Result<(), AppError> {
    tracing::info!("[VLC] 🎬 Preparing to launch VLC player...");
    if let Ok(mut cache) = state.stats_cache.write() {
        *cache = None;
    }
    if state.is_maintenance_mode.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(AppError::Custom("System Busy: Maintenance mode is currently active.".to_string()));
    }

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

    kill_active_vlc().await;

    match play_in_vlc(&canonical_vlc.to_string_lossy(), &file_path, start_sec) {
        Ok(mut proc) => {
            // Non-blocking loop to check stderr for 2 seconds
            if let Some(mut stderr) = proc.stderr.take() {
                let start_time = std::time::Instant::now();
                let mut error_detected = false;
                let mut buffer = [0; 1024];

                while start_time.elapsed() < Duration::from_secs(2) {
                    if let Ok(bytes_read) = tokio::time::timeout(Duration::from_millis(100), stderr.read(&mut buffer)).await {
                        if let Ok(n) = bytes_read {
                            if n > 0 {
                                let err_str = String::from_utf8_lossy(&buffer[..n]).to_lowercase();
                                if err_str.contains("password") || err_str.contains("bind") || err_str.contains("error") {
                                    error_detected = true;
                                    break;
                                }
                            } else {
                                break; // EOF
                            }
                        }
                    }

                    // Check if heartbeat is already responding, meaning VLC is fully up and running
                    if get_vlc_status().await.is_some() {
                        break;
                    }
                }

                if error_detected {
                    let _ = proc.kill().await;
                    return Err(AppError::Custom("VLC_AUTH_ERROR".to_string()));
                }
            }

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
                    let last_session_id: Option<String> = row.get(0).unwrap_or_default();
                    let last_timestamp: i64 = row.get(1).unwrap_or_default();

                    let now = chrono::Utc::now().timestamp();
                    if now - last_timestamp <= 21600 {
                        if let Some(sid) = last_session_id {
                            if !sid.is_empty() {
                                session_id = sid;
                            }
                        }
                    }
                }
            }

            let current_timestamp = chrono::Utc::now().timestamp();

            let _ = conn.execute(
                "INSERT INTO History (episode_id, timestamp, session_id, is_legacy, start_time, pause_count, completion_ratio)
                    VALUES (?, ?, ?, 0, ?, 0, 0.0)",
                params![episode_id, current_timestamp, session_id, start_dt_str],
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

            let proc_id = proc.id();
            if let Some(pid) = proc_id {
                let mut lock = ACTIVE_VLC.lock().unwrap();
                *lock = Some(pid);
            }

            tokio::spawn(async move {
                vlc_heartbeat(proc, episode_id, session_id, start_dt_str, app_handle).await;

                // Clean up PID when process naturally exits, but only if it's OUR process
                if let Some(pid) = proc_id {
                    let mut lock = ACTIVE_VLC.lock().unwrap();
                    if *lock == Some(pid) {
                        *lock = None;
                    }
                }
            });

            Ok(())
        }
        Err(e) => Err(e)
    }
}
