import re
from pathlib import Path
from typing import List, Tuple, Optional, Any
from .data import DataManager

VIDEO_EXTENSIONS = {'.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.webm'}

def parse_filename(filename: str) -> Tuple[Optional[str], Optional[int], Optional[int]]:
    """
    Parses a filename to extract Series Name, Season, and Episode.
    Expects S01E01 or s1e1 format (case-insensitive).
    Returns (Series Name, Season, Episode) or (None, None, None) if parsing fails.
    """
    base_name = Path(filename).stem
    pattern = r"^(.*?)[ \.\-_]*[sS](\d{1,2})[ \.\-_]*[eE](\d{1,3})"
    match = re.search(pattern, base_name)
    if not match:
        return None, None, None

    raw_series = match.group(1)
    season_num = match.group(2)
    episode_num = match.group(3)

    series_name = re.sub(r'[\.\_]', ' ', raw_series).strip()
    series_name = re.sub(r' (19|20)\d{2}$', '', series_name).strip()

    if not series_name:
         return None, None, None

    return series_name, int(season_num), int(episode_num)

def scan_directory(directory: str, data_manager: DataManager) -> List[dict]:
    """
    Scans a directory for video files.
    Matches S01E01 files to the database (if the show is tracked).
    Returns a list of unmatched files for manual assignment.
    """
    root_path = Path(directory)
    if not root_path.is_dir():
        return []

    conn = data_manager.get_db_connection()
    cursor = conn.cursor()
    unmatched = []

    for file_path in root_path.rglob('*'):
        if file_path.is_file() and file_path.suffix.lower() in VIDEO_EXTENSIONS:
            series_name, season_num, episode_num = parse_filename(file_path.name)
            str_path = str(file_path)

            matched_ep_id = None

            if series_name and season_num is not None and episode_num is not None:
                # Try to find a tracked show with a very similar name
                # Simplistic match: compare lowercase, alphanumeric only
                safe_series = "".join(c for c in series_name.lower() if c.isalnum())

                cursor.execute("SELECT id, title FROM Media WHERE type='TV'")
                shows = cursor.fetchall()

                matched_media_id = None
                for show in shows:
                    safe_db_name = "".join(c for c in show['title'].lower() if c.isalnum())
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

            if matched_ep_id:
                # Upsert to Local_Files
                cursor.execute("""
                    INSERT INTO Local_Files (episode_id, file_path)
                    VALUES (?, ?)
                    ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path
                """, (matched_ep_id, str_path))
                conn.commit()
            else:
                unmatched.append({
                    "file_path": str_path,
                    "filename": file_path.name,
                    "parsed_series": series_name,
                    "parsed_season": season_num,
                    "parsed_episode": episode_num
                })

    conn.close()
    return unmatched
