# WatchMark Media Tracker (Tauri 2.0 Edition)

WatchMark is a high-performance, standalone desktop application designed to bridge the gap between online media metadata (what exists in the world) and local file storage (what you actually have downloaded).

Originally built in Python, **WatchMark has been completely re-architected in Tauri 2.0 (Rust + React + Tailwind CSS)**. It acts as a **Desktop Media Tracking Diary** to track your watch progress, completion statuses, and rewatch counts. Designed with a clean, cinematic "Plex / Apple TV" interface, WatchMark is a minimalist alternative to bloated media servers, specifically tailored for single-user offline desktop environments.

---

## 🚀 Core Features

*   **100% Pure Rust Backend:** Lightning-fast, self-contained executable utilizing `rusqlite`, `reqwest`, and `tokio` for zero-dependency Python-free performance.
*   **Cinematic UI (React + Tailwind):** Edge-to-edge backdrop banners, frosted glassmorphism overlays, and butter-smooth `framer-motion` page transitions.
*   **Universal Search (TMDB):** Type a Movie or TV Show name to search The Movie Database (TMDB).
*   **Smart Media Scanner:** Point the app to your local media folders. It uses advanced regex to parse filenames (e.g., `S01E01` or `1x01`) and automatically links the local files to your fetched online library.
*   **Persistent Inbox (Triage):** When files fail to auto-match, they are intelligently grouped by their extracted series name in a persistent "Inbox" UI for manual assignment.
*   **Smart Playhead Tracking (VLC Heartbeat):** The Rust backend silently connects to your external VLC player's local HTTP API in the background. It polls the playhead position every 5 seconds.
    *   If you close VLC after watching **90% or more** of the video, WatchMark automatically logs it as "Completed" and increments your watch count.
    *   If you close it earlier, it **remembers exactly where you left off** (in absolute seconds) so you can resume perfectly later.
*   **Binge-Session Auto-Grouping:** Rust algorithms group sequential episodes watched within a 6-hour window into unified "Binge Blocks" in your History timeline.
*   **Offline First Mode:** Uses local SQLite storage and caches posters locally so the app remains fully functional even if the TMDB API is unreachable.

---

## 📋 Requirements & Installation

### System Requirements
*   **Node.js 18+**
*   **Rust / Cargo**
*   **VLC Media Player** installed on your system.
*   **TMDB API Key** (Free, required to search and add new shows).

### Installation Guide

1.  **Clone this repository:**
    ```bash
    git clone https://github.com/yourusername/timemark.git
    cd timemark
    ```

2.  **Run the development environment:**
    *   **Windows:** Double-click the provided `run.bat` file in the root directory.
    *   **Direct CLI:**
        ```bash
        cd watchmark-tauri
        npm install
        npm run tauri dev
        ```

3.  **Build a Standalone Executable:**
    ```bash
    cd watchmark-tauri
    npm run tauri build
    ```
    The compiled `.exe` or `.app` will be located in `src-tauri/target/release/`.

---

## 📖 Usage Guide

### 1. Initial Setup
1.  **Get a TMDB API Key:** Create a free account at [The Movie Database (TMDB)](https://www.themoviedb.org/) and generate an API key from your profile settings.
2.  **Configure WatchMark:** Open WatchMark, click `⚙️ Settings` in the bottom left sidebar, and paste your TMDB API Key. You must also manually configure your VLC Executable Path (e.g., `C:\Program Files\VideoLAN\VLC\vlc.exe`).

### 2. Adding Shows to Your Library
1.  Go to the `🔍 Search` tab.
2.  Type the name of a show or movie you are watching (e.g., "The Sopranos") and hit Enter.
3.  Click the **"+ Add to Tracker"** button. WatchMark will download all metadata and seasons via the Rust backend.

### 3. Scanning Your Local Files
1.  Navigate to the `Inbox` tab.
2.  Click `Scan Directory` and provide the absolute path to your local media. The Rust `walkdir` scanner will recursively parse and link files.
3.  Any unmatched files will appear in the Inbox group list for manual triaging.

### 4. Watching Media
1.  Click on any Show or Movie poster in your library or dashboard to open the cinematic deep-dive view.
2.  Episodes with a linked local file will have a bright orange `▶` Play button.
3.  Click Play. WatchMark will spawn VLC and monitor your progress automatically.

---

## 🏗 Technical Architecture & Data Storage

WatchMark 2.0 has completely abandoned Python and CustomTkinter in favor of a modern Tauri 2.0 architecture.

### Database Schema (SQLite)
All user data is stored in a relational `watchmark.db` SQLite database located in a portable `WatchMark` folder next to the executable. **The Rust application automatically preserves and migrates legacy databases from the Python version using safe `ALTER TABLE` execution.**

### VLC Integration
WatchMark communicates with VLC without needing any complicated plugins. It uses Rust's `std::process::Command` to launch the VLC executable with a command-line flag enabling its local HTTP interface (`--extraintf=http --http-password=watchmark`). WatchMark then spawns a background thread using `reqwest` to silently poll `http://127.0.0.1:8080/requests/status.json` to parse the `length` and `time` nodes. When the process closes, an event is emitted to the React frontend to refresh the UI.

---

## 🛠 Troubleshooting

**Q: VLC opens, but my progress isn't being saved.**
A: WatchMark polls VLC's HTTP interface on port `8080` using the password `watchmark`.
1. Ensure no other application on your machine is using port 8080.
2. If VLC warns you about HTTP interface security, ensure you have allowed it through your firewall, or check your VLC preferences under `Main interfaces > Lua` to ensure the password is not conflicting with an existing manual setup.
