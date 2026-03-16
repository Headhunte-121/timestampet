## Micro-Feature 7: TMDB Search Execution & UI updates
- Added offline mode detection (16.4) and UI notification using native onLine and Rust Heartbeat.
- Formatted TMDB search results in a responsive grid, with filtering toggles and special handling for 1-2 item lists (16.5).
- Enhanced SafeImage component for robust Image Load Error Handling, swapping failed urls with fallbacks and hiding broken image icons (16.12).

## 3.12 Original Release Date / First Air Date extraction
- Upgraded `sanitizer::sanitize_date` in the Rust backend to intercept partial strings like `YYYY` and pad them to `YYYY-01-01` before storage.
- Assured correct `NULLS LAST` sorting in chronological queries (`Release Year`) via standard `ORDER BY CASE WHEN m.release_date IS NULL OR m.release_date = '' THEN 1 ELSE 0 END, m.release_date DESC` logic.
- Built a localized date parser for the frontend (`utils/dateFormatter.ts`) that wraps TMDB dates into `Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', year: 'numeric' }).format(date)`.

## 3.13 Specific Episode Air Date tracking
- Filtered timeline episode states via `is_unaired` boolean injected directly from the Rust backend (utilizing `chrono` exact comparison logic).
- Flagged unaired episodes visually in `MediaDetails.tsx` by assigning a `grayscale-[0.5] opacity-70` overlay to the primary thumbnail, completely disabling the primary `Play` action button, and conditionally wrapping a `Calendar` vector icon above the row layout.

## 3.15 Total Episode count aggregation
- Enhanced core SQL progress logic by manually integrating `total_available` counts against the `completed_eps` denominator throughout Dashboard widgets and the Library grid.
- Assured the math skips unreleased episodes mapping specifically to `air_date <= date('now')`.
- Appended filtering specifically excluding `season_num = 0` specials from standard completion progress tracking so percentages remain completely consistent.

## 3.7 High-resolution primary Poster extraction
- Configured the Rust `ImageConfig` struct to query active `app.primary_monitor().scale_factor()` resolution limits identically across both `poster_size` and `backdrop_size` variables.
- Integrated precise high-DPI scaling directly across internal TMDB download caches (`original` resolution on >1.0 scales vs standard `w500` fallbacks on 1.0).
