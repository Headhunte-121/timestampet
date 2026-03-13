use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrFloat {
        Int(i32),
        Float(f64),
    }

    match IntOrFloat::deserialize(deserializer) {
        Ok(IntOrFloat::Int(i)) => Ok(i),
        Ok(IntOrFloat::Float(f)) => Ok(f.round() as i32),
        Err(e) => Err(e),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MediaType {
    TV,
    Movie,
    Unknown,
}

impl MediaType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "TV" => MediaType::TV,
            "Movie" => MediaType::Movie,
            _ => MediaType::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::TV => "TV",
            MediaType::Movie => "Movie",
            MediaType::Unknown => "Unknown",
        }
    }
}

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

#[allow(dead_code)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub vlc_path: String,
    #[serde(deserialize_with = "deserialize_to_i32", default = "default_width")]
    pub width: i32,
    #[serde(deserialize_with = "deserialize_to_i32", default = "default_height")]
    pub height: i32,
    #[serde(deserialize_with = "deserialize_to_i32", default = "default_x")]
    pub x: i32,
    #[serde(deserialize_with = "deserialize_to_i32", default = "default_y")]
    pub y: i32,
    pub tmdb_api_key: String,
    pub cinema_mode: bool,

    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_auto_complete_threshold")]
    pub auto_complete_threshold: i32,
    #[serde(default = "default_binge_grouping_hours")]
    pub binge_grouping_hours: i32,
    #[serde(default = "default_auto_resume")]
    pub auto_resume: bool,
    #[serde(default = "default_auto_scan_on_boot")]
    pub auto_scan_on_boot: bool,
    #[serde(default = "default_logging_level")]
    pub logging_level: String,
}

fn default_width() -> i32 { 1280 }
fn default_height() -> i32 { 800 }
fn default_x() -> i32 { 100 }
fn default_y() -> i32 { 100 }
fn default_language() -> String { "en-US".to_string() }
fn default_auto_complete_threshold() -> i32 { 90 }
fn default_binge_grouping_hours() -> i32 { 6 }
fn default_auto_resume() -> bool { true }
fn default_auto_scan_on_boot() -> bool { false }
fn default_logging_level() -> String { "Info".to_string() }

impl Default for Settings {
    fn default() -> Self {
        Self {
            vlc_path: String::new(),
            width: default_width(),
            height: default_height(),
            x: default_x(),
            y: default_y(),
            tmdb_api_key: String::new(),
            cinema_mode: true,
            language: default_language(),
            auto_complete_threshold: default_auto_complete_threshold(),
            binge_grouping_hours: default_binge_grouping_hours(),
            auto_resume: default_auto_resume(),
            auto_scan_on_boot: default_auto_scan_on_boot(),
            logging_level: default_logging_level(),
        }
    }
}
