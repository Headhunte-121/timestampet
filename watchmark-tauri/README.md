# WatchMark Tauri Workspace (`watchmark-tauri`)

This directory contains the complete desktop application source code for **WatchMark Media Tracker**, powered by **Tauri 2.0**, **Rust**, and **React 19**.

For full user documentation, feature overviews, and architecture diagrams, see the [Main Project README](../README.md).

---

## 🛠 Directory Layout

* **`src/`**: React 19 + TypeScript frontend.
  * `components/`: UI views (`Dashboard`, `Library`, `MediaDetails`, `History`, `BingeBlock`, `InboxView`, `SearchTMDB`, `SettingsView`).
  * `components/ui/`: Atomic UI widgets (`VirtualPoster`, `SafeImage`, `Modal`, `StarRating`).
  * `hooks/`: Custom React hooks (`useAsyncInvoke`, `useHorizontalScroll`).
  * `store/`: Zustand state management (`useAppStore`, `useTaskStore`, `uiStore`).
  * `utils/`: Path utilities, date formatting, and Sonner toast bridges.
* **`src-tauri/`**: Pure Rust backend.
  * `src/main.rs`: Application lifecycle, window management, system tray, and custom URI streaming protocol (`watchmark://`).
  * `src/commands.rs`: Tauri IPC command definitions.
  * `src/db.rs`: SQLite connection pooling, migrations, and relational queries.
  * `src/vlc.rs`: Tokio async VLC process launcher and HTTP telemetry polling.
  * `src/scanner.rs`: Recursive file scanner with regex matching.
  * `src/tmdb.rs`: TMDB API v3 async client with caching.
  * `src/settings.rs`: JSON preferences and OS Credential Manager (Keyring).
  * `src/backup.rs`: SQLite atomic backup/restore with SHA-256 verification.

---

## 💻 Development Commands

From this directory (`watchmark-tauri`):

```bash
# Install NPM dependencies
npm install

# Start development desktop environment (Vite HMR + Cargo dev)
npm run tauri dev

# Typecheck TypeScript code
npx tsc --noEmit

# Build production bundle
npm run tauri build
```

From the Rust backend directory (`watchmark-tauri/src-tauri`):

```bash
# Check Rust compilation
cargo check

# Run unit and integration tests
cargo test
```

