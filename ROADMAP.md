# WatchMark Project Roadmap

This document outlines the architectural milestones, completed systems, and forward-looking product roadmap for the **WatchMark Media Tracker**.

---

## 🎯 Project Vision
WatchMark is engineered to be the definitive **client-side, privacy-focused media tracking diary and local file bridge**. It combines the beauty and richness of modern streaming frontends (Apple TV, Plex) with the speed, memory safety, and independence of pure desktop software—zero server daemons, zero continuous CPU transcoding, and zero telemetries.

---

## 🏆 Completed Milestones

### ✅ Milestone 1: Core Engine & SQLite Architecture (`v1.0.0`)
* [x] **Tauri 2.0 Architectural Overhaul**: Replaced legacy Python with a 100% pure Rust backend and React 19 + TypeScript + Tailwind CSS frontend.
* [x] **SQLite Relational Foundation**: Multi-table schema running in Write-Ahead Logging (`PRAGMA journal_mode = WAL;`) with atomic evolutionary migrations.
* [x] **VLC Telemetry Engine**: Tokio async event loops polling VLC's local HTTP API with 0% idle CPU and dynamic port probing (8080–8090).
* [x] **Smart Playhead Resumption**: Remembers exact second positions (`last_position`) and auto-completes at ≥90% watch ratio.
* [x] **TMDB Metadata Integration**: Async client with exponential backoff retries and local image caching in OS AppData.

### ✅ Milestone 2: Filesystem Integration & Data Integrity (`v1.1.0`)
* [x] **High-Speed Media Scanner**: Recursive directory traversal using Rust's `walkdir` and multi-pattern regex (`S01E01`, `1x01`, `Ep 01`).
* [x] **Windows Long Path & Shortcut Support**: Windows `\\?\` prefix normalization and `.lnk` shortcut parsing via `parselnk`.
* [x] **Persistent Triage Inbox**: Grouping unmatched media files by parsed series name for 1-click matching or manual linking.
* [x] **Custom Streaming Protocol (`watchmark://`)**: Native asynchronous URI protocol supporting Range header parsing for video seeking.
* [x] **Database Safety**: Atomic in-app database backups with SHA-256 integrity verification, auto-rotation, and SQLite `VACUUM` optimization.
* [x] **OS Credential Manager**: Secure TMDB API key storage via OS Keyring (Windows Credential Manager / Keychain / Secret Service).

### ✅ Milestone 3: Cinema UI & Virtualized Timeline (`v1.2.0`)
* [x] **BingeBlock Accordion Timeline**: Master-detail accordion grouping consecutive episodes with Framer Motion auto-height physics.
* [x] **Midnight Crossover Detection**: Dynamic epoch timestamp calculations displaying a `<Moon />` icon when sessions span past midnight.
* [x] **Distinct Show Session Isolation**: Auto-chains consecutive episodes watched within 6 hours (< 21,600s) while immediately isolating different shows.
* [x] **DOM Virtualization (`VirtualPoster`)**: Custom `IntersectionObserver` windowing engine that unmounts off-screen elements, supporting 1,000+ shows with low RAM.
* [x] **Edge-to-Edge Hero Banner**: 450px banner with diagonal gradient fades, bold typography, and 1-click resume actions.
* [x] **Global Quick Search**: Floating dark-glass search input in top navigation bar filtering rows and grids in real-time.

## 🚀 Roadmap & Upcoming Milestones

### 🚀 Milestone 4: GPU Compositor Optimization & Smart Recommendation Engine (`v1.3.0` - Active Dev)
* [x] **Compositor Load Elimination**: Eliminated continuous scale transforms behind CSS `backdrop-filter: blur(...)` surfaces in `SafeImage`, reducing idle GPU from ~70% to 0.0% – 1.0%.
* [x] **Intelligent Hero Spotlight Recommendation Engine**: 8-dimensional weighted candidate scoring algorithm prioritizing paused in-progress shows, newly aired episodes, and season finales with dynamic library fallback.
* [x] **Continuous Integration Pipeline**: Automated GitHub Actions workflow (`.github/workflows/ci.yml`) performing multi-stage verification on Rust backend (`cargo check`) and React frontend (`npm run build`).

### 📌 Milestone 5: Deep Media & Granular Inspector (`v1.4.0`)
* [ ] **Container & Stream Probing**: Preview embedded audio codecs, bitrates, channels, and subtitle tracks via `ffprobe` / native container readers.
* [ ] **VLC Stream Flags**: Pass preferred audio/subtitle languages (`--sub-language`, `--audio-language`) directly to VLC process invocation.

### 📌 Milestone 6: Automated Filesystem Watching & Advanced Scanner (`v1.5.0`)
* [ ] **Native File Watcher**: Background filesystem watcher using Rust's `notify` crate to detect newly added episodes automatically without manual scan triggers.
* [ ] **Background Indexing Queue**: Asynchronous processing queue with debounce windows to index downloaded or moved files with zero UI lag.

### 📌 Milestone 7: History Analytics & Trakt Cloud Sync (`v1.6.0`)
* [ ] **Trakt.tv Two-Way Scrobbling**: Optional two-way synchronization to automatically scrobble watched episodes and import watch history from Trakt.
* [ ] **Viewing Habit Visualizations**: Interactive GitHub-style watch activity heatmaps, completion rates, and genre breakdowns.
* [ ] **Universal History Export**: Export/import watch records in standardized JSON and CSV formats.

### 📌 Milestone 8: Advanced Media Management & Batch Organizer (`v1.7.0`)
* [ ] **Intelligent Batch File Renamer**: Standardize local file naming (`{Show} - S{s:02d}E{e:02d} - {Title}.{ext}`) with atomic OS rename transactions and dry-run previews.
* [ ] **Offline Metadata & Artwork Exporter**: Generate Kodi/Jellyfin-compatible `.nfo` XML files and local artwork sidecars (`poster.jpg`, `fanart.jpg`) for 100% offline environments.
* [ ] **Library Health & Integrity Diagnostics**: Scan for corrupt video container headers, missing season episode gaps, and duplicate files.

### 📌 Milestone 9: Multi-Profile Ecosystem & Web Companion (`v2.0.0`)
* [ ] **Local Multi-Profile Support**: Multiple user profiles (e.g. Household Members) sharing local media files with isolated watch histories and ratings.
* [ ] **Local Network Remote / Web Companion**: Lightweight optional local web companion allowing playback control and progress inspection from a smartphone on the same Wi-Fi network.

---

<div align="center">
  <sub>Have a feature request? Open an issue on GitHub using our Feature Request template!</sub>
</div>
