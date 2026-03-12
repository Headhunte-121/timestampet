import re
from pathlib import Path
from typing import List, Tuple, Optional, Any
from .data import DataManager

VIDEO_EXTENSIONS = {'.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.webm'}

def parse_filename(filename: str) -> Tuple[Optional[str], Optional[int], Optional[int]]:
    """
    Parses a filename to extract Name, Season, and Episode.
    Expects S01E01 for TV, or Title (Year) for Movies.
    Returns (Name, Season, Episode) for TV.
    Returns (Name, None, None) for Movies.
    Returns (None, None, None) if parsing fails.
    """
    base_name = Path(filename).stem

    # Try TV show format first: S01E01
    pattern_tv = r"^(.*?)[ \.\-_]*[sS](\d{1,2})[ \.\-_]*[eE](\d{1,3})"
    match_tv = re.search(pattern_tv, base_name)
    if match_tv:
        raw_series = match_tv.group(1)
        season_num = match_tv.group(2)
        episode_num = match_tv.group(3)

        series_name = re.sub(r'[\.\_]', ' ', raw_series).strip()
        series_name = re.sub(r' (19|20)\d{2}$', '', series_name).strip()

        if series_name:
            return series_name, int(season_num), int(episode_num)

    # Try Movie format: Title (Year) or just Title.Year
    # Matches strings ending with a 4-digit year in parens, brackets, or after a dot/space
    pattern_movie = r"^(.*?)[ \.\-_\[\(]*(19\d{2}|20\d{2})[\]\)]*"
    match_movie = re.search(pattern_movie, base_name)
    if match_movie:
        raw_movie = match_movie.group(1)
        movie_name = re.sub(r'[\.\_]', ' ', raw_movie).strip()
        if movie_name:
             return movie_name, None, None

    return None, None, None

def scan_directory(directory: str, data_manager: DataManager) -> int:
    """
    Scans a directory for video files.
    Matches S01E01 files to the database (if the show is tracked).
    Inserts unmatched files into Unmatched_Files table.
    Returns the number of new unmatched files found.
    """
    root_path = Path(directory)
    if not root_path.is_dir():
        return 0

    conn = data_manager.get_db_connection()
    cursor = conn.cursor()
    new_unmatched_count = 0

    for file_path in root_path.rglob('*'):
        if file_path.is_file() and file_path.suffix.lower() in VIDEO_EXTENSIONS:
            series_name, season_num, episode_num = parse_filename(file_path.name)
            str_path = str(file_path)

            matched_ep_id = None

            if series_name:
                safe_series = "".join(c for c in series_name.lower() if c.isalnum())

                if season_num is not None and episode_num is not None:
                    # TV Show Match
                    cursor.execute("SELECT id, title FROM Media WHERE type='TV'")
                    shows = cursor.fetchall()

                    matched_media_id = None
                    for show in shows:
                        safe_title = (show['title'] or 'Unknown Title').lower()
                        safe_db_name = "".join(c for c in safe_title if c.isalnum())
                        if safe_series == safe_db_name or safe_series in safe_db_name or safe_db_name in safe_series:
                            matched_media_id = show['id']
                            break

                    if matched_media_id:
                        cursor.execute("""
                            SELECT id FROM Episodes
                            WHERE media_id = ? AND season_num = ? AND ep_num = ?
                        """, (matched_media_id, season_num, episode_num))
                        ep = cursor.fetchone()
                        if ep:
                            matched_ep_id = ep['id']
                else:
                    # Movie Match
                    cursor.execute("SELECT id, title FROM Media WHERE type='Movie'")
                    movies = cursor.fetchall()

                    matched_media_id = None
                    for movie in movies:
                        safe_title = (movie['title'] or 'Unknown Title').lower()
                        safe_db_name = "".join(c for c in safe_title if c.isalnum())
                        if safe_series == safe_db_name:
                            matched_media_id = movie['id']
                            break

                    if matched_media_id:
                        # Movies only have 1 dummy episode (s=1, e=1)
                        cursor.execute("""
                            SELECT id FROM Episodes
                            WHERE media_id = ? AND season_num = 1 AND ep_num = 1
                        """, (matched_media_id,))
                        ep = cursor.fetchone()
                        if ep:
                            matched_ep_id = ep['id']

            if matched_ep_id:
                # Upsert to Local_Files
                import sqlite3
                try:
                    cursor.execute("""
                        INSERT INTO Local_Files (episode_id, file_path)
                        VALUES (?, ?)
                        ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path
                    """, (matched_ep_id, str_path))
                    conn.commit()
                except sqlite3.IntegrityError:
                    pass # File path already claimed by another episode
            else:
                # Determine group key for unmatched files
                group_key = series_name
                if not group_key:
                    # Fallback regex
                    fallback_match = re.search(r"^(.+?)(?=\.[sS]\d\d|\.\d{4})", file_path.stem)
                    if fallback_match:
                        group_key = fallback_match.group(1)
                    else:
                        group_key = file_path.stem

                # Clean group key
                if group_key:
                    group_key = re.sub(r'[\.\_]', ' ', group_key).strip()

                import sqlite3
                try:
                    cursor.execute("""
                        INSERT INTO Unmatched_Files (file_path, filename, parsed_series, parsed_season, parsed_episode, group_key)
                        VALUES (?, ?, ?, ?, ?, ?)
                    """, (str_path, file_path.name, series_name, season_num, episode_num, group_key))
                    conn.commit()
                    new_unmatched_count += 1
                except sqlite3.IntegrityError:
                    pass # Already in Unmatched_Files

    conn.close()
    return new_unmatched_count
