// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use std::fs;
use std::path::PathBuf;
use tauri::App;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

pub fn boot_time_guard(_app: &mut App, app_dir: &PathBuf) -> Result<(), String> {
    let required_paths = vec![
        app_dir.clone(),
        app_dir.join("db"),
        app_dir.join("cache").join("posters"),
        app_dir.join("cache").join("backdrops"),
        app_dir.join("cache").join("stills"),
    ];

    for path in required_paths {
        tracing::info!("[FS] 📁 Verifying directory: {:?}", path);

        // 1. Idempotent Creation
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(&path) {
                tracing::error!(action = "fs_guard_fail", path = ?path, error = %e, "Permission or I/O error during startup - Creation Failed");
                return Err(format!("Fatal Error: WatchMark lacks permission to create its data folders.\nPath: {:?}\nError: {}\nPlease check your Antivirus or Folder Permissions.", path, e));
            }
            tracing::info!("[FS] ✨ Created Directory: {:?}", path);
        } else if path.is_file() {
            // If a file exists where a directory should be, it's a conflict
            tracing::error!(action = "fs_guard_fail", path = ?path, "Conflict: File exists where directory was expected");
            return Err(format!("Fatal Error: WatchMark found a file where a directory was expected.\nPath: {:?}\nPlease remove the file and restart the app.", path));
        } else {
            tracing::info!("[FS] ✅ Verified Directory Exists: {:?}", path);
        }

        // 2. Functional Probe - Read (Execute/Search)
        if let Err(e) = fs::read_dir(&path) {
            tracing::error!(action = "fs_guard_fail", path = ?path, error = %e, "Permission or I/O error during startup - Read Probe Failed");
            return Err(format!("Fatal Error: WatchMark cannot read the required data folder.\nPath: {:?}\nError: {}\nPlease check your Antivirus or Folder Permissions.", path, e));
        }

        // 3. Functional Probe - Write
        let probe_file = path.join(".probe");
        if let Err(e) = fs::write(&probe_file, b"probe") {
            tracing::error!(action = "fs_guard_fail", path = ?path, error = %e, "Permission or I/O error during startup - Write Probe Failed");
            return Err(format!("Fatal Error: WatchMark lacks permission to write to its data folders.\nPath: {:?}\nError: {}\nPlease check your Antivirus or Folder Permissions.", path, e));
        } else {
            // Clean up the probe file immediately
            let _ = fs::remove_file(probe_file);
        }
    }

    Ok(())
}

pub fn execute_guard_and_exit_on_failure(app: &mut App, app_dir: &PathBuf) {
    if let Err(err_msg) = boot_time_guard(app, app_dir) {
        // Native Intercept: Trigger a native Windows message box bypassing React
        app.dialog()
            .message(err_msg)
            .kind(MessageDialogKind::Error)
            .title("WatchMark - Fatal Error")
            .blocking_show();

        std::process::exit(1);
    }
}
