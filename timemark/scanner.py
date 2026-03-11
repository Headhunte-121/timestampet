import re
from pathlib import Path
from typing import List, Tuple, Optional
from .data import DataManager, Episode, Series, Season

VIDEO_EXTENSIONS = {'.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.webm'}

def parse_filename(filename: str) -> Tuple[Optional[str], Optional[str], Optional[str]]:
    """
    Parses a filename to extract Series Name, Season, and Episode.
    Expects S01E01 or s1e1 format (case-insensitive).
    Returns (Series Name, Season, Episode) or (None, None, None) if parsing fails.
    """
    # Remove file extension
    base_name = Path(filename).stem

    # Pattern to find S01E01 or s1e1 format
    # Match something like S01E01, s1e1, S1.E1, S01-E01, etc.
    # Group 1: Series Name (everything before)
    # Group 2: Season Number
    # Group 3: Episode Number
    # The non-greedy capture for series name followed by some separator(s) and then the season/episode marker.
    pattern = r"^(.*?)[ \.\-_]*[sS](\d{1,2})[ \.\-_]*[eE](\d{1,2})"

    match = re.search(pattern, base_name)
    if not match:
        return None, None, None

    raw_series = match.group(1)
    season_num = match.group(2)
    episode_num = match.group(3)

    # Clean up series name: replace dots and underscores with spaces
    series_name = re.sub(r'[\.\_]', ' ', raw_series).strip()

    # Remove trailing years right before the season marker (e.g. "Dark Desire 2020")
    # Matches a space followed by a 4-digit year at the end of the string
    series_name = re.sub(r' (19|20)\d{2}$', '', series_name).strip()

    # Format numbers (e.g., "01" -> "1")
    season_num = str(int(season_num))
    episode_num = str(int(episode_num))

    # If series name is empty after cleaning, it's a failed parse
    if not series_name:
         return None, None, None

    return series_name, season_num, episode_num

def scan_directory(directory: str, data_manager: DataManager) -> None:
    """
    Recursively scans a directory for video files, parses them,
    and adds them to the data manager's library.
    """
    root_path = Path(directory)
    if not root_path.is_dir():
        return

    for file_path in root_path.rglob('*'):
        if file_path.is_file() and file_path.suffix.lower() in VIDEO_EXTENSIONS:
            series_name, season_num, episode_num = parse_filename(file_path.name)

            if series_name and season_num and episode_num:
                # Add to structured library

                # Check Series
                if series_name not in data_manager.library.series:
                    data_manager.library.series[series_name] = Series(name=series_name)
                series_obj = data_manager.library.series[series_name]

                # Check Season
                if season_num not in series_obj.seasons:
                    series_obj.seasons[season_num] = Season(number=season_num)
                season_obj = series_obj.seasons[season_num]

                # Check Episode - if exists update path, else create new
                if episode_num in season_obj.episodes:
                    season_obj.episodes[episode_num].file_path = str(file_path)
                else:
                    episode_title = f"Episode {episode_num}"
                    season_obj.episodes[episode_num] = Episode(
                        title=episode_title,
                        season=season_num,
                        series=series_name,
                        file_path=str(file_path)
                    )
            else:
                # Add to unmatched using raw filename (without extension)
                raw_title = file_path.stem
                if raw_title in data_manager.library.unmatched:
                    data_manager.library.unmatched[raw_title].file_path = str(file_path)
                else:
                    data_manager.library.unmatched[raw_title] = Episode(
                        title=raw_title,
                        season="?",
                        series="?",
                        file_path=str(file_path)
                    )

    # Save after scanning
    data_manager.save_library()

if __name__ == "__main__":
    # Test cases
    test_filenames = [
        "Dark.Desire.2020.S01E11.720p.HDTV.x265-MiNX.mkv",
        "High.Potential.S01E13.mp4",
        "The.Office.US.S02E04.avi",
        "Some.Random.Video.Without.Tags.mp4",
        "Breaking_Bad_s1e1.mkv",
        "Doctor Who 2005 S01E01.mp4"
    ]

    for fname in test_filenames:
        print(f"Parsing '{fname}':")
        print(f"  Result: {parse_filename(fname)}\n")
