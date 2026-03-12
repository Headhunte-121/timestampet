import os
import requests
from pathlib import Path
from .data import POSTER_CACHE_DIR
from typing import List, Dict, Any, Optional

TMDB_API_BASE = "https://api.themoviedb.org/3"
TMDB_IMAGE_BASE = "https://image.tmdb.org/t/p/w500"

def get_headers(api_key: str) -> Dict[str, str]:
    # Support both Bearer token and direct API key query param,
    # but the recommended way is using the api_key parameter in requests
    return {"accept": "application/json"}

def search_media(api_key: str, query: str, include_movies: bool = True) -> List[Dict[str, Any]]:
    """Search TMDB for movies or TV shows matching the query."""
    if not api_key:
        return []

    url = f"{TMDB_API_BASE}/search/multi"
    params = {
        "api_key": api_key,
        "query": query,
        "language": "en-US",
        "page": 1,
        "include_adult": "false"
    }

    try:
        response = requests.get(url, params=params, headers=get_headers(api_key), timeout=5.0)
        response.raise_for_status()
        data = response.json()
        results = data.get("results", [])

        filtered = []
        for r in results:
            media_type = r.get("media_type")
            if media_type == "tv":
                filtered.append({
                    "tmdb_id": str(r["id"]),
                    "type": "TV",
                    "title": r.get("name") or "",
                    "synopsis": r.get("overview") or "",
                    "poster_path": r.get("poster_path") or "",
                    "backdrop_path": r.get("backdrop_path") or "",
                    "release_date": r.get("first_air_date") or ""
                })
            elif include_movies and media_type == "movie":
                filtered.append({
                    "tmdb_id": str(r["id"]),
                    "type": "Movie",
                    "title": r.get("title") or "",
                    "synopsis": r.get("overview") or "",
                    "poster_path": r.get("poster_path") or "",
                    "backdrop_path": r.get("backdrop_path") or "",
                    "release_date": r.get("release_date") or ""
                })
        return filtered

    except requests.exceptions.RequestException as e:
        print(f"TMDB Search Error: {e}")
        raise e

def get_media_details(api_key: str, tmdb_id: str, media_type: str) -> Optional[Dict[str, Any]]:
    """Fetch detailed information for a specific TV show or movie."""
    if not api_key: return None

    endpoint = "tv" if media_type == "TV" else "movie"
    url = f"{TMDB_API_BASE}/{endpoint}/{tmdb_id}"
    params = {
        "api_key": api_key,
        "language": "en-US"
    }

    try:
        response = requests.get(url, params=params, headers=get_headers(api_key), timeout=5.0)
        response.raise_for_status()
        r = response.json()

        details = {
            "tmdb_id": str(r["id"]),
            "type": media_type,
            "title": (r.get("name") or "") if media_type == "TV" else (r.get("title") or ""),
            "synopsis": r.get("overview") or "",
            "poster_path": r.get("poster_path") or "",
            "backdrop_path": r.get("backdrop_path") or "",
            "total_episodes": r.get("number_of_episodes") or 1 if media_type == "TV" else 1,
            "seasons": r.get("seasons") or [] if media_type == "TV" else [],
            "runtime": r.get("runtime") or 0 if media_type == "Movie" else 0,
            "status": "Plan to Watch",
            "vote_average": r.get("vote_average") or 0.0,
            "release_date": (r.get("first_air_date") or "") if media_type == "TV" else (r.get("release_date") or "")
        }
        return details
    except requests.exceptions.RequestException as e:
        print(f"TMDB Details Error: {e}")
        raise e

def get_tv_season_episodes(api_key: str, tmdb_id: str, season_num: int) -> List[Dict[str, Any]]:
    """Fetch all episodes for a specific TV season."""
    if not api_key: return []

    url = f"{TMDB_API_BASE}/tv/{tmdb_id}/season/{season_num}"
    params = {
        "api_key": api_key,
        "language": "en-US"
    }

    try:
        response = requests.get(url, params=params, headers=get_headers(api_key), timeout=5.0)
        response.raise_for_status()
        data = response.json()
        episodes = data.get("episodes", [])

        formatted = []
        for ep in episodes:
            formatted.append({
                "season_num": season_num,
                "ep_num": ep.get("episode_number") or 0,
                "title": ep.get("name") or "",
                "overview": ep.get("overview") or "",
                "runtime": ep.get("runtime") or 0,
                "still_path": ep.get("still_path") or ""
            })
        return formatted
    except requests.exceptions.RequestException as e:
        print(f"TMDB Season Error: {e}")
        raise e

def download_image(image_path: str, size: str = "w500") -> Optional[str]:
    """
    Downloads an image (poster, backdrop, still) from TMDB and caches it locally.
    Returns the local path or None if failed.
    """
    if not image_path:
        return None

    # Prefix filename with size to cache different resolutions separately
    filename = f"{size}_{image_path.lstrip('/')}"
    local_path = POSTER_CACHE_DIR / filename

    if local_path.exists():
        return str(local_path)

    base_url = "https://image.tmdb.org/t/p/"
    url = f"{base_url}{size}{image_path}"
    try:
        response = requests.get(url, stream=True, timeout=10.0)
        response.raise_for_status()
        with open(local_path, 'wb') as f:
            for chunk in response.iter_content(chunk_size=8192):
                f.write(chunk)
        return str(local_path)
    except requests.exceptions.RequestException as e:
        print(f"Poster Download Error: {e}")
        return None
