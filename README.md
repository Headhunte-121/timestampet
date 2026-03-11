# TimeMark / VLC-Bookmarker

TimeMark is a lightweight, modern desktop application designed to organize video files (like TV Shows and Anime) by Series, Season, and Episode. It is built as a highly-optimized "Highlight Reel Generator", allowing users to quickly log scenes, seamlessly skip "filler", and export stitched scenes losslessly.

## Features

*   **Automated Library Generation:** Recursively scans your media folders and automatically parses standard TV show filenames (e.g., `S01E01`) to build a clean Series > Season > Episode hierarchy.
*   **VLC Highlight Reel Generation:** Automatically connects to VLC's local HTTP API to fetch precise playback positions. Plays your saved highlights seamlessly, skipping filler by generating temporary `.m3u` playlists with pre-caching enabled.
*   **"Rapid-Cut" Logging Mode:** Watch complex sequences with rapid cuts? Enable Rapid-Cut Mode and use a "dead man's switch" global hotkey (Hold `Alt` to capture, Release to stop) to seamlessly log micro-segments without typing.
*   **Seamless FFmpeg Export:** Extracted a multi-segment scene? Click the `[🎬 Export Seamless]` button, and TimeMark silently commands `ffmpeg` to extract and stitch your 2-second clips into a brand new, uninterrupted `.mp4` video file *losslessly* and *instantly* via stream-copy.
*   **Global Hotkeys & Quick Tags:** Log scenes entirely hands-free in the background. Press `Ctrl+Shift+[` to log Start, `Ctrl+Shift+]` to log End. Easily tag scenes using custom pre-set buttons to skip typing descriptions.
*   **Filler Filtration:** Toggle `[👁️ Show Only Highlighted]` to instantly hide any empty seasons or episodes, allowing you to focus purely on your library of best moments.
*   **Manual Overrides & Editing:** Did the regex miss a weird filename? TimeMark dumps unmatched files into a special category. You can easily rename them in the UI to an `S01E01` format, and the app will re-parse and automatically place them into your library hierarchy.
*   **Persistent State:** Saves your library, custom VLC paths, and window geometry automatically to your OS-specific application data directory (`%APPDATA%` on Windows, `~/.config` on Linux, `~/Library/Application Support` on macOS) so you never lose your data or preferences.

## Requirements

*   **Python 3.8+**
*   **VLC Media Player** installed on your system.
*   **FFmpeg** installed and accessible on your system's PATH (required for the Seamless Export feature).
*   Python Packages:
    *   `customtkinter`
    *   `requests`
    *   `keyboard`
    *   `packaging`
    *   `darkdetect`

## Installation

1.  Clone this repository.
2.  Install the required Python packages:

    ```bash
    pip install customtkinter requests keyboard
    ```

3.  Run the application:

    ```bash
    python run.py
    ```

## Usage

1.  **Scan Your Media:** Click the primary `[📂 Scan Media Folder]` button on the left pane and select the directory containing your video files. TimeMark will organize them automatically.
2.  **Log a Scene (UI):** Watch a video in VLC. When a scene starts, click `[ Get Start Time ]` to ping the VLC server and log the exact second. Click `[ Get End Time ]` when it finishes. Add tags, and hit `Save Highlight`.
3.  **Log a Scene (Hotkeys):** With VLC open, just press `Ctrl+Shift+[` to log the start time in the background, and `Ctrl+Shift+]` to log the end time.
4.  **Rapid-Cut Sequence Logging:** Enable the `[⚡ Enable Rapid-Cut Mode]` toggle. While watching VLC, hold the `Alt` key down to capture exactly when the good scene flashes on screen, and release `Alt` when it cuts away. Repeat. Save the multi-segment sequence into one clean block.
5.  **Watch the Highlight Reel:** Click `[🎬 Play Highlight Reel]` on an episode to automatically launch VLC with a seamless, pre-cached playlist that plays only your saved clips and skips all the filler.
6.  **Export a Scene:** Click `[🎬 Export Seamless]` next to any saved timestamp to instantly extract and merge that specific scene into a permanent video file on your hard drive.
7.  **Edit Unmatched Files:** If a file ends up in the `[?] Unmatched Files` category, select it and click `✎ Edit Name`. Rename it using the `S01E01` format (e.g., `Breaking Bad S01E01`), and TimeMark will instantly re-parse and move it!

## Architecture & Data Storage

TimeMark is built using a pure Python stack. It relies on standard libraries for file parsing (`pathlib`, `re`), safe system execution (`subprocess`), HTTP requests to VLC's local API (`requests`), and object serialization (`dataclasses`, `json`).

The UI is driven by `customtkinter` and standard `tkinter.ttk` elements.

All user data is stored in a simple, portable `library.json` file in your system's application data folder, separating application logic from user state.
