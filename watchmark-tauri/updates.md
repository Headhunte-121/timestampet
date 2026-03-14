
### Fix Compilation Errors in perform_tmdb_search (Task 1.11 Support)
- Fixed borrowing lifetime issues (`E0505`, `E0521`) inside the `tokio::task::spawn` in `perform_tmdb_search`.
- Cloned `tauri::AppHandle` and extracted `AppState` dynamically within the spawned task to safely mutate `is_api_authorized` and `db_queue` states.
- Re-implemented the `tokio::select!` block inside the spawned task to correctly race the TMDB network request against the `CancellationToken` (Task 1.11).
- Addressed dead code warnings in `models::Stats` and `models::DashboardData` by adding `#[allow(dead_code)]` decorators.
- Fixed test compilation failures by properly initializing `is_api_authorized` in `commands_tests_optimize` and `commands_tests_tmdb_auth`.
