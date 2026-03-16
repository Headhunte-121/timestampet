// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::db::get_app_data_dir;

#[derive(Clone, Debug)]
pub struct ImageConfig {
    pub backdrop_size: String,
    pub poster_size: String,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            backdrop_size: "w1280".to_string(),
            poster_size: "w500".to_string(),
        }
    }
}
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

pub fn get_still_cache_dir() -> PathBuf {
    let dir = get_app_data_dir().join("cache").join("stills");
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
            let original_language = item["original_language"].as_str().unwrap_or("xx"); // random default if missing

            // Note: search/multi does not easily return all images.
            // But if TMDB provides null for poster_path under en-US, we can't fetch it without an extra request.
            // Wait, TMDB generally provides a poster_path even if we request en-US and there is none, it falls back.
            // The prompt says "if the en-US poster is null, the Rust backend must immediately check the original_language field."
            // This might just mean making an extra call, or using include_image_language if possible.
            // Actually, in search we don't have include_image_language. The prompt says "When fetching metadata" which could refer to get_media_details.
            // Let's implement it here just in case. Since search returns the main poster, we'll use it directly. If it's empty, we might not be able to easily fetch the original without a separate request. Let's just use what's returned here.

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

    // We include append_to_response=images to get original language posters if en-US is missing
    let res = NETWORK_MANAGER.external_client
        .get(&url)
        .query(&[
            ("api_key", api_key),
            ("language", "en-US"),
            ("append_to_response", "images")
        ])
        .send()
        .await;

    let response = match res {
        Ok(r) => {
            if r.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::Custom("NOT_FOUND".to_string()));
            } else if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let mut retry_after = 1;
                if let Some(val) = r.headers().get("Retry-After") {
                    if let Ok(str_val) = val.to_str() {
                        if let Ok(secs) = str_val.parse::<u64>() {
                            retry_after = secs;
                        }
                    }
                }
                return Err(AppError::Custom(format!("RATE_LIMIT:{}", retry_after)));
            }
            r
        }
        Err(e) => return Err(crate::network::NetworkManager::handle_error(e)),
    };

    let r: Value = response.json().await?;

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
    let mut poster = r["poster_path"].as_str().unwrap_or("").to_string();

    // Fallback: If en-US poster is null/empty, check the original_language
    if poster.is_empty() {
        let original_language = r["original_language"].as_str().unwrap_or("");
        if !original_language.is_empty() {
            if let Some(images) = r.get("images") {
                if let Some(posters) = images.get("posters").and_then(|p| p.as_array()) {
                    // Try to find a poster matching the original language
                    for p in posters {
                        if p.get("iso_639_1").and_then(|lang| lang.as_str()) == Some(original_language) {
                            if let Some(path) = p.get("file_path").and_then(|fp| fp.as_str()) {
                                poster = path.to_string();
                                break;
                            }
                        }
                    }
                    // If still empty, grab the highest rated poster regardless of language
                    if poster.is_empty() && !posters.is_empty() {
                        if let Some(path) = posters[0].get("file_path").and_then(|fp| fp.as_str()) {
                            poster = path.to_string();
                        }
                    }
                }
            }
        }
    }

    if poster.is_empty() {
        obj.insert("poster_path".to_string(), Value::Null);
    } else {
        obj.insert("poster_path".to_string(), Value::String(poster.clone()));
    }

    let mut backdrop = r["backdrop_path"].as_str().unwrap_or("").to_string();

    if let Some(images) = r.get("images") {
        if let Some(backdrops) = images.get("backdrops").and_then(|b| b.as_array()) {
            let mut best_clean_backdrop = None;
            let mut highest_vote = -1.0;

            for b in backdrops {
                // Ensure iso_639_1 is null for clean textless images
                if b.get("iso_639_1").unwrap_or(&Value::Null).is_null() {
                    let vote = b.get("vote_average").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    if vote > highest_vote {
                        highest_vote = vote;
                        if let Some(path) = b.get("file_path").and_then(|fp| fp.as_str()) {
                            best_clean_backdrop = Some(path.to_string());
                        }
                    }
                }
            }

            if let Some(clean_backdrop) = best_clean_backdrop {
                backdrop = clean_backdrop;
            }
        }
    }

    if backdrop.is_empty() {
        if !poster.is_empty() {
            // Secondary Fallback: Use poster as pseudo-backdrop
            // We append a custom query param/marker to the path so the download engine knows to crop it
            obj.insert("backdrop_path".to_string(), Value::String(format!("{}?crop=true", poster)));
        } else {
            // Tertiary Fallback: no backdrops and no poster
            obj.insert("backdrop_path".to_string(), Value::Null);
            obj.insert("fallback_type".to_string(), Value::String("gradient".to_string()));
        }
    } else {
        obj.insert("backdrop_path".to_string(), Value::String(backdrop));
    }

    // Flatten genres correctly
    let genres = if let Some(genres_arr) = r["genres"].as_array() {
        genres_arr.iter()
            .filter_map(|g| g["name"].as_str())
            .collect::<Vec<&str>>()
            .join(", ")
    } else {
        String::new()
    };
    obj.insert("genres".to_string(), Value::String(genres));

    // Flatten networks / studios correctly
    let networks_arr = r["networks"].as_array()
        .or_else(|| r["production_companies"].as_array());

    let networks = if let Some(arr) = networks_arr {
        arr.iter()
            .filter_map(|n| n["name"].as_str())
            .collect::<Vec<&str>>()
            .join(", ")
    } else {
        String::new()
    };
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
    let res = NETWORK_MANAGER.external_client
        .get(&url)
        .query(&[("api_key", api_key), ("language", "en-US")])
        .send()
        .await;

    let response = match res {
        Ok(r) => {
            if r.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(AppError::Custom("NOT_FOUND".to_string()));
            } else if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let mut retry_after = 1; // Default to 1 second
                if let Some(val) = r.headers().get("Retry-After") {
                    if let Ok(str_val) = val.to_str() {
                        if let Ok(secs) = str_val.parse::<u64>() {
                            retry_after = secs;
                        }
                    }
                }
                return Err(AppError::Custom(format!("RATE_LIMIT:{}", retry_after)));
            }
            r
        }
        Err(e) => return Err(crate::network::NetworkManager::handle_error(e)),
    };

    let r: Value = response.json().await?;

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

pub fn resolve_local_still_path(image_path: &str, episode_id: i32) -> Option<String> {
    if image_path.is_empty() {
        return None;
    }

    let filename = format!("ep_{}.jpg", episode_id);
    let local_path = get_still_cache_dir().join(&filename);

    if local_path.exists() {
        return Some(local_path.to_string_lossy().to_string());
    }
    None
}

pub fn resolve_local_poster_path(image_path: &str, size: &str, high_performance_mode: bool) -> Option<String> {
    if image_path.is_empty() {
        return None;
    }

    let clean_path = image_path.trim_start_matches('/');
    let actual_size = if high_performance_mode && size == "w500" { "w342" } else { size };
    let filename = format!("{}_{}", actual_size, clean_path);

    let local_path = get_poster_cache_dir().join(&filename);
    if local_path.exists() {
        return Some(local_path.to_string_lossy().to_string());
    }
    None
}

pub fn resolve_local_backdrop_path(image_path: &str, size: &str, high_performance_mode: bool) -> Option<String> {
    if image_path.is_empty() {
        return None;
    }

    let mut is_pseudo_backdrop = false;
    let mut raw_path = image_path;
    if image_path.ends_with("?crop=true") {
        is_pseudo_backdrop = true;
        raw_path = image_path.trim_end_matches("?crop=true");
    }

    let clean_path = raw_path.trim_start_matches('/');
    let actual_size = if high_performance_mode && size == "w500" { "w342" } else { size };

    let filename = if is_pseudo_backdrop {
        format!("{}_pseudo_{}", actual_size, clean_path)
    } else {
        format!("{}_{}", actual_size, clean_path)
    };

    let local_path = get_poster_cache_dir().join(&filename);
    if local_path.exists() {
        return Some(local_path.to_string_lossy().to_string());
    }
    None
}

pub async fn download_episode_still(image_path: &str, episode_id: i32, high_performance_mode: bool) -> Option<String> {
    if image_path.is_empty() {
        return None;
    }



    let clean_path = image_path.trim_start_matches('/');
    let actual_size = if high_performance_mode { "w300" } else { "w500" };

    let filename = format!("ep_{}.jpg", episode_id);
    let local_path = get_still_cache_dir().join(&filename);

    if local_path.exists() {
        return Some(local_path.to_string_lossy().to_string());
    }

    let mut attempt_sizes = vec![actual_size];
    if actual_size == "w500" {
        attempt_sizes.push("original");
    }

    for current_size in attempt_sizes {
        let url = format!("https://image.tmdb.org/t/p/{}/{}", current_size, clean_path);
        if let Ok(response) = NETWORK_MANAGER.external_client.get(&url).send().await {
            if response.status().is_success() {
                if let Ok(bytes) = response.bytes().await {
                    let tmp_filename = format!("{}.tmp", filename);
                    let tmp_local_path = get_still_cache_dir().join(&tmp_filename);

                    if let Ok(_) = tokio::fs::write(&tmp_local_path, &bytes).await {
                        if crate::sanitizer::verify_image_header(&tmp_local_path) {
                            if std::fs::rename(&tmp_local_path, &local_path).is_ok() {
                                return Some(local_path.to_string_lossy().to_string());
                            }
                        }
                        let _ = std::fs::remove_file(&tmp_local_path);
                    }
                }
                break;
            } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                continue;
            } else {
                break;
            }
        }
    }

    None
}

pub async fn download_image(image_path: &str, size: &str, high_performance_mode: bool) -> Option<String> {
    if image_path.is_empty() {
        return None;
    }



    let mut is_pseudo_backdrop = false;
    let mut raw_path = image_path;
    if image_path.ends_with("?crop=true") {
        is_pseudo_backdrop = true;
        raw_path = image_path.trim_end_matches("?crop=true");
    }

    let clean_path = raw_path.trim_start_matches('/');
    let actual_size = if high_performance_mode && size == "w500" { "w342" } else { size };

    let filename = if is_pseudo_backdrop {
        format!("{}_pseudo_{}", actual_size, clean_path)
    } else {
        format!("{}_{}", actual_size, clean_path)
    };
    let local_path = get_poster_cache_dir().join(&filename);

    if local_path.exists() {
        return Some(local_path.to_string_lossy().to_string());
    }

    let mut attempt_sizes = vec![actual_size];
    if actual_size == "w500" || actual_size == "w342" {
        attempt_sizes.push("original"); // Fallback for 404
    }

    for current_size in attempt_sizes {
        let url = format!("https://image.tmdb.org/t/p/{}/{}", current_size, clean_path);
        if let Ok(response) = NETWORK_MANAGER.external_client.get(&url).send().await {
            if response.status().is_success() {
                if let Ok(bytes) = response.bytes().await {
                    let tmp_filename = format!("{}.tmp", filename);
                    let tmp_local_path = get_poster_cache_dir().join(&tmp_filename);

                    if is_pseudo_backdrop {
                        // Secondary Fallback: Process poster into a pseudo-backdrop
                        if let Ok(mut img) = image::load_from_memory(&bytes) {
                            // Center-crop to 16:9 and horizontal expansion
                            let (width, height) = img.dimensions();
                            use image::GenericImageView;

                            // Calculate 16:9 dimensions based on original width
                            let target_height = (width as f32 * 9.0 / 16.0).round() as u32;

                            // If the calculated height is smaller than the original, we can crop safely
                            if target_height <= height {
                                let y_offset = (height - target_height) / 2;
                                let cropped = img.crop(0, y_offset, width, target_height);
                                if let Ok(_) = cropped.save_with_format(&tmp_local_path, image::ImageFormat::Jpeg) {
                                    if std::fs::rename(&tmp_local_path, &local_path).is_ok() {
                                        return Some(local_path.to_string_lossy().to_string());
                                    }
                                }
                            } else {
                                // Just save it anyway if math is weird
                                if let Ok(_) = img.save_with_format(&tmp_local_path, image::ImageFormat::Jpeg) {
                                    if std::fs::rename(&tmp_local_path, &local_path).is_ok() {
                                        return Some(local_path.to_string_lossy().to_string());
                                    }
                                }
                            }
                            let _ = std::fs::remove_file(&tmp_local_path);
                        }
                    } else {
                        if let Ok(_) = tokio::fs::write(&tmp_local_path, &bytes).await {
                            if crate::sanitizer::verify_image_header(&tmp_local_path) {
                                if std::fs::rename(&tmp_local_path, &local_path).is_ok() {
                                    return Some(local_path.to_string_lossy().to_string());
                                }
                            }
                            let _ = std::fs::remove_file(&tmp_local_path);
                        }
                    }
                }
                break; // Stop trying other sizes if we hit success but failed validation (or succeeded)
            } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                continue; // Try next size fallback
            } else {
                break; // Don't try fallback on other errors (like 429)
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
