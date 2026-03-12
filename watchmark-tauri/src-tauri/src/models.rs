use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub id: i32,
    pub tmdb_id: String,
    pub r#type: String,
    pub title: String,
    pub synopsis: String,
    pub poster_path: String,
    pub backdrop_path: String,
    pub total_episodes: i32,
    pub status: String,
    pub vote_average: f64,
    pub user_rating: i32,
    pub release_date: String,

    // Virtual fields
    pub completed_eps: Option<i32>,
    pub last_watched: Option<String>,
    pub min_year: Option<String>,
    pub max_year: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Episode {
    pub id: i32,
    pub media_id: i32,
    pub season_num: i32,
    pub ep_num: i32,
    pub title: String,
    pub runtime: i32,
    pub still_path: String,
    pub watch_count: i32,
    pub last_position: i32,
    pub status: String,
    pub file_path: Option<String>,
    pub overview: String,
    pub air_date: String,

    // Virtual fields joined
    pub show_title: Option<String>,
    pub backdrop_path: Option<String>,
    pub media_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryEntry {
    pub hist_id: i32,
    pub timestamp: String,
    pub session_id: Option<String>,
    pub is_legacy: i32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub pause_count: i32,
    pub completion_ratio: f64,

    // Joined episode data
    pub episode_id: i32,
    pub season_num: i32,
    pub ep_num: i32,
    pub ep_title: String,
    pub still_path: String,
    pub air_date: String,

    // Joined media data
    pub media_id: i32,
    pub show_title: String,
    pub poster_path: String,
    pub backdrop_path: String,
    pub media_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnmatchedFile {
    pub file_path: String,
    pub filename: String,
    pub parsed_series: Option<String>,
    pub parsed_season: Option<i32>,
    pub parsed_episode: Option<i32>,
    pub group_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub vlc_path: String,
    pub window_geometry: String,
    pub window_position: String,
    pub tmdb_api_key: String,
}