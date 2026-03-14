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

#[cfg(test)]
#[path = "backup_tests.rs"]
mod backup_tests;

#[cfg(test)]
fn backup_tests_module_trigger() {}

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, MouseButton, TrayIconEvent},
    Manager, Emitter
};
use tauri_plugin_notification::NotificationExt;
use native_dialog::{MessageDialog, MessageType};

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
    if let Err(e) = canary_check() {
        let error_msg = format!("Fatal Error: Could not initialize application data directory.\n\nPermissions Required to write to: {:?}\n\nError details: {}", db::get_app_data_dir(), e);
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("WatchMark - Fatal Error")
            .set_text(&error_msg)
            .show_alert()
            .unwrap();
        std::process::exit(1);
    }

    if let Err(e) = db::ensure_directories() {
         let error_msg = format!("Fatal Error: Could not create nested application data directories.\n\nError details: {}", e);
         MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("WatchMark - Fatal Error")
            .set_text(&error_msg)
            .show_alert()
            .unwrap();
        std::process::exit(1);
    }

    db::init_db().expect("Failed to initialize database");

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

    let db_queue = std::sync::Arc::new(task_queue::DbTaskQueue::new());

    // Spawn Background Backup Task
    let backup_settings_arc = std::sync::Arc::new(std::sync::RwLock::new(initial_settings.clone()));
    let app_backup_settings_arc = backup_settings_arc.clone();

    tauri::Builder::default()
        .manage(commands::AppState {
            settings: app_backup_settings_arc,
            settings_tx,
            db_queue: db_queue.clone(),
        })
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
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
                                            if let Err(e) = crate::vlc::play_episode_cmd(app_handle.clone(), ep_id, file_path, last_position).await {
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

            tokio::spawn(async move {
                // Determine the allowed directory (the user's app data directory or specific library folders).
                // For a robust implementation, this would pull from the `db` or `settings`.
                // For now, we restrict this strictly to paths that exist and are absolute,
                // BUT we also add a canonicalization check to ensure they are real files and don't traverse.

                if !decoded_path.contains("..") && safe_path.is_absolute() {
                    if let Ok(real_path) = std::fs::canonicalize(&safe_path) {
                        // In a fully locked down app, we would verify `real_path` starts with
                        // one of the configured library root paths or the app data path.
                        // Since WatchMark plays arbitrary local files the user adds, it needs
                        // access to potentially any drive. We rely on the `is_absolute` and
                        // `canonicalize` to at least ensure it's a direct, valid file path.

                        if real_path.is_file() {
                            if let Ok(data) = tokio::fs::read(&real_path).await {
                                responder.respond(
                                    http::Response::builder()
                                        .header("Access-Control-Allow-Origin", "*")
                                        .body(data)
                                        .unwrap(),
                                );
                                return;
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
            commands::assign_unmatched_to_tracker,
            commands::get_media_history_count,
            commands::update_media_rating,
            vlc::play_episode_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
