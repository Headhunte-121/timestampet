// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod error;
mod models;
mod scanner;
mod settings;
mod tmdb;
mod vlc;

fn main() {
    db::init_db().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
            commands::get_settings,
            commands::save_settings,
            commands::delete_media_cmd,
            commands::get_media_details_db,
            commands::add_to_tracker,
            commands::get_dashboard_data,
            commands::get_library_data,
            commands::fetch_unmatched_files,
            commands::fetch_history,
            commands::run_scan_directory,
            commands::perform_tmdb_search,
            commands::assign_unmatched_to_tracker,
            vlc::play_episode_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
