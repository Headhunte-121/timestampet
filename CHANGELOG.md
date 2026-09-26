# Changelog

All notable changes to the WatchMark Media Tracker project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.2.0] - 2026-09-26

### Added
* **Intelligent Hero Spotlight Recommendation Engine**: 8-dimensional weighted candidate scoring algorithm prioritizing paused in-progress shows, newly aired episodes, and season finales with dynamic library fallback backfill and slide counters.
* **Continuous Integration Pipeline**: Automated GitHub Actions CI workflow (`.github/workflows/ci.yml`) performing multi-stage verification on both the Rust backend (`cargo check`) and React frontend (`npm run build`).
* **AI Orchestration & Systems Breakdown Whitepaper**: Comprehensive systems architecture whitepaper (`docs/architecture/AI_ORCHESTRATION.md`) documenting hierarchical decomposition, compiler-in-the-loop verification, and agentic workflows.
* **Cross-Platform Requirements & Installation Suite**: Standardized, script-free CLI installation and build guide supporting Windows, Linux, and macOS without relying on local `.bat` wrappers.
* **BingeBlock Accordion Timeline**: Dedicated interactive accordion components (`BingeBlock.tsx`) grouping consecutive watch events with fluid Framer Motion auto-height animations.
* **Midnight Crossover Detection**: Dynamic epoch timestamp calculations identifying binge sessions that span past midnight, automatically rendering a `<Moon />` indicator.
* **Distinct Show Session Isolation**: Auto-chains consecutive episodes watched within 6 hours (< 21,600s), but immediately terminates and mints a new session ID if the user switches to a different show.
* **DOM Virtualization (`VirtualPoster`)**: High-performance `IntersectionObserver` windowing engine that unmounts off-screen poster elements, enabling smooth 60 FPS scrolling across 1,000+ media titles with low memory footprint.
* **Edge-to-Edge Hero Banner**: 450px cinematic hero banner with multi-stop diagonal gradient fades and instant **Resume** actions.
* **Global Quick Search**: Floating dark-glass search input in the top navigation bar with instant client-side filtering across Dashboard rows and Library grids.
* **Accessibility & Power Optimizations**: Integrated `useReducedMotion` hooks so heavy layout transitions instantaneously drop to 0s when OS battery-saver or reduced-motion flags are active.

### Changed
* **Repository Architecture & Git Branching**: Established formal two-tier branching strategy: `main` (protected stable production) and `develop` (active integration), deprecating legacy auto-generated branches.
* **Requirements & Setup Documentation**: Completely replaced legacy `.bat` execution references with cross-platform npm and cargo commands.
* **History Data Transfer Object**: Standardized DTO payload key to `ui_type` (`"SINGLE"` vs `"BINGE"`), eliminating potential SQL and JavaScript reserved keyword collisions.
* **Season Tab Layout**: Evolved season tabs from flex-wrapped buttons into a single-line horizontal pill-style momentum scrolling row.
* **Rating Pill Badges**: Standardized rating badge positioning and styling with frosted dark backing (`bg-[#0D0F14]/60 backdrop-blur-md`) and tabular numeral alignment.

### Fixed
* **WebView2 70.3% GPU Compositor Spike**: Resolved critical idle GPU load in WebView2 by eliminating continuous infinite scale transforms beneath CSS `backdrop-filter: blur(...)` layers in `SafeImage.tsx`. Replaced with static hardware fade-ins, dropping idle GPU to **0.0% – 1.0%**.
* **Hero Spotlight CSS Repaints**: Replaced infinite `animate-pulse` on spotlight tags with high-contrast static neon styling, eliminating compositor redraw cycles over blurred backdrop banners.
* **Cascading Delete Safety**: Shifted media deletions completely to atomic SQLite `ON DELETE CASCADE` triggers, preventing orphaned episode records.
* **VLC Argument Parsing**: Automatically strips Windows verbatim prefixing (`\\?\`) before passing file paths to VLC to prevent URI encoding crashes.
* **Synopsis Overflow Safety**: Truncates show and episode overviews to 10,000 characters before database insertion to prevent IPC payload lockups on edge cases.


---

## [1.1.0] - 2026-08-15

### Added
* **Smart Media File Scanner**: Multi-threaded recursive directory traversal using Rust's `walkdir` and multi-pattern regex matching (`S01E01`, `1x01`, `Ep 01`, etc.).
* **Windows Long Path & Shortcut Support**: Full compatibility with deep paths via Windows `\\?\` prefixing and automatic Windows shortcut (`.lnk`) resolution using `parselnk`.
* **Persistent Triage Inbox**: Unmatched media files are parsed and grouped by extracted title into a dedicated triage UI for 1-click matching or manual linking.
* **Custom Streaming Protocol (`watchmark://`)**: Native asynchronous URI protocol supporting Range header parsing and percent-decoded streaming to bypass WebView local-asset restrictions.
* **Database Backup & Restore**: One-click in-app database backups with SHA-256 integrity verification, automated backup rotation, and safe restore routines.
* **Database Optimization Engine**: Integrated SQLite `VACUUM` and `PRAGMA optimize` maintenance tools to maintain lightning-fast queries.
* **OS Credential Manager (Keyring)**: Secure storage of TMDB API keys in the Windows Credential Manager, macOS Keychain, or Linux Secret Service, with fallback to `TMDB_API_KEY` environment variables.
* **System Tray & Global Hotkeys**: Native system tray integration with quick-action triggers (Scan Directory, Check for Updates) and global keyboard shortcuts (`Ctrl + Shift + S`).

### Changed
* **Chunked Scanner Streaming**: Scanner emits batches of 50 files (`scan-match-batch`) over the Tauri event bus to eliminate memory spikes during initial scans of large libraries.
* **Centralized Async IPC Hook**: All Tauri backend invocations migrated to `useAsyncInvoke` for standardized timeout handling, retry logic, and toast alerts.

### Fixed
* **SQLite PRAGMA Panic**: Fixed database connection initialization by updating PRAGMAs (`journal_mode = WAL;`) using `conn.pragma_update` rather than `conn.execute`.
* **Corrupted Settings Auto-Repair**: Defensive deserialization resets invalid or malformed JSON settings to defaults without application crashes.

---

## [1.0.0] - 2026-08-01

### Added
* **Tauri 2.0 Architectural Overhaul**: Complete migration from legacy Python/CustomTkinter to a pure Rust backend and React 19 + TypeScript + Tailwind CSS frontend.
* **Zero-CPU VLC Background Telemetry**: Asynchronous child process management using `tokio::process::Command` and `tokio::select!` event loops that consume zero background CPU while watching.
* **Dynamic VLC Port Probing**: Automatically detects free local ports between 8080 and 8090 and generates cryptographic one-time HTTP passwords for secure VLC telemetry.
* **Smart Playhead Resumption**: Tracks exact playhead position in seconds (`last_position`) and resumes playback seamlessly using VLC's `--start-time` flag.
* **Intelligent Completion Thresholds**: Automatically logs episodes as completed and increments rewatch counts when watched past 90% or within 120s of the credits.
* **Relational SQLite Storage**: Multi-table schema (`Media`, `Episodes`, `Local_Files`, `History`, `Unmatched_Files`) running in Write-Ahead Logging mode (`PRAGMA journal_mode = WAL;`).
* **Evolutionary Migrations**: Safe `ALTER TABLE` transactional migration system ensuring seamless schema updates without data loss.
* **Apple TV / Plex Cinema Aesthetic**: Dark-mode visual system (`#0D0F14`), translucent glassmorphism (`bg-[#1F222A]/60` with `backdrop-blur-md`), and signature VLC Orange accents (`#FF6B00`).
* **Universal TMDB Integration**: Asynchronous TMDB v3 API client with exponential backoff retries (`reqwest-retry`) and local offline poster/backdrop caching.

---

<div align="center">
  <sub>For full conversational development archives, refer to the local development documentation.</sub>
</div>
