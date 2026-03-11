# TimeMark / VLC-Bookmarker

TimeMark is a lightweight, modern desktop application designed to organize video files (like TV Shows and Anime) by Series, Season, and Episode. It allows users to save specific timestamps with notes and instantly launch them in VLC Media Player exactly at that timestamp.

## Features

*   **Automated Library Generation:** Recursively scans your media folders and automatically parses standard TV show filenames (e.g., `S01E01`) to build a clean Series > Season > Episode hierarchy.
*   **VLC Integration:** Instantly launches VLC Media Player via the command line, jumping straight to your saved timestamp.
*   **Flexible Time Input:** Enter timestamps in any standard format (`45`, `12:10`, or `01:15:30`), and TimeMark automatically converts and stores it as raw seconds for VLC while displaying it beautifully in the UI.
*   **Manual Overrides & Editing:** Did the regex miss a weird filename? TimeMark dumps unmatched files into a special category. You can easily rename them in the UI to an `S01E01` format, and the app will re-parse and automatically place them into your library hierarchy.
*   **Modern UI:** Built with `customtkinter`, featuring a clean, responsive dark mode interface with orange accents, complete with a collapsible navigation tree.
*   **Persistent State:** Saves your library, custom VLC paths, and window geometry automatically to your OS-specific application data directory (`%APPDATA%` on Windows, `~/.config` on Linux, `~/Library/Application Support` on macOS) so you never lose your data or preferences.
*   **Export/Import:** Easily back up your library to a `.json` file and restore it across devices.

## Requirements

*   **Python 3.8+**
*   **VLC Media Player** installed on your system.
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

1.  **Scan Your Media:** Click the primary `[📂 Scan Media Folder]` button on the left pane and select the directory containing your video files. TimeMark will do the rest.
2.  **Add a Timestamp:** Select an episode from the tree. At the bottom of the right pane, enter a time (e.g., `12:10`) and a description (e.g., `Cool explosion`), then click `Save Timestamp`.
3.  **Play a Timestamp:** Click the `▶` button next to any saved timestamp to instantly launch VLC at that exact moment.
4.  **Edit Unmatched Files:** If a file ends up in the `[?] Unmatched Files` category, select it and click `✎ Edit Name`. Rename it using the `S01E01` format (e.g., `Breaking Bad S01E01`), and TimeMark will automatically move it to the correct Series and Season folder!
5.  **Settings:** Click `⚙ Settings` to manually configure your VLC executable path if auto-detection fails, or to back up your library.

## Architecture & Data Storage

TimeMark is built using a pure Python stack. It relies on standard libraries for file parsing (`pathlib`, `re`), system execution (`subprocess`), and object serialization (`dataclasses`, `json`).

The UI is driven by `customtkinter` and standard `tkinter.ttk` elements.

All user data is stored in a simple, portable `library.json` file in your system's application data folder, separating application logic from user state.
