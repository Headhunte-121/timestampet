// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use chrono::NaiveDate;
use regex::Regex;
use std::sync::OnceLock;

// Ensure regexes are only compiled once
fn html_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<[^>]+>").unwrap())
}

fn space_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\s+$").unwrap())
}

/// Sanitizes an incoming date string.
/// Returns a tuple of (sanitized_date_string, is_exact_date, is_date_known)
pub fn sanitize_date(raw_date: &str) -> (String, bool, bool) {
    let trimmed = raw_date.trim();

    if trimmed.is_empty() || trimmed.to_lowercase() == "tbd" || trimmed.to_lowercase() == "null" {
        return ("0000-00-00".to_string(), false, false);
    }

    // Handle "Year Only" bug
    if trimmed.len() == 4 {
        if trimmed.parse::<i32>().is_ok() {
            return (format!("{}-01-01", trimmed), false, true);
        } else {
            return ("0000-00-00".to_string(), false, false);
        }
    }

    // Handle normal valid date
    if NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").is_ok() {
        return (trimmed.to_string(), true, true);
    }

    // Fallback for completely malformed garbage strings
    ("0000-00-00".to_string(), false, false)
}

/// Sanitizes a text description (synopsis or overview)
/// Strips HTML tags, trims whitespace.
/// If the resulting string is empty, returns the provided fallback string.
pub fn sanitize_text(raw_text: &str, fallback: &str) -> String {
    let no_html = html_regex().replace_all(raw_text, "");

    // Convert to owned String to avoid returning a reference from the regex replace
    let mut cleaned = no_html.into_owned();

    // If it's just spaces/newlines, clean it out entirely
    if space_regex().is_match(&cleaned) {
        cleaned.clear();
    }

    let trimmed = cleaned.trim();

    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Calculate safe progress percentage
pub fn calculate_progress_percentage(last_position: i32, runtime: i32) -> f64 {
    let safe_runtime = if runtime <= 0 { 1 } else { runtime };
    if runtime <= 0 {
        return 0.0;
    }

    let progress = (last_position as f64 / (safe_runtime as f64 * 60.0)) * 100.0;

    // Clamp between 0.0 and 100.0
    if progress < 0.0 {
        0.0
    } else if progress > 100.0 {
        100.0
    } else {
        progress
    }
}

pub fn verify_image_header(path: &std::path::Path) -> bool {
    use std::io::Read;
    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = [0u8; 8];
    if file.read_exact(&mut buffer).is_err() {
        return false;
    }

    // JPEG magic bytes: FF D8 FF
    if buffer[0] == 0xFF && buffer[1] == 0xD8 && buffer[2] == 0xFF {
        return true;
    }

    // PNG magic bytes: 89 50 4E 47
    if buffer[0] == 0x89 && buffer[1] == 0x50 && buffer[2] == 0x4E && buffer[3] == 0x47 {
        return true;
    }

    false
}
