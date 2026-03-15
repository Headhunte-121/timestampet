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
        let _ = conn.pragma_update(None, "busy_timeout", 5000); // Feature 1.10 busy timeout

        let mutex = Mutex::new(conn);
        DB_CONNECTION
            .set(mutex)
            .map_err(|_| rusqlite::Error::InvalidPath(Default::default()))?;
        Ok(DB_CONNECTION.get().unwrap().lock().unwrap())
    }
}

pub fn get_readonly_connection() -> Result<Connection, rusqlite::Error> {
    let _ = ensure_directories();
    let db_path = get_db_path();

    let conn = Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    )?;

    let _ = conn.pragma_update(None, "cache_size", "-2000");
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "synchronous", "NORMAL");
    let _ = conn.pragma_update(None, "temp_store", "MEMORY");
    let _ = conn.pragma_update(None, "foreign_keys", "ON");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);

    Ok(conn)
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
        conn.pragma_update(None, "busy_timeout", 5000)?;

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
            user_rating INTEGER CHECK(user_rating >= 0 AND user_rating <= 10) DEFAULT NULL,
            release_date TEXT,
            is_exact_date BOOLEAN DEFAULT 1,
            genres TEXT DEFAULT '',
            networks TEXT DEFAULT '',
            collection_id INTEGER,
            collection_name TEXT,
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
            season_overview TEXT DEFAULT '',
            watch_count INTEGER DEFAULT 0,
            last_position INTEGER NOT NULL DEFAULT 0,
            status TEXT DEFAULT 'Unwatched',
            completed_date TEXT,
            air_date TEXT,
            is_exact_date BOOLEAN DEFAULT 1,
            is_air_date_manual BOOLEAN DEFAULT 0,
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
            episode_id INTEGER NOT NULL,
            timestamp INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            last_position INTEGER DEFAULT 0,
            status TEXT DEFAULT 'Completed',
            is_legacy BOOLEAN DEFAULT 0,
            session_id TEXT,
            start_time DATETIME,
            end_time DATETIME,
            pause_count INTEGER DEFAULT 0,
            completion_ratio REAL DEFAULT 0.0,
            FOREIGN KEY (episode_id) REFERENCES Episodes (id) ON DELETE CASCADE
        )",
        (),
    )?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_history_timestamp ON History(timestamp DESC, id DESC)", ())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_media_title ON Media(title ASC)", ())?;

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

    if user_version < 5 {
        let tx = conn.transaction()?;
        let v5_migrations = vec![
            "ALTER TABLE History ADD COLUMN last_position INTEGER DEFAULT 0",
            "ALTER TABLE History ADD COLUMN status TEXT DEFAULT 'Completed'",
        ];
        for query in v5_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }

        // Migrate string timestamps to epoch integers safely using SQLite strftime, catching bad formats
        let _ = tx.execute(
            "UPDATE History SET timestamp = CAST(strftime('%s', timestamp) AS INTEGER) WHERE typeof(timestamp) = 'text'",
            ()
        );

        tx.execute("CREATE INDEX IF NOT EXISTS idx_history_timestamp ON History(timestamp DESC, id DESC)", ())?;
        tx.execute("PRAGMA user_version = 5", ())?;
        tx.commit()?;
    }

    if user_version < 6 {
        let tx = conn.transaction()?;
        let v6_migrations = vec![
            "ALTER TABLE Media ADD COLUMN is_exact_date BOOLEAN DEFAULT 1",
            "ALTER TABLE Episodes ADD COLUMN is_exact_date BOOLEAN DEFAULT 1",
        ];
        for query in v6_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        tx.execute("PRAGMA user_version = 6", ())?;
        tx.commit()?;
    }

    if user_version < 7 {
        let tx = conn.transaction()?;
        let v7_migrations = vec![
            "ALTER TABLE History ADD COLUMN is_legacy INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE Episodes ADD COLUMN is_air_date_manual BOOLEAN DEFAULT 0",
        ];
        for query in v7_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        // Ensure legacy rows that were added before are 0 by default if null
        let _ = tx.execute("UPDATE History SET is_legacy = 0 WHERE is_legacy IS NULL", ());
        tx.execute("PRAGMA user_version = 7", ())?;
        tx.commit()?;
    }

    if user_version < 8 {
        let tx = conn.transaction()?;
        let _ = tx.execute("UPDATE Media SET user_rating = NULL WHERE user_rating = 0", ());
        tx.execute("PRAGMA user_version = 8", ())?;
        tx.commit()?;
    }

    if user_version < 9 {
        let tx = conn.transaction()?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_media_tmdb_rating ON Media(vote_average)", ())?;
        tx.execute("PRAGMA user_version = 9", ())?;
        tx.commit()?;
    }

    if user_version < 10 {
        let tx = conn.transaction()?;
        // The column already exists in the CREATE TABLE Episodes statement (added as INTEGER DEFAULT 0),
        // but if we are migrating an existing database, we need to add it via ALTER TABLE.
        let v10_migrations = vec![
            "ALTER TABLE Episodes ADD COLUMN last_position INTEGER NOT NULL DEFAULT 0",
        ];
        for query in v10_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        tx.execute("PRAGMA user_version = 10", ())?;
        tx.commit()?;
    }

    if user_version < 11 {
        let tx = conn.transaction()?;
        let _ = tx.execute("UPDATE History SET session_id = NULL", ());
        tx.execute("PRAGMA user_version = 11", ())?;
        tx.commit()?;
    }

    if user_version < 12 {
        let tx = conn.transaction()?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_media_title ON Media(title ASC)", ())?;
        tx.execute("PRAGMA user_version = 12", ())?;
        tx.commit()?;
    }

    if user_version < 13 {
        let tx = conn.transaction()?;
        let v13_migrations = vec![
            "ALTER TABLE Media ADD COLUMN genres TEXT DEFAULT ''",
            "ALTER TABLE Media ADD COLUMN networks TEXT DEFAULT ''",
            "ALTER TABLE Episodes ADD COLUMN season_overview TEXT DEFAULT ''",
        ];
        for query in v13_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        tx.execute("PRAGMA user_version = 13", ())?;
        tx.commit()?;
    }

    if user_version < 14 {
        let tx = conn.transaction()?;
        let v14_migrations = vec![
            "ALTER TABLE Media ADD COLUMN collection_id INTEGER",
            "ALTER TABLE Media ADD COLUMN collection_name TEXT",
        ];
        for query in v14_migrations {
            if let Err(e) = tx.execute(query, ()) {
                if !e.to_string().contains("duplicate column name") {
                    return Err(crate::error::AppError::DbError(e));
                }
            }
        }
        tx.execute("PRAGMA user_version = 14", ())?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod db_tests;

#[cfg(test)]
#[path = "db_tests_append.rs"]
mod db_tests_append;
