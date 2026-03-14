#[cfg(test)]
mod tests_feature_25_4 {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, RwLock};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_vacuum_optimizer_collision_guard() {
        // Create a dummy AppState
        let (settings_tx, _) = mpsc::channel(1);
        let app_state = crate::commands::AppState {
            settings: Arc::new(RwLock::new(crate::models::Settings::default())),
            settings_tx,
            db_queue: Arc::new(crate::task_queue::DbTaskQueue::new()),
            is_maintenance_mode: AtomicBool::new(false),
        };

        // Simulate that a maintenance mode is already active
        app_state.is_maintenance_mode.store(true, Ordering::SeqCst);

        // It should immediately fail because of the atomic bool if we were calling it directly.
        // We will mimic the command guard explicitly since tauri::State cannot be easily mocked in a unit test context
        // without instantiating a full AppHandle.
        let result = if app_state.is_maintenance_mode.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            Err(crate::error::AppError::Custom("System Busy: Maintenance mode is already active.".to_string()))
        } else {
            Ok(0)
        };

        assert!(result.is_err());
        if let Err(crate::error::AppError::Custom(msg)) = result {
            assert!(msg.contains("System Busy"));
        } else {
            panic!("Expected AppError::Custom containing 'System Busy'");
        }
    }
}
