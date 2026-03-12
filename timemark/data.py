import os
import json
import sqlite3
from pathlib import Path
from dataclasses import dataclass, field, asdict
from typing import List, Dict, Optional, Tuple, Callable, Any
import shutil
import threading
import queue
from .config import APP_DATA_DIR, SETTINGS_FILE

DB_FILE = APP_DATA_DIR / "watchmark.db"
POSTER_CACHE_DIR = APP_DATA_DIR / "cache" / "posters"
POSTER_CACHE_DIR.mkdir(parents=True, exist_ok=True)

@dataclass
class Media:
    id: int
    tmdb_id: str
    type: str # 'TV' or 'Movie'
    title: str
    synopsis: str
    poster_path: str
    backdrop_path: str
    total_episodes: int
    status: str # 'Plan to Watch', 'Watching', 'Completed'

@dataclass
class Episode:
    id: int
    media_id: int
    season_num: int
    ep_num: int
    title: str
    runtime: int
    still_path: str
    watch_count: int
    last_position: int # Absolute seconds
    status: str # 'Unwatched', 'Watching', 'Completed'

@dataclass
class LocalFile:
    id: int
    episode_id: int
    file_path: str

@dataclass
class HistoryEntry:
    id: int
    episode_id: int
    timestamp: str


class DataManager:
    def __init__(self):
        self.settings = self.load_settings()
        self.db_path = str(DB_FILE)
        self.write_queue = queue.Queue()
        self._start_write_worker()
        self._init_db()

    def _start_write_worker(self):
        def worker():
            conn = sqlite3.connect(self.db_path)
            conn.row_factory = sqlite3.Row
            while True:
                item = self.write_queue.get()
                if item is None:
                    break

                task, callback = item
                try:
                    task(conn)
                    conn.commit()
                    if callback:
                        callback()
                except Exception as e:
                    print(f"DB Write Queue Error: {e}")
                    conn.rollback()
                finally:
                    self.write_queue.task_done()
            conn.close()

        self.worker_thread = threading.Thread(target=worker, daemon=True)
        self.worker_thread.start()

    def submit_write_task(self, task: Callable[[sqlite3.Connection], Any], callback: Optional[Callable] = None):
        self.write_queue.put((task, callback))

    def _init_db(self):
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.cursor()

            cursor.execute('''
                CREATE TABLE IF NOT EXISTS Media (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    tmdb_id TEXT UNIQUE,
                    type TEXT,
                    title TEXT,
                    synopsis TEXT,
                    poster_path TEXT,
                    backdrop_path TEXT,
                    total_episodes INTEGER,
                    status TEXT,
                    vote_average REAL DEFAULT 0.0,
                    user_rating INTEGER DEFAULT 0,
                    release_date TEXT
                )
            ''')

            cursor.execute('''
                CREATE TABLE IF NOT EXISTS Episodes (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    media_id INTEGER,
                    season_num INTEGER,
                    ep_num INTEGER,
                    title TEXT,
                    runtime INTEGER,
                    still_path TEXT,
                    overview TEXT,
                    watch_count INTEGER DEFAULT 0,
                    last_position INTEGER DEFAULT 0,
                    status TEXT DEFAULT 'Unwatched',
                    completed_date TEXT,
                    air_date TEXT,
                    FOREIGN KEY (media_id) REFERENCES Media (id),
                    UNIQUE(media_id, season_num, ep_num)
                )
            ''')

            cursor.execute('''
                CREATE TABLE IF NOT EXISTS Local_Files (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    episode_id INTEGER UNIQUE,
                    file_path TEXT UNIQUE,
                    FOREIGN KEY (episode_id) REFERENCES Episodes (id)
                )
            ''')

            cursor.execute('''
                CREATE TABLE IF NOT EXISTS History (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    episode_id INTEGER,
                    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (episode_id) REFERENCES Episodes (id)
                )
            ''')

            cursor.execute('''
                CREATE TABLE IF NOT EXISTS Unmatched_Files (
                    file_path TEXT PRIMARY KEY,
                    filename TEXT,
                    parsed_series TEXT,
                    parsed_season INTEGER,
                    parsed_episode INTEGER,
                    group_key TEXT
                )
            ''')

            # Migrations for new columns
            try:
                cursor.execute("ALTER TABLE Media ADD COLUMN backdrop_path TEXT")
            except sqlite3.OperationalError:
                pass # Column already exists

            try:
                cursor.execute("ALTER TABLE Episodes ADD COLUMN still_path TEXT")
            except sqlite3.OperationalError:
                pass # Column already exists

            try:
                cursor.execute("ALTER TABLE Episodes ADD COLUMN overview TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Media ADD COLUMN vote_average REAL DEFAULT 0.0")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Media ADD COLUMN user_rating INTEGER DEFAULT 0")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Media ADD COLUMN release_date TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Episodes ADD COLUMN completed_date TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Episodes ADD COLUMN air_date TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Media ADD COLUMN release_date TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Episodes ADD COLUMN overview TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Episodes ADD COLUMN completed_date TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE Episodes ADD COLUMN air_date TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE History ADD COLUMN is_legacy INTEGER DEFAULT 0")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE History ADD COLUMN session_id TEXT")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE History ADD COLUMN start_time DATETIME")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE History ADD COLUMN end_time DATETIME")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE History ADD COLUMN pause_count INTEGER DEFAULT 0")
            except sqlite3.OperationalError:
                pass

            try:
                cursor.execute("ALTER TABLE History ADD COLUMN completion_ratio REAL DEFAULT 0.0")
            except sqlite3.OperationalError:
                pass

            conn.commit()

    def get_db_connection(self):
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        return conn

    def load_settings(self) -> dict:
        home = str(Path.home())
        default_settings = {
            "vlc_path": "",
            "window_geometry": "1000x700",
            "window_position": "+100+100",
            "tmdb_api_key": ""
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
