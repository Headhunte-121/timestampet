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

---

## 🚀 Future Milestones

### 📌 Milestone 4: External Integrations & Cloud Sync (`v1.3.0`)
* [ ] **Trakt.tv Two-Way Scrobbling**: Optional two-way synchronization to automatically scrobble watched episodes and import watch history from Trakt.
* [ ] **Custom Metadata Scrapers**: Support for user-defined NFO files and local `.nfo` metadata parsing alongside TMDB.
* [ ] **Multi-Folder Scanner Monitoring**: Background filesystem watcher using Rust's `notify` crate to detect newly added episodes automatically without manual scan triggers.

### 📌 Milestone 5: Audio & Subtitle Track Selection (`v1.4.0`)
* [ ] **Media Stream Probing**: Use `ffprobe` / native metadata readers to preview embedded audio languages and subtitle tracks directly inside Media Details.
* [ ] **Default Language Preferences**: Configure preferred audio and subtitle languages in Settings, automatically passed to VLC via command-line parameters (`--sub-language`, `--audio-language`).

### 📌 Milestone 6: Multi-Profile & Companion Interface (`v2.0.0`)
* [ ] **Local Multi-Profile Support**: Multiple user profiles (e.g. Household Members) sharing local media files with isolated watch histories and ratings.
* [ ] **Local Network Remote / Web Companion**: Lightweight optional local web companion allowing playback control and progress inspection from a smartphone on the same Wi-Fi network.

---

<div align="center">
  <sub>Have a feature request? Open an issue on GitHub using our Feature Request template!</sub>
</div>
