# TimeMark / VLC-Bookmarker

TimeMark is a lightweight, modern desktop application designed to organize video files (like TV Shows and Anime) by Series, Season, and Episode. It is built as a highly-optimized "Highlight Reel Generator", allowing users to quickly log scenes, seamlessly skip "filler", and export stitched scenes losslessly.

## Features

*   **Automated Library Generation:** Recursively scans your media folders and automatically parses standard TV show filenames (e.g., `S01E01`) to build a clean Series > Season > Episode hierarchy.
*   **VLC Highlight Reel Generation:** Automatically connects to VLC's local HTTP API to fetch precise playback positions. Plays your saved highlights seamlessly, skipping filler by generating temporary `.m3u` playlists with pre-caching enabled.
*   **"Rapid-Cut" Logging Mode & Segment Timeline:** Watch complex sequences with rapid cuts? Enable Rapid-Cut Mode and use a "dead man's switch" global hotkey (Hold `Alt` to capture, Release to stop) to seamlessly log micro-segments. Your captured segments instantly appear as visual "Pills" in a horizontal timeline.
*   **Pro-Editor Segment Management:** Did you hold the capture key for too long? You can instantly delete accidental micro-segments (e.g. 0.5s clips) by clicking the `✖` on a pill. Missed the start of the action? Use the Nudge buttons (`<`, `>`) on any pill to perfectly adjust the start/end times by 0.5 seconds without re-recording!
*   **Frame-Accurate FFmpeg Export:** Extracted a multi-segment scene? Click the `[🎬 Export Seamless]` button. TimeMark silently commands `ffmpeg` in the background to automatically name, extract, and stitch your clips into a brand new, uninterrupted `.mp4` video file. We use *smart re-encoding* (`libx264 -preset ultrafast`) to ensure 100% frame-perfect cuts and absolutely zero "keyframe mismatch" artifacts!
*   **"Ghost-Clipper" Mini-Mode:** Click `[🔲 Switch to Mini-Mode]` to shrink the app into a sleek, draggable, always-on-top overlay. The Mini-Bar glows with a red `🔴` REC indicator when you hold your global hotkey, giving you instant visual feedback without ever taking your eyes off the video.
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
4.  **Rapid-Cut Sequence Logging:** Enable the `[⚡ Enable Rapid-Cut Mode]` toggle. While watching VLC, hold the `Alt` key down to capture exactly when the good scene flashes on screen, and release `Alt` when it cuts away. Repeat.
5.  **Review & Nudge Segments:** Look at the visual pills that appear after logging segments. If you accidentally grabbed a 0.2s clip, click `✖` to delete it. If a clip needs to be slightly longer, click the `>` button to nudge the end time by 0.5s.
6.  **Export a Scene:** Click `[🎬 Export Seamless]` next to any saved timestamp (or directly from the Mini-Bar for pending segments) to instantly extract, stitch, and save that scene to your `Videos/TimeMark` folder. A toast notification will pop up when it's done.
7.  **Ghost-Clipper Overlay:** Click `[🔲 Switch to Mini-Mode]` to collapse the UI. The app will float over your video, giving you a live count of your captured segments and a direct export button while staying out of your way.
8.  **Watch the Highlight Reel:** Click `[🎬 Play Highlight Reel]` on an episode to automatically launch VLC with a seamless, pre-cached playlist that plays only your saved clips and skips all the filler.
9.  **Edit Unmatched Files:** If a file ends up in the `[?] Unmatched Files` category, select it and click `✎ Edit Name`. Rename it using the `S01E01` format (e.g., `Breaking Bad S01E01`), and TimeMark will instantly re-parse and move it!

## Architecture & Data Storage

TimeMark is built using a pure Python stack. It relies on standard libraries for file parsing (`pathlib`, `re`), safe system execution (`subprocess`), HTTP requests to VLC's local API (`requests`), and object serialization (`dataclasses`, `json`).

The UI is driven by `customtkinter` and standard `tkinter.ttk` elements.

All user data is stored in a simple, portable `library.json` file in your system's application data folder, separating application logic from user state.
