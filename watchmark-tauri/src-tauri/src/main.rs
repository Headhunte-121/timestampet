// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod error;
mod models;
mod hash;
mod scanner;
mod settings;
mod tmdb;
mod vlc;
pub mod task_queue;
mod backup;
mod sanitizer;
pub mod network;

#[cfg(test)]
#[path = "sanitizer_tests.rs"]
mod sanitizer_tests;

#[cfg(test)]
#[path = "backup_tests.rs"]
mod backup_tests;

#[cfg(test)]
#[path = "restore_tests.rs"]
mod restore_tests;

#[cfg(test)]
#[path = "task_queue_tests.rs"]
mod task_queue_tests;

#[cfg(test)]
fn backup_tests_module_trigger() {}

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, MouseButton, TrayIconEvent},
    Manager, Emitter
};
use tauri_plugin_notification::NotificationExt;
use native_dialog::{MessageDialog, MessageType};

pub fn execute_cold_swap(app_dir: &std::path::Path) {
    let trigger_file_path = app_dir.join(".restore_pending");

    if trigger_file_path.exists() {
        let db_dir = app_dir.join("db");
        let active_db_path = db_dir.join("watchmark.db");
        let pending_db_path = db_dir.join("watchmark.db.pending");
        let old_db_path = db_dir.join("watchmark.db.old");

        // Only proceed if the pending database was successfully staged
        if pending_db_path.exists() {
            // Move active to .old (overwrite if exists)
            if active_db_path.exists() {
                let _ = std::fs::rename(&active_db_path, &old_db_path);
            }

            // Move pending to active
            if let Err(e) = std::fs::rename(&pending_db_path, &active_db_path) {
                // Critical failure during rename. Attempt to revert.
                if old_db_path.exists() {
                    let _ = std::fs::rename(&old_db_path, &active_db_path);
                }

                let error_msg = format!("Fatal Error: Database Restore Failed during cold-swap. Changes reverted.\n\nError details: {}", e);
                MessageDialog::new()
                    .set_type(MessageType::Error)
                    .set_title("WatchMark - Restore Error")
                    .set_text(&error_msg)
                    .show_alert()
                    .unwrap();
            } else {
                // Success! Clean up the trigger file.
                let _ = std::fs::remove_file(&trigger_file_path);
            }
        } else {
            // Trigger file exists but pending DB is missing. Corrupt state. Clean up flag.
            let _ = std::fs::remove_file(&trigger_file_path);
        }
    }
}

fn canary_check() -> Result<(), std::io::Error> {
    let app_dir = db::get_app_data_dir();
    // Try to create base dir if not exist
    if !app_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&app_dir) {
            return Err(e);
        }
    }

    let canary_path = app_dir.join(".canary");
    if let Err(e) = std::fs::write(&canary_path, b"canary") {
        return Err(e);
    }
    let _ = std::fs::remove_file(canary_path);
    Ok(())
}

fn main() {
    log::info!("Starting application: Running canary check...");
    if let Err(e) = canary_check() {
        let error_msg = format!("Fatal Error: Could not initialize application data directory.\n\nPermissions Required to write to: {:?}\n\nError details: {}", db::get_app_data_dir(), e);
        log::error!("{}", error_msg);
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("WatchMark - Fatal Error")
            .set_text(&error_msg)
            .show_alert()
            .unwrap();
        std::process::exit(1);
    }
    log::info!("Canary check passed successfully.");

    if let Err(e) = db::ensure_directories() {
         let error_msg = format!("Fatal Error: Could not create nested application data directories.\n\nError details: {}", e);
         log::error!("{}", error_msg);
         MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("WatchMark - Fatal Error")
            .set_text(&error_msg)
            .show_alert()
            .unwrap();
        std::process::exit(1);
    }

    // Cold-Swap Database Restore Logic
    execute_cold_swap(&db::get_app_data_dir());

    log::info!("Initializing SQLite database...");
    db::init_db().expect("Failed to initialize database");
    log::info!("Database initialized successfully.");

    let initial_settings = match settings::load_settings() {
        Ok(s) => s,
        Err(e) => {
            let error_msg = format!("Fatal Error: Could not load or generate configuration files.\n\nError details: {}", e);
            MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("WatchMark - Fatal Error")
                .set_text(&error_msg)
                .show_alert()
                .unwrap();
            std::process::exit(1);
        }
    };

    let (settings_tx, mut settings_rx) = tokio::sync::mpsc::channel::<models::Settings>(100);

    // Spawn Background Backup Task
    let backup_settings_arc = std::sync::Arc::new(std::sync::RwLock::new(initial_settings.clone()));
    let app_backup_settings_arc = backup_settings_arc.clone();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("app".into()),
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app_handle = window.app_handle().clone();
                if let Some(state) = app_handle.try_state::<commands::AppState>() {
                    let db_queue = state.db_queue.clone();

                    api.prevent_close(); // Prevent immediate exit
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    db_queue.shutdown(tx);

                    tauri::async_runtime::spawn(async move {
                        // Wait for maximum 2 seconds for worker thread to drain
                        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), rx).await;
                        app_handle.exit(0);
                    });
                }
            }
        })
        .setup(move |app| {
            log::info!("Tauri setup hook triggered. Initializing state...");
            let db_queue = std::sync::Arc::new(task_queue::DbTaskQueue::new(app.handle().clone()));

            let available_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
            let pool_size = (available_cores.saturating_sub(1)).max(1);

            let failed_image_sync_queue = std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new()));

            app.manage(commands::AppState {
                settings: app_backup_settings_arc,
                settings_tx,
                db_queue: db_queue.clone(),
                is_maintenance_mode: std::sync::atomic::AtomicBool::new(false),
                is_api_authorized: std::sync::atomic::AtomicBool::new(true),
                is_rate_limited: std::sync::atomic::AtomicBool::new(false),
                rate_limit_reset: std::sync::atomic::AtomicI64::new(0),
                stats_cache: std::sync::Arc::new(std::sync::RwLock::new(None)),
                read_semaphore: std::sync::Arc::new(tokio::sync::Semaphore::new(pool_size)),
                cancel_tokens: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
                failed_image_syncs: failed_image_sync_queue.clone(),
            });
            log::info!("AppState successfully managed by Tauri. Thread pool restricted to {}", pool_size);

            #[cfg(test)]
            crate::backup_tests_module_trigger();

            let app_handle_for_backup = app.handle().clone();

            // Background Backup Task
            tauri::async_runtime::spawn(async move {
                loop {
                    let mut should_backup = false;
                    let current_ts = chrono::Utc::now().timestamp();
                    {
                        if let Ok(settings) = backup_settings_arc.read() {
                            // 24 hours = 86400 seconds
                            if current_ts - settings.last_backup_timestamp > 86400 || settings.last_backup_timestamp == 0 {
                                should_backup = true;
                            }
                        }
                    }

                    if should_backup {
                        // Offload blocking backup task to a dedicated thread with low priority feel
                        let _app_h = app_handle_for_backup.clone();
                        let backup_res = tokio::task::spawn_blocking(move || {
                            crate::backup::perform_backup()
                        }).await;

                        match backup_res {
                            Ok(Ok(_)) => {
                                log::info!("Automatic database backup successful.");
                                let mut next_settings = None;
                                {
                                    if let Ok(mut settings) = backup_settings_arc.write() {
                                        settings.last_backup_timestamp = current_ts;
                                        settings.last_backup_status = "success".to_string();
                                        settings.last_backup_error = "".to_string();
                                        next_settings = Some(settings.clone());
                                    }
                                }
                                if let Some(updated_settings) = next_settings {
                                    let _ = crate::settings::save_settings(&updated_settings);
                                    let _ = app_handle_for_backup.emit("backup-finished", serde_json::json!({
                                        "status": "success",
                                        "timestamp": current_ts
                                    }));
                                }
                            }
                            Ok(Err(e)) => {
                                log::error!("Automatic database backup failed: {}", e);
                                let fail_ts = chrono::Utc::now().timestamp();
                                let mut next_settings = None;
                                {
                                    if let Ok(mut settings) = backup_settings_arc.write() {
                                        settings.last_backup_status = "error".to_string();
                                        settings.last_backup_error = e.to_string();
                                        next_settings = Some(settings.clone());
                                    }
                                }
                                if let Some(updated_settings) = next_settings {
                                    let _ = crate::settings::save_settings(&updated_settings);
                                }
                                let _ = app_handle_for_backup.emit("backup-finished", serde_json::json!({
                                    "status": "error",
                                    "error": e.to_string(),
                                    "timestamp": fail_ts
                                }));
                            }
                            Err(_) => {
                                log::error!("Backup task panicked or timed out.");
                            }
                        }
                    }

                    // Sleep for 1 hour before checking again
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                }
            });

            // Spawn low priority background image retry loop (runs every 30 mins)
            let app_handle_for_retry = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1800)); // 30 minutes
                loop {
                    interval.tick().await;

                    let mut retries = Vec::new();
                    {
                        let mut queue = failed_image_sync_queue.write().unwrap();
                        for item in queue.drain() {
                            retries.push(item);
                        }
                    }

                    if !retries.is_empty() {
                        if let Some(state) = app_handle_for_retry.try_state::<commands::AppState>() {
                            let hp_mode = state.settings.read().unwrap().high_performance_mode;
                            for (path, size) in retries {
                                tokio::task::yield_now().await; // prevent blocking
                                let _ = crate::tmdb::download_image(&path, &size, hp_mode).await;
                            }
                        }
                    }
                }
            });

            // Spawn debouncer task for saving settings inside Tauri's managed tokio runtime
            tauri::async_runtime::spawn(async move {
                let mut last_settings: Option<models::Settings> = None;
                let mut timeout = tokio::time::interval(tokio::time::Duration::from_millis(500));
                timeout.tick().await; // consume first tick immediately

                loop {
                    tokio::select! {
                        Some(settings) = settings_rx.recv() => {
                            last_settings = Some(settings);
                            timeout.reset();
                        }
                        _ = timeout.tick() => {
                            if let Some(settings) = last_settings.take() {
                                if let Err(e) = crate::settings::save_settings(&settings) {
                                    log::error!("Failed to save debounced settings: {}", e);
                                } else {
                                    log::info!("Settings successfully saved to disk.");
                                }
                            }
                        }
                    }
                }
            });
            use tauri_plugin_global_shortcut::{ShortcutState, GlobalShortcutExt};

            let _app_handle = app.handle().clone();

            // Register hotkey: Ctrl+Shift+S (Scan Directory)
            let scan_shortcut = "CommandOrControl+Shift+S";
            let hide_shortcut = "CommandOrControl+Shift+H";

            if let Err(e) = app.global_shortcut().on_shortcut(scan_shortcut, {
                move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("tray-scan", ());
                        }
                    }
                }
            }) {
                let _ = app.notification().builder().title("WatchMark").body(format!("Could not register hotkey {}: {:?}", scan_shortcut, e)).show();
            }

            if let Err(e) = app.global_shortcut().on_shortcut(hide_shortcut, {
                move |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                }
            }) {
                let _ = app.notification().builder().title("WatchMark").body(format!("Could not register hotkey {}: {:?}", hide_shortcut, e)).show();
            }

            use tauri::menu::PredefinedMenuItem;
            let scan_i = MenuItem::with_id(app, "scan", "Scan Directory", true, None::<&str>)?;
            let resume_i = MenuItem::with_id(app, "resume", "Resume Last Show", true, None::<&str>)?;
            let update_i = MenuItem::with_id(app, "update", "Check for Updates", true, None::<&str>)?;
            let sep_i = PredefinedMenuItem::separator(app)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&scan_i, &resume_i, &update_i, &sep_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "scan" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("tray-scan", ());
                            }
                        }
                        "resume" => {
                            // Resume last show logic to spawn VLC directly
                            if let Ok(conn) = db::get_db_connection() {
                                // Simplified: Query to get the last watched episode path.
                                // Assuming history table has timestamp and episode_id.
                                let result: rusqlite::Result<(i32, String)> = conn.query_row(
                                    "SELECT History.episode_id, Local_Files.file_path FROM History
                                     JOIN Local_Files ON History.episode_id = Local_Files.episode_id
                                     ORDER BY History.timestamp DESC LIMIT 1",
                                    [],
                                    |row: &rusqlite::Row| Ok((row.get(0)?, row.get(1)?)),
                                );

                                match result {
                                    Ok((ep_id, file_path)) => {
                                        // Fetch the position from episodes if available (optional)
                                        let last_position: i32 = conn.query_row("SELECT last_position FROM Episodes WHERE id = ?1", [ep_id], |row: &rusqlite::Row| row.get(0)).unwrap_or(0);

                                        // Spawn VLC (non-blocking) using vlc.rs functionality or directly.
                                        let app_handle = app.clone();

                                        tokio::spawn(async move {
                                            let state = app_handle.state::<commands::AppState>();
                                            if let Err(e) = crate::vlc::play_episode_cmd(app_handle.clone(), ep_id, file_path, last_position, state).await {
                                                let _ = app_handle.notification()
                                                    .builder()
                                                    .title("WatchMark")
                                                    .body(format!("Cannot resume: {}", e))
                                                    .show();
                                            }
                                        });
                                    }
                                    Err(_) => {
                                        let _ = app.notification()
                                            .builder()
                                            .title("WatchMark")
                                            .body("Cannot resume: File not found.")
                                            .show();
                                    }
                                }
                            }
                        }
                        "update" => {
                            // Placeholder for update check
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray: &tauri::tray::TrayIcon, event| match event {
                    TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("watchmark", |_app, request, responder| {
            let path = request.uri().path();
            if path.is_empty() {
                responder.respond(
                    http::Response::builder()
                        .status(400)
                        .body(Vec::new())
                        .unwrap(),
                );
                return;
            }
            let path = path[1..].to_string();
            // URL decode the path
            let decoded_path = percent_encoding::percent_decode_str(&path)
                .decode_utf8_lossy()
                .to_string();

            let safe_path = std::path::PathBuf::from(&decoded_path);

            // Clone request headers before moving them into the task
            let range_header = request.headers().get("Range").and_then(|h| h.to_str().ok().map(|s| s.to_string()));

            tokio::spawn(async move {
                if !decoded_path.contains("..") && safe_path.is_absolute() {
                    if let Ok(real_path) = std::fs::canonicalize(&safe_path) {
                        if real_path.is_file() {
                            let mime_type = match real_path.extension().and_then(|e| e.to_str()) {
                                Some("png") => "image/png",
                                Some("jpg") | Some("jpeg") => "image/jpeg",
                                Some("webp") => "image/webp",
                                Some("mp4") => "video/mp4",
                                Some("mkv") => "video/x-matroska",
                                _ => "application/octet-stream",
                            };

                            if let Ok(metadata) = tokio::fs::metadata(&real_path).await {
                                let file_size = metadata.len();
                                use std::io::SeekFrom;
                                use tokio::io::AsyncSeekExt;
                                use tokio::io::AsyncReadExt;

                                if let Some(range_str) = range_header {
                                    // Parse Range header (e.g., "bytes=0-1023")
                                    let range_str = range_str.trim_start_matches("bytes=");
                                    let mut parts = range_str.split('-');
                                    let start_str = parts.next().unwrap_or("");
                                    let end_str = parts.next().unwrap_or("");

                                    let start = if start_str.is_empty() { 0 } else { start_str.parse::<u64>().unwrap_or(0) };

                                    // Default read chunk size is 1MB to keep memory footprint low
                                    let max_chunk_size = 1024 * 1024;
                                    let mut end = if end_str.is_empty() {
                                        std::cmp::min(start + max_chunk_size - 1, file_size - 1)
                                    } else {
                                        end_str.parse::<u64>().unwrap_or(file_size - 1)
                                    };

                                    if start >= file_size {
                                        responder.respond(
                                            http::Response::builder()
                                                .status(416) // Range Not Satisfiable
                                                .header("Content-Range", format!("bytes */{}", file_size))
                                                .body(Vec::new())
                                                .unwrap(),
                                        );
                                        return;
                                    }

                                    // Enforce max chunk size to prevent memory exhaustion on large requested ranges
                                    if end - start + 1 > max_chunk_size {
                                        end = start + max_chunk_size - 1;
                                    }

                                    // Ensure end doesn't exceed file boundaries
                                    end = std::cmp::min(end, file_size - 1);
                                    let chunk_size = end - start + 1;

                                    if let Ok(mut file) = tokio::fs::File::open(&real_path).await {
                                        if file.seek(SeekFrom::Start(start)).await.is_ok() {
                                            let mut buffer = vec![0; chunk_size as usize];
                                            if let Ok(bytes_read) = file.read_exact(&mut buffer).await {
                                                // Shrink buffer if we read less than expected (shouldn't happen with read_exact on local file but safe)
                                                buffer.truncate(bytes_read);

                                                responder.respond(
                                                    http::Response::builder()
                                                        .status(206) // Partial Content
                                                        .header("Access-Control-Allow-Origin", "*")
                                                        .header("Content-Type", mime_type)
                                                        .header("Accept-Ranges", "bytes")
                                                        .header("Content-Range", format!("bytes {}-{}/{}", start, start + bytes_read as u64 - 1, file_size))
                                                        .header("Content-Length", bytes_read.to_string())
                                                        .body(buffer)
                                                        .unwrap(),
                                                );
                                                return;
                                            }
                                        }
                                    }
                                } else {
                                    // No Range request: Return the file. For 2GB video files this is still bad,
                                    // but typical for images. However, to keep memory low, we can limit even 200 OK
                                    // responses to 10MB chunks if needed. But for standard compatibility, if the client
                                    // doesn't request a range, we should stream it. `responder.respond` accepts `Vec<u8>`.
                                    // If we read the whole file here, we still blow RAM. Let's just limit max 200 OK read to 30MB.
                                    // BUT, we can just send the first chunk and add `Accept-Ranges: bytes`.
                                    // For images, they are small. For videos, HTML5 ALWAYS sends Range.

                                    let limit = std::cmp::min(file_size, 1024 * 1024 * 30); // Max 30MB

                                    if let Ok(mut file) = tokio::fs::File::open(&real_path).await {
                                        let mut buffer = vec![0; limit as usize];
                                        if let Ok(bytes_read) = file.read(&mut buffer).await {
                                            buffer.truncate(bytes_read);
                                            responder.respond(
                                                http::Response::builder()
                                                    .status(200)
                                                    .header("Access-Control-Allow-Origin", "*")
                                                    .header("Content-Type", mime_type)
                                                    .header("Accept-Ranges", "bytes")
                                                    .header("Content-Length", file_size.to_string())
                                                    .body(buffer)
                                                    .unwrap(),
                                            );
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                responder.respond(
                    http::Response::builder()
                        .status(404)
                        .body(Vec::new())
                        .unwrap(),
                );
            });
        })
        .invoke_handler(tauri::generate_handler![
            commands::cancel_task,
            commands::optimize_database,
            commands::repair_paths,
            commands::remove_local_link,
            commands::validate_and_hash_file,
            commands::update_local_file,
            commands::get_settings,
            commands::save_settings,
            commands::delete_media_cmd,
            commands::get_media_details_db,
            commands::add_to_tracker,
            commands::mark_season_watched,
            commands::archive_season,
            commands::toggle_episode_status,
            commands::get_dashboard_data,
            commands::get_library_data,
            commands::clear_unmatched_files,
            commands::fetch_unmatched_files,
            commands::fetch_history,
            commands::run_scan_directory,
            commands::perform_tmdb_search,
            commands::validate_tmdb_key,
            commands::assign_unmatched_to_tracker,
            commands::get_media_history_count,
            commands::update_media_rating,
            commands::export_database,
            commands::prepare_restore,
            vlc::play_episode_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
