This file serves as a chronological record of all significant updates, feature implementations, and bug fixes applied to the WatchMark project.


WatchMark Pro - Missing Feature Implementation Audit

The following discrepancies between the codebase and the Master List have been fixed:

1. Database Migrations (data.py):
   - Added `ALTER TABLE` execution for `release_date`, `overview`, `completed_date`, and `air_date` to ensure existing `watchmark.db` files do not throw `sqlite3.OperationalError: no such column` when upgrading to the Pro features.

2. Media Scanner Reliability (scanner.py):
   - Reverted the exact-match logic back to a flexible substring match (`safe_series in safe_db_name`) to ensure reliable local folder auto-matching.
   - Added `None` type data sanitization fallback (`Unknown Title`) when iterating through DB records during file matching.

3. Deep Dive Air-Date Integration (main.py):
   - Added parsing logic in `_show_media_details` to accurately calculate and display the Show-Level `Aired: YYYY - YYYY` context string directly below the media synopsis.

4. Library "At-a-Glance" Quick Stats (main.py):
   - Implemented an absolute-bottom aligned `CTkProgressBar` on all poster cards across the dashboard and library. Displays Gray (0%), Orange (>0%), or Green (100%).
   - Added small Star rating badges to the top-right corner of posters if the user has provided a personal score.

5. Data Sanitization (tmdb_api.py):
   - Added robust `.get()` checks and fallback defaults (e.g. `Unknown Title` and `No overview available.`) to API responses to prevent `NoneType` rendering crashes in CustomTkinter.

6. Dashboard Personal Stats Bar (main.py):
   - Replaced "Episodes Watched" with "Total Hours Watched", "Shows Completed", and dynamically calculated the "My Average Rating" widget using `SELECT AVG(user_rating)`.

7. In-Place UI Updates (main.py):
   - Handled edge cases by adding a `reset_scroll=False` flag to `_load_episodes()`, ensuring that marking checkmarks, marking seasons watched, and clicking the sync button updates the data without throwing the user back to the top of the scroll container.
   - Reset Dashboard view automatically if the Quick Search field is cleared.

Update 2. Portable Data Storage (config.py):
   - Changed the default application data storage directory from OS-specific roaming folders (like `%APPDATA%`) to a portable `WatchMark` folder located in the same directory as `run.py`. This ensures all settings, database files, and cache remain with the application files.

Update 3:
- Database Fetch Subqueries: Modified base SQL queries in main.py (`_render_library` and `_show_dashboard`) to fetch pre-calculated stats: MIN(air_date), MAX(air_date), and completed episode counts to prevent database lag during hover.
- Static Card Layer UI: Modified `_create_poster_card` to show Original Year in the bottom left, Progress bar on the bottom edge, and a persistent "★ {rating}/5" badge in the top right.
- Hover Overlay Rebuild: Created an `info_overlay` with `fg_color="#141519"` and a 10px implicit margin that displays the calculated Season Year Range, Watched Episodes fraction, truncated Synopsis, Last Watched human-readable string, and Play action center shifted downward.
- Hover Ghosting Bug Fix: Refactored Event Bindings by binding `<Enter>` and `<Leave>` to the parent `CTkFrame` container, using boundary coordinate checking alongside a `hover_active` flag, guaranteeing a fluid fade-in without flickering when hovering over text children.
- New Native Helper Function: Implemented `_get_time_ago(timestamp)` in main.py to gracefully convert history timestamps into "Time Ago" human-readable strings entirely using the native standard Python `datetime` library without timezone issues.
- Fixed TypeError in Watch History: Resolved an application crash (`can't compare offset-naive and offset-aware datetimes`) when sorting mixed history entries by explicitly assigning timezone-aware offsets to legacy SQL timestamps via `.astimezone()` before sorting.
- Added Status Tag visibility: Dynamically added the Status Tag (e.g., 'Returning Series' or 'Ended') to the Quick-View overlay with color-coded dark tints matching the application theme.
- Dynamic 'Present' Year Validation: Modified the season year range calculation to dynamically check if the `max_year` matches the current system year (`datetime.datetime.now().year`) to output '— Present' instead of using a hardcoded year.

Update 4: Tauri 2.0 Architectural Overhaul
- Core Migration: Completely migrated the Python/CustomTkinter application to a high-performance Tauri 2.0 executable.
- Pure Rust Backend: Rewrote all backend logic in 100% pure Rust (`rusqlite`, `reqwest`, `walkdir`, `tokio`). Removed all Python dependencies (no sidecars).
- Cinematic React UI: Replaced the CustomTkinter frontend with a modern React + Tailwind CSS + Framer Motion user interface. Implemented edge-to-edge backdrop banners, frosted glassmorphism elements, and spring-based page transitions for an "Apple TV / Plex" style experience.
- VLC Thread Polling: Reimplemented the infinite-loop VLC heartbeat monitor entirely in a Rust background thread using `std::process::Command` to spawn VLC and `reqwest` to poll the `status.json` HTTP interface, retaining exact sub-second playhead precision without Python.
- Legacy Schema Preservation: Rust `DataManager` opens the existing legacy SQLite `.db` files and safely applies evolutionary schemas (adding missing newer columns automatically using `IF NOT EXISTS` constructs) to ensure no user history or manual mappings are lost.
- Binge-Block Logic: Ported the sequential Auto-Binge session grouping algorithm to the new Rust application lifecycle.

Update 5: Cinema-Grade UI Overhaul
- Shell Refactor: Rebuilt `App.tsx` with a fixed-width glassy sidebar (`#141519/80`, `backdrop-blur-xl`), vibrant active states (`#FF6B00` borders), and a new top bar featuring a pill-shaped search input over a transparent blurred backdrop.
- Dashboard Redesign: Overhauled `Dashboard.tsx` to match premium media interfaces (Apple TV/Plex).
  - Implemented an immersive 450px high Hero Banner with gradient fades (`bg-gradient-to-t from-[#0D0F14]`) for the "Up Next" episode, including a calculated resume progress bar.
  - Added horizontal, snap-scrolling `Continue Watching` row with dynamic progress bars and animated hover states.
  - Added a `Recently Added` grid of posters with Framer Motion scaling, shadow lifting, and a glass-blur Play button overlay on hover.
  - Revamped the `Quick Stats` section into a row of sleek, frosted glass cards.
- Dynamic Data Integration: Restored Tauri `invoke` calls to populate the UI with actual backend data, seamlessly falling back to high-res Unsplash placeholders when TMDB image paths are missing.
- Async UX: Integrated Animated Skeleton Loaders (pulsing `#1F222A` layout boxes) to maintain layout structure and premium feel while Rust data fetches.

Update 6: Polishing the Cinematic UI
- Secondary Panels: Applied a glassy semi-transparent background (`bg-[#1F222A]/60 backdrop-blur-md`) to cards and panels.
- Sidebar Active State: Replaced the solid boxy active state with a glowing `#FF6B00` vertical line and a subtle gradient, with text turning bright white.
- Global Search Bar: Added a sleek, pill-shaped global quick search bar to `App.tsx` that filters the `Dashboard` and `Library` local items seamlessly.
- Hero Banner Polish: Enforced the vibrant orange "UP NEXT" label formatting, pure white title, and layered a directional gradient from bottom-left to top-right to ensure text readability against light backdrops while fading seamlessly into pure `#0D0F14`.
- Poster Badges: Transformed the top-right poster badges into slightly transparent `bg-black/60 backdrop-blur-md` pill shapes, utilizing the uniform `#FF6B00` for the stars and text.
- Stats Bar Refinement: Re-styled the Dashboard Stats cards into sleek, centered widgets with massive, bold `#FF6B00` text to emphasize immersion over raw data.

Update 7: Inbox & Tracker Stability Fixes
- Inbox Mapping Workflow: Completely redesigned `InboxView.tsx` to act as an active triage center. Users can now click on autogenerated local file groups, automatically populating an adjacent TMDB search bar to assign those files securely to specific database episodes.
- Tauri Backend Synchronicity: Rearchitected `commands.rs` to execute `add_to_tracker` and the new `assign_unmatched_to_tracker` strictly on the synchronous thread instead of deferring to background `thread::spawn` blocks, ensuring the UI correctly waits for actual DB commits before proceeding.
- Dev Watcher Hot-Reload Fix: Configured Tauri's dev server (`.taurignore`) to ignore the `WatchMark/` application data directory, preventing infinite restart loops when runtime configurations or databases are updated.
- Browser Safe-Guards: Wrapped Tauri IPC `.invoke()` calls with protective exception handlers in the React frontend that quietly suppress specific 'reading invoke' TypeErrors, ensuring testability in standard browser environments without intrusive alert boxes.

Update 8: Framer Motion Engine 1.5 Integration [Todo 1.5]
- Global Cinema Mode: Implemented a global toggle via Zustand (`useAppStore.ts`) that correctly pulls the default from OS reduced motion preferences and syncs with `settings.json` on the Rust backend.
- Navigation Fluidity: Wrapped the main view router in `<AnimatePresence mode="wait">` to provide smooth cross-fades when navigating the sidebar.
- Layout Animations: Utilized `<AnimatePresence mode="popLayout">` and `layout="position"` in `Library.tsx` grids and `Dashboard.tsx` carousels so items dynamically fill space when sorted or removed without snapping layout logic.
- Performance Scaling: Staggered entry animations are now strictly limited to the first 20 items in lists, with standard viewport fade-ins applied to subsequent items to handle 500+ item grids flawlessly without tanking the CPU.
- Animation Lockout: Implemented an `isAnimatingRef` in `MediaDetails.tsx` for destructive 'Remove Show' events. It optimistically triggers the exit animation immediately and then fires the actual backend SQL cleanup cleanly without locking the UI.
- Backend Sync: Built out the full cascading `delete_media_cmd` functionality in Rust, securely dropping all relational tracking rows from `History`, `Local_Files`, `Episodes` and wiping `cache` files from local storage on deletion.

Update 9 [Todo 1.6] - Performance Update: Implemented memory optimization layer (IntersectionObserver, SQLite Mutex, Release Strip).

Update 10: UI Polish - Desktop-Native Experience
- Custom Modals/Dialogs: Replaced native browser `window.alert`, `window.prompt`, and `window.confirm` with a custom lightweight React + Tailwind `Modal` component featuring dark backgrounds, rounded corners, and Framer Motion scale-up animations.
- Toast Notifications: Integrated `sonner` for non-blocking, premium toast notifications that slide in from the bottom right for success/error messages (e.g., "Added to Tracker!").
- Seasons List: Replaced the unstyled white horizontal scrollbar in the Seasons list with a clean `flex-wrap` layout (`gap-2`), ensuring seasons dynamically stack on multiple lines for better visibility and a cleaner desktop feel.
- Window Title Fixes: Updated `tauri.conf.json` and `index.html` to set the application window title to `WatchMark`, permanently removing "localhost:1420" and "watchmark-tauri" to create a cohesive native application feel.
- Replaced manual directory typing prompt with Native Tauri Dialog plugin (`@tauri-apps/plugin-dialog`) for a real OS File Picker in Inbox and Settings.

Update 11: Windows Native Optimization & System Integration [Todo 1.7]
- Ensured path separators (\ vs /) are handled dynamically using Windows native logic with `std::path::PathBuf` and the `\\?\` prefix for long paths.
- Created `pathUtils.ts` in React to normalize paths strictly to Windows `\` format visually.
- Implemented a robust Windows System Tray (`tauri-plugin-shell`, `tauri-plugin-notification`, `tray-icon`) with a Context Menu featuring Quick Actions: Scan Directory, Resume Last Show, Check for Updates, and Quit. Double-clicking the tray immediately restores and focuses the app.
- Configured NSIS installer with `installMode: "currentUser"` to bypass UAC prompts and handled `PermissionDenied` errors gracefully in the recursive directory scanner.
- Verified SmartScreen mitigations by appending valid company and copyright metadata in `tauri.conf.json`.
- Handled Global Keyboard Shortcuts securely using `tauri-plugin-global-shortcut` (Ctrl+Shift+S for Scan, Ctrl+Shift+H for Hide/Show) with native Toast fallback if shortcuts are already claimed by other Windows apps.

Update 12: Seamless IPC Bridging & Stability [Todo 1.9]
- Implemented timeout safety using `tokio::time::timeout` for heavy Tauri commands (e.g. `add_to_tracker`, `perform_tmdb_search`) to prevent the backend from hanging on deadlocks.
- Created a frontend `invokeWithTimeout` helper in React to stop waiting for frozen IPC calls and allow the UI to recover cleanly.
- Secured path inputs utilizing the `dunce` crate in Rust to safely canonicalize directory paths and prevent injection panics.
- Refactored `fetch_history` and `get_library_data` by moving their massive serialization loops into `tokio::task::spawn_blocking` to completely offload them from the main Tauri event loop and keep the UI responsive during massive data loads.
- Implemented proper pagination chunking in the History view (limit 100 per page) with a "Load More" action.
- Centralized event listener cleanup in React components (e.g., `App.tsx` and `SettingsView.tsx`) to ensure unlisten functions are explicitly awaited and correctly executed on unmount, preventing HMR ghosting and duplicate listeners.
- Integrated a global type-mismatch error handler in the frontend wrapper that gracefully intercepts `serde` deserialization mismatches and displays clear warnings rather than failing silently.

Update 13: Local App Data Directory Stability [Todo 1.13]
- Shifted application data resolution from portable logic directly to the OS-sanctioned `AppData\Local\WatchMark` (via `%LOCALAPPDATA%`) to prevent Windows Defender and Controlled Folder Access blocks.
- Implemented a pre-flight Canary Check in `main.rs`. If the directory lacks write access, Tauri does not boot and instead spawns a fatal native OS dialog via `native-dialog` crate.
- Added JIT (Just-In-Time) directory recreation in `db::ensure_directories()`. This runs immediately before any SQLite `Connection::open()` or external HTTP `reqwest` download logic, preventing crashes if users manually delete `/cache` folders during application runtime.
- Re-architected SQLite migration initialization with a rigid 3-Tier `PRAGMA user_version` engine. Ensures the base schema (V1), metadata columns (V2), and session history columns (V3) are transaction-safe, atomic, and will correctly rollback rather than throw 'column already exists' panics.

Update 14: Persistent Settings Storage & Auto-Repair [Todo 1.14]
- Refactored `Settings` schema to use strict `i32` integer types for window dimensions and positions instead of legacy string parsing, avoiding type-conversion rounding errors.
- Implemented robust `serde_json` defensive deserialization that handles missing keys, applies sanity bounds (e.g. width > 0), and auto-corrects floats saved by external editors.
- Engineered an Auto-Repair fallback loop in Rust that catches JSON syntax/corruption errors on boot, immediately generates a clean default configuration, and writes it back to disk to prevent fatal crashes.
- Secured the TMDB API Key storage using the `keyring` crate to persist sensitive data natively in the Windows Credential Manager. Fallback to Base64 obfuscation if the system credential service is unavailable.
- Introduced a debounce mechanism (500ms delay via `tokio::sync::mpsc`) in the backend to batch multiple rapid `save_settings` calls into a single atomic disk write, protecting against concurrent UI tab race conditions.
- Enhanced Boot Verification logic to explicitly re-read the `settings.json` file immediately after creation. If the OS denies write access (e.g. read-only AppData directory), the app throws a fatal, human-readable native dialog before Tauri initializes.
- Updated the React Settings UI to bind directly to the new `width` and `height` integer fields.

Update 15: Dedicated Settings UI Page Layout [Todo 16.8]
- Restructured `SettingsView.tsx` into a Master-Detail layout with a vertical sidebar navigating between 'General', 'Playback', 'Scanner', 'System', and 'Advanced' tabs, utilizing the glowing `#FF6B00` active state indicator.
- Implemented a precise `grid-cols-[250px_1fr]` Tailwind layout for all setting rows, ensuring text inputs, number fields, and pill selectors are perfectly left-aligned while descriptions sit neatly under bold headers.
- Engineered a robust `isDirty` state tracker that deep compares current form values against the `initialSettings` loaded from Rust.
- Built an animated Floating Action Bar (Footer) using `framer-motion` that gracefully slides up from the bottom of the viewport when unsaved changes are detected, offering distinct "Save" (solid orange) and "Discard" (transparent with border) actions.
- Integrated a Navigation Guard system in `App.tsx`. If the user attempts to switch sidebar tabs while `isDirty` is true, a 3-option confirmation modal ("Save & Leave", "Discard & Leave", "Cancel") intercepts the route transition securely.
- Expanded the Rust `models.rs` and `settings.rs` to include and serialize the new configuration parameters (`language`, `auto_complete_threshold`, `binge_grouping_hours`, `auto_resume`, `auto_scan_on_boot`, `logging_level`) without breaking backwards compatibility.

Update 16: Database Initialization, Concurrency & Foreign Key Cascading [Todo 5.1]
- DB Lazy Creation: Integrated explicit OpenFlags (`SQLITE_OPEN_CREATE | SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_URI`) ensuring rusqlite constructs the `watchmark.db` database properly on cold boot.
- PRAGMA Concurrency Tuning: Applied WAL (Write-Ahead Logging) to `db.rs` ensuring high-speed concurrent read/write behavior, preventing `database is locked` panics during bulk operations.
- AppData Canary Verification: Added `check_db_permissions` which tries to open the database file explicitly, or writes a temporary `.canary` file. If the OS denies write or the file is heavily locked, it triggers a custom `AppError::Fatal` resulting in a native Windows warning to the user, averting silent thread crashes.
- Atomic Migrations: Patched the SQLite schema updater in `init_db`. Wrapped schema alterations inside safe atomic `transaction()?` structures handling `duplicate column name` exceptions logically rather than bypassing standard integrity.
- Cascading Delete Hooks: Altered the base table `CREATE` commands for `Episodes`, `Local_Files`, and `History` to strictly implement `FOREIGN KEY (...) REFERENCES ... ON DELETE CASCADE`, shifting manual cleanup burden entirely onto native SQLite features while maintaining `PRAGMA foreign_keys = ON;`.

Update 17: Feature 5.2 - Media Table Schema Hardening & Polish [Todo 5.2]
- Enforced strict database constraints: `UNIQUE(tmdb_id, type)` correctly accommodates mixed Movie and TV entries.
- Implemented robust `ON CONFLICT DO UPDATE` upsert logic to ensure user ratings, history, and status survive metadata refreshes safely.
- Implemented a 10,000 character limit on synopsis text on the backend to prevent malicious API payloads from creating memory spikes.
- Integrated robust React frontend fallbacks: A "View More/Less" toggle limits long synopses to 300 chars, and clean "No overview available" fallback logic prevents empty layouts.
- Mapped explicit `"Unknown"` handling for the `MediaType` enum to guarantee database writes don't fail due to garbage API entries.

Update 18: Feature 5.3 & 5.14 - Episodes Schema, Data Sanitization, and Cascading Deletes [Todo 5.3, 5.14]
- Schema Hardening: Applied `UNIQUE(media_id, season_num, ep_num)` to the Episodes table and verified the `ON CONFLICT (...) DO UPDATE` logic safely catches API data changes without duplicating rows.
- Strict Typing: Refactored season and episode integer types from `i32` to `u32` across the Rust backend, and added `CHECK(season_num >= 0)` and `CHECK(ep_num >= 0)` at the SQLite schema level to physically prevent negative numbers.
- Safe TMDB Parsing: Implemented a robust `TmdbEpisode` struct in `models.rs` with a custom `deserialize_flexible_runtime` deserializer capable of handling strings, "N/A", nulls, or standard integers to prevent API noise from breaking the sync loop.
- UI Labels for Specials: Mapped Season 0 specifically to output the label "Specials" in the `MediaDetails.tsx` React component, and verified backend SQL logic strictly sorts by `season_num ASC, ep_num ASC` so they appear first.
- Cascading Deletes: Removed manual procedural row deletions (`History`, `Local_Files`, `Episodes`) in `delete_media()` and successfully tested full reliance on SQLite `ON DELETE CASCADE` relations.

Update 19: Feature 5.4: Local_Files Relational Table
- Added `file_size` and `file_hash` columns to `Local_Files` table.
- Added migration to version 4 of DB.
- Added `xxHash` crate for sparse hashing capability to evaluate and compare large media files.
- Built "Larger Wins" heuristic collision detection during filesystem scanning, enabling auto-upgrades to higher quality local file additions.
- Modified React Settings page to include Database path repair tools.
- Modified `EpisodeRow` in `MediaDetails` to include `Unlink Local File` using a Context Menu.
- Added Play Validation which actively queries disk availability directly when 'Play Next' or specific 'Play' buttons are clicked, launching interactive toast requests with 'Locate' file picker if unmapped.
- Validated tests confirming `Local_Files` drops safely without disrupting specific parent Episode metadata.

Update 20: Feature 5.5 History Relational Table
- Implemented the V5 database migration to convert legacy `History` table timestamps to Unix epochs (integers) using UTC `strftime('%s', 'now')`.
- Added missing `last_position` and `status` columns explicitly to `History` table schema.
- Built a smart indexing mechanism `idx_history_timestamp` across `timestamp DESC` and `id DESC` to ensure massive datasets (10,000+ entries) load seamlessly without sorting lag.
- Implemented `get_media_history_count(media_id)` directly fetching the sub-query mapped sum, integrating directly with `remove_show` logic.
- Rewrote Rust fetch_history logic to automatically format and return arrays clustered by `session_id` and 6-hour timestamp boundaries as "Binge-Blocks".
- Implemented `mark_season_watched` to explicitly handle "Archive Mode" (ignoring History logs while filling the Episode count metrics).

## TODO 5.6 Unmatched_Files Staging Table
- Handled edge cases where "Generic" filenames (e.g. `01.mkv`) are successfully processed by walking up to the parent directory and stripping out numeric noise.
- Handled parsing extremely dirty anime titles by aggressively stripping trailing `[1080p][CRC]` bracket blocks.
- Solved missing files prune-logic by creating a cross-platform directory tree walker. If the file is missing but its parent directory exists, it's purged. If the root drive is disconnected, it's gracefully kept.
- Re-architected the Rust `scanner.rs` to batch `INSERT` statements into chunks of 500, resolving extreme slow-downs when scanning 10,000 unrecognized anime files.
- Improved auto-migration logic: if a file in the Unmatched pool is matched manually or via automatic heuristics, it is atomically removed from the Inbox.
- Added a "Clear Inbox" soft-truncation button in the UI executing `DELETE FROM Unmatched_Files`.

Update 21: Feature 5.7 release_date column and precise library sorting
- Added `release_date` mapping for Movies and `first_air_date` mapping for TV Shows from TMDB.
- Added `is_exact_date` to `Media` and `Episodes` to differentiate between YYYY strings that have been padded to 'YYYY-01-01' and actual precise dates.
- Implemented robust date padding logic (YYYY becomes YYYY-01-01) in Rust before DB insertion for safe SQL DATE ordering.
- Implemented `NULLS LAST` sorting fallback directly in SQLite commands, keeping unreleased media at the bottom of standard `DESC` ordering.
- Replaced the "Play Next" and local "Play" buttons with a disabled "📅 Coming Soon" indicator and applied grayscale styling when the `air_date` or `release_date` evaluates to true for `is_unaired` using `chrono` logic.

## TODO 5.8: air_date column for episode 'time capsule' comparisons
- Added logic in Rust (`calculate_gap` in `commands.rs`) to calculate the Duration between `Episodes.air_date` and `History.timestamp`.
- Included accurate math conversions taking leap years and timezone boundaries into account via the `chrono` crate.
- Added structured output of "Gap Object" representing the difference in years, months, and days.
- Designed dynamic directional tracking allowing for negative duration reporting for "Early Watch" occurrences.
- Integrated accurate defaults when air times are omitted by falling back to assumed midnights.
- Configured frontend React component (`History.tsx`) to render visually distinct badges depending on the time capsule object.


## TODO 5.8 (Followup): Backdate Contextual Constraint Logic
- Implemented `is_air_date_manual` in the `Episodes` SQLite table and schema migrations (V7) to protect manually overridden dates from `Refresh Data` TMDB Syncing.
- Augmented the `add_to_tracker` sync engine to perform a dynamic check against `is_air_date_manual` before silently inserting data updates, safeguarding user custom input.
- Validated new strings like 'TBD' or fundamentally corrupted API returns dynamically dropping the update assignment and protecting the local DB string.
- Added `backdate_season` core logic enforcing contextual spreading, meaning if a user specifies a month with 30 days and 10 episodes, the history arrays naturally span apart safely.
- Wrote and tested verification edge cases against timeline gaps accurately assigning early viewing booleans.


## TODO 5.9 is_legacy boolean flag for handling archived/backdated history
- Added the `is_legacy` boolean strictly defaulting to 0 (false) for real-time natural watching.
- Ensured legacy rows can be safely updated (e.g. QoL date editing) without the `is_legacy` status being silently overwritten or destroyed.
- Implemented frontend React conditional rendering in `History.tsx` to display legacy items with 60% opacity "muted" styling, an "ARCHIVED" badge, and bypassing the expandable accordion pause logs.
- Engineered a background Task Queue (`task_queue.rs`) utilizing `std::sync::mpsc` channels to seamlessly perform low-priority mass insertions of 500+ legacy rows.
- Refactored `archive_season` to automatically backdate and spread legacy imports 24 hours apart, starting from the 1st of the month, successfully escaping database `is locked` deadlocks.
- Verified analytics counting stats directly query the database seamlessly ignoring `is_legacy`, so lifetime "Total Episodes Watched" dynamically integrates archives natively.
-
### Feature 5.10 - User Rating Integration
- Implemented `user_rating` column in SQLite `Media` table with `CHECK(user_rating >= 0 AND user_rating <= 10)` constraint.
- Setup DB migration `PRAGMA user_version = 8` to update legacy rows (from `0` to `NULL`).
- Created `update_media_rating` Rust command.
- Integrated "My Top Rated" `DESC NULLS LAST` sorting in `get_library_data`.
- Implemented `StarRating` React UI Component with Framer Motion, supporting 1-10 scores via half-star clicks.
- Updated `Dashboard.tsx` and `Library.tsx` logic to visually distinguish an explicit score of `0.0` from `Unrated`.

### Feature 5.11 - TMDB Score Caching (`vote_average`)
- Modified TMDB API parsing to round `vote_average` float to one decimal place immediately upon retrieval (e.g., 7.6666 -> 7.7) to prevent floating-point drift.
- Ensured a TMDB score of `0.0` is treated as a valid numeric state and stored successfully.
- Implemented frontend rendering in `MediaDetails.tsx` to strictly format the score to one decimal place (`Number(val).toFixed(1)`), maintaining UI consistency (e.g., 8.0, not 8).
- Rendered a specific "NR" (Not Rated) badge with distinct styling for new shows returning `0.0`, preventing misleading 0/10 scores.
- Added "Sort by TMDB Rating" filter option to the `Library.tsx` interface and connected it to the backend `get_library_data` logic using `DESC NULLS LAST` ordering.
- Created `idx_media_tmdb_rating` SQLite index in `PRAGMA user_version = 9` migration to optimize massive library sorting performance.

### Update Feature 5.12: Resume Playback
- **Database Schema:** Bumped `PRAGMA user_version` to 10 and implemented `last_position INTEGER NOT NULL DEFAULT 0` within the `Episodes` table.
- **VLC Throttling & Clamping:** Enhanced `vlc_heartbeat` to clamp `last_position` dynamically preventing overshoots, while batching writes intelligently on pause events, >30s scrubbing, or every 5-minutes avoiding excessive SSD locking.
- **UI Progress Parity:** Enforced mathematical consistency in React computing progress percentage dynamically, complete with a persistent 2px fallback for extremely minor viewing slivers and an automated green success-shift when >90% watched.
- **Auto-Complete Zone Logic:** Upgraded the heartbeat poller to ignore stale resume states globally inside the 10-second threshold bounds guaranteeing seamless auto-completion flows at file closure.
- **Strict Manual Reset:** Upgraded API endpoints ensuring any toggled status change immediately forces `last_position = 0` explicitly blocking stuck memory on repeat viewings.

### Update Feature 5.13: Binge-Block Session Grouping
- **UUID Session Engine:** Integrated Rust's `uuid` crate generating secure v4 UUIDs passed to a persistent SQLite `session_id` string-mapping column tracking continuity naturally across multiple shows and application restarts.
- **Smart Legacy Archival:** Ran an auto-migration ensuring prior existing unstructured History logs cleanly map as explicit `NULL` references, automatically funneling seamlessly into legacy/archived stateless UI blocks natively.
- **Vibe Label Calculations:** Advanced React's timeline rendering intelligently calculating date ranges ("Morning, Afternoon, Evening, Late Night") resolving complex midnight boundary edge-case splits naturally.
- **Temporal Grouping Rules:** Enforced rigid mathematical temporal logic enforcing < 6 hours as the core chaining rule, with Rust dynamically calculating trailing epochs to resurrect continuous sessions instantly on VLC spawn.

### Feature 5.14: Multi-table cascading deletes (Pro Stability Upgrade)
- **Asynchronous Teardown:** Refactored standard Tauri UI synchronous media deletion (`delete_media_cmd`) into an asynchronous task queued strictly within the Background Worker's Low Priority channel. This explicitly neutralizes UI locks and freeze frames previously caused by waiting for the standard SQLite thread handler to tear down massive datasets (e.g. 50,000+ History entries).
- **Processing State UI:** Implemented a rich, native "Processing..." `ProcessingModal` utilizing standard Framer Motion layout configurations to blur and disable the UI cleanly. Listens strictly for Rust's global `media-deleted` payload channel avoiding frontend race conditions entirely.
- **Industrial Integrity Tests:** Expanded Rust database unit tests matching standard requirements strictly assessing missing/malformed `PRAGMA foreign_keys = ON` execution constraints, Deep Tree mass relation cascading, and specific structural raw SQL Orphan Pass query executions guaranteeing table alignment unconditionally.

### Feature 25.1: Automated SQLite Backups
- **Backup Logic**: Added background Tokio task checking `last_backup_timestamp` periodically and executing `VACUUM INTO` over SQLite database file into `backups/watchmark_YYYY-MM-DD_HH-MM.bak` to ensure clean atomic copy without halting app operation.
- **Retention Logic**: Added automatic pruning feature to strictly retain only 3 recent backups in the data folder.
- **Safeguards**: Employed `fs3` to enforce 10MB free disk space requirement before vacuums to prevent disk corruption.
- **UI Updates**: Configured frontend `SettingsView` via IPC listener updates showing 'Last automated backup' status labels.

### Feature 25.2: Manual DB Backup Action
- **UI Component**: Converted 'Backup DB' button in the 'System' settings tab into an interactive `motion.button` tracking an `isBackingUp` state, rendering a `Loader2` spinning icon and transitioning into a `cursor-wait` state while actively executing.
- **Dialog Wiring**: Wired Tauri's `@tauri-apps/plugin-dialog` to launch standard OS Save File dialogs utilizing `documentDir` as a fallback, pre-filling a structured suggestion (e.g., `WatchMark-Backup-YYYY-MM-DD.db`), restricted specifically to `.db` or `.sqlite` formats.
- **High-Priority Atomic Copy**: Engineered an async `export_database` Rust command specifically pushing the SQLite `VACUUM INTO ?` SQL instruction exclusively into the `High Priority` DbTaskQueue, guaranteeing a pristine, non-corrupt snapshot.
- **Overwrite Safety**: Included a strict OS `fs::remove_file` catch immediately preceding execution to prevent standard `VACUUM INTO` operations natively from crashing when presented with existing identically-named destination files.

### Feature 25.3: Manual DB Restore (Completed)
- Added a full cold-swap architecture to bypass OS file-locking constraints on Windows when replacing the active SQLite database.
- Implemented `prepare_restore` which stages the backup file at `watchmark.db.pending` and drops a `.restore_pending` trigger file into the app data root.
- The Tauri Boot Sequence (`main.rs`) now checks for this trigger before initializing `rusqlite` and renames the files (`.old`, `.pending` -> `watchmark.db`) to ensure atomicity.
- Included an extensive React UI "Danger Zone" block with a "Type RESTORE" confirmation modal. Added safety interlocks in `useTaskStore` (`isScanning`, `isBackingUp`) to prevent the user from performing a restore while database writes are active.
- Integrated `localStorage` persistence before restart to guarantee a celebratory notification is emitted upon boot.

### TODO 25.4
- Implemented `optimize_database` command in Rust that runs `VACUUM` and `ANALYZE`.
- Implemented `is_maintenance_mode` lock via `AtomicBool` in `AppState` to prevent UI collisions during optimizations.
- Built a `OptimizationModal` using Framer Motion with a pulsing logo to block UI interactions while the process runs.
- Connected the Settings 'Clean Database' button.
- Added file size calculation logic to tell the user exactly how much disk space was saved in MB/GB formatting via a toast notification.
- Handled disk space edge cases using the `fs3` crate before starting the memory-intensive SQLite VACUUM operation.

### Feature 1.10: Single-thread centralized Task Queue
- **Task Queue Architecture:** Upgraded `DbTaskQueue` to support a typed `DbAction` Enum containing actions like `UpdateMediaRating`, `DeleteMedia`, and `Batch`.
- **Ephemeral Read Connections:** Introduced `db::get_readonly_connection()` to spawn ephemeral, read-only connections exclusively for React queries (like dashboard and library fetching) to avoid locking the writer thread.
- **Worker Thread Retry Logic:** Implemented an execution retry wrapper in the worker thread that catches `SQLITE_BUSY` errors, pauses for 100ms, and retries up to 3 times before discarding the task.
- **Failed Tasks Guard:** If a write operation hits the maximum retries, the worker writes the error payload to `watchmark_failed_tasks.log` and emits a `db-write-failed` event to the frontend.
- **Shutdown Drain Signal:** Intercepted Tauri's `WindowEvent::CloseRequested` to prevent immediate application termination. The app now pushes a `Shutdown` signal to the database queue with a `oneshot::channel` to await a safe drain (up to 2 seconds) before exiting.
- **Frontend Feedback:** Connected the new `db-write-failed` IPC event to a highly visible, Danger-themed `sonner` toast notification.
- **WebView Stabilization:** Addressed a Windows-specific WebView2 crash (where raw HTTP headers were dumped to the window output) by safely utilizing `try_state::<commands::AppState>()` inside early Tauri window lifecycle events, preventing unmanaged state panics.
- **Custom Protocol Hardening:** Injected strict dynamic MIME type headers (`Content-Type`) into the `watchmark://` custom URI scheme responder, preventing WebView2 from occasionally corrupting the document DOM stream by interpreting raw binary chunks as text.
- **Locking Performance:** Upgraded SQLite `busy_timeout` PRAGMAs to strictly use numeric types rather than strings, resolving a 15-second initialization deadlock.

### Bug Fix: Tauri/Vite Proxy Misconfiguration
- Resolved an issue where Tauri's internal proxy was throwing raw HTTP headers to the WebView2 engine instead of rendering the React application (`Access-Control-Allow-Origin: http://localhost:1420`).
- This was resolved by fixing a path misalignment issue in the Vite configuration. Although the `index.html` was correctly positioned at the project root (`watchmark-tauri/index.html`), Vite's default root behavior occasionally conflicts with Tauri v2's proxy expectations.
- Added `root: "."` directly to `vite.config.ts`, explicitly forcing Vite to serve the root directory to Tauri without ambiguity.
- Ensured `tauri.conf.json` strictly matched via `devUrl: http://localhost:1420` eliminating the possibility of older `devPath` schema conflicts causing the WebView misfire.

### Bug Fix: Localhost IPv6 Binding Conflict
- Fixed a bug where Tauri WebView2 on Windows 11 attempted to resolve `localhost` via IPv6 (`::1`), while the Vite dev server bound exclusively to IPv4 (`127.0.0.1`). This proxy mismatch resulted in raw HTTP headers and chunked streams rendering as plain text directly onto the main screen.
- Configured Vite (`vite.config.ts`) to explicitly bind to `127.0.0.1` instead of `localhost`.
- Configured Tauri (`tauri.conf.json`) to specifically target `http://127.0.0.1:1420`, strictly ensuring the underlying proxy handshakes succeed.

### Update 1.2: Tauri Plugin Log Configuration
- Upgraded the `tauri-plugin-log` integration in `main.rs` to instantiate dual targets during setup. It now logs robust application tracing implicitly to standard console output (`Stdout`) and persistently to the structured AppData `/logs` directory under `watchmark.log`, dramatically improving diagnostic resolution times for local environments.

### Update 1.2.1: Tauri Plugin Log Output Verbosity & Tracing
- Elevated the `tauri_plugin_log` output verbosity explicitly to `log::LevelFilter::Debug` inside `main.rs` to ensure network diagnostics and SQLite traces populate seamlessly into local outputs.
- Modified the file logging strategy targeting `app.log` inherently inside the `LogDir` instead of `watchmark.log`, enforcing strict Tauri OS-agnostic conventions.
- Deployed structured `log::info!` lifecycle checkpoints spanning critical architectural boot components (`canary_check()`, `db::init_db()`, and the Tauri `setup` hook state management phase) mapping accurate stack execution progress locally for debugging initialization faults.

## TODO 1.11 Asynchronous multi-threaded read operations
- Created database indices on `History(timestamp DESC)` and `Media(title ASC)`.
- Replaced direct backend execution of read queries with a dynamic Thread Pool managed by a global `tokio::sync::Semaphore` initialized to `available_parallelism() - 1`.
- Added UUID `tokio_util::sync::CancellationToken` tracking in `AppState`.
- Created a `useAsyncInvoke` hook in React that automatically aborts backend read tasks if the UI unmounts.
- Created an in-memory `stats_cache` in the Rust backend to prevent expensive SQL operations on global stats. Write-through invalidation clears this cache upon status changes.

### Update Feature 1.12: Strict NoneType/null data sanitization layer in Rust
- **Date Sanitization (`sanitizer::sanitize_date`)**: Built a robust Rust parser to strictly format `YYYY` to `YYYY-01-01` ensuring layout consistency. Invalid or `null` strings fallback immediately to `0000-00-00` safely.
- **Frontend TS Overhaul**: Refactored arrays out of optional chaining (`?`) directly into guaranteed instances (`[]`), integrating `is_date_known` dynamically into `TBD` rendering overlays.
- **Whitespace Scrubbing (`sanitizer::sanitize_text`)**: Engineered Regex scrubbers intercepting RAW HTML tags (`<p></p>`) and blank lines specifically replacing them seamlessly with `"No overview available"` fallbacks, guaranteeing layout grid heights remain stable.
- **Numeric Division Safety (`sanitizer::calculate_progress_percentage`)**: Intercepted missing/`0` runtime values explicitly routing them to `1` behind the scenes, outputting pre-calculated `progress_percentage` floats without passing `NaN` or `Infinity` payloads over the IPC.
- **Guaranteed Collections**: Validated strict `Vec::new()` and `#[serde(default)]` struct attributes over all nested serialization routines (`Media`, `Episode`, `HistoryEntry`) so React arrays `.map()` execute gracefully without exceptions on First-Boot empty DB configurations.

## Feature 3.1: Secure HTTP requests via Rust reqwest (Ultra-Stable Network Engine)
- **Middleware Pipeline:** Integrated `reqwest-middleware` and `reqwest-retry` to implement a robust retry policy for all external TMDB API requests using `ExponentialBackoff` logic (max 3 retries) to seamlessly recover from temporary network drops or 5xx server errors.
- **TLS/HTTPS Enforcement:** Migrated to `rustls-tls` backend natively in Rust and strictly enforced `.https_only(true)` for TMDB communication, preventing any protocol downgrade attacks or invalid certificate man-in-the-middle exploits.
- **Two-Tier Timeout System:** Added a 5-second `connect_timeout` and a 15-second `timeout` to catch hanging sockets and prevent the UI from spinning infinitely during slow network conditions.
- **Structured Error Mapping:** Extended the `AppError` enum to parse `NetworkTimeout`, `NetworkBlocked`, and `NetworkOffline` into specific React-friendly JSON objects (e.g., `{ "code": "TIMEOUT" }`), allowing the frontend to easily display localized feedback instead of generic errors.
- **Dual-Client Loopback Bypass:** Implemented a `NetworkManager` in `src/network.rs` that maintains two isolated clients: `external_client` for secure, retrying TMDB requests and a lightweight `local_client` for low-latency, HTTP VLC Heartbeat polling on `127.0.0.1`.

### Feature 3.2: On-the-fly TMDB API Key validation
- Added Rust backend command `validate_tmdb_key` calling the `/3/configuration` endpoint.
- Sanitized pasted keys to trim whitespaces and hidden zero-width characters.
- Modified Settings UI (`SettingsView.tsx`) to debounce typing and dynamically render validation states (spinner, check, error cross, warning triangle).
- Disabled saving logic globally if validation is pending or returned invalid.
- Mapped HTTP 429 response strictly to a retry-after state UI warning.
- Centralized auth state in Rust (`AppState::is_api_authorized`) and React (`useAppStore::isApiAuthorized`).
- Globally halted all TMDB queries (sync, search) instantly upon a 401 unauthorized fetch and updated the Search and MediaDetails UI to explicitly prompt users for the key.
- Addressed code review feedback: Made sure saving a valid key in Settings correctly restores the `isApiAuthorized` state back to `true`, and ensured `state.db_queue.clear()` is explicitly called upon an API key revocation (401 error) to clear pending sync metadata tasks.

### Rust Backend Fixes
- **Commands Sync:** Fixed missing `__cmd__save_settings` compiler error by adding the `#[tauri::command]` attribute back to the `save_settings` function in `src-tauri/src/commands.rs`.
- **Queue Logic:** Fixed `state.db_queue.clear()` compiler error by adding a `.clear()` method to `DbTaskQueue` in `src-tauri/src/task_queue.rs`. Note that due to `std::sync::mpsc::Sender` constraints, this serves as a logging placeholder while task cancellation effectively relies on the atomic `is_api_authorized` lock interceptor.



### Fix Compilation Errors in perform_tmdb_search (Task 1.11 Support)
- Fixed borrowing lifetime issues (`E0505`, `E0521`) inside the `tokio::task::spawn` in `perform_tmdb_search`.
- Cloned `tauri::AppHandle` and extracted `AppState` dynamically within the spawned task to safely mutate `is_api_authorized` and `db_queue` states.
- Re-implemented the `tokio::select!` block inside the spawned task to correctly race the TMDB network request against the `CancellationToken` (Task 1.11).
- Addressed dead code warnings in `models::Stats` and `models::DashboardData` by adding `#[allow(dead_code)]` decorators.
- Fixed test compilation failures by properly initializing `is_api_authorized` in `commands_tests_optimize` and `commands_tests_tmdb_auth`.

## TODO 3.3
- Implemented `search_media` with multi-search endpoint support in Rust.
- Included pagination parameter `page` and wired it up via `perform_tmdb_search`.
- Scraped control characters before fetching and natively let `reqwest` handle percent encoding of emojis and reserved characters.
- Built strict filtering and mapping: extracting `known_for` arrays from `person` results, deduplicating via TMDB ID, and explicitly ignoring `collection` media types.
- Revamped `SearchTMDB.tsx` utilizing a 500ms `useDebounce` hook to throttle API calls and minimize network load.
- Added `IntersectionObserver` at the bottom of the grid for "Infinite Scroll" capability on massive result sets.
- Distinctly badged mixed TV and Movie cards with custom Emerald/Blue borders and high-fidelity Lucide Icons.
- Handled edge cases for date formatting per requested rules (e.g., displaying `2013-` for TV Shows vs just `2013` for Movies).
- Attached extensive test suite ensuring same-name collisions, actor name flattening, and character encoding execute flawlessly.


## Feature 3.4: Dedicated TV Show deep-data fetching
- Added `genres` and `networks` flattened fields to `Media` schema via migration.
- Added `season_overview` to `Episodes` schema via migration.
- Integrated iterative Fetch-and-Commit streaming loop in `add_to_tracker` to prevent RAM spikes on 50+ season shows.
- Refactored `assign_unmatched_to_tracker` to also use iterative streaming loop.
- Added empty-string sanitization mapping missing images to `Value::Null` for data integrity.
- Integrated `SafeImage` component across Dashboard, Library, SearchTMDB, Inbox, and History for robust image fallbacks without layout shifts.

### Update 22: Strictness Bug Resolution (Library Missing Items)
- **Root Cause Identified:** The recent "Sanitization" updates introduced two new columns (`genres` and `networks`) to the `Media` SQLite table. However, queries in `get_library_data` and `get_dashboard_data` still relied on hardcoded `rusqlite` row indices (e.g. `row.get(13)`) that were mapped directly following `m.*`. This push resulted in pulling the `genres` TEXT field instead of the calculated `completed_eps` INTEGER, throwing a silent `InvalidColumnType` parsing panic, destroying the data payload, and rendering an empty UI.
- **Index Shift Applied:** Re-mapped indices accurately (e.g., `completed_eps` now correctly targets index 15, `last_watched` at 16, etc.) to reflect the newly expanded column widths, permanently resolving the "Strictness Bug" without database locking or logic regressions.
- **Frontend Restoration:** Verified the fix correctly revives the `SearchTMDB.tsx` and `Library.tsx` integration where successfully added shows immediately populate as expected.

### Update 14: Feature 3.5 - Dedicated Movie Deep-Data Fetching

**Summary:**
Implemented specialized deep-data fetching strategies for Movies, effectively mapping them into a unified "1 Unit" format to seamlessly integrate with the application's existing history tracking engine, while preserving critical cinematic metadata like Collections and explicit Release Date handling.

**Core Architectural Features Implemented:**
- **The "Unit" Logic (Virtual Episode Mapping):** Movies are treated as a single, watchable unit mapped to a Virtual Episode (Season 1, Episode 1) containing the main runtime. The episode title and overview strictly mirror the movie title and synopsis, allowing the timeline tracker and database relationships to function natively without conditional database joins.
- **Franchise/Collection Pre-Fetching Strategy:** Implemented `collection_id` and `collection_name` natively into the `Media` SQLite table and extracted `belongs_to_collection` data during TMDB synchronization. The backend completely pre-caches this relationship so the UI can instantly display related franchise properties entirely offline.
- **Strict Data Firewall (Fallback Placeholders):** Implemented the standard "No overview available." fallback and runtime math sanitization directly at the Rust backend layer.
- **Release Date Sentinel System (NULL DB storage):** If a movie lacks an explicit release date (TBA), the Rust backend strictly stores a `NULL` directly inside SQLite rather than dropping it. This specifically preserves native SQL `NULLS LAST` sorting. During frontend serialization, the Rust layer dynamically wraps this `NULL` into a type-safe string `"0000-00-00"` so the React UI never crashes on `undefined` values and can cleanly format "TBD" badges.
- **Logic Gatekeeping (Bypassing TV Loops):** The background `add_to_tracker` sync worker now strictly checks the extracted `MediaType` and executes exactly *one* query and bypasses all season-by-season iterators to prevent "Sync Loops" entirely. Applied a `UNIQUE(tmdb_id, type)` multi-column database constraint to protect against movies sharing an ID with a TV series.

**Database Schema Migrations:**
- Bumbed `user_version` to `14`.
- Added `collection_id INTEGER` and `collection_name TEXT` to the `Media` table safely via `conn.transaction()`.

**Edge Cases Tested & Verified:**
- Handled the "Zero-Minute Short" preventing UI divide-by-zero crashes.
- Ensured massive 10-hour runtimes cast gracefully into `i32` bounds.
- Parsed "Ghost" collection lists smoothly without throwing unwraps when sparsely defined by TMDB.
- Tested missing and NULL `release_date` strings and formatting loops.

### Update 15: Feature 3.6 - Season-by-season iterative API fetching

**Summary:**
Implemented global rate limit management and staggered iterative background fetching for massive TV shows.

**Core Architectural Features Implemented:**
- **Global Rate Limit Queue Pause:** Added `is_rate_limited` and `rate_limit_reset` to the Rust `AppState` to globally pause all background fetch loops based on TMDB's `Retry-After` HTTP 429 header.
- **Strict Array-Based Iteration:** Modifed the background fetch loop in `add_to_tracker` to strictly iterate over the `seasons` metadata array. This handles missing or non-sequential seasons (e.g. documentaries) by logging 404s gracefully without halting the overall sync.
- **Specials UI Semantic Mapping:** Ensured Season 0 is safely fetched and stored. Augmented `MediaDetails.tsx` to dynamically label `season_num === 0` as 'Specials', leveraging the backend's `ORDER BY season_num ASC`.
- **CancellationToken Sync Mapping:** Mapped `CancellationToken`s to `media_id` strings within `AppState`. The "Remove Show" action safely triggers cancellation, aborting the active fetch loop instantly before executing the cascading delete in the background queue.
- **Discrete UI Progress Binding:** Emitted granular `sync-progress` events containing both `mediaId` and `tmdbId` to cleanly track precise sync states in `useTaskStore`. Bound these explicitly to local UI elements like `MediaDetails` banners and `InboxView` search results, separating them from the global `App.tsx` multi-sync indicator.

**Edge Cases Tested & Verified:**
- "One Piece" Staggering Simulator: Verified a 250ms deterministic delay effectively staggers massive fetch queues.
- Add-Remove Sprint Cancellation: Validated that a rapid background delete cleanly intercepts and halts the active loop via token polling.
- Missing Season Skip Logic: Tested that a 404 response logs a warning, skips the iteration, and continues without raising fatal errors.

### Update Feature 3.8: High-resolution cinematic Backdrop image extraction
- **Dynamic Resolution Resolver:** Implemented `ImageConfig` in the Rust backend that interfaces with Tauri's `app.primary_monitor()` to dynamically select the ideal backdrop resolution (`original` for High-DPI displays, `w1280` for standard 1080p).
- **Textless Backdrop Fallback (Primary):** Refactored `get_media_details` to parse the `append_to_response=images` TMDB payload, specifically filtering the `backdrops` array for clean, textless variants (`iso_639_1` == null) and selecting the one with the highest `vote_average`.
- **Pseudo-Backdrop Generation (Secondary):** Integrated the Rust `image` crate into the download pipeline. If a show lacks backdrops entirely, the backend intercepts the primary poster path (appended with `?crop=true`), downloads it, and automatically performs a 16:9 center-crop and horizontal expansion to create a seamless "Pseudo-Backdrop" cached locally.
- **Gradient Fallback (Tertiary):** Updated SQLite schema (`PRAGMA user_version = 16`) to include a `backdrop_fallback` column. If both backdrops and posters are missing from TMDB, the DTO passes `fallback_type: "gradient"` to the React UI.
- **Eviction-Based Image Buffer:** Modified `SafeImage.tsx` to handle `asset://` object URLs for backdrops. Added a `useEffect` cleanup hook that explicitly calls `URL.revokeObjectURL` and sets the image source to null upon component unmount, strictly adhering to the 30MB RAM target during deep library navigation.
- **Cinematic Skeleton Loading:** Upgraded the `SafeImage` backdrop renderer with a z-index stack: a pulsing `#1F222A` to `#2A2D35` gradient bottom layer, overlaid with a `framer-motion` image that smoothly fades in (600ms ease-in-out) via the `onLoad` event to eliminate jarring layout snaps.

### Feature 16.13: Boot-Time Filesystem Guard
* Implemented `filesystem_guard.rs` with `boot_time_guard` for checking and verifying all necessary application directories.
* Moved `logging::init_tracing()` to the start of `main.rs` to allow filesystem errors to be logged early.
* Relocated `db::init_db()` into the Tauri `.setup()` hook explicitly so it only initializes after the filesystem is confirmed to be healthy.
* Removed deprecated `canary_check()` and `ensure_directories()` logic to avoid redundancies and potential race conditions.
* Configured `boot_time_guard` to perform idempotent directory creation and active probe writing to confirm disk write permissions.
* Connected `tauri_plugin_dialog` so a fatal GUI message is displayed bypassing the React frontend if filesystem integrity tests fail.

### Part 6 UI Styling and Layout
* **6.1 Pure #0D0F14 deep cinematic background color**: Updated `index.html` to inject #0D0F14 background class `bg-[#0D0F14]` strictly. Verified `tauri.conf.json` background_color parameter matches. Added `.hero-wrapper` for ultra-wide monitor support.
* **6.2 Translucent #1F222A surface cards**: Built global `.surface-card` CSS utility class in `index.css` applying `#1F222A` with `#2A2D35` solid borders and dynamic hover/nested states transitioning to `#252830`. Added pulsing `.skeleton-loader` animation alternating between these hex values.
* **6.3 Vibrant #FF6B00 (VLC Orange) global accent**: Added `.btn-primary` and `.active-glow` for strict hover states mapping to `#FF8533` and ensuring appropriate drop-shadows. Verified text inside these elements defaults to `#FFFFFF`.
* **6.7 Modern Inter sans-serif font family integration**: Added `.tabular-nums` class and confirmed standard anti-aliasing behavior in `index.css`. Tailwind is already configured correctly for the `Inter` font stack.
* **6.10 Custom thin, dark styled scrollbars**: Upgraded `::-webkit-scrollbar` with a 6px layout. Implemented `rgba(42, 45, 53, 0.4)` pseudo-opacity on the thumb track so it blends over the `#0D0F14` background cleanly, transitioning to the primary orange `#FF6B00` on hover.
* **6.11 Lucide-React high-fidelity vector icon integration**: Created centralized `Icon.tsx` component wrapper for all Lucide-React icons that strictly sets `strokeWidth=1.5` and `aria-hidden=true` to maintain globally consistent crisp stroke weights. Added `mix-blend-mode: multiply` edge case fallback to `img` CSS rules.
## [7.1, 7.5, 7.6] Update Sidebar, Top Navigation, and Logo UI to match architectural specs
- Fixed sidebar with w-64, backdrop-blur-xl and mobile drawer with hamburger trigger
- Unified Sidebar Logo with stylized Play icon (#FF6B00) and proper negative spacing
- New Sticky Top Global Navigation Bar with scrolling opacity and properly spaced elements
