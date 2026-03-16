# Updates

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

y navigation.
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

### [7.1, 7.5, 7.6] Update Sidebar, Top Navigation, and Logo UI to match architectural specs
- Fixed sidebar with w-64, backdrop-blur-xl and mobile drawer with hamburger trigger
- Unified Sidebar Logo with stylized Play icon (#FF6B00) and proper negative spacing
- New Sticky Top Global Navigation Bar with scrolling opacity and properly spaced elements

### Update: Frontend Plain-English Logging Blueprint Implementation

**Summary:**
Implemented a centralized, plain-English logging blueprint for the React frontend to replace all messy `console.log` statements with clear, story-like outputs categorizing user actions, navigation, IPC communication, state changes, and errors.

**Core Features Implemented:**
- **Centralized Logger Utility (`src/utils/logger.ts`):** Created a lightweight module that exports specific logging functions (e.g., `app`, `navTo`, `click`, `ipcSend`, `error`) prefixed with distinct tags and emojis (e.g., `[ACTION] 🖱️`, `[IPC: SUCCESS] ✨`).
- **Native Browser Method Mapping:** Mapped `[ACTION]` and `[NAV]` logs to `console.info()`, `[ERROR]` to `console.error()`, and all other standard events to `console.log()`.
- **Smart Payload Handling:** Enforced strict plain-text strings for all standard logs to prevent console clutter. Only the `logger.error` function natively appends the raw error object for debugging stack traces.
- **Automatic IPC Logging:** Integrated the logger directly into the `useAsyncInvoke` hook. The hook now automatically triggers `logger.ipcSend` on call, `logger.ipcCancel` on unmount/abort, and `logger.ipcSuccess` upon resolution, ensuring 100% logging coverage for backend communication without repetitive component code.
- **Global Log Purge:** Systematically removed all pre-existing raw `console.*` statements across `src/components`, `src/store`, and `src/utils`, replacing them with appropriate `logger.*` functions to guarantee a continuous, clean script of frontend activity.
- **Frontend-to-Backend Log Bridge:** Added an IPC hook (`frontend_log` inside `commands.rs`) that explicitly pipes all plain-English logs from the React UI directly to the Rust backend's `tracing` engine, ensuring the actual terminal logs tell the full story.
- **Backend Logging Refactor:** Mass-replaced existing `tracing::info!` statements throughout the Rust backend to strictly adopt the matching plain-English, emoji-prefixed format (e.g., `[APP] 🚀`, `[DB] ✨`, `[IPC: CANCEL] 🛑`).

### Update: Feature 1.8 & 1.15 Window Frame & Geometry Management

**Summary:**
Implemented robust native OS window frame integrations and window geometry restoration.

**Core Architectural Features Implemented:**
- **Titlebar Drag Regions:** Added native `data-tauri-drag-region` to the `<header>` in `App.tsx` and used `z-10` on children to cleanly decouple drag zones from clickable buttons.
- **Dynamic DPI Scaling & Layout Recovery:** Mapped listeners to `tauri://resize` to dispatch synthetic window resize events, ensuring virtualized lists (like `Library.tsx`) and Intersection Observers properly reflow from minimized states.
- **Snap-Assist Compatibility:** Enabled `decorations: true` and set `minWidth: 800` & `minHeight: 600` via `tauri.conf.json`. Upgraded `Library.tsx`'s auto-fill constraints to strictly calculate exact widths without breaking column alignments.
- **Focus Signal Filters:** Linked `tauri://focus` and `tauri://blur` events in `App.tsx` to conditionally render `grayscale-[20%]` and `opacity-90` classes to visually cue when WatchMark enters background OS states.
- **Off-Screen Recovery & Automatic Preservation:** Augmented the Rust boot sequence in `main.rs` to validate the `initial_settings` bounded X and Y coordinates explicitly against `window.available_monitors()`. If off-screen, it performs an immediate fall-back calculation bounding the app back to the primary center.
- **Maximized/Fullscreen Disambiguation:** Bound Tauri's `WindowEvent::CloseRequested` to serialize coordinates exclusively if the window is natively windowed (`!is_maximized` && `!is_fullscreen`), explicitly preventing maximized dimensions from overriding the preferred state on next launch.

### Feature 16.9: TMDB API Key text input, validation, and persistent local storage
- **Visual Obscuration**: Added a secure Eye/EyeOff toggle to the TMDB API key input field in the Settings UI to mask the key by default.
- **Background Validation**: Integrated debounced automatic validation against TMDB's `/3/configuration` endpoint when the key changes. Displays dynamic UI feedback (emerald checkmark for success, red cross for error, yellow triangle for rate limits).
- **Whitespace Scrubbing**: Enforced robust sanitization in `validate_tmdb_key` Rust command, stripping whitespace and non-alphanumeric characters before validation and storage.
- **Secure Storage**: Ensured the Windows Credential Manager integration (via the `keyring` crate) prioritizes secure storage of the validated key, falling back to Base64 encoding.
- **Test Connection Loop**: Added a manual "Test Connection" button that bypasses caches, triggers a fresh network handshake, displays a "Pinging TMDB..." loading state, and fires explicit Success/Error Toasts.

### Feature 16.1 & 16.2: Dedicated TMDB Search full-page UI & Input Handling
- **Expansive Layout**: Built a dedicated Search view utilizing a `max-w-[1800px]` container with `mx-auto` centering and `px-10` padding to perfectly organize results on ultra-wide monitors.
- **Visual Parity**: Reused the standard 2:3 aspect ratio Poster Card components with Framer Motion `whileHover` scale animations and "At-a-Glance" TV/Movie badges for a seamless transition from the Dashboard.
- **Fluid Grid**: Implemented responsive Tailwind auto-grid columns (`grid-cols-[repeat(auto-fill,minmax(180px,1fr))]`) ensuring graceful reflows across narrow and extreme wide-screens, animated via Framer Motion's `layout` prop.
- **Form-Based Input**: Wrapped the quick search and main search pills in native HTML `<form>` elements to capture `onSubmit` events, allowing instant 'Enter' key request firing that bypasses debounce delays.
- **Input UX**: Added inline 'X' clear button functionality to rapidly reset the view state, and implemented an initial page-load auto-focus to let users start typing immediately.
- **Navigation**: Integrated a smooth "<- Back to Home" button pinned to the top-left of the Top Global Navigation bar when in the Search view.
- **State Preservation**: Ensured the Search component safely receives and syncs initial query parameters (e.g., from the global quick search) to maintain state during navigation.

### Feature 16.3: API HTTP Error / Invalid Key graceful error toast catchers
- **Specific Error Parsing**: Intercepted 401 Unauthorized responses to render a structured "Danger" Toast guiding the user to check their Settings.
- **Timeout Handling**: Caught network timeouts and generic connection failures to display distinct "Warning" Toasts indicating TMDB is taking too long to respond.
- **UI Shell Resilience**: Wrapped API invocations in `try/catch` blocks within the frontend, ensuring failing network calls gracefully stop loading spinners and never crash the React application shell.

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