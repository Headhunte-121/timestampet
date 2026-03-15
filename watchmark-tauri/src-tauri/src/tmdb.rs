use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::db::get_app_data_dir;
use crate::network::NETWORK_MANAGER;
use crate::error::AppError;

const TMDB_API_BASE: &str = "https://api.themoviedb.org/3";

pub fn get_poster_cache_dir() -> PathBuf {
    let dir = get_app_data_dir().join("cache").join("posters");
    if !dir.exists() {
        fs::create_dir_all(&dir).unwrap_or_default();
    }
    dir
}

pub async fn validate_key(api_key: &str) -> Result<bool, AppError> {
    let url = format!("{}/configuration", TMDB_API_BASE);

    // Call the lightweight configuration endpoint
    let res = NETWORK_MANAGER.external_client
        .get(&url)
        .query(&[("api_key", api_key)])
        .send()
        .await;

    match res {
        Ok(response) => {
            if response.status().is_success() {
                Ok(true)
            } else if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                Ok(false)
            } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                // Return a specific error type for rate limiting so the caller can retry
                // Parse the Retry-After header
                let mut retry_after = 1; // Default to 1 second if header is missing
                if let Some(val) = response.headers().get("Retry-After") {
                    if let Ok(str_val) = val.to_str() {
                        if let Ok(secs) = str_val.parse::<u64>() {
                            retry_after = secs;
                        }
                    }
                }
                Err(AppError::Custom(format!("RATE_LIMIT:{}", retry_after)))
            } else {
                Err(crate::network::NetworkManager::handle_error(reqwest_middleware::Error::Reqwest(response.error_for_status().unwrap_err())))
            }
        }
        Err(e) => Err(crate::network::NetworkManager::handle_error(e)),
    }
}

use std::collections::HashSet;

pub async fn search_media(api_key: &str, query: &str, page: u32) -> Result<Vec<Value>, AppError> {
    // Strip non-printable control characters from the query
    let safe_query: String = query.chars().filter(|c| !c.is_control()).collect();

    let url = format!("{}/search/multi", TMDB_API_BASE);

    let res: Value = NETWORK_MANAGER.external_client
        .get(&url)
        .query(&[
            ("api_key", api_key),
            ("query", safe_query.as_str()),
            ("language", "en-US"),
            ("page", &page.to_string()),
            ("include_adult", "false"),
        ])
        .send()
        .await
        .map_err(crate::network::NetworkManager::handle_error)?
        .json()
        .await?;

    let results = res["results"].as_array().unwrap_or(&vec![]).clone();
    let mut filtered = Vec::new();
    let mut seen_ids = HashSet::new();

    for r in results {
        let media_type = r["media_type"].as_str().unwrap_or("");

        let mut process_items = Vec::new();

        if media_type == "person" {
            if let Some(known_for) = r["known_for"].as_array() {
                for item in known_for {
                    process_items.push(item.clone());
                }
            }
        } else if media_type == "tv" || media_type == "movie" {
            process_items.push(r.clone());
        }

        for item in process_items {
            let item_media_type = item["media_type"].as_str().unwrap_or("");
            if item_media_type != "tv" && item_media_type != "movie" {
                continue;
            }

            let tmdb_id = item["id"].as_i64().unwrap_or(0);
            let dedup_key = format!("{}-{}", item_media_type, tmdb_id);
            if !seen_ids.insert(dedup_key) {
                continue; // Already processed this item
            }

            let mut obj = serde_json::Map::new();
            obj.insert(
                "tmdb_id".to_string(),
                Value::String(tmdb_id.to_string()),
            );
            obj.insert(
                "type".to_string(),
                Value::String(if item_media_type == "tv" { "TV" } else { "Movie" }.to_string()),
            );

            let title = if item_media_type == "tv" {
                item["name"].as_str()
            } else {
                item["title"].as_str()
            }
            .unwrap_or("Unknown Title");
            obj.insert("title".to_string(), Value::String(title.to_string()));

            obj.insert(
                "synopsis".to_string(),
                Value::String(crate::sanitizer::sanitize_text(item["overview"].as_str().unwrap_or(""), "No overview available.")),
            );
            let poster = item["poster_path"].as_str().unwrap_or("");
            if poster.is_empty() {
                obj.insert("poster_path".to_string(), Value::Null);
            } else {
                obj.insert("poster_path".to_string(), Value::String(poster.to_string()));
            }

            let backdrop = item["backdrop_path"].as_str().unwrap_or("");
            if backdrop.is_empty() {
                obj.insert("backdrop_path".to_string(), Value::Null);
            } else {
                obj.insert("backdrop_path".to_string(), Value::String(backdrop.to_string()));
            }

            let release_date = if item_media_type == "tv" {
                item["first_air_date"].as_str()
            } else {
                item["release_date"].as_str()
            }
            .unwrap_or("");

            let (final_date, is_exact, is_known) = if item_media_type != "tv" && release_date.is_empty() {
                ("0000-00-00".to_string(), false, false)
            } else {
                crate::sanitizer::sanitize_date(release_date)
            };

            obj.insert(
                "release_date".to_string(),
                Value::String(final_date),
            );
            obj.insert(
                "is_exact_date".to_string(),
                Value::Bool(is_exact),
            );
            obj.insert(
                "is_date_known".to_string(),
                Value::Bool(is_known),
            );

            filtered.push(Value::Object(obj));
        }
    }

    Ok(filtered)
}

pub async fn get_media_details(
    api_key: &str,
    tmdb_id: &str,
    media_type: &str,
) -> Result<Value, AppError> {
    let endpoint = if media_type == "TV" { "tv" } else { "movie" };
    let url = format!("{}/{}/{}", TMDB_API_BASE, endpoint, tmdb_id);

    let r: Value = NETWORK_MANAGER.external_client
        .get(&url)
        .query(&[("api_key", api_key), ("language", "en-US")])
        .send()
        .await
        .map_err(crate::network::NetworkManager::handle_error)?
        .json()
        .await?;

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
        Value::String(crate::sanitizer::sanitize_text(r["overview"].as_str().unwrap_or(""), "No overview available.")),
    );
    let poster = r["poster_path"].as_str().unwrap_or("");
    if poster.is_empty() {
        obj.insert("poster_path".to_string(), Value::Null);
    } else {
        obj.insert("poster_path".to_string(), Value::String(poster.to_string()));
    }

    let backdrop = r["backdrop_path"].as_str().unwrap_or("");
    if backdrop.is_empty() {
        obj.insert("backdrop_path".to_string(), Value::Null);
    } else {
        obj.insert("backdrop_path".to_string(), Value::String(backdrop.to_string()));
    }

    // Flatten genres
    let genres = r["genres"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g["name"].as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        })
        .unwrap_or_else(|| "".to_string());
    obj.insert("genres".to_string(), Value::String(genres));

    // Flatten networks / studios
    let networks = r["networks"]
        .as_array()
        .or_else(|| r["production_companies"].as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n["name"].as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        })
        .unwrap_or_else(|| "".to_string());
    obj.insert("networks".to_string(), Value::String(networks));

    if media_type == "TV" {
        obj.insert(
            "total_episodes".to_string(),
            Value::Number(serde_json::Number::from(
                r["number_of_episodes"].as_i64().unwrap_or(1),
            )),
        );
        obj.insert("seasons".to_string(), r["seasons"].clone());
        let air_date = r["first_air_date"].as_str().unwrap_or("");
        let (final_date, is_exact, is_known) = crate::sanitizer::sanitize_date(air_date);

        obj.insert(
            "release_date".to_string(),
            Value::String(final_date),
        );
        obj.insert(
            "is_exact_date".to_string(),
            Value::Bool(is_exact),
        );
        obj.insert(
            "is_date_known".to_string(),
            Value::Bool(is_known),
        );
        obj.insert("collection_id".to_string(), Value::Null);
        obj.insert("collection_name".to_string(), Value::Null);
    } else {
        obj.insert(
            "total_episodes".to_string(),
            Value::Number(serde_json::Number::from(1)),
        );

        let runtime_val = match serde_json::from_value::<crate::models::TmdbEpisode>(r.clone()) {
            Ok(ep) => ep.runtime,
            Err(_) => r["runtime"].as_i64().unwrap_or(0) as i32,
        };
        obj.insert(
            "runtime".to_string(),
            Value::Number(serde_json::Number::from(runtime_val)),
        );

        let rel_date = r["release_date"].as_str().unwrap_or("");
        let (final_date, is_exact, is_known) = if rel_date.is_empty() {
            ("0000-00-00".to_string(), false, false)
        } else {
            crate::sanitizer::sanitize_date(rel_date)
        };

        obj.insert(
            "release_date".to_string(),
            Value::String(final_date),
        );
        obj.insert(
            "is_exact_date".to_string(),
            Value::Bool(is_exact),
        );
        obj.insert(
            "is_date_known".to_string(),
            Value::Bool(is_known),
        );

        if let Some(collection) = r["belongs_to_collection"].as_object() {
            obj.insert(
                "collection_id".to_string(),
                Value::Number(serde_json::Number::from(collection["id"].as_i64().unwrap_or(0))),
            );
            obj.insert(
                "collection_name".to_string(),
                Value::String(collection["name"].as_str().unwrap_or("").to_string()),
            );
        } else {
            obj.insert("collection_id".to_string(), Value::Null);
            obj.insert("collection_name".to_string(), Value::Null);
        }
    }

    obj.insert(
        "status".to_string(),
        Value::String("Plan to Watch".to_string()),
    );

    if let Some(vote) = r["vote_average"].as_f64() {
        let rounded_vote = (vote * 10.0).round() / 10.0;
        if let Some(num) = serde_json::Number::from_f64(rounded_vote) {
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

pub async fn get_tv_season_episodes(
    api_key: &str,
    tmdb_id: &str,
    season_num: u32,
) -> Result<Vec<Value>, AppError> {
    let url = format!("{}/tv/{}/season/{}", TMDB_API_BASE, tmdb_id, season_num);
    let r: Value = NETWORK_MANAGER.external_client
        .get(&url)
        .query(&[("api_key", api_key), ("language", "en-US")])
        .send()
        .await
        .map_err(crate::network::NetworkManager::handle_error)?
        .json()
        .await?;

    let season_overview = crate::sanitizer::sanitize_text(
        r["overview"].as_str().unwrap_or(""),
        "No season overview.",
    );

    let episodes = r["episodes"].as_array().unwrap_or(&vec![]).clone();
    let mut formatted = Vec::new();

    for ep_value in episodes {
        // Attempt to parse into our typed struct to safely extract runtime
        let tmdb_ep: crate::models::TmdbEpisode = serde_json::from_value(ep_value)?;

        let mut obj = serde_json::Map::new();
        obj.insert(
            "season_num".to_string(),
            Value::Number(serde_json::Number::from(season_num)),
        );
        obj.insert(
            "ep_num".to_string(),
            Value::Number(serde_json::Number::from(tmdb_ep.episode_number)),
        );
        obj.insert(
            "title".to_string(),
            Value::String(tmdb_ep.name.unwrap_or("Unknown Title".to_string())),
        );
        obj.insert(
            "overview".to_string(),
            Value::String(crate::sanitizer::sanitize_text(&tmdb_ep.overview.unwrap_or("".to_string()), "No episode summary.")),
        );
        obj.insert(
            "season_overview".to_string(),
            Value::String(season_overview.clone()),
        );
        obj.insert(
            "runtime".to_string(),
            Value::Number(serde_json::Number::from(tmdb_ep.runtime)),
        );
        let still = tmdb_ep.still_path.unwrap_or("".to_string());
        if still.is_empty() {
            obj.insert("still_path".to_string(), Value::Null);
        } else {
            obj.insert("still_path".to_string(), Value::String(still));
        }
        let raw_air_date = tmdb_ep.air_date.unwrap_or("".to_string());
        let (final_date, is_exact, is_known) = crate::sanitizer::sanitize_date(&raw_air_date);
        obj.insert(
            "air_date".to_string(),
            Value::String(final_date),
        );
        obj.insert(
            "is_exact_date".to_string(),
            Value::Bool(is_exact),
        );
        obj.insert(
            "is_date_known".to_string(),
            Value::Bool(is_known),
        );
        formatted.push(Value::Object(obj));
    }

    Ok(formatted)
}

pub async fn get_collection_details(api_key: &str, collection_id: i32) -> Result<Value, AppError> {
    let url = format!("{}/collection/{}", TMDB_API_BASE, collection_id);
    let r: Value = NETWORK_MANAGER.external_client
        .get(&url)
        .query(&[("api_key", api_key), ("language", "en-US")])
        .send()
        .await
        .map_err(crate::network::NetworkManager::handle_error)?
        .json()
        .await?;

    Ok(r)
}

pub async fn download_image(image_path: &str, size: &str) -> Option<String> {
    if image_path.is_empty() {
        return None;
    }

    let _ = crate::db::ensure_directories();

    let clean_path = image_path.trim_start_matches('/');
    let filename = format!("{}_{}", size, clean_path);
    let local_path = get_poster_cache_dir().join(&filename);

    if local_path.exists() {
        return Some(local_path.to_string_lossy().to_string());
    }

    let url = format!("https://image.tmdb.org/t/p/{}/{}", size, clean_path);

    if let Ok(response) = NETWORK_MANAGER.external_client.get(&url).send().await {
        if response.status().is_success() {
            if let Ok(bytes) = response.bytes().await {
                // Use blocking file IO inside async (or switch to tokio::fs, but since this isn't high concurrency, std::fs is okay here or we can use tokio::fs)
                if let Ok(_) = tokio::fs::write(&local_path, &bytes).await {
                    return Some(local_path.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
#[path = "tmdb_tests_3_4.rs"]
mod tmdb_tests_3_4;

#[cfg(test)]
#[path = "tmdb_tests_3_5.rs"]
mod tmdb_tests_3_5;
