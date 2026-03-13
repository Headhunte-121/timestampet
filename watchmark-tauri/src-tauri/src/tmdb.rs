use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::db::get_app_data_dir;

const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";

pub fn get_poster_cache_dir() -> PathBuf {
    let dir = get_app_data_dir().join("cache").join("posters");
    if !dir.exists() {
        fs::create_dir_all(&dir).unwrap_or_default();
    }
    dir
}

pub fn search_media(api_key: &str, query: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let url = format!("{}/search/multi", TMDB_API_BASE);

    let res: Value = client
        .get(&url)
        .query(&[
            ("api_key", api_key),
            ("query", query),
            ("language", "en-US"),
            ("page", "1"),
            ("include_adult", "false"),
        ])
        .send()?
        .json()?;

    let results = res["results"].as_array().unwrap_or(&vec![]).clone();
    let mut filtered = Vec::new();

    for r in results {
        let media_type = r["media_type"].as_str().unwrap_or("");
        if media_type == "tv" || media_type == "movie" {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "tmdb_id".to_string(),
                Value::String(r["id"].as_i64().unwrap_or(0).to_string()),
            );
            obj.insert(
                "type".to_string(),
                Value::String(if media_type == "tv" { "TV" } else { "Movie" }.to_string()),
            );

            let title = if media_type == "tv" {
                r["name"].as_str()
            } else {
                r["title"].as_str()
            }
            .unwrap_or("Unknown Title");
            obj.insert("title".to_string(), Value::String(title.to_string()));

            obj.insert(
                "synopsis".to_string(),
                Value::String(r["overview"].as_str().unwrap_or("").to_string()),
            );
            obj.insert(
                "poster_path".to_string(),
                Value::String(r["poster_path"].as_str().unwrap_or("").to_string()),
            );
            obj.insert(
                "backdrop_path".to_string(),
                Value::String(r["backdrop_path"].as_str().unwrap_or("").to_string()),
            );

            let release_date = if media_type == "tv" {
                r["first_air_date"].as_str()
            } else {
                r["release_date"].as_str()
            }
            .unwrap_or("");
            obj.insert(
                "release_date".to_string(),
                Value::String(release_date.to_string()),
            );

            filtered.push(Value::Object(obj));
        }
    }

    Ok(filtered)
}

pub fn get_media_details(
    api_key: &str,
    tmdb_id: &str,
    media_type: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let endpoint = if media_type == "TV" { "tv" } else { "movie" };
    let url = format!("{}/{}/{}", TMDB_API_BASE, endpoint, tmdb_id);

    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let r: Value = client
        .get(&url)
        .query(&[("api_key", api_key), ("language", "en-US")])
        .send()?
        .json()?;

    let mut obj = serde_json::Map::new();
    obj.insert(
        "tmdb_id".to_string(),
        Value::String(r["id"].as_i64().unwrap_or(0).to_string()),
    );
    obj.insert("type".to_string(), Value::String(media_type.to_string()));

    let title = if media_type == "TV" {
        r["name"].as_str()
    } else {
        r["title"].as_str()
    }
    .unwrap_or("Unknown Title");
    obj.insert("title".to_string(), Value::String(title.to_string()));

    obj.insert(
        "synopsis".to_string(),
        Value::String(r["overview"].as_str().unwrap_or("").to_string()),
    );
    obj.insert(
        "poster_path".to_string(),
        Value::String(r["poster_path"].as_str().unwrap_or("").to_string()),
    );
    obj.insert(
        "backdrop_path".to_string(),
        Value::String(r["backdrop_path"].as_str().unwrap_or("").to_string()),
    );

    if media_type == "TV" {
        obj.insert(
            "total_episodes".to_string(),
            Value::Number(serde_json::Number::from(
                r["number_of_episodes"].as_i64().unwrap_or(1),
            )),
        );
        obj.insert("seasons".to_string(), r["seasons"].clone());
        obj.insert(
            "release_date".to_string(),
            Value::String(r["first_air_date"].as_str().unwrap_or("").to_string()),
        );
    } else {
        obj.insert(
            "total_episodes".to_string(),
            Value::Number(serde_json::Number::from(1)),
        );
        obj.insert(
            "runtime".to_string(),
            Value::Number(serde_json::Number::from(r["runtime"].as_i64().unwrap_or(0))),
        );
        obj.insert(
            "release_date".to_string(),
            Value::String(r["release_date"].as_str().unwrap_or("").to_string()),
        );
    }

    obj.insert(
        "status".to_string(),
        Value::String("Plan to Watch".to_string()),
    );

    if let Some(vote) = r["vote_average"].as_f64() {
        if let Some(num) = serde_json::Number::from_f64(vote) {
            obj.insert("vote_average".to_string(), Value::Number(num));
        } else {
            obj.insert(
                "vote_average".to_string(),
                Value::Number(serde_json::Number::from(0)),
            );
        }
    } else {
        obj.insert(
            "vote_average".to_string(),
            Value::Number(serde_json::Number::from(0)),
        );
    }

    Ok(Value::Object(obj))
}

pub fn get_tv_season_episodes(
    api_key: &str,
    tmdb_id: &str,
    season_num: i64,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let url = format!("{}/tv/{}/season/{}", TMDB_API_BASE, tmdb_id, season_num);
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let r: Value = client
        .get(&url)
        .query(&[("api_key", api_key), ("language", "en-US")])
        .send()?
        .json()?;

    let episodes = r["episodes"].as_array().unwrap_or(&vec![]).clone();
    let mut formatted = Vec::new();

    for ep in episodes {
        let mut obj = serde_json::Map::new();
        obj.insert(
            "season_num".to_string(),
            Value::Number(serde_json::Number::from(season_num)),
        );
        obj.insert(
            "ep_num".to_string(),
            Value::Number(serde_json::Number::from(
                ep["episode_number"].as_i64().unwrap_or(0),
            )),
        );
        obj.insert(
            "title".to_string(),
            Value::String(ep["name"].as_str().unwrap_or("Unknown Title").to_string()),
        );
        obj.insert(
            "overview".to_string(),
            Value::String(ep["overview"].as_str().unwrap_or("").to_string()),
        );
        obj.insert(
            "runtime".to_string(),
            Value::Number(serde_json::Number::from(
                ep["runtime"].as_i64().unwrap_or(0),
            )),
        );
        obj.insert(
            "still_path".to_string(),
            Value::String(ep["still_path"].as_str().unwrap_or("").to_string()),
        );
        obj.insert(
            "air_date".to_string(),
            Value::String(ep["air_date"].as_str().unwrap_or("").to_string()),
        );
        formatted.push(Value::Object(obj));
    }

    Ok(formatted)
}

pub fn download_image(image_path: &str, size: &str) -> Option<String> {
    if image_path.is_empty() {
        return None;
    }

    let clean_path = image_path.trim_start_matches('/');
    let filename = format!("{}_{}", size, clean_path);
    let local_path = get_poster_cache_dir().join(&filename);

    if local_path.exists() {
        return Some(local_path.to_string_lossy().to_string());
    }

    let url = format!("https://image.tmdb.org/t/p/{}/{}", size, clean_path);
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .ok()?;

    if let Ok(mut response) = client.get(&url).send() {
        if response.status().is_success() {
            if let Ok(mut file) = std::fs::File::create(&local_path) {
                if let Ok(_) = response.copy_to(&mut file) {
                    return Some(local_path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}
