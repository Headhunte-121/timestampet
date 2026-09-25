# WatchMark Documentation Hub

Welcome to the official technical documentation for **WatchMark Media Tracker**. This directory contains comprehensive architectural blueprints, database schemas, formal engineering specifications, Quality Assurance test matrices, and developer guides.

---

## 📚 Documentation Index

```text
docs/
├── architecture/
│   ├── AI_ORCHESTRATION.md         # System decomposition, agentic harness, and empirical case studies
│   ├── SYSTEM_DESIGN.md            # Topology, Tokio async loops, and VLC telemetry protocol
│   └── DATABASE_SCHEMA.md          # Relational SQLite tables, WAL mode, PRAGMAs, and migrations
├── specifications/
│   └── SYSTEM_SPECIFICATIONS.md    # Formal engineering specs for all 16 core subsystems
├── testing/
│   └── TEST_MATRIX.md              # Systematic stress tests, permissions, and edge cases
└── dev/
    ├── LOCAL_DEVELOPMENT.md        # Source setup, debugging tools, logging, and release packaging
    └── AI_WORKFLOWS.md             # Transparent guide to the Human-in-the-Loop AI pairing workflow
```

---

## 🏛️ 1. Architecture & Design

* [**AI Orchestration & System Decomposition (`architecture/AI_ORCHESTRATION.md`)**](./architecture/AI_ORCHESTRATION.md)
  * Hierarchical system breakdown framework (6 core domains, 16 micro-features).
  * Human-in-the-Loop agentic workflow with strict SOP (`AGENTS.md`) and compiler-as-a-verifier gates.
  * Real-world empirical case studies (resolving 70% GPU compositor spike, zero-CPU VLC telemetry, recommendation scoring).
  * Ready-to-use resume and portfolio technical bullet points.

* [**System Design & Topology (`architecture/SYSTEM_DESIGN.md`)**](./architecture/SYSTEM_DESIGN.md)
  * High-level multi-process architecture (React 19 + Tauri v2 + Pure Rust).
  * 0% background CPU async execution loop (`tokio::process::Command` + `tokio::select!`).
  * Non-intrusive VLC telemetry protocol, socket probing (8080–8090), and playhead tracking.
  * Custom Range streaming protocol (`watchmark://`) and path security (`filesystem_guard.rs`).
  * `VirtualPoster` IntersectionObserver windowing engine for 1,000+ shows.

* [**Database Schema & Relational Storage (`architecture/DATABASE_SCHEMA.md`)**](./architecture/DATABASE_SCHEMA.md)
  * Complete SQLite ER diagram and table definitions (`Media`, `Episodes`, `Local_Files`, `History`, `Unmatched_Files`).
  * Write-Ahead Logging (`PRAGMA journal_mode = WAL;`) and cascading deletes (`ON DELETE CASCADE`).
  * Transactional evolutionary migration engine (`PRAGMA user_version`).
  * Point-in-time database backups with SHA-256 integrity checksums.

---

## 📋 2. Specifications

* [**Technical Specifications Suite (`specifications/README.md`)**](./specifications/README.md)
  * Formal, IEEE 29148 / RFC 2119 compliant systems engineering specifications across all core tiers:
    1. [**01: Core Shell, Lifecycle & OS Boundaries**](./specifications/01_CORE_SHELL_AND_STORAGE.md) — App initialization, canary access probe, OS Keyring, and window geometry clamping.
    2. [**02: Database Engine, WAL Concurrency & Integrity**](./specifications/02_DATABASE_AND_CONCURRENCY.md) — Relational schema, WAL concurrency, evolutionary migrations, and SHA-256 backup verification.
    3. [**03: TMDB Metadata Engine & Synchronization**](./specifications/03_TMDB_METADATA_ENGINE.md) — Upstream wire protocol, token-bucket limiter ($40\text{ req}/10\text{s}$), exponential backoff, and 3-tier image caching.
    4. [**04: High-Speed Media Scanner & Triage Inbox**](./specifications/04_MEDIA_SCANNER_AND_INBOX.md) — Recursive traversal (depth 15), Windows long path prefix (`\\?\`), hardware cycle detection, EBNF token grammar, and triage inbox.
    5. [**05: VLC Telemetry & Playback Orchestration**](./specifications/05_VLC_TELEMETRY_AND_PLAYBACK.md) — Child process supervisor ($0\%\text{ idle CPU}$), dynamic TCP port negotiation ($8080\dots8090$), loopback telemetry, completion thresholds ($\ge 90\%$), Oops Guard, and 6-hour binge clustering.
    6. [**06: Cinema UI Design System & Virtualization**](./specifications/06_CINEMA_UI_AND_COMPONENTS.md) — Semantic dark tokens, `VirtualPoster` windowing engine ($<50\text{MB}$ memory at $1,000+$ items), `SafeImage` spoiler blur, diurnal vibe classification, and battery-saver mode.
    7. [**Master Feature Specifications (`specifications/SYSTEM_SPECIFICATIONS.md`)**](./specifications/SYSTEM_SPECIFICATIONS.md) — Cross-cutting engineering matrix covering all 16 core subsystems and 80+ functional micro-requirements.

---

## 🧪 3. Quality Assurance & Testing

* [**Quality Assurance & Testing Matrix (`testing/TEST_MATRIX.md`)**](./testing/TEST_MATRIX.md)
  * Permission boundary tests (read-only AppData, restricted user profiles, canary write failures).
  * Crash resilience & migration tests (power-cut rollback, hot DB deletions, duplicate column suppression).
  * Filesystem stress tests (Windows MAX_PATH > 260 chars, circular `.lnk` shortcuts, 0-byte disk).
  * Settings auto-repair tests (corrupted syntax, floating-point dimensions, off-screen window recovery).
  * VLC telemetry edge cases (port collisions, sudden VLC termination, exact-second resumption).

---

## 💻 4. Development & AI Engineering

* [**Local Development Guide (`dev/LOCAL_DEVELOPMENT.md`)**](./dev/LOCAL_DEVELOPMENT.md)
  * Setting up Node.js, Rust 2021, and VLC build dependencies.
  * Running live development with HMR (`npm run tauri dev` or `run.bat`).
  * Inspecting diagnostic logs (`watchmark.log`) and querying SQLite via CLI.
  * Compiling standalone native executables (`npm run tauri build`).

* [**AI-Assisted Architecture Workflow (`dev/AI_WORKFLOWS.md`)**](./dev/AI_WORKFLOWS.md)
  * Comprehensive disclosure of the Human-in-the-Loop agentic AI methodology.
  * Explaining the Jules Agent Standard Operating Procedure ([`AGENTS.md`](../AGENTS.md)).
  * Mandatory verification gates (`cargo check`, `npx tsc --noEmit`) ensuring zero technical debt.

---

<div align="center">
  <sub>Back to the <a href="../README.md">Main Project README</a> • <a href="../ROADMAP.md">Roadmap</a> • <a href="../CHANGELOG.md">Changelog</a></sub>
</div>
