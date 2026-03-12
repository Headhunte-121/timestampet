import os
import sys
from pathlib import Path

def get_app_data_dir(app_name="WatchMark"):
    """
    Returns the application data directory alongside the application source.
    """
    # Path(__file__) is timemark/config.py
    # Path(__file__).parent is timemark/
    # Path(__file__).parent.parent is the root directory containing run.py
    base_dir = Path(__file__).parent.parent

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
