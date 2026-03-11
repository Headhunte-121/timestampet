import os
import sys
import subprocess
from pathlib import Path
import requests
import tempfile
from typing import List

VLC_HTTP_PASSWORD = "timemark"
VLC_HTTP_PORT = 8080

def get_vlc_path() -> str:
    """Attempts to auto-detect VLC path across different operating systems."""
    if sys.platform == "win32":
        paths = [
            r"C:\Program Files\VideoLAN\VLC\vlc.exe",
            r"C:\Program Files (x86)\VideoLAN\VLC\vlc.exe"
        ]
        for p in paths:
            if Path(p).exists():
                return p
        return ""
    elif sys.platform == "darwin":
        path = "/Applications/VLC.app/Contents/MacOS/VLC"
        if Path(path).exists():
            return path
        return ""
    else: # Linux
        try:
            # `which` returns 0 if found
            res = subprocess.run(["which", "vlc"], capture_output=True, text=True)
            if res.returncode == 0:
                return res.stdout.strip()
            return ""
        except Exception:
            return ""

def parse_time_input(time_str: str) -> int:
    """
    Parses flexible time input into total seconds.
    Supports formats like:
    - 45 (seconds)
    - 12:10 (MM:SS)
    - 01:15:30 (HH:MM:SS)
    """
    parts = time_str.split(":")
    total_seconds = 0

    try:
        if len(parts) == 1:
            total_seconds = int(parts[0])
        elif len(parts) == 2:
            minutes, seconds = int(parts[0]), int(parts[1])
            total_seconds = (minutes * 60) + seconds
        elif len(parts) == 3:
            hours, minutes, seconds = int(parts[0]), int(parts[1]), int(parts[2])
            total_seconds = (hours * 3600) + (minutes * 60) + seconds
        return total_seconds
    except ValueError:
        return 0

def format_time_display(total_seconds: int) -> str:
    """Formats total seconds nicely into HH:MM:SS or MM:SS."""
    hours, remainder = divmod(total_seconds, 3600)
    minutes, seconds = divmod(remainder, 60)

    if hours > 0:
        return f"{hours:02d}:{minutes:02d}:{seconds:02d}"
    else:
        return f"{minutes:02d}:{seconds:02d}"

def play_in_vlc(vlc_path: str, file_path: str, start_time: int = 0) -> None:
    """Launches VLC to play a specific file, optionally starting at a given time."""
    if not os.path.exists(vlc_path) and vlc_path != "vlc":
        # Check if vlc_path is literally just 'vlc' in which case the OS might know how to run it
        print(f"Error: VLC path '{vlc_path}' not found.")
        return

    if not os.path.exists(file_path):
         print(f"Error: File path '{file_path}' not found.")
         return

    # Add HTTP interface arguments so we can query the time later
    command = [
        vlc_path,
        file_path,
        "--extraintf", "http",
        f"--http-password={VLC_HTTP_PASSWORD}"
    ]
    if start_time > 0:
        command.append(f"--start-time={start_time}")

    try:
        # We use Popen instead of run so it runs asynchronously in the background
        subprocess.Popen(command)
        print(f"Launched VLC for '{file_path}' starting at {start_time}s.")
    except Exception as e:
        print(f"Failed to launch VLC: {e}")

def get_current_vlc_time() -> int:
    """
    Attempts to fetch the current playback time from VLC via its local HTTP API.
    Returns the time in total seconds, or -1 if it cannot connect.
    """
    url = f"http://localhost:{VLC_HTTP_PORT}/requests/status.json"
    try:
        # Basic auth: username is blank, password is the one we set via CLI
        response = requests.get(url, auth=("", VLC_HTTP_PASSWORD), timeout=1.0)
        response.raise_for_status()
        data = response.json()

        # VLC returns time in seconds
        return int(data.get("time", -1))
    except (requests.exceptions.RequestException, ValueError, KeyError):
        return -1

def generate_highlight_playlist(vlc_path: str, scenes: List[dict]) -> str:
    """
    Generates an .m3u playlist and launches VLC with it.
    scenes format: [{'file': 'C:/video.mp4', 'start': 120, 'end': 150, 'title': 'My Note'}]
    """
    if not os.path.exists(vlc_path) and vlc_path != "vlc":
        print(f"Error: VLC path '{vlc_path}' not found.")
        return ""

    if not scenes:
        return ""

    # Create a temporary m3u file
    fd, temp_path = tempfile.mkstemp(suffix=".m3u", prefix="timemark_highlights_")

    try:
        with os.fdopen(fd, 'w', encoding='utf-8') as f:
            f.write("#EXTM3U\n")
            for scene in scenes:
                file_path = scene.get('file', '')
                start = scene.get('start', 0)
                end = scene.get('end', 0)
                title = scene.get('title', 'Highlight')

                if not file_path or not os.path.exists(file_path):
                    continue

                f.write(f"#EXTINF:-1,{title}\n")
                f.write(f"#EXTVLCOPT:start-time={start}\n")
                if end > start:
                    f.write(f"#EXTVLCOPT:stop-time={end}\n")
                f.write(f"{file_path}\n")

    except Exception as e:
        print(f"Failed to write m3u: {e}")
        return ""

    # Launch VLC with the playlist
    command = [
        vlc_path,
        temp_path,
        "--extraintf", "http",
        f"--http-password={VLC_HTTP_PASSWORD}"
    ]

    try:
        subprocess.Popen(command)
        print(f"Launched VLC with highlight playlist: {temp_path}")
        return temp_path
    except Exception as e:
        print(f"Failed to launch VLC playlist: {e}")
        return ""

if __name__ == "__main__":
    print(f"Auto-detected VLC path: {get_vlc_path()}")

    # Test time parsing
    print(f"parse '45': {parse_time_input('45')} seconds")
    print(f"parse '12:10': {parse_time_input('12:10')} seconds")
    print(f"parse '01:15:30': {parse_time_input('01:15:30')} seconds")

    # Test formatting
    print(f"format 45: {format_time_display(45)}")
    print(f"format 730: {format_time_display(730)}")
    print(f"format 4530: {format_time_display(4530)}")
