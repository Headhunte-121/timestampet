# WatchMark Media Tracker

WatchMark is a pure-Python, desktop application designed to bridge the gap between online media metadata (what exists in the world) and local file storage (what you actually have downloaded).

It acts as a **Desktop Media Tracking Diary** to track your watch progress, completion statuses, and rewatch counts. Designed with a clean, dark-mode CustomTkinter interface, WatchMark acts as a minimalist alternative to bloated media servers like Plex or Jellyfin, specifically tailored for single-user offline desktop environments.

---

## 🚀 Core Features

*   **Universal Search (TMDB):** Type a Movie or TV Show name to search The Movie Database (TMDB). Clicking "+ Add to Tracker" automatically pulls down:
    *   Show Synopsis, Genres, and Release Dates.
    *   High-quality Poster Art and Banners.
    *   Complete Season/Episode structure (names, episode summaries, runtime).
*   **Granular Watch Tracking:** Track your progress at the Show, Season, and Episode levels.
*   **Watch Counts & History:** Keeps an exact count of how many times you have watched a specific episode or movie, along with a historical log of when you completed it.
*   **Smart Media Scanner:** Point the app to your local media folders (e.g., `D:\TV Shows`). It uses advanced regex to parse filenames (e.g., `S01E01` or `1x01`) and automatically links the local files to your fetched online library.
*   **Persistent Inbox (Triage):** When files fail to auto-match (due to weird naming conventions), they are intelligently grouped by their extracted series name in a persistent "Inbox" UI. You can search TMDB for the group once, and WatchMark will magically link all valid SxxExx files in that group to the selected show.
*   **Direct Playback:** Clicking "Play" on a tracked episode launches the linked local file directly in your system's VLC player.
*   **Smart Playhead Tracking (VLC Heartbeat):** No need to manually click "Mark as Watched"! When WatchMark launches VLC, it silently connects to VLC's local HTTP API in the background. It polls the playhead position every 5 seconds.
    *   If you close VLC after watching **90% or more** of the video, WatchMark automatically logs it as "Completed" and increments your watch count.
    *   If you close it earlier, it **remembers exactly where you left off** (in absolute seconds) so you can resume perfectly later.
*   **Missing Episodes View:** Easily view episodes you haven't watched that are missing from your hard drive, helping you figure out what to download next.
*   **Offline First Mode:** Uses local SQLite storage and caches posters locally (`Pillow` and `requests`) so the app remains fully functional, snappy, and beautiful even if the TMDB API is unreachable or you lose internet access.

---

## 📋 Requirements & Installation

### Requirements
*   **Python 3.10+**
*   **VLC Media Player** installed on your system.
*   **TMDB API Key** (Free, required to search and add new shows).

### Python Dependencies
*   `customtkinter` (Modern, dark-themed UI components)
*   `requests` (API communication and VLC polling)
*   `Pillow` (Image caching and processing)

### Installation Guide

1.  **Clone this repository:**
    ```bash
    git clone https://github.com/yourusername/timemark.git
    cd timemark
    ```

2.  **Install the required Python packages:**
    ```bash
    pip install customtkinter requests Pillow
    ```

3.  **Run the application:**
    Use one of the provided launch scripts to start the app cleanly in the background:
    *   **Windows:** Double-click `Start_WatchMark.vbs` (runs silently) or run `run.bat`.
    *   **Linux / macOS:** Run the `run.sh` script to launch the app detached in the background.
    *   **Direct CLI:** `python run.py`

---

## 📖 Usage Guide

### 1. Initial Setup
1.  **Get a TMDB API Key:** Create a free account at [The Movie Database (TMDB)](https://www.themoviedb.org/) and generate an API key from your profile settings.
2.  **Configure WatchMark:** Open WatchMark, click `⚙️ Settings` in the bottom left, and paste your TMDB API Key. You can also manually configure your VLC Executable Path if the app couldn't find it automatically.

### 2. Adding Shows to Your Library
1.  Go to the `🔍 Search` tab.
2.  Type the name of a show or movie you are watching (e.g., "The Sopranos") and hit Enter.
3.  Click the **"+ Add to Tracker"** button over the correct poster. WatchMark will download all metadata and seasons in the background.

### 3. Scanning Your Local Files
1.  Navigate to either the `📺 TV Shows` or `🎬 Movies` tab.
2.  Click the orange `[📂 Scan Local Folder]` button in the top right.
3.  Select the root directory containing your video files. WatchMark will recursively scan the folder, match the files to the shows in your library, and link them.

### 4. Managing Unmatched Files (The Inbox)
If WatchMark finds media files it cannot map to a show currently in your library, it places them in the **Inbox** tab.
1.  Click the `Inbox` tab in the sidebar.
2.  Select a group of unmatched files.
3.  Click "Search TMDB" to look up the correct show. Once added, the entire group of files will be instantly assigned to the correct seasons and episodes, clearing your inbox.

### 5. Watching Media
1.  Click on any Show or Movie poster in your library.
2.  Browse the seasons and episodes. Episodes with a linked local file will have a bright orange `▶` Play button.
3.  Click Play. VLC will open the file. WatchMark will monitor your progress automatically.

---

## 🏗 Technical Architecture & Data Storage

WatchMark uses a pure Python stack, completely avoiding web-framework overhead like Electron or CEF.

### Database Schema (SQLite)
All user data is stored in a relational `watchmark.db` SQLite database located in your system's application data folder (e.g., `%APPDATA%\WatchMark` on Windows).

*   **Media:** Stores root metadata for Movies and TV Shows (TMDB ID, title, synopsis, poster paths, ratings).
*   **Episodes:** Stores granular data for every episode of every tracked show (Season #, Ep #, runtime, watch status, absolute resume position in seconds, watch count).
*   **Local_Files:** A mapping table linking a specific `episode_id` to an absolute file path on your local drive.
*   **History:** A chronological log of every time an episode is marked as "Completed".
*   **Unmatched_Files:** A staging table for files discovered during a scan that could not be linked, grouped intelligently by regex heuristics.

### Local Image Caching
To ensure instant load times and offline support, all poster art (`poster_path`, `backdrop_path`, `still_path`) is downloaded once via the `requests` library and saved to an `images` directory alongside the database. The CustomTkinter UI loads these cached `.jpg` files lazily using threading to prevent the main UI loop from freezing.

### VLC Integration
WatchMark communicates with VLC without needing any complicated plugins. It uses python's `subprocess` to launch the VLC executable with a command-line flag enabling its local HTTP interface (`--extraintf=http`). WatchMark then uses `requests.get()` to silently poll `http://localhost:8080/requests/status.xml` to parse the `length` and `time` nodes, determining exactly how far you've watched.

---

## 🛠 Troubleshooting & FAQ

**Q: The app crashes silently when I click a specific show.**
A: Ensure your database isn't corrupted or missing fields from an old version. WatchMark relies on strict Dictionary key access for SQLite Rows (e.g., `row['vote_average']`). If a migration failed, backing up and deleting the `watchmark.db` file will force a clean schema rebuild.

**Q: VLC opens, but my progress isn't being saved.**
A: WatchMark polls VLC's HTTP interface on port `8080`.
1. Ensure no other application on your machine is using port 8080.
2. Check your VLC preferences: Go to `Tools > Preferences > Show settings: All > Interfaces > Main interfaces`. Ensure "Web" is checked.
3. Ensure the password under `Main interfaces > Lua` matches what WatchMark expects (defaults to `watchmark`).

**Q: A show isn't being matched during a folder scan.**
A: The scanner uses Regex to look for patterns like `S01E05`, `1x05`, or `Season 1 Episode 5`. If your file is named something completely ambiguous (e.g., `The.Show.Part.5.mkv`), the regex will fail. You can manually assign these files from the **Inbox** tab using the "Advanced / Manual Match" toggle.
