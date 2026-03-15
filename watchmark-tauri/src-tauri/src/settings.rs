// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use rusqlite::Result;
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use keyring::Entry;

use crate::db::get_app_data_dir;
use crate::models::Settings;

const KEYRING_SERVICE: &str = "WatchMark";
const KEYRING_USER: &str = "TMDB_API_KEY";

fn get_keyring_entry() -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| e.to_string())
}

pub fn load_settings() -> Result<Settings, String> {
    let default = Settings::default();

    let settings_file = get_app_data_dir().join("settings.json");
    if !settings_file.exists() {
        // Create the directory just in case it's missing.
        if let Err(e) = fs::create_dir_all(get_app_data_dir()) {
            return Err(format!("Could not create AppData directory: {}", e));
        }
        if let Err(e) = save_settings(&default) {
            return Err(format!("Could not write default settings to disk: {}. Is the AppData directory read-only?", e));
        }

        // Immediately re-read to verify disk I/O was successful
        if let Err(e) = fs::read_to_string(&settings_file) {
             return Err(format!("Disk write verification failed: {}", e));
        }

        let mut first_run_settings = default;
        if let Ok(env_key) = std::env::var("TMDB_API_KEY") {
            if !env_key.is_empty() {
                first_run_settings.tmdb_api_key = env_key;
            }
        }

        return Ok(first_run_settings);
    }

    let mut loaded_settings = default.clone();
    let mut needs_repair = false;

    match fs::read_to_string(&settings_file) {
        Ok(content) => {
            match serde_json::from_str::<Settings>(&content) {
                Ok(mut settings) => {
                    // Fix bounds if corrupted
                    if settings.width <= 0 { settings.width = 1280; }
                    if settings.height <= 0 { settings.height = 800; }

                    // Recover API key
                    let mut found_key = false;

                    // First, try Environment Variable (for cloud testing)
                    if let Ok(env_key) = std::env::var("TMDB_API_KEY") {
                        if !env_key.is_empty() {
                            settings.tmdb_api_key = env_key;
                            found_key = true;
                        }
                    }

                    // Next, try Keyring
                    if !found_key {
                        if let Ok(entry) = get_keyring_entry() {
                            if let Ok(password) = entry.get_password() {
                                settings.tmdb_api_key = password;
                                found_key = true;
                            }
                        }
                    }

                    // Finally, fallback to obfuscated json
                    if !found_key && !settings.tmdb_api_key.is_empty() {
                        if let Ok(decoded) = general_purpose::STANDARD.decode(&settings.tmdb_api_key) {
                            if let Ok(decoded_str) = String::from_utf8(decoded) {
                                settings.tmdb_api_key = decoded_str;
                            } else {
                                settings.tmdb_api_key = String::new(); // Bad UTF8
                            }
                        } else {
                             // Assuming it was cleartext or un-decodable
                             // We don't overwrite it here because they might have pasted it in.
                        }
                    }

                    loaded_settings = settings;
                },
                Err(e) => {
                    tracing::warn!(action = "load_settings", error = %e, "Corrupted settings.json detected. Resetting to defaults.");
                    needs_repair = true;
                }
            }
        },
        Err(e) => {
            tracing::warn!(action = "load_settings", error = %e, "Failed to read settings.json. Resetting to defaults.");
            needs_repair = true;
        }
    }

    if needs_repair {
        if let Err(e) = save_settings(&loaded_settings) {
            return Err(format!("Could not repair corrupted settings file: {}", e));
        }
    }

    Ok(loaded_settings)
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let settings_file = get_app_data_dir().join("settings.json");

    let mut key_stored_in_keyring = false;
    let mut is_env_var = false;

    // Check if the current key matches the environment variable exactly
    if let Ok(env_key) = std::env::var("TMDB_API_KEY") {
        if env_key == settings.tmdb_api_key && !settings.tmdb_api_key.is_empty() {
            is_env_var = true;
        }
    }

    // Attempt to store in Keyring only if it's not from the environment
    if !settings.tmdb_api_key.is_empty() {
        if !is_env_var {
            if let Ok(entry) = get_keyring_entry() {
                if entry.set_password(&settings.tmdb_api_key).is_ok() {
                    key_stored_in_keyring = true;
                }
            }
        }
    } else {
        // If empty, delete from keyring
        if let Ok(entry) = get_keyring_entry() {
            let _ = entry.delete_credential();
        }
        key_stored_in_keyring = true;
    }

    let mut settings_to_save = settings.clone();

    // If we successfully saved to keyring OR it is the environment variable, remove it from the JSON.
    // If keyring failed and it's not the environment variable, obfuscate it in the JSON.
    if key_stored_in_keyring || is_env_var {
        settings_to_save.tmdb_api_key = String::new();
    } else if !settings.tmdb_api_key.is_empty() {
        settings_to_save.tmdb_api_key = general_purpose::STANDARD.encode(&settings.tmdb_api_key);
    }

    if let Ok(json) = serde_json::to_string_pretty(&settings_to_save) {
        fs::write(settings_file, json).map_err(|e| e.to_string())
    } else {
        Err("Failed to serialize settings".to_string())
    }
}
