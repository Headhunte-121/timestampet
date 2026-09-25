# Master Engineering Specifications

This document defines the formal functional engineering specifications across the **16 Core Subsystems** of the **WatchMark Media Tracker**.

---

## 📑 Functional System Index

1. [App Bootstrapping & OS Shell](#1-app-bootstrapping--os-shell)
2. [UI Foundation & Theme Engine](#2-ui-foundation--theme-engine)
3. [TMDB Metadata Engine](#3-tmdb-metadata-engine)
4. [Local Storage & Cache Architecture](#4-local-storage--cache-architecture)
5. [SQLite Relational Database Engine](#5-sqlite-relational-database-engine)
6. [Cinema UI Design System](#6-cinema-ui-design-system)
7. [Responsive Layouts & Viewports](#7-responsive-layouts--viewports)
8. [High-Speed Media Scanner](#8-high-speed-media-scanner)
9. [Persistent Triage Inbox](#9-persistent-triage-inbox)
10. [VLC Player Integration & Telemetry](#10-vlc-player-integration--telemetry)
11. [Media Details & Episode Management](#11-media-details--episode-management)
12. [Watch History & Binge Timeline](#12-watch-history--binge-timeline)
13. [Universal Search & Discovery](#13-universal-search--discovery)
14. [Settings & Credential Management](#14-settings--credential-management)
15. [Database Maintenance, Backup & Recovery](#15-database-maintenance-backup--recovery)
16. [Windows Native Integration & Packaging](#16-windows-native-integration--packaging)

---

### 1. App Bootstrapping & OS Shell
* **Tauri v2 Architecture**: Multi-process architecture dividing the untrusted webview renderer from privileged native Rust system calls.
* **Canary Permission Probing**: On startup, attempts writing a temporary `.canary` file to `%LOCALAPPDATA%\WatchMark`. If permission is denied, displays a native OS fatal error dialog rather than silently crashing.
* **WebView2 Auto-Bootstrapping**: Windows NSIS installer configured with `downloadBootstrapper` to download Microsoft Edge WebView2 dynamically if missing on legacy Windows machines.
* **Window Bounds Persistence**: Restores window width, height, and screen position with sanity clamping (e.g. minimum width 800px, minimum height 600px).

---

### 2. UI Foundation & Theme Engine
* **Semantic Dark Palette**: Universal dark background (`#0D0F14`) applied to document root to eliminate white screen flashes during initial load.
* **Frosted Glassmorphism**: Standardized surface layers (`bg-[#1F222A]/60` with `backdrop-blur-md` and `border border-white/5`) creating depth without GPU rendering spikes.
* **Signature Accents**: Uniform VLC Orange (`#FF6B00`) for play buttons, rating stars, active navigation glow bars, and progress indicators.
* **Accessibility Reduced Motion**: Integrated `useReducedMotion` hooks; drops layout transitions to 0s when battery-saver or OS reduced-motion accessibility flags are active.

---

### 3. TMDB Metadata Engine
* **Asynchronous Client**: Native `reqwest` client with rustls-tls, connection timeouts (10s connect, 30s read), and rate-limiting queues.
* **Exponential Backoff (`reqwest-retry`)**: Automatically retries transient network dropouts and HTTP 429 rate limits up to 3 times before returning structured errors.
* **Rich Metadata Ingestion**: Ingests title, synopsis, release dates, runtime, poster/backdrop art, season lists, episode overviews, and movie franchise collection parts.
* **Defensive Parsing**: Safely handles null overviews ("No overview available."), missing release dates ("0000-00-00"), and string runtimes.

---

### 4. Local Storage & Cache Architecture
* **Native AppData Routing**: Uses OS-standard directories (`directories::ProjectDirs` for `com.WatchMark.WatchMark`) rather than relative folder paths.
* **Three-Tier Image Resolution**:
  1. *Primary*: Local disk cache in `%LOCALAPPDATA%\WatchMark\cache\posters\` and `backdrops\`.
  2. *Secondary*: Remote TMDB CDN fallback.
  3. *Tertiary*: Offline dark gradient "Ghost Cards" with centered titles to ensure complete functionality without internet access.

---

### 5. SQLite Relational Database Engine
* **High-Concurrency WAL Mode**: `PRAGMA journal_mode = WAL;` enables simultaneous background scanner writes and frontend UI reads without lock contention.
* **Atomic Cascades**: `ON DELETE CASCADE` on all foreign keys ensures removing a media entity completely purges episodes and watch history without orphan records.
* **Evolutionary Migrations**: Multi-tier schema migrations using `PRAGMA user_version` wrapped in atomic transactions.

---

### 6. Cinema UI Design System
* **Edge-to-Edge Hero Banner**: 450px banner with diagonal gradient fade (`bg-gradient-to-tr`), tiny bold orange tags ("UP NEXT"), massive white titles, and 1-click **Resume** actions.
* **"Continue Watching" Carousel**: Wide 16:9 episode thumbnails with embedded progress bars indicating resume points in absolute minutes/seconds.
* **Interactive Poster Badges**: 2:3 poster cards with hover scaling, frosted glass dimming, and centered play buttons.
* **Personal Statistics Widgets**: Three frosted glass stat widgets displaying Hours Watched, Shows Completed, and Average Rating.

---

### 7. Responsive Layouts & Viewports
* **Fluid Poster Grids**: CSS grid auto-fill clamping `grid-cols-[repeat(auto-fill,minmax(180px,1fr))]` preserving exact 2:3 aspect ratios across 1080p, 1440p, and 4K ultra-wide monitors.
* **Horizontal Momentum Scrolling**: Custom `useHorizontalScroll` hook transforming vertical mousewheel input into horizontal scroll for carousels.

---

### 8. High-Speed Media Scanner
* **Recursive File Traversal**: Multi-threaded traversal using Rust's `walkdir` reading nested TV show folders in milliseconds.
* **Multi-Pattern Regex Matching**: Extracts show title, season, and episode from patterns:
  - Standard: `Show.Name.S01E05.mkv`, `Show_Name_1x05.mp4`
  - Absolute/Anime: `[ReleaseGroup] Show Name - 05 [1080p].mkv`
  - Daily: `Show.Name.2024.03.15.mkv`
* **Windows Long Path (`\\?\`) Support**: Canonicalizes deep path strings to bypass the 260-character Windows MAX_PATH limitation.
* **Shortcut Resolution**: Parses Windows `.lnk` shortcuts using `parselnk` to discover media stored across external network shares.
* **Batch IPC Streaming**: Emits files in batches of 50 (`scan-match-batch`) to eliminate frontend UI freezing during 10,000+ file scans.

---

### 9. Persistent Triage Inbox
* **Intelligent Series Grouping**: Unmatched files are automatically grouped by parsed series name into expandable triage rows.
* **One-Click TMDB Assignment**: Search TMDB directly from the unmatched row, linking all files to newly downloaded show metadata with a single confirmation.
* **Safe Inbox Clearing**: Clear Inbox removes records from the triage queue without modifying or deleting video files on disk.

---

### 10. VLC Player Integration & Telemetry
* **Zero-CPU Async Loop**: Manages VLC via `tokio::process::Child` and `tokio::select!`, polling every 5 seconds with zero background CPU load when idle.
* **Dynamic Socket Probing**: Probes ports 8080–8090 to find a free TCP socket, generating a random one-time 16-character password for each session.
* **Playhead Resumption**: Launches playback passing `--start-time=<last_position>` to resume playback seamlessly.
* **Intelligent Completion**: Flags an episode as completed and increments watch count when watched ≥90% or closed within 120s of the credits.
* **Session Isolation**: Binge sessions (< 21,600s / 6 hours) automatically chain consecutive episodes of the same show, but isolate immediately if the user switches shows.

---

### 11. Media Details & Episode Management
* **Deep-Dive Metadata**: Displays 4K backdrop art, season tabs, episode air dates, runtimes, cast, crew, and franchise collection parts.
* **Horizontal Season Selector**: Pill-style horizontal momentum scrolling tabs (`flex-row overflow-x-auto whitespace-nowrap`).
* **Manual File Relinking**: Allows manually binding local files to specific episodes if naming conventions are non-standard.
* **Star Rating System**: Interactive 10-star rating widget updating `user_rating` and instantly recalculating library averages.

---

### 12. Watch History & Binge Timeline
* **Interactive BingeBlocks**: Dedicated accordion components (`BingeBlock.tsx`) grouping binge sessions with Framer Motion auto-height physics.
* **Midnight Crossover Detection**: Dynamic epoch timestamp comparison rendering a distinct `<Moon />` icon when sessions span past midnight.
* **Sub-Episode Breakdown**: Shows individual watch durations, pause frequency counts, and 16:9 thumbnail fallbacks.
* **Auto-Scroll Anchoring**: Maintains viewport lock on the expanded header when opening long binge blocks.

---

### 13. Universal Search & Discovery
* **Debounced Live Query**: Instant search querying TMDB with 300ms input debouncing to prevent API quota exhaustion.
* **One-Click Library Onboarding**: "+ Add to Tracker" downloads full show metadata, seasons, episodes, and artwork in the background.

---

### 14. Settings & Credential Management
* **OS Keyring Integration**: Encrypts TMDB API keys using Windows Credential Manager, macOS Keychain, or Linux Secret Service via `keyring`.
* **Environment Variable Fallback**: Automatically checks `TMDB_API_KEY` system environment variables for automated setups.
* **Defensive Auto-Repair**: Resets corrupted `settings.json` files to defaults without crashing the application shell.

---

### 15. Database Maintenance, Backup & Recovery
* **In-App Optimization**: Executes SQLite `VACUUM` and `PRAGMA optimize` to reclaim disk space and rebuild B-tree indexes.
* **Atomic Backup Engine**: Uses the SQLite Online Backup API to produce point-in-time snapshots with companion SHA-256 checksums.
* **Automated Rotation**: Maintains a rolling window of recent backups, pruning older archives to conserve disk space.

---

### 16. Windows Native Integration & Packaging
* **System Tray Menu**: Native system tray integration with quick-action triggers (Scan Directory, Check for Updates).
* **Global Hotkeys**: Registers system-wide shortcuts (`Ctrl + Shift + S`) for instant library directory scanning.
* **NSIS Standalone Installer**: Produces standalone `.exe` and NSIS installers with bundled icons and automatic WebView2 bootstrapper.
