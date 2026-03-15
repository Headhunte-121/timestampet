// WATCHMARK TRACING DIRECTIVE:
// 1. Use tracing::instrument on all public commands/logic blocks.
// 2. Prefer structured logging: info!(action = "...", id = ?, "Message").
// 3. No raw println! allowed.

use tracing_subscriber::{
    filter::EnvFilter,
    fmt,
    reload,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling;
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;

use crate::db::get_app_data_dir;

pub const CORE_MODULES: &[&str] = &[
    "watchmark_tauri_lib::main",
    "watchmark_tauri_lib::db",
    "watchmark_tauri_lib::scanner",
    "watchmark_tauri_lib::tmdb",
    "watchmark_tauri_lib::vlc",
    "watchmark_tauri_lib::commands",
    "watchmark_tauri_lib::settings",
    "watchmark_tauri_lib::network",
    "watchmark_tauri_lib::task_queue",
    "watchmark_tauri_lib::backup",
    "watchmark_tauri_lib::sanitizer",
];

lazy_static! {
    pub static ref LOG_MANAGER: RwLock<Option<LogManager>> = RwLock::new(None);
}

pub struct LogManager {
    reload_handle: reload::Handle<EnvFilter, tracing_subscriber::Registry>,
    _worker_guard: WorkerGuard, // Must be kept alive to flush logs
}

impl LogManager {
    pub fn update_filter(&self, filter_string: &str) -> Result<(), String> {
        let new_filter = EnvFilter::try_new(filter_string)
            .map_err(|e| format!("Invalid log filter '{}': {}", filter_string, e))?;
        self.reload_handle
            .reload(new_filter)
            .map_err(|e| format!("Failed to reload tracing filter: {}", e))?;
        Ok(())
    }
}

pub fn init_tracing(global_level: &str, module_logs: &HashMap<String, String>) {
    let app_dir = get_app_data_dir();

    // Set up file appender for WatchMark.log
    let file_appender = rolling::never(app_dir, "WatchMark.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter_string = build_filter_string(global_level, module_logs);

    let filter = EnvFilter::try_new(&filter_string)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let (reload_filter, reload_handle) = reload::Layer::new(filter);

    let subscriber = tracing_subscriber::registry()
        .with(reload_filter)
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
        )
        .with(
            fmt::layer()
                .with_writer(non_blocking)
                .with_target(true)
                .with_ansi(false)
                .with_thread_ids(true)
                .with_thread_names(true)
        );

    // If this fails, it might mean another subscriber was already set. We can ignore in tests,
    // but in production it should succeed.
    let _ = subscriber.try_init();

    let mut manager = LOG_MANAGER.write().unwrap();
    *manager = Some(LogManager {
        reload_handle,
        _worker_guard: guard,
    });

    tracing::info!(action = "init_tracing", filter = %filter_string, "Dynamic Tracing Engine initialized.");
}

pub fn build_filter_string(global_level: &str, module_logs: &HashMap<String, String>) -> String {
    let mut directives = Vec::new();

    // Default catch-all
    let g_level = match global_level.to_lowercase().as_str() {
        "debug" | "info" | "warn" | "error" | "trace" | "off" => global_level.to_lowercase(),
        _ => "info".to_string(),
    };

    // Apply global level to both possible crate names
    directives.push(format!("watchmark_tauri={}", g_level));
    directives.push(format!("watchmark_tauri_lib={}", g_level));

    // As a fallback for any other uncaught internal modules, we also set the global default
    directives.push(format!("{}", g_level));

    // Silence reqwest, html5ever, rustls, etc. by default unless specified
    directives.push("reqwest=warn".to_string());
    directives.push("hyper=warn".to_string());
    directives.push("rustls=warn".to_string());

    // Merge explicitly provided module logs
    for (module, level) in module_logs {
        let level_lower = level.to_lowercase();
        if ["off", "error", "warn", "info", "debug", "trace"].contains(&level_lower.as_str()) {
            directives.push(format!("{}={}", module, level_lower));
        }
    }

    directives.join(",")
}

pub fn set_levels(global_level: &str, module_logs: &HashMap<String, String>) -> Result<(), String> {
    let filter_string = build_filter_string(global_level, module_logs);

    if let Some(manager) = LOG_MANAGER.read().unwrap().as_ref() {
        manager.update_filter(&filter_string)?;
        tracing::info!(action = "set_levels", filter = %filter_string, "Tracing filter dynamically updated.");
        Ok(())
    } else {
        Err("Tracing engine not initialized".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_filter_string() {
        let mut modules = HashMap::new();
        modules.insert("watchmark_tauri_lib::db".to_string(), "debug".to_string());

        let filter = build_filter_string("info", &modules);
        assert!(filter.contains("watchmark_tauri=info"));
        assert!(filter.contains("watchmark_tauri_lib=info"));
        assert!(filter.contains("info"));
        assert!(filter.contains("watchmark_tauri_lib::db=debug"));
    }
}
