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
