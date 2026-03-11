import json
from pathlib import Path
from dataclasses import dataclass, field, asdict
from typing import List, Dict, Optional
import shutil
from .config import LIBRARY_FILE, SETTINGS_FILE

@dataclass
class Timestamp:
    time_seconds: int
    display_time: str
    description: str

@dataclass
class Episode:
    title: str
    season: str
    series: str
    file_path: Optional[str] = None
    timestamps: List[Timestamp] = field(default_factory=list)

@dataclass
class Season:
    number: str
    episodes: Dict[str, Episode] = field(default_factory=dict)

@dataclass
class Series:
    name: str
    seasons: Dict[str, Season] = field(default_factory=dict)

@dataclass
class Library:
    series: Dict[str, Series] = field(default_factory=dict)
    unmatched: Dict[str, Episode] = field(default_factory=dict)

class DataManager:
    def __init__(self):
        self.settings = self.load_settings()
        self.library = self.load_library()

    def load_library(self) -> Library:
        if not LIBRARY_FILE.exists():
            return Library()
        try:
            with open(LIBRARY_FILE, 'r', encoding='utf-8') as f:
                data = json.load(f)

            lib = Library()

            for series_name, series_data in data.get("series", {}).items():
                series = Series(name=series_name)
                for season_num, season_data in series_data.get("seasons", {}).items():
                    season = Season(number=season_num)
                    for ep_key, ep_data in season_data.get("episodes", {}).items():
                        timestamps = [Timestamp(**ts) for ts in ep_data.get("timestamps", [])]
                        episode = Episode(
                            title=ep_data["title"],
                            season=ep_data["season"],
                            series=ep_data["series"],
                            file_path=ep_data.get("file_path"),
                            timestamps=timestamps
                        )
                        season.episodes[ep_key] = episode
                    series.seasons[season_num] = season
                lib.series[series_name] = series

            for ep_key, ep_data in data.get("unmatched", {}).items():
                timestamps = [Timestamp(**ts) for ts in ep_data.get("timestamps", [])]
                episode = Episode(
                    title=ep_data["title"],
                    season=ep_data["season"],
                    series=ep_data["series"],
                    file_path=ep_data.get("file_path"),
                    timestamps=timestamps
                )
                lib.unmatched[ep_key] = episode

            return lib
        except Exception as e:
            print(f"Error loading library: {e}")
            return Library()

    def save_library(self):
        try:
            with open(LIBRARY_FILE, 'w', encoding='utf-8') as f:
                json.dump(asdict(self.library), f, indent=4)
        except Exception as e:
            print(f"Error saving library: {e}")

    def load_settings(self) -> dict:
        default_settings = {
            "vlc_path": "",
            "window_geometry": "1000x700",
            "window_position": "+100+100"
        }
        if not SETTINGS_FILE.exists():
            return default_settings
        try:
            with open(SETTINGS_FILE, 'r', encoding='utf-8') as f:
                settings = json.load(f)
                # merge with defaults for any missing keys
                for key, value in default_settings.items():
                    if key not in settings:
                        settings[key] = value
                return settings
        except Exception as e:
            print(f"Error loading settings: {e}")
            return default_settings

    def save_settings(self):
        try:
            with open(SETTINGS_FILE, 'w', encoding='utf-8') as f:
                json.dump(self.settings, f, indent=4)
        except Exception as e:
            print(f"Error saving settings: {e}")

    def export_library(self, dest_path: Path) -> bool:
        try:
            self.save_library() # Ensure latest is saved
            shutil.copy2(LIBRARY_FILE, dest_path)
            return True
        except Exception as e:
            print(f"Error exporting library: {e}")
            return False

    def import_library(self, src_path: Path) -> bool:
        try:
            shutil.copy2(src_path, LIBRARY_FILE)
            self.library = self.load_library()
            return True
        except Exception as e:
            print(f"Error importing library: {e}")
            return False

if __name__ == "__main__":
    dm = DataManager()
    print("Initial Library:", dm.library)

    # Test saving some data
    dm.library.series["Breaking Bad"] = Series("Breaking Bad")
    dm.library.series["Breaking Bad"].seasons["1"] = Season("1")
    ep = Episode("Pilot", "1", "Breaking Bad", "C:/video.mp4")
    ep.timestamps.append(Timestamp(120, "02:00", "Start of action"))
    dm.library.series["Breaking Bad"].seasons["1"].episodes["1"] = ep

    dm.save_library()

    dm2 = DataManager()
    print("Loaded Library:", dm2.library)
