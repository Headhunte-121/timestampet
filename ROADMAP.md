# WatchMark Project Roadmap

This document outlines the completed architectural milestones and the detailed forward-looking product roadmap for the **WatchMark Media Tracker**.

---

## 🎯 Project Vision
WatchMark is engineered as the definitive **client-side, privacy-focused desktop media tracking diary and local file bridge**. It combines the aesthetics and richness of modern streaming frontends (Apple TV, Plex) with the speed, memory safety, and independence of pure desktop software—zero server daemons, zero continuous CPU transcoding, and zero telemetry.

---

## 🏆 Completed Milestones

### ✅ Milestone 1: Core Engine & SQLite Architecture (`v1.0.0`)
* [x] **Tauri 2.0 Architectural Overhaul**: Pure Rust backend and React 19 + TypeScript + Tailwind CSS frontend.
* [x] **SQLite Relational Foundation**: Multi-table schema running in Write-Ahead Logging (`PRAGMA journal_mode = WAL;`) with atomic evolutionary migrations.
* [x] **Zero-CPU VLC Telemetry**: Tokio async event loops polling VLC's local HTTP API with 0% idle CPU and dynamic port probing (8080–8090).
* [x] **Smart Playhead Resumption**: Remembers exact second positions (`last_position`) and auto-completes at ≥90% watch ratio.
* [x] **TMDB Metadata Integration**: Async client with exponential backoff retries and local image caching in OS AppData.

### ✅ Milestone 2: Filesystem Integration & Data Integrity (`v1.1.0`)
* [x] **High-Speed Media Scanner**: Recursive directory traversal using Rust's `walkdir` and multi-pattern regex (`S01E01`, `1x01`, `Ep 01`).
* [x] **Windows Long Path & Shortcut Support**: Windows `\\?\` prefix normalization and `.lnk` shortcut parsing via `parselnk`.
* [x] **Persistent Triage Inbox**: Grouping unmatched media files by parsed series name for 1-click matching or manual linking.
* [x] **Custom Streaming Protocol (`watchmark://`)**: Native asynchronous URI protocol supporting Range header parsing for video seeking.
* [x] **Database Safety**: Atomic in-app database backups with SHA-256 integrity verification, auto-rotation, and SQLite `VACUUM` optimization.
* [x] **OS Credential Manager**: Secure TMDB API key storage via OS Keyring (Windows Credential Manager / Keychain / Secret Service).

### ✅ Milestone 3: Cinema Spotlight, Virtualized Timeline & GPU Fix (`v1.2.0`)
* [x] **Intelligent Hero Spotlight Recommendation Engine**: 8-dimensional weighted candidate scoring algorithm prioritizing paused in-progress shows, newly aired episodes, and season finales with dynamic library fallback.
* [x] **BingeBlock Accordion Timeline**: Master-detail accordion grouping consecutive episodes with Framer Motion auto-height physics.
* [x] **Midnight Crossover Detection**: Dynamic epoch timestamp calculations displaying a `<Moon />` icon when sessions span past midnight.
* [x] **Distinct Show Session Isolation**: Auto-chains consecutive episodes watched within 6 hours (< 21,600s) while immediately isolating different shows.
* [x] **DOM Virtualization (`VirtualPoster`)**: Custom `IntersectionObserver` windowing engine unmounting off-screen elements, supporting 1,000+ shows with low RAM.
* [x] **WebView2 GPU Optimization**: Eliminated continuous scale transforms beneath CSS `backdrop-filter: blur(...)` surfaces in `SafeImage`, reducing idle GPU from ~70% to 0.0% – 1.0%.
* [x] **Edge-to-Edge Hero Banner**: 450px banner with multi-stop diagonal gradient fades and instant resume actions.
* [x] **Global Quick Search**: Floating dark-glass search input in top navigation bar filtering rows and grids in real-time.

---

## 🚀 Forward Product Roadmap

```
                       WATCHMARK UPCOMING MILESTONES
 ═════════════════════════════════════════════════════════════════════════
   v1.3.0 (Milestone 4) ──▶  Library Intelligence, Deep Metadata & Bulk Actions
   v1.4.0 (Milestone 5) ──▶  VLC Telemetry Pro, Playback Controls & Tray Daemon
   v1.5.0 (Milestone 6) ──▶  Analytics, History Deep-Dive & Data Portability
   v2.0.0 (Milestone 7) ──▶  Ambient Cinema UI, Command Palette & Hardening
 ═════════════════════════════════════════════════════════════════════════
```

---

### 📌 Milestone 4: Library Intelligence, Deep Metadata & Bulk Operations (`v1.3.0`)
> **Theme:** Transform the library from a static poster grid into an interactive, multi-dimensional media management suite with TMDB cast/crew deep-linking, quick-view cards, and bulk actions.  
> **Target Release:** Q4 2026 | **Primary Branch:** `develop` | **Feature Branch:** `feat/v1.3.0-library-intelligence`

#### 1. Core Feature Specifications
* **Multi-Dimensional Library Filtering & Sorting** (Micro-Features 32, 35)
  * Multi-criteria sorting: *Recently Added*, *Last Watched*, *Community Rating*, *Release Year*, *Title (A-Z)*, *Total Runtime*.
  * Multi-select genre chips: combine multiple genres (`Action` + `Sci-Fi`) with instantaneous client-side filtering.
  * Status filter pills: *All*, *In Progress / Watching*, *Completed*, *Unwatched*, *Dropped / Archived*, *Missing Local Video Files*.
  * Release year dual-thumb range slider (e.g. `1970` – `2026`).
* **"Quick-View" Hover Cards & Overlays** (Micro-Feature 33)
  * Floating preview card triggered after a 300ms hover delay over any poster card.
  * Displays 16:9 backdrop banner, runtime, rating pill, next unplayed episode title, and 1-click **"Play Now"** button.
  * Automatic boundary collision detection: automatically flips anchor positioning (left vs. right / top vs. bottom) to stay within viewport bounds.
* **Rich Cast, Crew & Content Rating Inspector** (Micro-Features 37, 39)
  * Asynchronous TMDB `/credits` integration with local SQLite caching.
  * Horizontal actor carousel in `MediaDetails.tsx` displaying actor headshots, character names, and direct links to other shows in the local library featuring the same actor.
  * Official content rating badges (`TV-MA`, `PG-13`, `R`, `TV-14`) with dark frosted pill backing.
  * Episode guest star credits, director/creator bios, and production studio tags.
* **Personal Media Interactions & Tagging** (Micro-Feature 38)
  * Favorite toggle (heart icon) pinning favorite titles to the top or dedicated view.
  * Custom user tags system (e.g. `"comfort-rewatch"`, `"weekend-binge"`, `"classic"`).
  * Personal user notes & review input modal per media item.
* **Bulk Data Mutators** (Micro-Feature 29)
  * **"Mark Season as Watched"** and **"Mark Season as Unwatched"** buttons with confirmation toasts.
  * Multi-select episode mode: shift-click or checkbox selection to bulk-link files or bulk-delete history logs.
  * Atomic SQLite transactional execution (`BEGIN TRANSACTION ... COMMIT`).
* **Library Micro-Interactions & View Modes** (Micro-Feature 36)
  * View density switcher: Standard Poster Grid, Compact Grid, and Detailed List View.
  * Subtle hover lift and border glow on active media cards with 0% compositor overhead.

#### 2. Architectural & Database Requirements
* **Schema Extension:** Add `user_tags` (TEXT/JSON), `user_notes` (TEXT), and `is_favorite` (BOOLEAN DEFAULT 0) to `Media` table.
* **Credits Cache Table:** Create `Media_Credits` (`id`, `media_id`, `cast_json`, `crew_json`, `last_updated_at`) with 30-day TTL invalidation.
* **State Management:** Introduce client-side multi-select context (`selectedMediaIds: Set<number>`) for bulk mutation dispatch.

---

### 📌 Milestone 5: VLC Telemetry Pro, Playback Controls & Background Lifecycle (`v1.4.0`)
> **Theme:** Deep OS-level integration, automated background media discovery, and advanced playback controls directly synced with VLC.  
> **Target Release:** Q1 2027 | **Primary Branch:** `develop` | **Feature Branch:** `feat/v1.4.0-vlc-and-os-lifecycle`

#### 1. Core Feature Specifications
* **Advanced VLC Audio & Subtitle Track Switcher** (Micro-Feature 40)
  * Query VLC HTTP status API to extract available audio streams (languages, channels) and embedded subtitle tracks.
  * Pre-playback stream selector drawer allowing users to pick audio language and subtitle track before spawning VLC.
  * Pass selected stream arguments directly to VLC spawn flags (`--audio-track`, `--sub-track`).
* **Playback Quality of Life Shortcuts** (Micro-Feature 42)
  * In-app **Skip Intro (85s)** and **Skip Outro / Next Episode (120s)** action triggers via VLC HTTP API.
  * Playback speed multiplier toggles: `1.0x`, `1.25x`, `1.5x`, `2.0x`.
* **VLC Window Focus & Native OS Management** (Micro-Feature 41)
  * Window focus coordination: automatically minimize or hide WatchMark window when VLC starts playback.
  * Auto-restore WatchMark window focus when VLC process exits.
  * Single-instance VLC window reuse when consecutive episodes are launched.
* **In-App Playback Toasts & Volume Sync** (Micro-Feature 43)
  * Real-time in-app playback status toast (e.g. `"Playing: Severance S02E01 • 14:32 / 52:10"`).
  * Volume level synchronization and mute toggling directly from the WatchMark interface.
* **Multi-Directory Management & Non-Blocking Scanner** (Micro-Features 54, 55)
  * Multi-directory watch folder configuration in Settings: add, label, and manage multiple drives.
  * Optional startup auto-scan running quietly in a background Rust thread with zero UI lag.
  * Native OS directory change monitoring via `notify` crate to detect newly downloaded episodes.
  * `.watchmarkignore` file support inside folders to ignore bonus clips or sample files.
* **Temporary Download Shield & Inbox Safety** (Micro-Feature 56)
  * Automatic filtering of active download extensions (`.crdownload`, `.part`, `.tmp`, `.aria2`).
  * Ingest and match files automatically the moment download file locks are released.
* **Native System Tray Daemon & OS Lifecycle** (Micro-Feature 58)
  * Minimize-to-tray and close-to-tray options in Settings (`minimize_to_tray: true`).
  * System tray context menu: *Show WatchMark*, *Scan Libraries Now*, *Active Playback Status*, *Quit*.
  * Optional "Launch on Windows Startup" registry toggle.

#### 2. Architectural & Backend Requirements
* **VLC HTTP API Expansion:** Expand HTTP client to poll `/requests/status.json` for audio stream track indices (`audio_track`), subtitle track IDs (`spu_track`), and aspect ratio controls.
* **File System Watcher:** Integrate `notify` crate in `src-tauri` to monitor configured library folders for real-time filesystem events (`Create`, `Modify`, `Rename`).
* **Tauri System Tray:** Configure native tray icon with minimize-to-tray and auto-start on Windows boot via registry/startup shortcut.

---

### 📌 Milestone 6: Analytics, History Deep-Dive & Data Portability (`v1.5.0`)
> **Theme:** Comprehensive personal viewing analytics, historical backdating, Trakt/Letterboxd data portability, and queue hygiene.  
> **Target Release:** Q2 2027 | **Primary Branch:** `develop` | **Feature Branch:** `feat/v1.5.0-analytics-and-history`

#### 1. Core Feature Specifications
* **Manual History Backdating & Session Logger** (Micro-Feature 28)
  * Interactive modal to manually log past watch events: date picker, time input, and season/episode selector.
  * Ability to edit historical watch timestamps or delete accidental logs.
  * Manually logged entries integrate cleanly into BingeBlock session auto-chaining logic.
* **History Timeline Search & Infinite Scroll** (Micro-Feature 44)
  * Full-text search across entire watch history by show name, episode title, or date.
  * Filter history by specific series, movies, or date range.
  * Virtualized timeline infinite scrolling handling 10,000+ historical entries smoothly.
* **Personal Viewing Analytics Dashboard** (Micro-Features 34, 46)
  * Dedicated Analytics View: Total time watched (Months, Days, Hours), TV vs. Movie ratio pie chart, favorite genres breakdown, most-watched shows.
  * Dynamic time-of-day greetings: Morning, Afternoon, Evening, and Late-Night Binge greeting ("*Still up watching?*").
  * **"On This Day"** historical flashback card on Dashboard ("*1 year ago today, you watched the Season 2 finale of...*").
* **Multi-Platform Data Portability & Sync** (Micro-Features 45, 57)
  * Full portable JSON data export containing all media, custom tags, notes, and playhead positions.
  * CSV export formatted for Trakt.tv and Letterboxd imports.
  * CSV import parser for standard Trakt.tv and Letterboxd export files with interactive match confirmation triage.
  * Automated daily database backup with 3-backup rolling retention.
* **Continue Watching Queue Pruning & Drop Shelf** (Micro-Features 47, 48)
  * Hover dismiss button (`×`) on Continue Watching cards to hide titles without deleting history.
  * Dedicated "Dropped / Archived" category accessible from Library filters.
  * Upcoming Release Radar: shows next air dates for in-progress series using TMDB scheduled dates.

#### 2. Architectural & Data Requirements
* **Backdating Engine:** Support manual insertion into `History` with custom `watched_at` timestamps and manual duration overrides.
* **Aggregations Engine:** Pre-compiled SQL views (`v_monthly_stats`, `v_top_genres`, `v_binge_velocity`) for real-time charting without expensive table scans.
* **Export / Import Serialization:** Type-safe JSON and CSV schema specifications for full database export and third-party import validation.

---

### 📌 Milestone 7: Ambient Cinema UI, Command Palette & Hardening (`v2.0.0`)
> **Theme:** Desktop aesthetics, dynamic chromatic backdrops, universal keyboard-first navigation, and enterprise defensive stability.  
> **Target Release:** Q3 2027 | **Primary Branch:** `develop` | **Feature Branch:** `feat/v2.0.0-cinema-ui`

#### 1. Core Feature Specifications
* **Dynamic Chromatic Ambient Lighting** (Micro-Feature 51)
  * Extract vibrant accent color from active show poster/backdrop.
  * Dynamically blend with `#0D0F14` background at 12% opacity to give each show a unique atmospheric tone.
  * Parallax scrolling effect on top backdrop banners.
* **Universal Command Palette (`Ctrl+K`)** (Micro-Feature 49)
  * Floating dark-glass command palette accessible via `Ctrl+K` / `Cmd+K`.
  * Instant fuzzy search through library, settings toggles, and direct VLC launch actions.
  * Full keyboard navigability (Arrow up/down, Enter to select, Escape to dismiss).
* **Native Desktop Hardware Navigation** (Micro-Feature 50)
  * Support for mouse navigation buttons (Mouse Button 4 = Previous Screen, Mouse Button 5 = Forward).
  * Scroll position restoration: preserves exact scroll offsets when navigating back to grids or lists.
* **Cinematic Transitions & Micro-Interactions** (Micro-Features 52, 53)
  * Staggered entry wave animations using Framer Motion springs for library grid loading.
  * Glowing orange accent playhead indicator on active progress bars.
  * Interactive hover star rating components with live preview.
* **Persistent Image Caching Engine** (Micro-Feature 30)
  * Local disk storage of TMDB backdrops and posters in AppData with LRU cache eviction and offline fallback placeholders.
* **Desktop Hardening & UX Polish** (Micro-Features 59, 60, 61)
  * **Reset Layout & UI Cache** in Settings without altering watch history or library mappings.
  * Graceful empty states when TMDB announces a show with 0 episodes aired.
  * Desktop-grade `user-select: none` hardening on chrome while preserving text selection on synopses.
  * Frameless custom window dragging handle optimizations.

#### 2. Architectural & Systems Requirements
* **Dynamic Palette Engine:** Rust-side or canvas-based dominant color extraction from poster artwork (`fast_image_resize` + color quantizer), caching dominant hex colors in SQLite.
* **Universal Command Palette:** Global modal listener (`Ctrl+K` / `Cmd+K`) with fuzzy search across shows, settings, quick-actions, and navigation routes.
* **Hardware Mouse Integration:** Native listener for Mouse Button 4 (Back) and Button 5 (Forward).

---

## 🛡️ Enterprise Compliance Notice
Per architectural review, all unauthorized third-party download scrapers and MTProto sourcing engines (Micro-Features 62–64) have been permanently excluded from WatchMark. The application operates strictly as a legitimate, local file player and personal viewing diary.

---

<div align="center">
  <sub>WatchMark Media Tracker • Built with Tauri, Rust, and React</sub>
</div>
