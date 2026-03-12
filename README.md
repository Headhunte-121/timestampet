# WatchMark Media Tracker

WatchMark is a pure-Python desktop application designed to bridge the gap between online media metadata (what exists in the world) and local file storage (what you actually have downloaded).

It acts as a **Desktop Media Tracking Diary** to track your watch progress, completion statuses, and rewatch counts.

## Features

*   **Universal Search (TMDB):** Type a Movie or TV Show name to search The Movie Database (TMDB). Clicking "+ Add to Tracker" automatically pulls down:
    *   Show Synopsis, Genres, and Release Dates.
    *   High-quality Poster Art and Banners.
    *   Complete Season/Episode structure (names, episode summaries, runtime).
*   **Granular Watch Tracking:** Track your progress at the Show, Season, and Episode levels.
*   **Watch Counts & History:** Keeps an exact count of how many times you have watched a specific episode or movie, along with a historical log of when you completed it.
*   **Smart Media Scanner:** Point the app to your local media folders (e.g., `D:\TV Shows`). It uses regex to parse names (e.g., `S01E01`) and automatically links the local files to your fetched online library.
*   **Persistent Unmatched Files Inbox:** When files fail to auto-match, they are intelligently grouped by their extracted series name in a persistent Inbox UI. You can search TMDB for the group once, and WatchMark will magically link all valid SxxExx files in that group to the selected show.
*   **Direct Playback:** Clicking "Play" on a tracked episode launches the linked local file in VLC automatically.
*   **Smart Playhead Tracking (VLC Heartbeat):** No need to manually click "Mark as Watched"! When WatchMark launches VLC, it silently connects to VLC's local HTTP API in the background. It polls the playhead position every 5 seconds. If you close VLC after watching 90% or more of the video, WatchMark automatically logs it as "Completed" and increments your watch count. If you close it earlier, it remembers exactly where you left off (in precise seconds, not just a vague percentage).
*   **Missing Episodes View:** A filter that shows episodes you haven't watched, which are also missing from your hard drive, helping you figure out what to download next.
*   **Offline Mode:** Uses local SQLite storage and caches posters locally (`Pillow` and `requests`) so the app is fully functional even if the TMDB API is unreachable or you lose internet access.
*   **Persistent State:** Saves your library database (`watchmark.db`) and preferences to your OS-specific application data directory (`%APPDATA%` on Windows, `~/.config` on Linux, `~/Library/Application Support` on macOS).

## Requirements

*   **Python 3.10+**
*   **VLC Media Player** installed on your system.
*   **TMDB API Key** (Free, required to search and add new shows).
*   Python Packages:
    *   `customtkinter`
    *   `requests`
    *   `Pillow`

## Installation

1.  Clone this repository.
2.  Install the required Python packages:

    ```bash
    pip install customtkinter requests Pillow
    ```

3.  Run the application using the one-click scripts (see below).

## Usage

### One-Click Start

To avoid keeping a messy command prompt window open while you manage your media, use the provided launch scripts:

*   **Windows:** Double-click `Start_WatchMark.vbs`. This will launch the application entirely in the background without a flashing terminal window. (Alternatively, run `run.bat`).
*   **Linux / macOS:** Run the `run.sh` script to launch the app detached in the background.

### Initial Setup

1.  **Get a TMDB API Key:** Create a free account at [The Movie Database (TMDB)](https://www.themoviedb.org/) and generate an API Key.
2.  **Settings:** Open WatchMark, go to the `⚙️ Settings` tab in the sidebar, and paste your TMDB API Key. You can also verify or manually set your VLC Executable Path here.
3.  **Add Shows:** Go to the `🔍 Search` tab, search for a show you are watching, and click "+ Add to Tracker".
4.  **Scan Your Media:** Go to the `📺 TV Shows` or `🎬 Movies` tab and click `[📂 Scan Local Folder]`. Select the directory containing your video files. WatchMark will organize them and link them to your tracked shows automatically.
5.  **Watch:** Click on a show poster to view its episodes. Click `▶` to play a linked episode. WatchMark will monitor your VLC session and automatically mark the episode as watched when you finish!

## Architecture & Data Storage

WatchMark uses a pure Python stack with a `customtkinter` UI.

All user data is stored in a relational `watchmark.db` SQLite database in your system's application data folder, ensuring fast queries and reliable history tracking. Poster images are downloaded once and cached locally to provide a snappy, offline-first experience.
