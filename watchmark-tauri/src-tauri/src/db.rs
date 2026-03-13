use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::fs;

use directories::ProjectDirs;

pub fn get_app_data_dir() -> PathBuf {
    let app_dir = if let Some(proj_dirs) = ProjectDirs::from("com", "WatchMark", "WatchMark") {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        // Fallback to local execution directory if OS doesn't support appdata paths
        let mut exe_dir = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        exe_dir.pop();
        exe_dir.join("WatchMark")
    };

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
