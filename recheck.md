Update 1: The Tauri 2.0 Overhaul
✅ Intact & Active:
- 100% Pure Rust Backend replacing Python, spanning multiple files (`src-tauri/src/db.rs`, `vlc.rs`, `scanner.rs`, `tmdb.rs`, `commands.rs`).
- `db.rs`: SQLite connection logic using `rusqlite` and evolutionary `ALTER TABLE` migrations.
- `scanner.rs`: Media file scanner using Rust's `walkdir` and `regex` crates.
- `tmdb.rs`: Asynchronous TMDB API fetching and local image caching using Rust.
- `commands.rs`: Tauri endpoints connecting frontend requests to database queries (e.g., `get_dashboard_data`, `fetch_history`).
- Modern UI Stack (`src/`): Rewritten frontend using React, Tailwind CSS, and TypeScript.
- Cinematic aesthetics: Edge-to-edge backdrop hero banners, frosted glass sidebars (`backdrop-blur-xl`), scalable Lucide React icons.

🔄 Superseded & Evolved:
- `vlc.rs` Integration: Originally launched via `std::process::Command` and polled with a blocking background thread. This was explicitly rewritten in Update 5 to use `tokio::process::Command` and `tokio::select!` for a 0% background CPU async refactor.

❌ Missing or Broken:
None identified.

Update 2: The Cinema-Grade Overhaul
✅ Intact & Active:
- The App Shell (`src/App.tsx`): Frosted navigation panel with active-state styling (pure white text with a glowing vertical line), floating Top Bar, and Framer Motion `<AnimatePresence>` for smooth cross-fading page transitions.
- The Dashboard (`src/components/Dashboard.tsx`):
  - "Continue Watching" Row: Horizontal, snap-scrolling row of wide episode thumbnails with bottom-edge progress bars and Framer Motion scaling on hover.
  - "Recently Added" Grid: Cinematic 2:3 poster grid with hover scaling, frosted glass overlays, and centered Play buttons.
  - Personal Stats Widget: Three sleek cards showcasing Hours Watched, Shows Completed, and Average Rating.
- Dynamic Data & Async UX: Wired `get_dashboard_data` endpoint with pulsing dark-gray Skeleton Loaders to prevent flashing while SQLite fetches data.

🔄 Superseded & Evolved:
- The Hero Banner (`src/components/Dashboard.tsx`): Originally built as a 450px banner with a deep black gradient fade (`bg-gradient-to-t from-[#0D0F14]`). This was evolved in Update 3 (directional diagonal fade) and completely superseded in Update 89 which forced it to be perfectly edge-to-edge (`w-[calc(100%+5rem)] -mx-10`) bleeding under the navigation bar with a highly specific multi-stop gradient.
- Unsplash Placeholders (`src/components/Dashboard.tsx`): Initially, missing TMDB backdrops/posters fell back to high-res Unsplash URLs. This was superseded by Update 52 (Tertiary gradient fallback) and Update 77 (Ghost Cards rendering text on `#1F222A` backgrounds) to ensure offline support.

❌ Missing or Broken:
None identified.

Update 3: Premium Vibe Polish
✅ Intact & Active:
- Fixed the Styling Engine: `index.css` imported into `main.tsx` so Tailwind compiles the layouts properly.
- Glassmorphism: Replaced solid dark gray panels with frosted glass (`bg-[#1F222A]/60` with `backdrop-blur-md`) to create visual depth across Dashboard, History, Inbox, and Media Details.
- The Sidebar Glow (`src/App.tsx`): Active tabs feature a glowing vibrant orange (`#FF6B00`) vertical line on the far left.
- Persistent Global Search (`src/App.tsx`): Floating, pill-shaped "Quick Search" bar that instantly filters rows and grids in real-time.
- Hero Typography Rules (`src/components/Dashboard.tsx`): Enforced tiny bold orange labels ("UP NEXT"), massive pure white titles, and muted silver subtitles.
- Stats Bar Redesign (`src/components/Dashboard.tsx`): Overhauled into center-aligned, frosted glass widgets showcasing bold orange numbers.

🔄 Superseded & Evolved:
- Poster Rating Badges: Changed from harsh black squares with yellow stars into blurred pill shapes featuring uniform VLC Orange stars. This was further superseded and explicitly refined in Update 78 to enforce `absolute top-2 right-2 bg-[#0D0F14]/60 backdrop-blur-md`, tabular-nums, and logic that vanishes entirely if the rating is null or 0.

❌ Missing or Broken:
None identified.

Update 4: Webview Protocols & Missing Dependencies
✅ Intact & Active:
- WebView configuration (`src-tauri/tauri.conf.json`): Added `"webviewInstallMode": { "type": "downloadBootstrapper" }` to automatically download the Microsoft WebView2 bootstrapper if a Windows user is missing it.
- Custom Protocol (`src-tauri/src/main.rs`): Registered an asynchronous URI scheme protocol `watchmark://` that decodes percent-encoded strings, validates absolute file paths (preventing traversal attacks), checks MIME types, supports Range header parsing for video seeking, and securely streams local assets over HTTP to bypass WebView security restrictions.

🔄 Superseded & Evolved:
None identified. While Tauri v2's native `assetProtocol` was enabled later in Update 57, the `watchmark://` handler remains intact, functional, and explicitly defined inside `main.rs` as a custom layer of the architecture.

❌ Missing or Broken:
None identified.

Update 5: Zero Background CPU & Error Architecture
✅ Intact & Active:
- Zero Background CPU Logic (`src-tauri/src/vlc.rs`): `tokio::process::Command` async child process handle, `tokio::select!` for concurrent waiting on process exit or 5-second `tokio::time::interval`, idle/zombie detection (`consecutive_failures`), and async HTTP polling (`reqwest::Client`).
- Graceful Panic Handling (`src-tauri/src/error.rs`): `thiserror` based custom `AppError` enum wrapping standard errors into user-friendly serialized strings.
- Global `catch_unwind` Boundary: Wrote `handle_panic` that intercepts raw strings, appends to `watchmark.log`, and catches panics before tearing down Tauri.
- Audited `.unwrap()` Calls: Safely propagated `?` error mappings across `commands.rs`.

🔄 Superseded & Evolved:
None identified.

❌ Missing or Broken:
None identified.

Update 6: Layout Integrity & Hardware Acceleration
✅ Intact & Active:
- Tailwind Configuration (`tailwind.config.js`): Safelists array, Rust pathing in content array, and Semantic color palette injected (`brand-orange`, `cinema-black`, `surface-gray`).
- Global Shell Architecture (`src/App.tsx`): Flex container (`flex-1 overflow-hidden min-w-0`) wrapped around the main content to preserve sidebar scroll behavior.
- Semantic Colors Applied: Swapped hardcoded shell colors for semantic equivalents like `bg-cinema-black` and `selection:bg-brand-orange/30` in `src/App.tsx`.
- Responsive Layout Integrity (`src/components/Library.tsx`): Clamp logic `grid-cols-[repeat(auto-fill,minmax(180px,1fr))]` utilized to preserve exact 2:3 aspect ratio posters on larger screens.
- Global Typography: The `antialiased` font smoothing class was confirmed intact via `src/index.css`.

🔄 Superseded & Evolved:
- Extreme Aspect Ratios (`src/components/Dashboard.tsx`): The Hero Banner previously relied on `aspect-video max-h-[450px]`. This was completely superseded in Update 89 by `h-[450px] min-h-[400px] lg:h-[50vh] -mt-24 -mx-10` to force edge-to-edge bleed under the new sticky navigation bar.

❌ Missing or Broken:
- Hardware Acceleration & Blur Fallbacks (`src/App.tsx`): The `transform-gpu`, `will-change-transform`, and `motion-reduce:bg-surface-gray motion-reduce:backdrop-blur-none` classes originally applied to the glassy sidebar are missing and have been stripped/lost from the component's `className`.

Update 7: Data Stability & Inbox Triage Flow
✅ Intact & Active:
- Secure Database Location (`src-tauri/src/db.rs`): DB connection properly routes `watchmark.db` and the cache into the OS's native AppData directory (via `ProjectDirs::from("com", "WatchMark", "WatchMark")`), preventing Vite hot-reloading loop crashes.
- Inbox Triage Modal (`src/components/InboxView.tsx`): The Inbox functions as an active triage center. Clicking a group lists files in a secondary pane, and there is a `+ Search TMDB & Add Tracker` button that launches the Search Modal.
- `assign_unmatched_to_tracker` Command (`src-tauri/src/commands.rs`): Securely takes unmatched files, links them to episodes inside `Local_Files`, and auto-deletes them from the `Unmatched_Files` database queue.
- Async `tokio::task::spawn_blocking` (`src-tauri/src/commands.rs`): Both `add_to_tracker` and `assign_unmatched_to_tracker` wrap their heavy DB operations in `spawn_blocking` to prevent UI freezing and race conditions.
- Missing TMDB API Key Silence (`src-tauri/src/commands.rs`): Hard checks exist in `add_to_tracker` and `assign_unmatched_to_tracker` to return a `Missing TMDB API Key.` error if the user's key is blank.

🔄 Superseded & Evolved:
- UI Responsiveness Breakpoints: Originally applied rigid width breakpoints (`w-[140px] md:w-[160px] lg:w-[180px]`) to the "Recently Added" grid in `Library.tsx` and `Dashboard.tsx`. `Library.tsx` was explicitly superseded by the fluid CSS grid logic `grid-cols-[repeat(auto-fill,minmax(180px,1fr))]` in Update 16.1.
- `TypeError` browser crash handlers: Originally wrapped raw `invoke()` calls in `InboxView.tsx` to prevent browser crashes. This was superseded by the global `useAsyncInvoke.ts` hook implementation in Update 43 which centralizes IPC error handling.

❌ Missing or Broken:
None identified.

Update 8: Framer Motion 1.5 & Zustand
✅ Intact & Active:
- Zustand & Global Cinema Mode: Managed in `src/store/useAppStore.ts` with an `isCinemaMode` boolean state. Applied globally in `App.tsx` via `<MotionConfig transition={isCinemaMode ? ... : { duration: 0 }}>`.
- Navigation Fluidity (`src/App.tsx`): View switching is wrapped inside `<AnimatePresence mode="wait">` to create smooth crossfades between pages instead of instant swaps.
- Advanced Grid Rendering (`src/components/Library.tsx` & `Dashboard.tsx`): Poster grids and carousels use `<AnimatePresence mode="popLayout">`. `Dashboard.tsx` utilizes `layout="position"` to allow elements to slide dynamically.
- Cascading Deletes & UI Lockout (`src/components/MediaDetails.tsx`): An `isAnimatingRef` locks the "Remove Show" button to prevent spam clicking. The `delete_media_cmd` in `commands.rs` performs a secure SQLite `DELETE FROM Media` leveraging `ON DELETE CASCADE` and properly wipes cached images.
- Subpixel Anti-Aliasing: `transform-gpu` utilities are appended to interactive motion elements (e.g. `Dashboard.tsx` posters) to leverage hardware acceleration and prevent text blurring.

🔄 Superseded & Evolved:
- Staggered entry animations: The `isInitialStagger` logic for the first 20 items using `whileInView` optimizations in `Library.tsx` is structurally present but effectively bypassed/superseded by the `VirtualPoster` IntersectionObserver windowing rendering logic from Update 77.

❌ Missing or Broken:
- 30-Second "Ken Burns" scale effect: The slow, 30-second panning/scaling animation applied to the Hero Banner background image in `Dashboard.tsx` is completely missing.

Update 9: Memory Optimizations & Modal Polish
✅ Intact & Active:
- SQLite Connection Pooling (`src-tauri/src/db.rs`): Utilizes `OnceLock` with `Mutex<Connection>` to maintain a single global connection.
- SQLite PRAGMA Tuning (`src-tauri/src/db.rs`): Explicitly executes `PRAGMA cache_size = -2000;`.
- Background Scanning IPC (`src-tauri/src/scanner.rs` & `commands.rs`): Scanner loop batches unmatched files into chunks of 50 and uses `app_handle.emit("scan-match-batch", ...)` to send them to the React frontend iteratively, preventing RAM spikes.
- Rust Compilation Size (`src-tauri/Cargo.toml`): Contains `[profile.release]` block configured for maximum size reduction (`opt-level = "z"`, `lto = true`, `codegen-units = 1`).
- VLC Loop Verification (`src-tauri/src/vlc.rs`): The `vlc_heartbeat` loop correctly breaks when the VLC process dies or fails 3 consecutive HTTP probes, ensuring no zombie threads leak memory.
- Window Title Fix (`index.html`): The `<title>` tag is explicitly set to "WatchMark" to ensure it looks like a native OS executable.

🔄 Superseded & Evolved:
- React Image Caches (`src/components/Library.tsx`): The custom `LazyImage` component replacing `src` with a 1x1 transparent pixel data-URI on unmount was completely superseded by the `VirtualPoster` component in Update 77, which leverages a custom `IntersectionObserver` to entirely unmount off-screen DOM nodes and replace them with empty layout-preserving `div` blocks.
- Fixed White Horizontal Scrollbars (`src/components/MediaDetails.tsx`): The Season Tabs were updated to cleanly wrap to a new line using `flex-wrap` without scrollbars. This layout was completely superseded by Update 81, which rewrote them into horizontal scrolling pill-style tabs via `flex-row overflow-x-auto whitespace-nowrap`.

❌ Missing or Broken:
- Replaced Native Browser Dialogs (`src/components/SettingsView.tsx` & `MediaDetails.tsx`): The specific custom `CustomDialog` and `CustomConfirmDialog` components mentioned are missing/not found as distinct components. However, generic native replacements and UI Store modals (`Modal.tsx`, `useUiStore.showConfirm`) currently handle this logic. (Flagging as missing strictly because the explicitly named `CustomDialog` component is absent).

Update 10: Native Desktop UI Modernization
✅ Intact & Active:
- Toast Notifications (`src/utils/toast.ts`): The `sonner` package is actively installed and utilized across the application to provide sleek dark notifications instead of blocking `window.alert()` popups.
- Native File Browser (`src/components/SettingsView.tsx`): Uses the `@tauri-apps/plugin-dialog` Rust/NPM plugin to open the actual native OS file explorer when clicking "Scan Directory" or browsing for the VLC executable.
- Window Title Fix (`src-tauri/tauri.conf.json` & `index.html`): Permanently sets the application title to "WatchMark".

🔄 Superseded & Evolved:
- Clean Flex-Wrap Seasons List (`src/components/MediaDetails.tsx`): Removed the horizontal scroll restriction and added `flex-wrap` and `gap-2` so season buttons automatically flowed onto a second line. As noted in Update 9, this was completely superseded by Update 81, which reverted them back to a single-line horizontal scrolling row (`overflow-x-auto`) using momentum side-scrolling.
- Custom Desktop Modals: The custom lightweight React+Tailwind `Modal` components replacing native `window.confirm()` actions were evolved. Currently, `showConfirm` relies on `useUiStore` and `Modal.tsx` directly.

❌ Missing or Broken:
None identified.
