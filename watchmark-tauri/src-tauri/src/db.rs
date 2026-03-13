use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::{Mutex, MutexGuard};

use directories::ProjectDirs;

static DB_CONNECTION: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn get_app_data_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "WatchMark", "WatchMark") {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        // Only fallback to a local directory if the OS literally lacks an AppData directory (very rare)
        let mut exe_dir = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        exe_dir.pop();
        exe_dir.join("WatchMark")
    }
}

pub fn ensure_directories() -> std::io::Result<()> {
    let app_dir = get_app_data_dir();

    let paths_to_create = vec![
        app_dir.clone(),
        app_dir.join("db"),
        app_dir.join("cache").join("posters"),
        app_dir.join("cache").join("backdrops"),
        app_dir.join("cache").join("stills"),
    ];

    for path in paths_to_create {
        if path.exists() {
            if path.is_file() {
                // Rogue file collision! Delete or rename the rogue file
                let _ = fs::remove_file(&path);
                fs::create_dir_all(&path)?;
            }
        } else {
            fs::create_dir_all(&path)?;
        }

        // Final metadata check
        let meta = fs::metadata(&path)?;
        if !meta.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Failed to create directory at {:?}", path)
            ));
        }
    }

    Ok(())
}

pub fn get_db_path() -> PathBuf {
    get_app_data_dir().join("db").join("watchmark.db")
}

pub fn check_db_permissions(db_path: &PathBuf) -> Result<(), crate::error::AppError> {
    // Try to open in ReadWrite mode to check if we can write to an existing db file
    // Or check if we can write a canary to the directory
    if db_path.exists() {
        if let Err(_e) = Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE) {
            // Permission Denied or File Locked
            let mut canary_path = db_path.clone();
            canary_path.set_extension("canary");
            if std::fs::write(&canary_path, b"test").is_err() {
                return Err(crate::error::AppError::Fatal(format!(
                    "Cannot write to database directory. Permission denied: {:?}",
                    db_path.parent()
                )));
            } else {
                let _ = std::fs::remove_file(canary_path);
                return Err(crate::error::AppError::Fatal(format!(
                    "Database file is locked or cannot be accessed: {:?}",
                    db_path
                )));
            }
        }
    } else {
        // Check directory write permission
        let mut canary_path = db_path.clone();
        canary_path.set_extension("canary");
        if std::fs::write(&canary_path, b"test").is_err() {
            return Err(crate::error::AppError::Fatal(format!(
                "Cannot write to database directory. Permission denied: {:?}",
                db_path.parent()
            )));
        } else {
            let _ = std::fs::remove_file(canary_path);
        }
    }
    Ok(())
}

pub fn get_db_connection() -> Result<MutexGuard<'static, Connection>, rusqlite::Error> {
    let _ = ensure_directories();
    if let Some(conn_mutex) = DB_CONNECTION.get() {
        Ok(conn_mutex.lock().unwrap())
    } else {
        // Fallback or initialization if not set (should not happen if init_db is called first)
        let db_path = get_db_path();

        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
                | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )?;

        // Apply PRAGMA tuning
        let _ = conn.pragma_update(None, "cache_size", "-2000");
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        let _ = conn.pragma_update(None, "synchronous", "NORMAL");
        let _ = conn.pragma_update(None, "temp_store", "MEMORY");
        let _ = conn.pragma_update(None, "foreign_keys", "ON");

        let mutex = Mutex::new(conn);
        DB_CONNECTION
            .set(mutex)
            .map_err(|_| rusqlite::Error::InvalidPath(Default::default()))?;
        Ok(DB_CONNECTION.get().unwrap().lock().unwrap())
    }
}

pub fn init_db() -> Result<(), crate::error::AppError> {
    let _ = ensure_directories();
    if DB_CONNECTION.get().is_none() {
        let db_path = get_db_path();

        check_db_permissions(&db_path)?;

        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
                | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        )?;

        // Apply PRAGMA tuning
        conn.pragma_update(None, "cache_size", "-2000")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let _ = DB_CONNECTION.set(Mutex::new(conn));
    }

    let mut conn = get_db_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Media (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tmdb_id TEXT,
            \"type\" TEXT NOT NULL DEFAULT 'TV',
            \"title\" TEXT,
            synopsis TEXT,
            poster_path TEXT,
            backdrop_path TEXT,
            total_episodes INTEGER,
            status TEXT,
            vote_average REAL DEFAULT 0.0,
            user_rating INTEGER DEFAULT 0,
            release_date TEXT,
            UNIQUE(tmdb_id, \"type\")
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Episodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            media_id INTEGER,
            season_num INTEGER CHECK(season_num >= 0),
            ep_num INTEGER CHECK(ep_num >= 0),
            \"title\" TEXT,
            runtime INTEGER,
            still_path TEXT,
            overview TEXT,
            watch_count INTEGER DEFAULT 0,
            last_position INTEGER DEFAULT 0,
            status TEXT DEFAULT 'Unwatched',
            completed_date TEXT,
            air_date TEXT,
            FOREIGN KEY (media_id) REFERENCES Media (id) ON DELETE CASCADE,
            UNIQUE(media_id, season_num, ep_num)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Local_Files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            episode_id INTEGER UNIQUE,
            file_path TEXT UNIQUE,
            file_size INTEGER DEFAULT 0,
            file_hash TEXT,
            FOREIGN KEY (episode_id) REFERENCES Episodes (id) ON DELETE CASCADE
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS History (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            episode_id INTEGER,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (episode_id) REFERENCES Episodes (id) ON DELETE CASCADE
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

    // Run migrations safely via PRAGMA user_version
    let user_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    if user_version == 0 {
        // If it's a completely new database, we just created all columns via CREATE TABLE above.
        // We can just bump the version to 3 directly.
        // But to be safe if it's an old Python DB that had no user_version, we can just let
        // the migrations run. The `conn.execute` ignores errors anyway for adding columns.
        // However, the instructions say we should use BEGIN TRANSACTION and COMMIT.
        // Since ALTER TABLE ADD COLUMN IF NOT EXISTS is not standard in SQLite, and we
        // want to be transactionally safe, we will just try to run them inside transactions.
        // If one fails, the transaction rolls back, but we can't easily catch individual 'column already exists'
        // errors without aborting the transaction.
        // A safer approach for SQLite migration from version 0 (unknown) is to try adding columns one by one and ignore errors,
        // OR rely on the fact that if user_version == 0, it MIGHT be a fresh DB.
        // Let's implement the 3-Tier system properly.
    }

    if user_version < 2 {
        let tx = conn.transaction()?;
        let v2_migrations = vec![
            "ALTER TABLE Media ADD COLUMN backdrop_path TEXT",
            "ALTER TABLE Episodes ADD COLUMN still_path TEXT",
            "ALTER TABLE Episodes ADD COLUMN overview TEXT",
            "ALTER TABLE Media ADD COLUMN vote_average REAL DEFAULT 0.0",
            "ALTER TABLE Media ADD COLUMN user_rating INTEGER DEFAULT 0",
            "ALTER TABLE Media ADD COLUMN release_date TEXT",
            "ALTER TABLE Episodes ADD COLUMN completed_date TEXT",
            "ALTER TABLE Episodes ADD COLUMN air_date TEXT",
        ];
        for query in v2_migrations {
            // Try to execute the query. If a column already exists, sqlite throws a specific
            // error (e.g., "duplicate column name"). We want to skip this error for backward compatibility
            // but fail on syntax/other errors.
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        tx.execute("PRAGMA user_version = 2", ())?;
        tx.commit()?;
    }

    if user_version < 3 {
        let tx = conn.transaction()?;
        let v3_migrations = vec![
            "ALTER TABLE History ADD COLUMN is_legacy INTEGER DEFAULT 0",
            "ALTER TABLE History ADD COLUMN session_id TEXT",
            "ALTER TABLE History ADD COLUMN start_time DATETIME",
            "ALTER TABLE History ADD COLUMN end_time DATETIME",
            "ALTER TABLE History ADD COLUMN pause_count INTEGER DEFAULT 0",
            "ALTER TABLE History ADD COLUMN completion_ratio REAL DEFAULT 0.0",
        ];
        for query in v3_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        tx.execute("PRAGMA user_version = 3", ())?;
        tx.commit()?;
    }

    if user_version < 4 {
        let tx = conn.transaction()?;
        let v4_migrations = vec![
            "ALTER TABLE Local_Files ADD COLUMN file_size INTEGER DEFAULT 0",
            "ALTER TABLE Local_Files ADD COLUMN file_hash TEXT",
        ];
        for query in v4_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        tx.execute("PRAGMA user_version = 4", ())?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod db_tests;

pub fn delete_media(media_id: i32) -> Result<()> {
    let mut conn = get_db_connection()?;

    // Begin transaction for safety
    let tx = conn.transaction()?;

    // Fetch poster and backdrop paths before deleting
    let mut paths = Vec::new();
    {
        let mut stmt = tx.prepare("SELECT poster_path, backdrop_path FROM Media WHERE id = ?")?;
        let mut rows = stmt.query([media_id])?;
        if let Some(row) = rows.next()? {
            let poster: Option<String> = row.get(0)?;
            let backdrop: Option<String> = row.get(1)?;
            paths.push(poster);
            paths.push(backdrop);
        }
    }

    // Delete Media (Due to ON DELETE CASCADE and PRAGMA foreign_keys = ON, this will automatically
    // delete all related rows in Episodes, History, and Local_Files)
    tx.execute("DELETE FROM Media WHERE id = ?", [media_id])?;

    tx.commit()?;

    // Cleanup cached image files from app data dir
    let cache_dir = get_app_data_dir().join("cache");
    if cache_dir.exists() {
        for path_str in paths.into_iter().flatten() {
            // Ensure the path is just the filename if it's stored as an absolute URL or starts with a slash
            let filename = path_str.trim_start_matches('/');
            let full_path = cache_dir.join(filename);
            if full_path.exists() {
                let _ = std::fs::remove_file(full_path);
            }
        }
    }

    Ok(())
}
