# Local Development Guide

This guide walks you through setting up, running, debugging, and compiling **WatchMark Media Tracker** from source code.

---

## 🛠️ Prerequisites

Before you begin, ensure you have the following installed on your machine:

1. **Node.js**: `v18.0.0` or higher (Recommended: Node 20 LTS) — [nodejs.org](https://nodejs.org/)
2. **Rust Toolchain**: Latest stable Rust compiler and Cargo — [rustup.rs](https://rustup.rs/)
   ```bash
   rustc --version
   cargo --version
   ```
3. **VLC Media Player**: Installed in its default directory (or custom path) — [videolan.org/vlc](https://www.videolan.org/vlc/)
4. **Microsoft C++ Build Tools** (Windows only): Ensure the "Desktop development with C++" workload is installed via Visual Studio Installer.
5. **TMDB API Key**: Free account required from [The Movie Database](https://www.themoviedb.org/documentation/api).

---

## 🚀 Setting Up the Repository

```bash
# 1. Clone the repository
git clone https://github.com/Headhunte-121/timestampet.git
cd timestampet

# 2. Enter the Tauri workspace
cd watchmark-tauri

# 3. Install frontend dependencies
npm install
```

---

## 💻 Running the Development Environment

Start the development environment with live Hot Module Replacement (HMR) for React and incremental compilation for Rust:

```bash
cd watchmark-tauri
npm run tauri dev
```

This command will:
1. Start the Vite frontend server on `http://127.0.0.1:1420`.
2. Compile the Rust backend using `cargo`.
3. Launch the native desktop window connecting to the Vite dev server with DevTools enabled.

---

## 🔍 Debugging & Inspection

### 1. Inspecting Application Logs
WatchMark writes structured, multi-level diagnostic logs using `tracing`:
* **Windows**: `%LOCALAPPDATA%\WatchMark\watchmark.log`
* **macOS**: `~/Library/Application Support/com.WatchMark.WatchMark/watchmark.log`
* **Linux**: `~/.local/share/WatchMark/watchmark.log`

You can monitor logs live in PowerShell:
```powershell
Get-Content -Path "$env:LOCALAPPDATA\WatchMark\watchmark.log" -Wait -Tail 50
```

### 2. Inspecting the SQLite Database
The database file is located at:
`%LOCALAPPDATA%\WatchMark\watchmark.db`

You can inspect the database directly using any SQLite viewer (e.g. [DB Browser for SQLite](https://sqlitebrowser.org/)) or the SQLite CLI:
```bash
sqlite3 "%LOCALAPPDATA%\WatchMark\watchmark.db"
```
Helpful diagnostic queries:
```sql
-- Check total tracked media
SELECT id, title, type, user_rating, total_episodes, completed_eps FROM Media;

-- Check latest watch history
SELECT id, episode_id, session_id, duration, completion_ratio FROM History ORDER BY id DESC LIMIT 10;

-- Check unmatched triage files
SELECT id, file_path, parsed_series FROM Unmatched_Files;
```

### 3. Frontend Webview DevTools
To inspect the React DOM and console in development mode, right-click anywhere within the WatchMark window and select **Inspect Element** (or press `F12` / `Ctrl + Shift + I`).

---

## 🧪 Verification & Typechecking

Always verify your changes before submitting code:

```bash
# 1. Check Rust compilation and type-safety
cd watchmark-tauri/src-tauri
cargo check

# 2. Run unit and integration tests
cargo test

# 3. Typecheck React/TypeScript frontend
cd ..
npx tsc --noEmit

# 4. Verify static production bundle
npm run build
```

---

## 📦 Packaging Standalone Release Executables

To build an optimized, standalone release executable with embedded webview assets:

```bash
cd watchmark-tauri
npm run tauri build
```

### Build Outputs:
* **Standalone Portable Executable**:
  `watchmark-tauri/src-tauri/target/release/watchmark-tauri.exe`
* **NSIS Windows Installer**:
  `watchmark-tauri/src-tauri/target/release/bundle/nsis/WatchMark_1.2.0_x64-setup.exe`

The release binary embeds all React, CSS, and SQLite logic directly into the `.exe`. It requires zero external dependencies (no Node, no npm, no Vite) and runs on any standard Windows 10/11 machine.
