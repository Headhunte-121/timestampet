use regex::Regex;
use std::sync::OnceLock;

static JUNK_DICTIONARY: OnceLock<Regex> = OnceLock::new();
static PROTECTED_DOTS: OnceLock<Regex> = OnceLock::new();
static RESTORE_DOTS: OnceLock<Regex> = OnceLock::new();
static GEOMETRY_VACUUM: OnceLock<Regex> = OnceLock::new();

pub fn clean_string(name: &str) -> String {
    let mut cleaned = name.to_string();

    // 1. Technical Token Stripping (Codecs & Resolutions)
    let junk_regex = JUNK_DICTIONARY.get_or_init(|| {
        Regex::new(r"(?i)\b(x264|x265|hevc|aac|dts|bluray|webrip|720p|1080p|4k|2160p|1080i|2160i)\b").unwrap()
    });
    cleaned = junk_regex.replace_all(&cleaned, "").to_string();

    // 2. Delimiter Normalization (Expansion Protection)
    // Identify dots surrounded by letters (e.g., Mr. Robot) and protect them by temporarily swapping
    let protected_dots_regex = PROTECTED_DOTS.get_or_init(|| {
        Regex::new(r"(?<=[A-Za-z])\.(?=[A-Za-z])").unwrap()
    });
    cleaned = protected_dots_regex.replace_all(&cleaned, "%%DOT%%").to_string();

    // Replace occurrences of . and _ with standard spaces
    cleaned = cleaned.replace('.', " ").replace('_', " ");

    // Restore protected dots
    let restore_dots_regex = RESTORE_DOTS.get_or_init(|| {
        Regex::new(r"%%DOT%%").unwrap()
    });
    cleaned = restore_dots_regex.replace_all(&cleaned, ".").to_string();

    // 3. Geometry Trimming
    // Vacuum: strips any trailing hyphens, brackets, or multiple consecutive spaces.
    let vacuum_regex = GEOMETRY_VACUUM.get_or_init(|| {
        Regex::new(r"[-[\]()]+$|\s{2,}").unwrap()
    });
    cleaned = vacuum_regex.replace_all(&cleaned, " ").trim().to_string();

    cleaned
}
