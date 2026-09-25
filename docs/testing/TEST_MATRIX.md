# Quality Assurance & Testing Matrix

This document outlines the systematic test cases, stress tests, and edge-case recovery verifications performed on the **WatchMark Media Tracker** architecture.

---

## 🧪 Testing Categories

1. [Permission & Storage Boundaries](#1-permission--storage-boundaries)
2. [Database Crash Resilience & Migrations](#2-database-crash-resilience--migrations)
3. [Filesystem & Scanner Stress Tests](#3-filesystem--scanner-stress-tests)
4. [Settings Corruption & Auto-Repair](#4-settings-corruption--auto-repair)
5. [VLC Telemetry & Playhead Synchronization](#5-vlc-telemetry--playhead-synchronization)

---

## 1. Permission & Storage Boundaries

| Test Scenario | Test Method | Expected Behavior | Verification Status |
| :--- | :--- | :--- | :--- |
| **Restricted User Profile** | Run WatchMark under a restricted child or guest account where `%LOCALAPPDATA%` write access is disabled. | App detects canary write failure, halts before UI render, and displays a native fatal error dialog explaining permission requirements. | ✅ **Pass** |
| **Windows Defender Sandbox** | Add WatchMark `.exe` to the "Blocked Apps" list in Windows Security. | Catches `std::io::Error` cleanly and displays the native dialog without silent hanging or white-screen freezes. | ✅ **Pass** |
| **The "Shadow File" Collision** | Manually create a file named `posters` (no extension) inside the `cache` folder. | JIT `ensure_directories()` detects that the path is a file, removes it safely, and recreates the intended directory structure. | ✅ **Pass** |
| **Zero-Byte Disk (Full Storage)** | Launch app on a partition with 0 bytes remaining. | `fs::create_dir_all` and SQLite errors are intercepted early in Rust; native dialog alerts the user with "No space left on device". | ✅ **Pass** |
| **Mid-Session Cache Wipe** | Delete `%LOCALAPPDATA%\WatchMark\cache\posters\` while browsing the Library. | Next image request dynamically checks and recreates the cache folder on the fly without crashing the UI. | ✅ **Pass** |

---

## 2. Database Crash Resilience & Migrations

| Test Scenario | Test Method | Expected Behavior | Verification Status |
| :--- | :--- | :--- | :--- |
| **Corrupted Migration (Power Cut)** | Forcibly terminate the process (`taskkill /F`) halfway through a database migration. | Transaction rollbacks cleanly on next boot via atomic `conn.unchecked_transaction()`. Re-running migration succeeds without deadlocks. | ✅ **Pass** |
| **Legacy Python DB Jump** | Open an original Version 1 database from the Python codebase with the Tauri 2.0 app. | `PRAGMA user_version` detects version `0`, initiates the evolutionary migration loop, and safely adds missing columns via `ALTER TABLE`. | ✅ **Pass** |
| **Hot Database Deletion** | Forcibly delete `watchmark.db` from Windows Explorer while a background directory scan is actively writing records. | Next call to `get_db_connection()` detects missing database, initializes fresh schema, and logs warning without thread panics. | ✅ **Pass** |
| **Duplicate Column Defense** | Manually execute an `ALTER TABLE` to add an existing column before running the app. | Migration logic catches `duplicate column name` error and safely swallows it, preventing boot failure. | ✅ **Pass** |
| **Cascading Delete Integrity** | Delete a show containing 100+ episodes and 200+ history records. | SQLite `ON DELETE CASCADE` purges all related rows atomically, leaving 0 orphaned records in `Episodes` and `History`. | ✅ **Pass** |

---

## 3. Filesystem & Scanner Stress Tests

| Test Scenario | Test Method | Expected Behavior | Verification Status |
| :--- | :--- | :--- | :--- |
| **Windows MAX_PATH (Long Paths)** | Place video files in a folder path exceeding 260 characters (e.g. 300+ characters). | Scanner normalizes paths using Windows verbatim prefixing (`\\?\`), traversing and reading files without string overflow. | ✅ **Pass** |
| **Circular Shortcut Loops** | Create two Windows `.lnk` files pointing to each other inside a scanned directory. | Scanner detects path recursion, tracks visited canonical paths, and breaks infinite loops cleanly. | ✅ **Pass** |
| **Massive Library Scan (10,000+ Files)** | Scan a media directory containing over 10,000 video and non-video files. | Batching mechanism emits files in chunks of 50 (`scan-match-batch`); UI remains fluid at 60 FPS without memory spikes. | ✅ **Pass** |
| **Unicode & Special Character Names** | Scan filenames containing emojis, Japanese kanji, Arabic script, and URL reserved characters (`?`, `%`, `#`). | Paths are percent-decoded and UTF-8 validated across the Tauri IPC bridge without string mangling. | ✅ **Pass** |

---

## 4. Settings Corruption & Auto-Repair

| Test Scenario | Test Method | Expected Behavior | Verification Status |
| :--- | :--- | :--- | :--- |
| **Syntax Corrupted `settings.json`** | Manually delete the last 10 characters of `settings.json` to produce broken JSON syntax. | Defensive deserialization detects parse error, logs warning, and writes a fresh default `settings.json` on boot. | ✅ **Pass** |
| **Floating-Point Dimension Bug** | Manually edit `settings.json` to set `"width": 1280.99`. | Custom Serde deserializer rounds and casts float to integer `1281`, preventing a type mismatch crash. | ✅ **Pass** |
| **Off-Screen Window Coordinates** | Set window coordinates to `x: -5000, y: -5000` (off-screen). | Window manager clamps bounds to the primary display area, preventing an invisible window. | ✅ **Pass** |
| **OS Credential Manager Locked** | Run app with the Windows Credential Manager service disabled. | API key storage falls back to Base64-obfuscated JSON storage without panicking. | ✅ **Pass** |
| **Rapid Save Toggle Spam** | Toggle a settings switch 20 times in 2 seconds. | Debounced 500ms write loop combines modifications, resulting in only 1 physical disk write. | ✅ **Pass** |

---

## 5. VLC Telemetry & Playhead Synchronization

| Test Scenario | Test Method | Expected Behavior | Verification Status |
| :--- | :--- | :--- | :--- |
| **Port 8080 Collision** | Bind a dummy HTTP server to port 8080 before launching playback in WatchMark. | Socket probe detects port 8080 is occupied, loops to 8081, and launches VLC successfully on the free port. | ✅ **Pass** |
| **Sudden VLC Crash Mid-Playback** | Forcibly terminate the VLC process (`taskkill /F /IM vlc.exe`) during playback. | Tokio async child handle catches process death immediately; polling breaks cleanly without zombie threads. | ✅ **Pass** |
| **Midnight Binge Crossover** | Play consecutive episodes starting at 11:30 PM and ending at 1:15 AM. | BingeBlock calculates date boundary difference across epochs and renders the `<Moon />` indicator in History. | ✅ **Pass** |
| **Show Switching Isolation** | Watch an episode of Show A, then immediately watch Show B within 5 minutes. | Show isolation logic detects `new_media_id != last_media_id` and mints a brand-new `session_id`. | ✅ **Pass** |
| **Exact Second Resumption** | Stop playback at second 1,427, close VLC, and click Play again. | Spawns VLC with `--start-time=1427`, jumping precisely to the stop point without drift. | ✅ **Pass** |
