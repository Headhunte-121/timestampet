use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::fs;

pub fn get_app_data_dir() -> PathBuf {
    let mut exe_dir = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    // If we're running in debug mode (target/debug/app), move up to the project root for local data.
    if exe_dir.ends_with("watchmark-tauri") || exe_dir.parent().map_or(false, |p| p.ends_with("debug") || p.ends_with("release")) {
        // Just use current directory if development, otherwise alongside exe
        if cfg!(debug_assertions) {
            exe_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        } else {
            exe_dir.pop();
        }
    } else {
        exe_dir.pop();
    }

    let app_dir = exe_dir.join("WatchMark");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).unwrap_or_default();
    }
    app_dir
}

pub fn get_db_path() -> PathBuf {
    get_app_data_dir().join("watchmark.db")
}

pub fn get_db_connection() -> Result<Connection> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;
    Ok(conn)
}

pub fn init_db() -> Result<()> {
    let conn = get_db_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Media (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tmdb_id TEXT UNIQUE,
            type TEXT,
            title TEXT,
            synopsis TEXT,
            poster_path TEXT,
            backdrop_path TEXT,
            total_episodes INTEGER,
            status TEXT,
            vote_average REAL DEFAULT 0.0,
            user_rating INTEGER DEFAULT 0,
            release_date TEXT
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Episodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_id INTEGER,
            season_num INTEGER,
            ep_num INTEGER,
            title TEXT,
            runtime INTEGER,
            still_path TEXT,
            overview TEXT,
            watch_count INTEGER DEFAULT 0,
            last_position INTEGER DEFAULT 0,
            status TEXT DEFAULT 'Unwatched',
            completed_date TEXT,
            air_date TEXT,
            FOREIGN KEY (media_id) REFERENCES Media (id),
            UNIQUE(media_id, season_num, ep_num)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Local_Files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            episode_id INTEGER UNIQUE,
            file_path TEXT UNIQUE,
            FOREIGN KEY (episode_id) REFERENCES Episodes (id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS History (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            episode_id INTEGER,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (episode_id) REFERENCES Episodes (id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Unmatched_Files (
            file_path TEXT PRIMARY KEY,
            filename TEXT,
            parsed_series TEXT,
            parsed_season INTEGER,
            parsed_episode INTEGER,
            group_key TEXT
        )",
        (),
    )?;

    // Run migrations safely
    let migrations = vec![
        "ALTER TABLE Media ADD COLUMN backdrop_path TEXT",
        "ALTER TABLE Episodes ADD COLUMN still_path TEXT",
        "ALTER TABLE Episodes ADD COLUMN overview TEXT",
        "ALTER TABLE Media ADD COLUMN vote_average REAL DEFAULT 0.0",
        "ALTER TABLE Media ADD COLUMN user_rating INTEGER DEFAULT 0",
        "ALTER TABLE Media ADD COLUMN release_date TEXT",
        "ALTER TABLE Episodes ADD COLUMN completed_date TEXT",
        "ALTER TABLE Episodes ADD COLUMN air_date TEXT",
        "ALTER TABLE History ADD COLUMN is_legacy INTEGER DEFAULT 0",
        "ALTER TABLE History ADD COLUMN session_id TEXT",
        "ALTER TABLE History ADD COLUMN start_time DATETIME",
        "ALTER TABLE History ADD COLUMN end_time DATETIME",
        "ALTER TABLE History ADD COLUMN pause_count INTEGER DEFAULT 0",
        "ALTER TABLE History ADD COLUMN completion_ratio REAL DEFAULT 0.0",
    ];

    for query in migrations {
        let _ = conn.execute(query, ()); // Ignore errors if column already exists
    }

    Ok(())
}
