use rusqlite::Result;
use std::fs;

use crate::db::get_app_data_dir;
use crate::models::Settings;

pub fn load_settings() -> Settings {
    let default = Settings {
        vlc_path: String::new(),
        window_geometry: "1200x800".to_string(),
        window_position: "+100+100".to_string(),
        tmdb_api_key: String::new(),
        cinema_mode: true,
    };

    let settings_file = get_app_data_dir().join("settings.json");
    if !settings_file.exists() {
        return default;
    }

    if let Ok(content) = fs::read_to_string(&settings_file) {
        if let Ok(settings) = serde_json::from_str(&content) {
            return settings;
        }
    }

    default
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let settings_file = get_app_data_dir().join("settings.json");
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        fs::write(settings_file, json).map_err(|e| e.to_string())
    } else {
        Err("Failed to serialize settings".to_string())
    }
}
