use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("A critical backend error occurred: {0}")]
    Panic(String),

    #[error("Fatal Database/Permission error: {0}")]
    Fatal(String),

    #[error("{0}")]
    Custom(String),
}

// Serialize the error to send to the frontend
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let message = match self {
            AppError::DbError(e) => format!("Database error: {}", e),
            AppError::NetworkError(e) => format!("Network error: {}", e),
            AppError::IoError(e) => format!("IO error: {}", e),
            AppError::JsonError(e) => format!("Data Mismatch/Parse Error: {}", e),
            AppError::Panic(ref msg) => format!("Critical Panic: {}", msg),
            AppError::Fatal(ref msg) => format!("Fatal Error: {}", msg),
            AppError::Custom(ref msg) => msg.to_string(),
        };

        serializer.serialize_str(&message)
    }
}

// Helper for generic From conversions if necessary (e.g. standard errors into Custom)
impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Custom(s.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Custom(s)
    }
}

pub fn handle_panic<F, R>(f: F) -> Result<R, AppError>
where
    F: FnOnce() -> Result<R, AppError> + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(f).unwrap_or_else(|err| {
        let msg = if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic".to_string()
        };

        // Log the panic to a file
        if let Ok(mut dir) = std::env::current_exe() {
            if cfg!(debug_assertions) {
                dir = std::env::current_dir().unwrap_or_default();
            } else {
                dir.pop();
            }
            let log_dir = dir.join("WatchMark");
            let log_path = log_dir.join("watchmark.log");

            let _ = std::fs::create_dir_all(&log_dir);
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
            {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(file, "[{}] PANIC: {}", timestamp, msg);
            }
        }

        Err(AppError::Panic(msg))
    })
}
