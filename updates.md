
### Feature 16.13: Boot-Time Filesystem Guard
* Implemented `filesystem_guard.rs` with `boot_time_guard` for checking and verifying all necessary application directories.
* Moved `logging::init_tracing()` to the start of `main.rs` to allow filesystem errors to be logged early.
* Relocated `db::init_db()` into the Tauri `.setup()` hook explicitly so it only initializes after the filesystem is confirmed to be healthy.
* Removed deprecated `canary_check()` and `ensure_directories()` logic to avoid redundancies and potential race conditions.
* Configured `boot_time_guard` to perform idempotent directory creation and active probe writing to confirm disk write permissions.
* Connected `tauri_plugin_dialog` so a fatal GUI message is displayed bypassing the React frontend if filesystem integrity tests fail.