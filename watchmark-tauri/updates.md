
## Fix: Prevent rusqlite deserialization panics for Episodes
* Replaced fragile `SELECT e.*` queries with explicit column selections in `get_media_details_db` and `get_dashboard_data` to prevent mapping errors from schema changes.
* Wrapped missing/NULL columns with `Option<T>` and graceful `unwrap_or_default()` fallbacks.
* Added a dedicated unit test ensuring that missing metadata for episodes does not throw panics and crash the payload.
