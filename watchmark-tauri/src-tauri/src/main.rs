// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod scanner;
mod settings;
mod tmdb;
mod vlc;

fn main() {
    db::init_db().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_media_details_db,
            commands::add_to_tracker,
            commands::get_dashboard_data,
            commands::get_library_data,
            commands::fetch_unmatched_files,
            commands::fetch_history,
            commands::run_scan_directory,
            commands::perform_tmdb_search,
            vlc::play_episode_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
