import os
import sys
from pathlib import Path

def get_app_data_dir(app_name="WatchMark"):
    """
    Returns the appropriate application data directory based on the OS.
    Windows: %APPDATA%/WatchMark
    Mac: ~/Library/Application Support/WatchMark
    Linux: ~/.config/WatchMark
    """
    home = Path.home()
    if sys.platform == "win32":
        app_data = os.environ.get("APPDATA")
        if app_data:
            base_dir = Path(app_data)
        else:
            base_dir = home / "AppData" / "Roaming"
    elif sys.platform == "darwin":
        base_dir = home / "Library" / "Application Support"
    else:  # Linux and other Unix-like
        config_home = os.environ.get("XDG_CONFIG_HOME")
        if config_home:
            base_dir = Path(config_home)
        else:
            base_dir = home / ".config"

    app_dir = base_dir / app_name
    app_dir.mkdir(parents=True, exist_ok=True)
    return app_dir

APP_DATA_DIR = get_app_data_dir()
LIBRARY_FILE = APP_DATA_DIR / "library.json"
SETTINGS_FILE = APP_DATA_DIR / "settings.json"

if __name__ == "__main__":
    print(f"App Data Directory: {APP_DATA_DIR}")
    print(f"Library File: {LIBRARY_FILE}")
    print(f"Settings File: {SETTINGS_FILE}")
