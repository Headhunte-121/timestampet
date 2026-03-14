# WatchMark Pro: Definitive Master To-Do List

This document represents the complete functional blueprint and state of the WatchMark codebase across 16 primary systems. Every single requested feature is expanded with functional sub-steps, edge cases, and behavior details.

## 🟢 Part 1: Core Architecture & Backend Engine (Rust/Tauri) (Incomplete)

**1.1 Built on the modern Tauri 2.0 framework for maximum efficiency.** (Complete)

- [x] Verify compatibility with the latest Tauri 2.0 release candidate and plugins.
- [x] Ensure standard webview protocols are correctly registered across OSs.
- [x] Handle edge cases where the system webview is outdated or missing entirely.
- [x] Confirm that local development hot-reloading does not leak memory over time.
- [x] Test core feature degradation if running on unsupported legacy OS versions.

**1.2 100% Rust backend ensuring native OS performance.** (Complete)

- [x] Ensure zero background CPU usage when the app is idle.
- [x] Gracefully handle panics in Rust threads without crashing the entire app shell.
- [x] Optimize memory allocation for large string parsing operations.
- [x] Provide detailed Rust logging to local files for production debugging.
- [x] Test edge cases involving low-memory environments to ensure the app doesn't forcefully terminate.

**1.3 React (TypeScript) frontend architecture.** (Complete)

- [x] Enforce strict typing across all IPC payload boundaries.
- [x] Handle React hydration errors gracefully during hot reloads.
- [x] Ensure deeply nested state updates do not trigger unnecessary whole-page re-renders.
- [x] Implement a global error boundary to catch and display unhandled UI exceptions.
- [x] Test component unmounting cleanup to prevent memory leaks in long-lived sessions.

**1.4 Tailwind CSS styling engine integration.** (Complete)

- [x] Verify Tailwind purge paths catch all dynamically constructed class names.
- [x] Ensure custom color palettes (e.g., VLC Orange) compile down correctly without overriding defaults.
- [x] Handle edge cases where viewport resizing breaks flexbox or grid container limits.
- [x] Check that `backdrop-blur` utilities don't cause extreme GPU spikes on lower-end devices.
- [x] Test text contrast ratios in Tailwind configurations for accessibility compliance.

**1.5 Framer Motion engine for 60FPS hardware-accelerated animations.** (Complete)

- [x] Provide a global toggle to disable heavy animations for battery-saving mode.
- [x] Ensure `<AnimatePresence>` correctly cleans up unmounted nodes to prevent ghost elements.
- [x] Handle edge cases where rapid clicking queues up conflicting animation states.
- [x] Verify complex layout animations do not cause text to jitter or blur.
- [x] Test animation performance limits when rendering grids of 500+ items.

**1.6 Ultra-lightweight binary footprint (~30MB RAM usage).** (Complete)

- [x] Monitor and aggressively clear image caches in React memory after unmounting views.
- [x] Optimize SQLite connection pooling to minimize constant RAM footprint.
- [x] Handle edge cases where background scanning spikes RAM by chunking operations.
- [x] Verify Rust build is compiled with `opt-level = 'z'` or `s` for size reduction.
- [x] Test idle RAM usage after leaving the app open in the background for 48 hours.

**1.7 Cross-platform compatibility (Windows, macOS, Linux).** (Complete)

- [x] Ensure path separators (`\` vs `/`) are handled dynamically across all OSs.
- [x] Handle edge cases where Linux distributions lack a standard system tray implementation.
- [x] Verify macOS specific permissions for accessing external drives or Documents.
- [x] Test Windows SmartScreen false-positive mitigations by ensuring proper signing.
- [x] Ensure global keyboard shortcuts do not conflict with native OS defaults.

**1.8 Native OS window frame integration (supports Windows snap-assist and native drop shadows).** (Incomplete)

- [ ] Ensure titlebar drag regions do not overlap with clickable UI elements like tabs.
- [ ] Handle edge cases where restoring from minimized state breaks layout dimensions.
- [ ] Test multi-monitor dragging where DPI scales drastically change between screens.
- [ ] Verify Windows Snap-Assist snapping triggers proper layout recalculations.
- [ ] Ensure custom macOS traffic light buttons (close/minimize/maximize) align perfectly.

**1.9 Seamless IPC (Inter-Process Communication) bridging via Tauri invoke.** (Complete)

- [x] Implement timeout safety for IPC calls that hang or take longer than expected.
- [x] Sanitize all string inputs sent from React to Rust to prevent injection or panics.
- [x] Handle edge cases where large JSON payloads block the main IPC thread.
- [x] Ensure type mismatches between frontend TS and backend Rust structs log clear errors.
- [x] Test IPC event listener cleanup so multiple listeners aren't attached on page reloads.

**1.10 Single-thread centralized Task Queue for database writes to prevent locking.** (Incomplete)

- [ ] Queue sudden bursts of write operations (e.g., bulk updates) sequentially.
- [ ] Handle edge cases where the app is closed while the queue is still processing.
- [ ] Ensure read queries can still execute concurrently while writes are queued.
- [ ] Test SQLite `database is locked` error mitigation during rapid click events.
- [ ] Provide a fallback retry mechanism if a specific queued write fails.

**1.11 Asynchronous multi-threaded read operations so the UI never blocks.** (Incomplete)

- [ ] Ensure pagination queries yield fast initial load times.
- [ ] Handle edge cases where the user navigates away before a massive read completes.
- [ ] Verify thread pool sizes do not exceed OS thread limits on low-end CPUs.
- [ ] Test concurrent reads happening alongside background poster downloading.
- [ ] Implement caching for highly repetitive reads (like global stats) to bypass DB entirely.

**1.12 Strict NoneType/null data sanitization layer in Rust before data reaches React.** (Incomplete)

- [ ] Map all missing TMDB dates to a safe default instead of null crashing UI components.
- [ ] Ensure empty string descriptions do not collapse layout margins in React.
- [ ] Handle edge cases where expected integers (like `runtime`) return as null or 0 from DB.
- [ ] Provide placeholder arrays for empty history or library results.
- [ ] Test deeply nested JSON serialization ensuring no unexpected `undefined` bubbles up.

**1.13 Auto-generation of local application data directories on first boot.** (Complete)

- [x] Handle edge cases where the OS denies write permissions to the intended install directory.
- [x] Verify nested folders (`/cache/posters`, `/db`) are created successfully.
- [x] Provide a fatal error UI if directory creation completely fails.
- [x] Test behavior if a user manually deletes the data folder while the app is running.
- [x] Ensure proper fallback logic if migrating from an older schema.

**1.14 Persistent JSON settings storage (Window size, position, API keys).** (Complete)

- [x] Handle corrupted JSON files by cleanly resetting to defaults instead of crashing.
- [x] Ensure API keys are stored securely or at least obscured from plain sight.
- [x] Test saving settings concurrently from multiple UI tabs.
- [x] Verify that deleting the config JSON dynamically generates a new one on next boot.
- [x] Handle edge cases where integer sizes/positions are saved as floats and break parsing.

**1.15 Automatic window geometry and position restoration on launch.** (Incomplete)

- [ ] Handle edge cases where the saved position is now off-screen (e.g., disconnected secondary monitor).
- [ ] Ensure maximized states are restored correctly without hiding the taskbar.
- [ ] Verify width/height are constrained to minimum allowable app dimensions.
- [ ] Test rapid opening/closing of the app to ensure bounds save correctly.
- [ ] Ensure fullscreen vs. windowed states are properly distinguished and saved.

## 📁 Part 2: Local File Scanning & Smart Parser (Incomplete)

**2.1 Native OS directory selection dialog.** (Incomplete)

- [ ] Ensure dialog strictly restricts selection to directories, not individual files.
- [ ] Handle edge cases where the user cancels the dialog (should fail gracefully).
- [ ] Test behavior when a selected directory is read-only or restricted by the OS.
- [ ] Verify default paths open to logical locations (e.g., user's Video folder).
- [ ] Handle edge cases where a network drive is selected and suddenly disconnects.

**2.2 Recursive subfolder deep-scanning.** (Incomplete)

- [ ] Implement a maximum depth limit to prevent infinite recursion in complex directory trees.
- [ ] Handle circular symlinks that could cause scanning loops.
- [ ] Ensure the UI visually indicates that a deep scan is actively running.
- [ ] Test scanning performance on folders containing 10,000+ files.
- [ ] Provide a manual 'Cancel Scan' button to halt the background thread.

**2.3 Real-time file extension filtering (.mkv, .mp4, .avi, .mov, etc.).** (Incomplete)

- [ ] Ensure file extension matching is entirely case-insensitive (.MKV vs .mkv).
- [ ] Handle edge cases with double extensions (e.g., `video.tar.gz` skipped, `video.mkv` parsed).
- [ ] Provide a settings option to manually add or remove supported formats.
- [ ] Verify hidden OS files (like `.DS_Store` or `Thumbs.db`) are strictly ignored.
- [ ] Test behavior when a valid file extension masks a corrupted or zero-byte file.

**2.4 Complex Regex Engine: Extracts Series Name, Season, and Episode from standard SxxExx formats.** (Incomplete)

- [ ] Handle spaces, dots, and hyphens preceding the SxxExx block.
- [ ] Ensure single-digit seasons and episodes (e.g., S1E5) parse equally well as S01E05.
- [ ] Handle multi-episode files gracefully (e.g., S01E01-E02).
- [ ] Verify text following the episode block (like episode titles) is cleanly stripped.
- [ ] Test edge cases where the series name contains numbers matching the regex.

**2.5 Complex Regex Engine: Parses alternative TV formats (e.g., Sxx.Exx, [Sxx][Exx]).** (Incomplete)

- [ ] Handle bracketed formats often used in anime releases.
- [ ] Parse explicit 'Season X Episode Y' full text strings.
- [ ] Ensure year-based episode formats (e.g., 2023.10.05) fall back to specific parsing logic.
- [ ] Verify absolute episode numbers (e.g., Episode 105 instead of S3E05) flag for manual review.
- [ ] Test edge cases where multiple format styles appear in the same filename.

**2.6 Complex Regex Engine: Parses Movie formats extracting Title and (Year).** (Incomplete)

- [ ] Extract the year strictly from 4-digit blocks surrounded by parenthesis or brackets.
- [ ] Ensure resolutions like '1080p' or '4K' are not mistaken for a release year.
- [ ] Handle edge cases where the movie title itself contains a year (e.g., 'Blade Runner 2049 (2017)').
- [ ] Verify standard release group tags are ignored during title extraction.
- [ ] Test titles with colons or hyphens replacing spaces.

**2.7 Non-blocking background thread execution for massive directory scans.** (Incomplete)

- [ ] Ensure the UI remains 60FPS responsive while scanning a 5TB drive.
- [ ] Implement a progress channel sending batch updates to React rather than single file events.
- [ ] Handle thread panics gracefully if the drive is unexpectedly ejected.
- [ ] Test pausing and resuming the scan queue.
- [ ] Ensure CPU prioritization is set to lower levels so system performance isn't tanked.

**2.8 Automated String-Cleaning (removing dots, underscores, resolution tags).** (Incomplete)

- [ ] Strip all known codec tags (x264, x265, HEVC, AAC).
- [ ] Remove resolution tags (720p, 1080p, 4K, 2160p).
- [ ] Replace periods and underscores with standard space characters.
- [ ] Handle edge cases where the actual show title contains dots (e.g., 'Mr. Robot').
- [ ] Verify trailing hyphens and brackets are entirely trimmed.

**2.9 Auto-matching scanned files directly to existing tracked database entries.** (Incomplete)

- [ ] Implement fuzzy string matching to account for slight spelling differences.
- [ ] Ensure season/episode integers precisely match before automatically linking the file.
- [ ] Handle edge cases where two shows have the identical parsed name but different years.
- [ ] Do not overwrite an existing matched file path without explicit confirmation.
- [ ] Provide a detailed log of which files were auto-matched vs left unmatched.

**2.10 Path collision detection (prevents duplicate file paths in the DB).** (Incomplete)

- [ ] Ensure the database strictly enforces UNIQUE constraints on the file path column.
- [ ] Handle edge cases where a file is renamed but the content/hash remains identical.
- [ ] Verify that scanning the exact same directory twice results in zero new additions.
- [ ] Handle case-sensitivity issues on Windows (treating `C:\File` and `c:\file` as identical).
- [ ] Test resolving path conflicts seamlessly without throwing a red UI error.

**2.11 Symlink and shortcut resolution for external drives.** (Incomplete)

- [ ] Resolve `.lnk` files on Windows to their absolute target paths.
- [ ] Resolve standard Unix symlinks accurately on macOS/Linux.
- [ ] Handle edge cases where the symlink points to a deleted or non-existent file.
- [ ] Ensure the database stores the final resolved path, not the symlink path.
- [ ] Test recursive scanning through directory symlinks while preventing infinite loops.

**2.12 "Scan Complete" dynamic system toast notification with match counts.** (Incomplete)

- [ ] Ensure the toast auto-dismisses after a sensible timeout (e.g., 5 seconds).
- [ ] Display exact numerical data (e.g., 'Added 15 episodes, 3 unmatched').
- [ ] Handle edge cases where multiple scans finish simultaneously, preventing toast spam.
- [ ] Allow the user to click the toast to navigate directly to the Inbox/Unmatched view.
- [ ] Verify toasts render smoothly above all other modal z-indexes.

## 🌐 Part 3: TMDB Metadata & Syncing (Incomplete)

**3.1 Secure HTTP requests via Rust reqwest.** (Incomplete)

- [ ] Ensure strict TLS/SSL validation is enforced for all API calls.
- [ ] Handle connection timeouts gracefully on slow networks.
- [ ] Implement exponential backoff retry logic for temporary network drops.
- [ ] Verify user-agent headers are explicitly set to prevent API blocking.
- [ ] Test behavior when the host OS firewall entirely blocks the application.

**3.2 On-the-fly TMDB API Key validation.** (Incomplete)

- [ ] Provide instant visual feedback (Green Check / Red X) in the Settings UI when entering a key.
- [ ] Handle whitespace or accidental hidden characters pasted into the key field.
- [ ] Ensure invalid keys instantly halt all background API requests to prevent bans.
- [ ] Test edge cases where the API key is valid but the TMDB account is rate-limited.
- [ ] Display clear instructions/links for users to obtain their own API key.

**3.3 Multi-search API endpoint integration for mixed TV/Movie results.** (Incomplete)

- [ ] Ensure the UI visually distinguishes TV vs Movie results returned in the same payload.
- [ ] Handle edge cases where the multi-search returns unexpected media types (e.g., 'Person').
- [ ] Implement pagination support for broad search terms yielding hundreds of results.
- [ ] Verify search string URL encoding prevents crashes on special characters.
- [ ] Test response times and add debounce logic to the search input.

**3.4 Dedicated TV Show deep-data fetching.** (Incomplete)

- [ ] Ensure the payload strictly maps to the local SQLite `Media` table schema.
- [ ] Handle edge cases where a TV show lacks a synopsis entirely.
- [ ] Verify network data structures for complex fields like genres and networks are flattened.
- [ ] Test data fetching for shows with 50+ seasons (e.g., Soap Operas).
- [ ] Handle missing poster/backdrop URLs gracefully.

**3.5 Dedicated Movie deep-data fetching.** (Incomplete)

- [ ] Map movie specific data (like total runtime) accurately to the database.
- [ ] Ensure movie collections/franchises are handled visually if applicable.
- [ ] Test fetching data for extremely obscure or newly announced movies.
- [ ] Handle edge cases where the release date is completely undefined.
- [ ] Verify movie data doesn't accidentally trigger TV episode sync loops.

**3.6 Season-by-season iterative API fetching.** (Incomplete)

- [ ] Ensure requests for Season 1, 2, 3 etc., occur in staggered batches to avoid rate limits.
- [ ] Handle missing seasons (e.g., a show with Season 1 and 3, but no 2).
- [ ] Verify 'Specials' (Season 0) are fetched and categorized appropriately.
- [ ] Test canceling a fetch operation halfway if the user deletes the show.
- [ ] Provide UI progress bars for massive shows with hundreds of episodes.

**3.7 High-resolution primary Poster image extraction.** (Incomplete)

- [ ] Target specific TMDB image width configurations (e.g., `w500` or `original`).
- [ ] Ensure fallbacks are strictly enforced if the primary locale poster is missing.
- [ ] Handle corrupt or incomplete image byte streams during download.
- [ ] Test image fetching on extremely slow connections to ensure timeouts don't hang the app.
- [ ] Verify that poster dimensions are enforced regardless of the source aspect ratio.

**3.8 High-resolution cinematic Backdrop image extraction.** (Incomplete)

- [ ] Target `w1280` or `original` paths for crisp high-dpi display.
- [ ] Handle edge cases where a show has zero backdrops available on TMDB.
- [ ] Ensure backdrops are completely stripped of textual logos if clean variants exist.
- [ ] Test memory usage when rendering 10+ backdrops in memory simultaneously.
- [ ] Verify backdrop loading states show smooth CSS skeleton pulses.

**3.9 Episode-specific 16:9 still-image extraction.** (Incomplete)

- [ ] Ensure missing episode stills fall back to the show's main backdrop automatically.
- [ ] Handle API rate limiting strictly, as querying 200 episode images simultaneously will fail.
- [ ] Test edge cases where the still image is flagged as a spoiler and blurred.
- [ ] Verify the exact 16:9 crop is maintained in the UI regardless of the raw image.
- [ ] Ensure cached stills are tied to the specific episode ID.

**3.10 Show-level Synopsis/Overview text downloading.** (Incomplete)

- [ ] Ensure character encoding correctly handles foreign languages and emojis.
- [ ] Handle incredibly long synopses by ensuring DB schemas allow TEXT columns.
- [ ] Verify missing synopses are replaced with 'No overview available.' strings.
- [ ] Test edge cases where synopses contain markdown or HTML tags (strip them).
- [ ] Ensure localized overviews match the user's OS locale if possible.

**3.11 Episode-level Synopsis text downloading.** (Incomplete)

- [ ] Ensure spoiler synopses (often hidden by TMDB before airing) are handled.
- [ ] Test database size impact when storing thousands of episode synopses.
- [ ] Handle edge cases where the episode is a two-parter with duplicate synopses.
- [ ] Ensure text overflow in UI components uses standard ellipsis truncation.
- [ ] Verify synchronization updates synopses if they were previously blank.

**3.12 Original Release Date / First Air Date extraction.** (Incomplete)

- [ ] Parse 'YYYY-MM-DD' formats strictly into the SQLite `release_date` column.
- [ ] Handle edge cases where only the Year ('YYYY') is returned.
- [ ] Ensure null air dates do not crash the sorting algorithms.
- [ ] Test timeline timezone conversions if exact UTC timestamps are provided.
- [ ] Verify release dates update correctly if a 'TBD' show gets an official date.

**3.13 Specific Episode Air Date tracking.** (Incomplete)

- [ ] Ensure episodes aired in the future are visually flagged as 'Unaired' in the UI.
- [ ] Handle edge cases where episode 3 airs before episode 2.
- [ ] Verify sorting inside season tabs strictly follows the `episode_num` regardless of air date.
- [ ] Test missing air dates to ensure they default to the bottom of sorting arrays.
- [ ] Ensure mathematical calculations (e.g., 'Aired 5 years ago') handle leap years safely.

**3.14 Accurate Runtime/Duration metadata pulling.** (Incomplete)

- [ ] Handle API responses where runtime is an array instead of an integer.
- [ ] Ensure missing runtimes default to a sensible value (e.g., 0) for math calculations.
- [ ] Test edge cases where runtimes are extremely long (e.g., 200+ minute movies).
- [ ] Verify runtime updates dynamically if local file FFmpeg length overrides TMDB data.
- [ ] Ensure UI cleanly formats 135m as '2h 15m'.

**3.15 Total Episode count aggregation.** (Incomplete)

- [ ] Ensure 'Specials' (Season 0) do not inflate the standard episode count artificially.
- [ ] Handle edge cases where TMDB lists unaired episodes in the total count.
- [ ] Verify real-time completion percentages update properly when new episodes are added.
- [ ] Test aggregation math against locally tracked vs globally available numbers.
- [ ] Ensure discrepancies between local files and TMDB counts highlight missing files.

**3.16 Series Life-Cycle Status tracking (e.g., 'Returning Series', 'Ended', 'Canceled').** (Incomplete)

- [ ] Map specific TMDB status strings to visual UI badge colors (Green=Returning, Red=Ended).
- [ ] Ensure sync operations check for status changes on older shows.
- [ ] Handle edge cases where status is null or undefined.
- [ ] Test UI wrapping behavior if the status string is unusually long.
- [ ] Allow users to manually filter the Library based on these specific status flags.

**3.17 TMDB Global Vote Average (Score) fetching.** (Incomplete)

- [ ] Ensure floating-point numbers are rounded to a single decimal (e.g., 8.54 -> 8.5).
- [ ] Handle edge cases where the show has 0 votes.
- [ ] Verify UI components conditionally hide the score badge if no data exists.
- [ ] Test database updates so the score stays fresh upon manual sync.
- [ ] Ensure the TMDB score is visually distinct from the user's personal 5-star rating.

**3.18 Asynchronous background image downloading pipeline.** (Incomplete)

- [ ] Implement a task queue specifically for images to prevent thread starvation.
- [ ] Ensure failed image downloads automatically retry up to 3 times before skipping.
- [ ] Test concurrent downloads ensuring network bandwidth isn't completely saturated.
- [ ] Handle edge cases where the app closes mid-download (delete partial temp files).
- [ ] Provide a visual UI indicator if massive background caching is occurring.

**3.19 Local file caching for Posters (saving bandwidth and enabling offline mode).** (Incomplete)

- [ ] Store posters using their ID as the filename to prevent collision.
- [ ] Ensure the React frontend serves the `file://` cached path if available.
- [ ] Handle edge cases where the cached file is completely corrupted.
- [ ] Test cache invalidation if a new poster is selected or refreshed.
- [ ] Ensure the cache directory is excluded from OS backups or indexing.

**3.20 Local file caching for Backdrops.** (Incomplete)

- [ ] Verify high-res backdrops don't silently consume gigabytes of disk space.
- [ ] Implement an automated cleanup task to delete orphaned backdrops (shows removed from DB).
- [ ] Handle edge cases where write permissions fail during caching.
- [ ] Test UI fallback behavior transitioning smoothly from remote URL to local cache.
- [ ] Ensure filenames specify resolution (e.g., `w1280_12345.jpg`).

**3.21 Local file caching for Episode Stills.** (Incomplete)

- [ ] Strictly limit caching to recently viewed or 'Up Next' episodes to save space.
- [ ] Handle edge cases where thousands of stills rapidly fill the cache.
- [ ] Ensure cache eviction policies delete oldest stills when a size limit is hit.
- [ ] Test offline mode ensuring the UI gracefully loads stills from disk.
- [ ] Verify missing still states apply the specific blur/darken fallback design.

## 🎬 Part 4: VLC Playback & Smart Heartbeat (Incomplete)

**4.1 Auto-detection of VLC installation paths across OS defaults.** (Incomplete)

- [ ] Ensure fallback checks for default `C:\Program Files\VideoLAN\VLC\vlc.exe` on Windows.
- [ ] Ensure checks for `/Applications/VLC.app/Contents/MacOS/VLC` on macOS.
- [ ] Verify `/usr/bin/vlc` or snap/flatpak paths on Linux.
- [ ] Handle edge cases where VLC is installed but the executable is restricted.
- [ ] Provide a clear fatal error UI if automatic detection completely fails.

**4.2 Manual VLC executable path override in Settings.** (Incomplete)

- [ ] Provide a file picker dialog strictly filtering for executable files.
- [ ] Handle edge cases where the user selects a non-VLC executable.
- [ ] Ensure manual paths are saved persistently to JSON and override auto-detection.
- [ ] Verify that changing the path does not require an app restart.
- [ ] Test behavior if the manually specified path is deleted or moved externally.

**4.3 Spawning VLC as a detached external background process.** (Incomplete)

- [ ] Ensure closing the WatchMark app gracefully kills the spawned VLC instance (or leaves it running based on preference).
- [ ] Handle edge cases where VLC takes an unusually long time to spawn.
- [ ] Verify the external process doesn't lock the local video file from being renamed.
- [ ] Test spawning multiple VLC instances concurrently (should be prevented).
- [ ] Ensure Rust strictly uses `Command::new` without capturing standard output to prevent deadlocks.

**4.4 Passing secure local file paths directly to the VLC Command Line.** (Incomplete)

- [ ] Handle absolute paths containing spaces, foreign characters, and emojis safely.
- [ ] Ensure network drive paths (`//SERVER/Share`) are formatted correctly for VLC.
- [ ] Test escaping shell characters (quotes, ampersands) to prevent command injection.
- [ ] Verify long paths exceeding Windows 256-character limits are handled.
- [ ] Handle edge cases where the file was deleted immediately prior to clicking Play.

**4.5 Passing the --start-time CLI argument for exact-second resuming.** (Incomplete)

- [ ] Ensure the database value `last_position` is properly converted to a raw integer string.
- [ ] Handle edge cases where `last_position` is within 5 seconds of the end of the video.
- [ ] Verify VLC accepts the flag and jumps instantly without dropping video frames.
- [ ] Test behavior when `last_position` is explicitly 0.
- [ ] Ensure resuming doesn't break external subtitle file loading.

**4.6 Programmatically enabling VLC's local HTTP interface (--extraintf http).** (Incomplete)

- [ ] Ensure the `--http-port` flag explicitly defines a specific port (e.g., 8080).
- [ ] Handle edge cases where port 8080 is already in use by another application.
- [ ] Verify the `--http-password` is dynamically injected securely per session.
- [ ] Test OS firewall popups preventing the HTTP interface from binding.
- [ ] Ensure the interface binds strictly to `localhost/127.0.0.1` to prevent network snooping.

**4.7 Injecting a secure, randomized custom HTTP password for the session.** (Incomplete)

- [ ] Generate a random alphanumeric 16-character string on every playback event.
- [ ] Ensure Rust securely holds the password in memory for polling.
- [ ] Handle edge cases where password injection fails to parse in VLC.
- [ ] Verify older VLC versions support the specific password syntax flag.
- [ ] Test unauthorized external queries to the port to ensure rejection.

**4.8 Rust-based HTTP polling loop triggering every 5000ms.** (Incomplete)

- [ ] Ensure the thread sleeps correctly without blocking Tauri's main event loop.
- [ ] Handle connection refused errors gracefully while VLC is still booting up.
- [ ] Verify timeout limits on the HTTP request prevent hanging the thread.
- [ ] Test behavior when the system goes to sleep during the polling loop.
- [ ] Ensure the polling frequency can be configured via a strict constant.

**4.9 Real-time fetching of current playback time (in exact seconds).** (Incomplete)

- [ ] Parse the XML/JSON response from VLC safely handling missing nodes.
- [ ] Ensure the time value correctly updates local memory state.
- [ ] Handle edge cases where VLC reports negative time or garbage data during seeking.
- [ ] Test rapid seeking by the user to ensure the poller captures the final position.
- [ ] Verify performance overhead of parsing the payload every 5 seconds.

**4.10 Real-time fetching of total video length.** (Incomplete)

- [ ] Ensure the length accurately reflects the local file, overriding TMDB runtime.
- [ ] Handle edge cases where VLC reports a length of 0 (e.g., corrupted file index).
- [ ] Test files with variable frame rates that might cause length fluctuations.
- [ ] Verify the database is actively updated with the precise local length.
- [ ] Ensure UI calculations rely on this precise length immediately upon playback.

**4.11 Playback state monitoring (playing, paused, stopped).** (Incomplete)

- [ ] Ensure specific logic triggers specifically when the state shifts to 'paused'.
- [ ] Handle edge cases where the user rapidly spam-clicks pause/play.
- [ ] Verify 'stopped' events instantly finalize session logic and DB commits.
- [ ] Test behavior when playback reaches the absolute end of the file automatically.
- [ ] Ensure state strings from VLC are case-insensitively matched.

**4.12 Real-time mathematical completion percentage calculation.** (Incomplete)

- [ ] Ensure floating point math errors don't trigger false completions.
- [ ] Handle edge cases where total length is 0 (prevent divide by zero exceptions).
- [ ] Verify the percentage is calculated based on exact seconds, not rough minutes.
- [ ] Test calculations on incredibly short files (e.g., 2-minute clips).
- [ ] Ensure the percentage is logged accurately in the DB session row.

**4.13 >90% Auto-Completion Logic: Automatically marks episode watched and increments history.** (Incomplete)

- [ ] Ensure reaching 90.00% strictly triggers a database `INSERT` to History.
- [ ] Verify the episode status instantly shifts to 'Completed'.
- [ ] Handle edge cases where the user seeks past 90%, rewinds, and stops.
- [ ] Test rapid scrubbing to the end of the video to bypass actual viewing.
- [ ] Ensure the UI updates immediately to reflect the completed state globally.

**4.14 <90% Resume Logic: Saves the exact last_position to the database.** (Incomplete)

- [ ] Ensure the exact final second is recorded in the `last_position` column.
- [ ] Handle edge cases where the user watched 89.9% (should still resume).
- [ ] Verify the progress bar UI updates directly relative to this position.
- [ ] Test consecutive resume sessions updating the same position value accurately.
- [ ] Ensure zero-second positions reset the status entirely if manually triggered.

**4.15 Process termination detection (Rust knows instantly when the user closes the VLC window).** (Incomplete)

- [ ] Ensure `Child::wait()` or process monitoring instantly detects termination.
- [ ] Handle edge cases where VLC crashes or is forcefully killed by the OS.
- [ ] Verify the HTTP poller instantly halts when the process dies.
- [ ] Test behavior when multiple instances accidentally exist.
- [ ] Ensure final database commits occur before the process thread fully drops.

**4.16 Real-time IPC event emission to React to update the UI the second VLC closes.** (Incomplete)

- [ ] Emit a custom `vlc-closed` payload cleanly to the React listener.
- [ ] Ensure React instantly triggers a DB refetch to update all UI statuses.
- [ ] Handle edge cases where the React component was unmounted during playback.
- [ ] Verify no ghost events duplicate the refresh action.
- [ ] Test the latency between VLC closing and the React UI fully updating.

**4.17 Minimum threshold safety (ignoring accidental clicks watched for <5%).** (Incomplete)

- [ ] Ensure closing the video before 5% does not overwrite previous progress.
- [ ] Handle edge cases where a user starts a video by mistake and instantly closes it.
- [ ] Verify that >5% strictly engages the resume logic tracking.
- [ ] Test edge cases where an episode is less than 5 minutes long.
- [ ] Ensure UI cleanly ignores these short sessions without rendering messy history rows.

## 🗄️ Part 5: Database & Schema Design (SQLite) (Incomplete)

**5.1 Local, offline-first SQLite database (watchmark.db).** (Complete)

- [x] Ensure database initialization creates a new file seamlessly if one is not found.
- [x] Verify PRAGMA settings enable Write-Ahead Logging (WAL) for faster performance.
- [x] Handle edge cases where the directory lacks file write permissions.
- [x] Test database migration mechanisms for future schema updates.
- [x] Ensure foreign key constraints are strictly enabled on every connection.

**5.2 Media table schema (ID, TMDB ID, title, type, synopsis).** (Complete)

- [x] Verify string constraints prevent buffer overflow on massive synopsis texts.
- [x] Ensure TMDB ID is strictly UNIQUE to prevent duplicate show entries.
- [x] Handle edge cases where 'title' contains raw SQL reserved keywords.
- [x] Test fallback default values if 'type' (Movie/TV) is explicitly omitted.
- [x] Verify NULL handling for synopsis to prevent query failures.

**5.3 Episodes table schema (ID, media ID, season, episode num, title, runtime).** (Complete)

- [x] Ensure a composite UNIQUE constraint exists for (media ID, season, episode num).
- [x] Handle edge cases where runtime data arrives as a string instead of an integer.
- [x] Verify season and episode numbers cannot be negative integers.
- [x] Test inserting 'Specials' as season 0 without breaking numeric sorting.
- [x] Ensure cascading deletes work correctly when the parent Media row is removed.

**5.4 Local_Files relational table (1-to-1 linking of episodes to hard drive paths).** (Complete)

- [x] Verify path columns support extreme length strings (Windows maximums).
- [x] Handle edge cases where multiple files point to the exact same episode ID.
- [x] Ensure paths are updated gracefully if a user manually renames a drive letter.
- [x] Test deleting a path explicitly from this table without deleting the Episode data.
- [x] Verify file size or hash metadata fields are available for future collision checks.

**5.5 History relational table (Tracking timestamp, episode ID).** (Complete)

- [x] Ensure timestamps are uniformly stored as UTC integers (UNIX epoch).
- [x] Handle edge cases where a user marks 50 episodes watched in the exact same millisecond.
- [x] Verify that deleting an episode correctly cascades to wipe all its history entries.
- [x] Test indexing on the timestamp column to ensure timeline UI fetching is instant.
- [x] Ensure missing timestamps dynamically fall back to current time on insert.

**5.6 Unmatched_Files staging table for the Inbox.** (Complete)

- [x] Verify the string key extracted from regex is stored separately from the raw path.
- [x] Handle edge cases where a previously unmatched file is naturally moved by the OS.
- [x] Ensure clearing the Inbox strictly truncates this table safely.
- [x] Test rapid inserts when scanning 10,000 completely unrecognized anime files.
- [x] Verify path collision constraints prevent duplicating items already in Local_Files.

**5.7 release_date column for precise library sorting.** (Complete)

- [x] Ensure DATE columns are properly formatted 'YYYY-MM-DD' for SQLite sorting.
- [x] Handle edge cases where TMDB provides only 'YYYY'.
- [x] Verify sorting queries use `NULLS LAST` so unreleased shows fall to the bottom.
- [x] Test filtering by explicit year ranges using SQL `BETWEEN`.
- [x] Ensure dates in the future correctly flag the UI as 'Unaired'.

**5.8 air_date column for episode 'time capsule' comparisons.** (Complete)

- [x] Verify math operations (e.g., difference between watch date and air date) work accurately.
- [x] Handle edge cases where air date exists but time is unknown (assume midnight).
- [x] Ensure backdating logic can leverage this column for historical reconstruction.
- [x] Test updating this column gracefully without overwriting local manual changes.
- [x] Verify leap year air dates do not cause day-offset bugs.

**5.9 is_legacy boolean flag for handling archived/backdated history.** (Complete)

- [x] Ensure this flag strictly defaults to 0 (false) for real-time natural watching.
- [x] Handle edge cases where a legacy row is modified; does it stay legacy?
- [x] Verify the UI timeline parser bypasses expandable accordion logic for legacy items.
- [x] Test mass-insertion of 500 legacy rows ensuring database locks are avoided.
- [x] Ensure analytics/stats math still correctly counts legacy rows.

**5.10 user_rating integer column for 1-10 star personal scores.** (Complete)

- [x] Ensure constraints strictly restrict integers to between 0 and 10.
- [x] Handle edge cases where null represents unrated versus 0 representing terrible.
- [x] Verify 'My Top Rated' sort logic places 10-star shows at the top and 0 at bottom.
- [x] Test rapid UI clicking of stars accurately reflects the final state in the DB.
- [x] Ensure this data is entirely independent from the TMDB `vote_average`.

**5.11 vote_average float column for TMDB score caching.** (Complete)

- [x] Verify data precision does not drift (e.g., 8.5 does not become 8.5000001).
- [x] Handle edge cases where TMDB returns a 0.0 for a brand new show.
- [x] Ensure this column is forcibly overwritten every time the user clicks 'Refresh Data'.
- [x] Test formatting to strictly one decimal place when returned to the UI.
- [x] Ensure queries can sort by this value without crashing on nulls.

**5.12 last_position integer column for sub-episode pause tracking.** (Complete)

- [x] Ensure default value is strictly 0 on episode insertion.
- [x] Handle edge cases where last_position exceeds the stored total runtime.
- [x] Verify this column resets strictly to 0 if the episode is manually marked complete.
- [x] Test updating this value rapidly via the background VLC HTTP poller.
- [x] Ensure UI progress bars calculate width based on `(last_position / runtime) * 100`.

**5.13 session_id mapping column for grouping binges.** (Complete)

- [x] Ensure the session ID is a universally unique identifier (UUID) generated by Rust.
- [x] Handle edge cases where a binge stretches exactly across midnight.
- [x] Verify the 6-hour gap logic properly triggers a brand new session_id generation.
- [x] Test UI mapping that groups History rows by matching session_ids.
- [x] Ensure manually backdated rows completely ignore/bypass this column.

**5.14 Multi-table cascading deletes (Deleting Media safely wipes Episodes, Files, and History).** (Complete)

- [x] Verify `PRAGMA foreign_keys = ON;` is fired immediately on database connect.
- [x] Test explicitly deleting a Media show and querying History to ensure orphans do not exist.
- [x] Handle edge cases where deleting millions of rows locks the DB for several seconds.
- [x] Ensure Local_Files rows are wiped but the actual physical file on disk remains untouched.
- [x] Verify error handling if a constraint violation accidentally occurs.

## 🎨 Part 6: Global UI, Styling & Motion (Cinema-Grade) (Incomplete)

**6.1 Pure #0D0F14 deep cinematic background color.** (Incomplete)

- [ ] Ensure the color is uniformly applied to `<body>` to prevent white flashes on load.
- [ ] Verify scrollbar track colors match or blend seamlessly into this hex value.
- [ ] Handle edge cases where transparent images render awkwardly over this specific dark tone.
- [ ] Test contrast ratios ensuring standard silver text remains readable against it.
- [ ] Ensure fullscreen modes maintain this background color on ultra-wide monitors.

**6.2 Translucent #1F222A surface cards for depth.** (Incomplete)

- [ ] Ensure borders use a subtle lighter tone (e.g., `#2A2D35`) to define card edges.
- [ ] Verify nested cards (cards within cards) step up lightness correctly for visual hierarchy.
- [ ] Handle edge cases where surface cards overlap each other during Framer Motion transitions.
- [ ] Test hover states slightly brightening this hex value to indicate interactivity.
- [ ] Ensure loading skeletons utilize a pulsed version of this exact color.

**6.3 Vibrant #FF6B00 (VLC Orange) global accent/interaction color.** (Incomplete)

- [ ] Verify this color is strictly reserved for primary actions, not passive text.
- [ ] Ensure hover states utilize a slightly lighter variant (e.g., `#FF8533`) for feedback.
- [ ] Handle edge cases where this color is used as a drop-shadow glow effect.
- [ ] Test colorblind accessibility, ensuring text inside an orange pill remains pure `#FFFFFF`.
- [ ] Ensure progress bars use this exact color to fill their active width.

**6.4 Tailwind backdrop-blur-xl heavy Frosted Glass effects on sidebars.** (Incomplete)

- [ ] Ensure Safari and older browsers that don't support `backdrop-filter` fall back to a solid color.
- [ ] Verify performance does not tank to 15FPS when scrolling massive lists behind the blur.
- [ ] Handle edge cases where nested blur components cancel each other out.
- [ ] Test the sidebar blur over complex, highly detailed hero images.
- [ ] Ensure the blur radius is strictly consistent across the entire app shell.

**6.5 Tailwind backdrop-blur-md light glass effects on floating elements.** (Incomplete)

- [ ] Apply this specifically to hover tooltips, dropdown menus, and quick-view overlays.
- [ ] Verify z-index stacking context ensures floating glass always remains on top.
- [ ] Handle edge cases where clicking through the blurred element accidentally triggers background events.
- [ ] Test transition speeds of the blur fading in and out (should be < 200ms).
- [ ] Ensure floating glass elements cast a harsh dark drop shadow to separate from the background.

**6.6 High-contrast typography hierarchy (Pure white headers, silver body text).** (Incomplete)

- [ ] Verify `h1`, `h2`, and `h3` tags globally default to `#FFFFFF`.
- [ ] Ensure `p` and `span` tags globally default to a silver tone (e.g., `#A0AEC0`).
- [ ] Handle edge cases where text is rendered over extremely bright poster images (require shadows).
- [ ] Test font weights, strictly using `font-bold` for headers and `font-normal` for body.
- [ ] Ensure letter spacing (tracking) is slightly tightened on massive hero text.

**6.7 Modern Inter sans-serif font family integration.** (Incomplete)

- [ ] Ensure the Inter font files are locally bundled to prevent network-dependent font loading.
- [ ] Verify font anti-aliasing is explicitly enabled in CSS (`antialiased`, `subpixel-antialiased`).
- [ ] Handle edge cases where foreign languages fall back gracefully to standard system sans-serif.
- [ ] Test number rendering, ensuring tabular figures are used in data-heavy stat grids.
- [ ] Ensure variable font weights map correctly without causing layout jumps.

**6.8 Perfect rounded-xl and rounded-2xl corner radii.** (Incomplete)

- [ ] Verify primary surface cards strictly utilize `rounded-2xl`.
- [ ] Ensure smaller elements like buttons and chips strictly utilize `rounded-xl`.
- [ ] Handle edge cases where images inside a rounded container bleed out of the corners (`overflow-hidden`).
- [ ] Test nested rounded elements to ensure corner radius math looks visually parallel.
- [ ] Ensure focus rings (for keyboard navigation) precisely follow the rounded path.

**6.9 Global scrollbar hiding (scrollbar-hide) for all horizontal carousels.** (Incomplete)

- [ ] Verify horizontal scrolling is purely driven by mouse-wheel capture or trackpad swiping.
- [ ] Ensure visual affordances (cut-off images on the edge) exist since scrollbars are hidden.
- [ ] Handle edge cases where users lack a scroll wheel (provide optional left/right arrow buttons).
- [ ] Test smooth scrolling CSS properties to ensure flick-scrolling feels natural.
- [ ] Ensure hiding the scrollbar doesn't accidentally disable keyboard accessibility (arrow keys).

**6.10 Custom thin, dark styled scrollbars for vertical lists.** (Incomplete)

- [ ] Verify the scrollbar track is entirely transparent or matches `#0D0F14`.
- [ ] Ensure the scrollbar thumb uses a subtle gray that slightly brightens on hover.
- [ ] Handle cross-browser specific CSS (`::-webkit-scrollbar` vs standard `scrollbar-width`).
- [ ] Test that the scrollbar overlay doesn't shift the entire page width when it appears.
- [ ] Ensure standard scrollbar width is incredibly thin (e.g., `4px` or `6px`).

**6.11 Lucide-React high-fidelity vector icon integration.** (Incomplete)

- [ ] Verify all icons scale perfectly without pixelation regardless of size.
- [ ] Ensure `strokeWidth` is globally consistent across all imported icons.
- [ ] Handle edge cases where icons require filling (like the 5-star rating stars).
- [ ] Test icon rendering overhead when rendering grids containing hundreds of them.
- [ ] Ensure screen readers correctly ignore icons flagged with `aria-hidden`.

**6.12 Advanced linear gradient fades over all background images for text legibility.** (Incomplete)

- [ ] Verify gradients fade strictly from pure dark at the text origin to transparent at the focal point.
- [ ] Handle edge cases where an image is entirely white, ensuring the gradient is heavy enough.
- [ ] Test resizing the window to ensure the gradient dynamically covers the correct percentage.
- [ ] Ensure gradients do not trigger banding artifacts on low-quality displays.
- [ ] Verify bottom-to-top gradients exist on all poster cards to make titles readable.

**6.13 'No-Jump' state management (React updates specific DOM nodes without page reloads).** (Incomplete)

- [ ] Verify that clicking 'Mark Watched' instantly morphs the icon without shifting the layout.
- [ ] Ensure React strictly uses stable `key` props on list items to prevent full DOM recreation.
- [ ] Handle edge cases where multiple state changes happen in the exact same millisecond.
- [ ] Test scroll positions remain completely untouched when background data refreshes.
- [ ] Ensure API syncs update the UI precisely in-place without causing white screen flashes.

**6.14 Pulse/Skeleton animated loading states during API data fetching.** (Incomplete)

- [ ] Verify skeletons precisely match the dimensions of the final loaded component.
- [ ] Ensure the CSS pulse animation is smooth and synchronized across all active skeletons.
- [ ] Handle edge cases where the API returns instantly, bypassing the skeleton to prevent a flicker.
- [ ] Test skeleton rendering when navigating explicitly to a deep-linked URL.
- [ ] Ensure skeletons utilize the `#1F222A` surface color to blend perfectly with the theme.

**6.15 Custom global 'Turbo-Scroll' implementation (4x-5x mouse wheel multiplier).** (Incomplete)

- [ ] Verify the multiplier is explicitly disabled on components that require precise scrolling (like date pickers).
- [ ] Ensure trackpad pinch-to-zoom gestures are not accidentally multiplied and broken.
- [ ] Handle edge cases where custom scrolling completely breaks on Linux Wayland environments.
- [ ] Test scrolling performance on massive lists (10,000+ items) with the multiplier active.
- [ ] Ensure the implementation doesn't interfere with Framer Motion scroll-linked animations.

## 🧭 Part 7: The Application Shell & Navigation (Incomplete)

**7.1 Fixed, full-height frosted glass Sidebar (w-64).** (Incomplete)

- [ ] Ensure the sidebar width is exactly 16rem/256px consistently across views.
- [ ] Verify the sidebar overlays or displaces the main content area correctly on resize.
- [ ] Handle edge cases on narrow window widths where a hamburger menu might be preferred.
- [ ] Test the `backdrop-blur-xl` on the sidebar over moving or heavily colored video elements.
- [ ] Ensure scroll bars inside the sidebar are strictly hidden unless hovering.

**7.2 Glowing orange vertical line indicator for the 'Active Tab'.** (Incomplete)

- [ ] Ensure the line uses a neon glow effect (e.g., `box-shadow` or Framer Motion aura).
- [ ] Verify it precisely animates its `y` position between tabs when clicked.
- [ ] Handle edge cases where no specific tab is logically active (e.g., Settings page).
- [ ] Test that the line thickness (e.g., `w-1`) perfectly aligns to the absolute left edge.
- [ ] Ensure the indicator is strictly `#FF6B00`.

**7.3 Text brightening (silver to pure white) for active tabs.** (Incomplete)

- [ ] Verify inactive tabs use `#A0AEC0` (silver) and active use `#FFFFFF`.
- [ ] Ensure the transition time is smooth (e.g., `duration-200`).
- [ ] Handle edge cases where an active tab is hovered again (should remain white).
- [ ] Test keyboard focus also triggers the brighten effect for accessibility.
- [ ] Ensure icons next to the text also simultaneously brighten to pure white.

**7.4 Hover transitions on sidebar links (color shifts, no boxy backgrounds).** (Incomplete)

- [ ] Ensure hover effects explicitly avoid solid rectangular backgrounds behind text.
- [ ] Verify the text subtly brightens or shifts on hover without triggering the full 'active' state.
- [ ] Handle edge cases where rapid movement up and down the sidebar flickers the UI.
- [ ] Test specific Framer Motion scale effects (e.g., `scale-105`) strictly on the text/icon.
- [ ] Ensure the padding is large enough to create an easy click target.

**7.5 Unified sidebar Logo featuring a stylized Play icon.** (Incomplete)

- [ ] Ensure the logo SVG is razor sharp at all resolutions and precisely centered in the header.
- [ ] Verify the stylized Play icon seamlessly incorporates the primary `#FF6B00` orange.
- [ ] Handle edge cases where the app is resized vertically causing the logo to overlap content.
- [ ] Test clicking the logo reliably routes the user back to the Dashboard.
- [ ] Ensure the logo is visually isolated with proper margins from the top edge.

**7.6 Top global navigation bar (sticky top-0).** (Incomplete)

- [ ] Ensure the bar is exactly 64px in height across the entire app.
- [ ] Verify its `z-index` strictly places it above all scrolling content but below modals.
- [ ] Handle edge cases where content bleeds through the edges of the sticky header.
- [ ] Test that the `<- Back` button perfectly aligns on the left axis inside this bar.
- [ ] Ensure the user profile/settings icon aligns perfectly on the right axis.

**7.7 Top bar background transition (transparent to blurred when scrolled).** (Incomplete)

- [ ] Verify the header is completely transparent when `scrollY === 0` over hero images.
- [ ] Ensure a heavy `backdrop-blur` and a semi-transparent `#1F222A` background fades in immediately when scrolling starts.
- [ ] Handle edge cases where rapid scrolling causes the header to flicker between states.
- [ ] Test the transition duration ensuring the fade is elegant and not abrupt.
- [ ] Ensure elements strictly behind the header do not suddenly snap or shift.

**7.8 Persistent Global Quick Search input pill centered in the top bar.** (Incomplete)

- [ ] Ensure the input field is shaped exactly as a pill (`rounded-full`).
- [ ] Verify its background is a dark translucent tone (`bg-black/20`).
- [ ] Handle edge cases where the user types an impossibly long string.
- [ ] Test a hotkey (like `Ctrl+K`) perfectly focusing the input instantly.
- [ ] Ensure an empty state perfectly centers a magnifying glass icon and placeholder text.

**7.9 Smooth Framer Motion <AnimatePresence> page cross-fades.** (Incomplete)

- [ ] Verify entering components fade in (`opacity: 1`) while exiting components fade out (`opacity: 0`) simultaneously.
- [ ] Ensure the duration is swift enough (< 0.3s) to feel instantly responsive.
- [ ] Handle edge cases where rapid navigation clicks queue up multiple fading animations.
- [ ] Test nested routes ensuring they do not completely fade out the parent layout shell.
- [ ] Ensure scroll positions reset to the top precisely when the new page fades in.

## 🏠 Part 8: Dashboard (Home Screen) (Incomplete)

**8.1 Edge-to-edge relative Hero Banner container.** (Incomplete)

- [ ] Ensure the banner spans the absolute maximum width of the routing layout container.
- [ ] Verify the height is locked to a specific dramatic ratio (e.g., `450px` or `50vh`).
- [ ] Handle edge cases where the browser window is extremely wide (ensure image covers via `object-cover`).
- [ ] Test the top edge seamlessly bleeds underneath the transparent Top Navigation bar.
- [ ] Ensure the container strictly crops and hides any image overflow.

**8.2 Custom Hero directional gradient (Solid Black Bottom-Left -> Transparent Top-Right).** (Incomplete)

- [ ] Verify the gradient originates strictly from the bottom-left corner.
- [ ] Ensure the starting color matches the `body` background `#0D0F14` entirely to anchor the text.
- [ ] Handle edge cases where bright images wash out the gradient (may require a secondary overlay).
- [ ] Test the diagonal angle (e.g., `to-tr`) to ensure the center focal point remains visible.
- [ ] Ensure text nested inside the bottom-left gradient region is perfectly legible.

**8.3 Smart Hero detection logic (Automatically surfaces the most recently watched, unfinished show).** (Incomplete)

- [ ] Verify the backend query strictly targets `status = 'Watching'` and sorts by the most recent `History` timestamp.
- [ ] Ensure it precisely calculates the lowest SxxExx episode that remains strictly 'Unwatched'.
- [ ] Handle edge cases where all active shows are 100% finished (should fallback to a newly added show).
- [ ] Test behavior if the database is completely empty (should display a welcoming fallback image).
- [ ] Ensure the surfaced show dynamically updates the very second playback completes.

**8.4 Dynamic Hero Status tag ('RESUME SESSION' if partially watched, 'UP NEXT' if new).** (Incomplete)

- [ ] Verify 'RESUME SESSION' strictly appears if the specific episode's `last_position` is > 0.
- [ ] Ensure 'UP NEXT' strictly appears if `last_position` is exactly 0 and it follows a completed episode.
- [ ] Handle edge cases where it's the very first episode of a brand new show (e.g., 'START SERIES').
- [ ] Test the UI tag uses the `#FF6B00` color consistently to grab attention.
- [ ] Ensure the text is fully capitalized and uses tight tracking.

**8.5 Massive 48pt bold Hero title rendering.** (Incomplete)

- [ ] Ensure the font size strictly scales or wraps elegantly on smaller window widths.
- [ ] Verify standard truncation or CSS clamping (e.g., `line-clamp-2`) prevents infinite vertical expansion.
- [ ] Handle edge cases where titles have incredibly long single words (e.g., German titles).
- [ ] Test text shadow properties ensure readability regardless of the background.
- [ ] Ensure the title accurately pulls from the Show, not the specific Episode.

**8.6 Hero Season/Episode specific subtitle formatting.** (Incomplete)

- [ ] Ensure strings perfectly match standard format: 'Season X • Episode Y'.
- [ ] Verify the episode's specific title is appended dynamically (e.g., '• Pilot').
- [ ] Handle edge cases where the episode is a 'Special' (Season 0).
- [ ] Test missing episode titles default securely to 'Episode Y' without throwing undefined.
- [ ] Ensure font color strictly uses silver (`#A0AEC0`) for this sub-text.

**8.7 Prominent Hero '▶ Resume' action button.** (Incomplete)

- [ ] Ensure the button features a distinct orange background and bold text.
- [ ] Verify clicking instantly triggers the Rust VLC spawn command with the correct local path.
- [ ] Handle edge cases where the file was deleted (button should instantly disable or grey out).
- [ ] Test hover scaling animations ensuring the button feels highly tactile.
- [ ] Ensure the button provides clear visual feedback (pulse/spinner) if VLC takes a second to boot.

**8.8 Hero precise progress bar rendering exact minutes remaining.** (Incomplete)

- [ ] Verify mathematical logic accurately subtracts `last_position` from `total_runtime`.
- [ ] Ensure the string formats cleanly (e.g., '22m remaining' instead of '1320s remaining').
- [ ] Handle edge cases where `last_position` is somehow larger than the runtime (display 0m).
- [ ] Test the visual bar width perfectly maps to the calculated percentage.
- [ ] Ensure the progress bar strictly anchors to the bottom of the action button row.

**8.9 Fallback Unsplash image logic if TMDB backdrop is missing.** (Incomplete)

- [ ] Verify the logic accurately fetches a dark, generic cinematic texture from an API or local asset.
- [ ] Ensure the fallback strictly avoids returning completely black or unstyled squares.
- [ ] Handle edge cases where the network is completely down (load a pure local gradient).
- [ ] Test UI cross-fades strictly occur after the fallback has fully finished loading.
- [ ] Ensure the fallback image still properly applies the directional gradient.

**8.10 'Continue Watching' horizontal smart queue section.** (Incomplete)

- [ ] Verify the horizontal row utilizes hidden scrollbars and enables swiping.
- [ ] Ensure the section strictly filters out shows that are 100% completed or completely unwatched.
- [ ] Handle edge cases where only 1 show is active (should center or pad the card beautifully).
- [ ] Test horizontal mouse-wheel capture accurately converts vertical scroll inputs into a horizontal slide.
- [ ] Ensure empty states completely hide this section from the Dashboard entirely.

**8.11 Dynamic generation of 'Next unwatched episode' cards for all active shows.** (Incomplete)

- [ ] Verify database queries correctly calculate the exact SxxExx needed per active show.
- [ ] Ensure UI strictly fetches the 16:9 still image specific to that exact unwatched episode.
- [ ] Handle edge cases where multiple shows are binge-watched simultaneously (sort by last activity).
- [ ] Test the title overlay perfectly combines Show Name + 'S1:E2'.
- [ ] Ensure clicking the card directly launches VLC without navigating to the details page.

**8.12 16:9 episode-specific still image cards in Continue Watching.** (Incomplete)

- [ ] Ensure the CSS aspect ratio strictly forces a `16/9` box regardless of the raw asset size.
- [ ] Verify fallback logic correctly blurs the show backdrop if the specific episode still is missing.
- [ ] Handle edge cases where TMDB returned a 4:3 image, ensuring it is strictly centered and cropped.
- [ ] Test image caching ensures the carousel loads instantly without network flicker.
- [ ] Ensure the cards feature the standard `#1F222A` backing to prevent transparent holes.

**8.13 Bottom-edge progress line on Continue Watching cards.** (Incomplete)

- [ ] Verify the line height is extremely thin (e.g., `h-1` or `h-2`).
- [ ] Ensure it strictly uses `#FF6B00` to indicate partially watched progress.
- [ ] Handle edge cases where the episode is 0% watched (should show a pure gray track).
- [ ] Test the absolute positioning specifically anchors it flush to the bottom corners.
- [ ] Ensure rounded card corners cleanly clip the sharp edges of the progress bar.

**8.14 'Recently Added' horizontal carousel highlighting new local scans.** (Incomplete)

- [ ] Verify it sorts strictly by the local `id` descending (most recent scans).
- [ ] Ensure it renders standard 2:3 posters, distinct from the 16:9 Continue Watching cards.
- [ ] Handle edge cases where newly added items are instantly completed by the user.
- [ ] Test the carousel limit (e.g., strictly max out at 20 items to prevent infinite scrolling).
- [ ] Ensure hover overlays correctly trigger the orange Play button over the poster.

**8.15 Global Personal Stats grid layout containing 3 glassy widgets.** (Incomplete)

- [ ] Verify the grid uses exactly 3 uniform columns.
- [ ] Ensure the background strictly uses a lighter glass effect (`bg-[#1F222A]/50`) to stand out.
- [ ] Handle edge cases on narrow mobile-like widths where the grid must collapse to 1 column.
- [ ] Test subtle padding (`p-6`) centers the numbers and text comfortably.
- [ ] Ensure the values dynamically count up or animate via Framer Motion on initial load.

**8.16 Real-time 'Total Episodes Watched' mathematical calculation.** (Incomplete)

- [ ] Verify the query accurately counts all valid `History` rows.
- [ ] Ensure it explicitly ignores 'Archive' rows completely to preserve true history tracking.
- [ ] Handle edge cases where a user clears their history (should safely drop to 0).
- [ ] Test integer formatting precisely handles massive numbers (e.g., '1,234' vs '1234').
- [ ] Ensure real-time sync immediately updates this number the second an episode finishes.

**8.17 Real-time 'Hours Watched' calculation (Summing completed episode runtimes).** (Incomplete)

- [ ] Verify the mathematical sum grabs exact seconds and reliably divides them.
- [ ] Ensure the display cleanly reads 'Hours' without massive decimal trails (e.g., '245 Hrs').
- [ ] Handle edge cases where TMDB runtime data is null across hundreds of episodes (default safely to 0 or an average).
- [ ] Test the math strictly only sums episodes formally flagged as 'Completed'.
- [ ] Ensure rounding favors the nearest whole integer.

**8.18 Real-time 'Shows Completed' mathematical calculation.** (Incomplete)

- [ ] Verify logic strictly evaluates series where the number of local 'Completed' episodes exactly equals `total_episodes`.
- [ ] Ensure shows flagged as 'Canceled' still count as completed if the user watched the final aired episode.
- [ ] Handle edge cases where TMDB dynamically increases `total_episodes` (the status should automatically revert).
- [ ] Test UI display securely formatting the integer.
- [ ] Ensure accuracy regardless of whether the episodes were watched naturally or bulk backdated.

## 📚 Part 9: Library Views (TV Shows & Movies) (Incomplete)

**9.1 Highly responsive, auto-wrapping poster grid layout.** (Incomplete)

- [ ] Verify CSS Grid `repeat(auto-fill, minmax(180px, 1fr))` naturally expands across monitors.
- [ ] Ensure the gap between posters is consistently applied (e.g., `gap-6` or `gap-8`).
- [ ] Handle edge cases on ultra-wide 4K monitors (e.g., limit max columns or grid width to prevent absurdity).
- [ ] Test the rendering performance of 5,000 grids loaded instantly via virtual mapping.
- [ ] Ensure vertical layout shift is completely absent when posters lazy-load.

**9.2 Strict 2:3 cinematic poster aspect ratio enforcement.** (Incomplete)

- [ ] Verify `aspect-[2/3]` strictly forces the container height relative to the computed width.
- [ ] Ensure images strictly use `object-cover` so mismatched posters never stretch horizontally.
- [ ] Handle edge cases where a poster completely fails to load (display the show title centered on a dark box).
- [ ] Test rounded corners are strictly preserved without being overridden by the child image.
- [ ] Ensure the ratio holds firm during browser resize events.

**9.3 Framer Motion whileHover={{ scale: 1.05 }} smooth expansion animation.** (Incomplete)

- [ ] Verify the expansion scales from the exact center of the poster without moving adjacent elements.
- [ ] Ensure standard CSS `transition-transform` is utilized if Framer Motion becomes too heavy for massive grids.
- [ ] Handle edge cases where rapid zig-zag mouse movements queue up stuttering scale animations.
- [ ] Test `z-index` strictly pulls the hovered poster above adjacent neighbors.
- [ ] Ensure the animation is exactly fast enough (e.g., `0.2s`) to feel snappy.

**9.4 Dynamic drop-shadow casting intensity increase on hover.** (Incomplete)

- [ ] Verify the shadow smoothly shifts from a low baseline (e.g., `shadow-md`) to massive intensity (`shadow-2xl`).
- [ ] Ensure the shadow color is customized to match the `#0D0F14` background cleanly.
- [ ] Handle edge cases where shadows might be clipped by a parent `overflow-hidden` rule.
- [ ] Test transition smoothness occurring simultaneously with the `scale` animation.
- [ ] Ensure the shadow correctly drops down and slightly outwards to simulate floating.

**9.5 Always-on Top-Right Star Rating pill (Conditionally rendered if rated > 0).** (Incomplete)

- [ ] Verify the pill is absolutely positioned specifically in the top right corner (`top-2 right-2`).
- [ ] Ensure it correctly evaluates the `user_rating` column, completely ignoring the TMDB `vote_average`.
- [ ] Handle edge cases where the value is 0 or null (the pill must completely vanish).
- [ ] Test the text formatting strictly outputs a string like '★ 4' or '★ 5/5'.
- [ ] Ensure the background is a translucent black to remain visible on bright white posters.

**9.6 Always-on Bottom-Left Original Release Year pill.** (Incomplete)

- [ ] Verify the pill accurately extracts purely the 4-digit Year string from `release_date`.
- [ ] Ensure the styling utilizes a completely solid dark badge to ground the poster base.
- [ ] Handle edge cases where the year is entirely unknown or null (do not display 'NaN' or '1970').
- [ ] Test exact absolute positioning to avoid overlapping with bottom progress bars.
- [ ] Ensure font size is tiny but highly legible (e.g., `text-xs`).

**9.7 Absolute bottom-edge library progress bar (Gray = Unwatched, Half Orange = Watching, Green = Finished).** (Incomplete)

- [ ] Verify mathematical logic perfectly calculates width percentage based on `completed_episodes / total_episodes`.
- [ ] Ensure the width transitions correctly (Gray for 0%, Orange for 1-99%, Green strictly for 100%).
- [ ] Handle edge cases where the show is 'Ended' but only 3 episodes ever aired.
- [ ] Test absolute positioning locking the bar to the lowest possible pixel of the poster.
- [ ] Ensure corner radius rounding neatly clips the bar without jutting out as a square.

**9.8 'Hide Completed' state toggle switch.** (Incomplete)

- [ ] Verify the React state correctly tracks a boolean passed to the backend query.
- [ ] Ensure the visual toggle button explicitly indicates its active state (e.g., turning orange).
- [ ] Handle edge cases where toggling leaves the entire library empty (should display friendly text).
- [ ] Test Framer Motion `layout` ensuring posters seamlessly rearrange without jarring jumps.
- [ ] Ensure the toggle preference is strictly saved to the persistent Settings JSON.

**9.9 Complex SQL Sub-query logic to filter out 100% watched series dynamically.** (Incomplete)

- [ ] Verify the Rust query accurately identifies 100% via `COUNT` and dynamically skips rows without returning false positives.
- [ ] Ensure this query is highly optimized (utilizing indexes) so the library doesn't lag on massive databases.
- [ ] Handle edge cases where a show has 0 total episodes (prevent divide by zero skipping).
- [ ] Test behavior if a new episode magically drops, immediately unhiding the previously completed show.
- [ ] Ensure pagination correctly adapts to the dynamically reduced result set.

**9.10 Dedicated Library Sort Dropdown component.** (Incomplete)

- [ ] Verify the dropdown accurately overlays content without shifting the page layout.
- [ ] Ensure it provides clear, human-readable options (e.g., 'Alphabetical A-Z').
- [ ] Handle edge cases where clicking entirely outside the dropdown instantly closes it.
- [ ] Test active state styling distinctly highlighting the currently selected sorting method.
- [ ] Ensure the list strictly updates instantly when a new method is clicked.

**9.11 Sort logic: 'Recently Added' (ID DESC).** (Incomplete)

- [ ] Verify `ID DESC` reliably places the newest exact scans at the absolute top of the grid.
- [ ] Ensure subsequent identical IDs do not break the deterministic order.
- [ ] Handle edge cases where multiple files were added in the exact same millisecond.
- [ ] Test integration with 'Hide Completed' (ensuring the sort purely acts on the remaining items).
- [ ] Ensure this acts as the absolute default sorting method globally.

**9.12 Sort logic: 'Alphabetical' (A-Z String matching).** (Incomplete)

- [ ] Verify SQL sorting is explicitly case-insensitive (treating 'A' and 'a' equally).
- [ ] Ensure articles like 'The', 'A', and 'An' at the start of strings are explicitly ignored in sorting.
- [ ] Handle edge cases with special characters (e.g., '!', '@') sorting appropriately.
- [ ] Test performance when sorting a local library of 10,000+ items entirely by string.
- [ ] Ensure numbers sort logically (e.g., '24' falls correctly before '300').

**9.13 Sort logic: 'Release Year' (release_date DESC, handling NULLs by placing them last).** (Incomplete)

- [ ] Verify `NULLS LAST` specifically works in the SQLite syntax to prevent TBD shows from ruining the top.
- [ ] Ensure older shows are correctly forced to the bottom in standard descending mode.
- [ ] Handle edge cases where shows released in the exact same year sort alphabetically via a secondary constraint.
- [ ] Test behavior with negative dates if legacy formats somehow trigger epoch bounds.
- [ ] Ensure month and day are completely ignored if only Year sorting is intended.

**9.14 Sort logic: 'My Top Rated' (user_rating DESC).** (Incomplete)

- [ ] Verify shows strictly grouped by their exact integer (e.g., all 5-stars together).
- [ ] Ensure the secondary constraint automatically sorts the grouped items alphabetically or by ID.
- [ ] Handle edge cases where 0-star (unrated) shows are strictly relegated to the absolute bottom.
- [ ] Test rapid UI updates ensuring a newly rated show leaps to the top instantly.
- [ ] Ensure TMDB `vote_average` is absolutely ignored.

**9.15 Smooth list re-ordering transitions when sorting/filtering.** (Incomplete)

- [ ] Verify `<motion.li layout>` seamlessly animates the precise XY translation of each poster.
- [ ] Ensure the DOM correctly unmounts filtered items strictly using `<AnimatePresence>` opacity fades.
- [ ] Handle edge cases where extreme sorting triggers 500 animations simultaneously (must not drop below 30FPS).
- [ ] Test visual tracking ensuring the user's eye can follow a poster if it moves slightly.
- [ ] Ensure the animation duration is rapid (e.g., `0.3s`).

**9.16 Friendly empty-state illustration/text for libraries with 0 items.** (Incomplete)

- [ ] Verify the exact text encourages the user to 'Scan for Files' or add media.
- [ ] Ensure the component is perfectly horizontally and vertically centered.
- [ ] Handle edge cases where a library is empty purely due to aggressive filtering, not a zero-item DB.
- [ ] Test an appropriate massive, dim Lucide-React icon renders beautifully in the center.
- [ ] Ensure the 'Hide Completed' toggle explicitly vanishes to avoid UI clutter in this state.

## ✨ Part 10: "Quick-View" Interactive Hover Overlay (Posters) (Incomplete)

**10.1 Instant overlay state toggle utilizing parent container onMouseEnter/Leave boundaries.** (Incomplete)

- [ ] Ensure state flipping strictly avoids flickering or bouncing by binding events exactly to the outer poster edge.
- [ ] Verify child elements (`pointer-events-none`) do not trigger premature `onMouseLeave` resets.
- [ ] Handle edge cases where rapid scrolling through rows forces stuck hover states.
- [ ] Test React timeout logic (e.g., `100ms` delay) ensuring intentional hovering prevents accidental popups.
- [ ] Ensure touch devices safely ignore this overlay entirely or bind it to a single tap.

**10.2 #141519 heavy dimming overlay with specific padding insets to preserve poster borders.** (Incomplete)

- [ ] Verify the exact hex color uses a solid overlay with an opacity of strictly `80-90%`.
- [ ] Ensure absolute positioning uses `inset-x-2 inset-y-2` (or similar) so the original poster art forms a perfect framing border.
- [ ] Handle edge cases where long titles accidentally bleed outside the inset box.
- [ ] Test transition durations ensuring the background strictly fades in without snapping.
- [ ] Ensure corner rounding matches the inset, preventing sharp overflow on rounded borders.

**10.3 Bold Title rendering on overlay.** (Incomplete)

- [ ] Verify `text-center` strictly forces the title to align horizontally inside the inset.
- [ ] Ensure `text-white font-bold` renders distinctly over the dimming layer.
- [ ] Handle edge cases where a 5-word movie title strictly line-wraps without overlapping other metrics.
- [ ] Test line clamping (e.g., `line-clamp-2`) truncating any absurdly long text with an ellipsis.
- [ ] Ensure the padding at the top provides ample breathing room.

**10.4 Dynamic Year Range calculation (Extracting Min and Max episode air dates, e.g., '2011 - 2019').** (Incomplete)

- [ ] Verify Rust or SQL accurately queries the exact mathematical `MIN(air_date)` and `MAX(air_date)`.
- [ ] Ensure strings are stripped cleanly to only four characters ('YYYY').
- [ ] Handle edge cases where the show premiered and ended in the exact same year (display just '2015').
- [ ] Test behavior where unaired future episodes artificially extend the range to 2026 erroneously.
- [ ] Ensure null air dates correctly fall back strictly to the primary show's `release_date`.

**10.5 Series Status tag conditional rendering (e.g., 'Ended', 'Returning').** (Incomplete)

- [ ] Verify strings exactly match the standardized TMDB dataset.
- [ ] Ensure it strictly renders only on TV Shows, not standard Movies.
- [ ] Handle edge cases where a status is officially 'Canceled' vs 'Ended' (color-code red if preferred).
- [ ] Test font sizes rendering extremely small but distinctly readable.
- [ ] Ensure a bold dot (`•`) smoothly separates the Year Range and the Status.

**10.6 'Watched X / Y Eps' real-time math string generation.** (Incomplete)

- [ ] Verify the total specifically reflects the global episode count fetched from the DB.
- [ ] Ensure X accurately sums all episodes completely watched by the user.
- [ ] Handle edge cases where the user watched 0 out of 500 (display '0 / 500 Eps').
- [ ] Test conditional coloring ensuring '10 / 10 Eps' strictly renders in green to match the UI language.
- [ ] Ensure rapid clicks in other windows automatically refresh this overlaid math.

**10.7 Advanced Synopsis truncation (Limits to 2 lines / 100 chars with an ellipsis).** (Incomplete)

- [ ] Verify `line-clamp-2` or `line-clamp-3` consistently limits paragraph height regardless of font size.
- [ ] Ensure the text color is specifically silver (`text-gray-300`) to remain secondary to the title.
- [ ] Handle edge cases where a synopsis completely lacks spaces, breaking CSS word-wrap.
- [ ] Test text centering ensuring it feels visually anchored and balanced.
- [ ] Ensure missing synopses completely skip rendering to avoid awkward blank gaps.

**10.8 'Last Viewed: X days ago' human-readable time-ago calculation.** (Incomplete)

- [ ] Verify the time math strictly compares the most recent `History` timestamp against the current OS clock.
- [ ] Ensure formatting cleanly steps through units (e.g., 'Just now', '2 hrs ago', '5 days ago', '1 yr ago').
- [ ] Handle edge cases where a user completely lacks History for the show (render 'Never Viewed').
- [ ] Test negative time loops if OS clock synchronization goes wrong (fallback strictly to 'Recently').
- [ ] Ensure it renders anchored specifically to the bottom of the overlay inset.

**10.9 Centralized, large Orange Play Button fade-in appearance.** (Incomplete)

- [ ] Verify the Play button strictly anchors exactly to the true horizontal and vertical center of the poster.
- [ ] Ensure `text-[#FF6B00]` and a slight glowing shadow make it impossible to miss.
- [ ] Handle edge cases where clicking strictly targets the exact next unwatched episode.
- [ ] Test scale effects (e.g., `hover:scale-110`) explicitly on the icon itself.
- [ ] Ensure disabling the button explicitly changes the color to dim gray if zero local files exist.

## 🔍 Part 11: Media Details View (The Deep Dive) (Incomplete)

**11.1 Custom <- Back button overlaying the header.** (Incomplete)

- [ ] Verify the button strictly floats in the top-left corner using absolute or sticky positioning.
- [ ] Ensure it utilizes a translucent, dark pill shape specifically contrasting the background banner.
- [ ] Handle edge cases where rapid clicking breaks the history stack in React Router.
- [ ] Test keyboard accessibility specifically focusing this button upon load.
- [ ] Ensure hovering slightly brightens the pill and scales the arrow.

**11.2 Massive 400px height cinematic backdrop banner.** (Incomplete)

- [ ] Verify the precise height (`h-96` or similar) perfectly anchors the top third of the page.
- [ ] Ensure `object-cover` strictly prevents stretching of low-resolution horizontal images.
- [ ] Handle edge cases where no backdrop exists (use a massive CSS linear gradient strictly matching the theme).
- [ ] Test the heavy gradient overlay seamlessly fading the bottom edge into pure `#0D0F14`.
- [ ] Ensure rapid image switching cross-fades gracefully instead of flashing white.

**11.3 Overlapping, left-aligned vertical main poster for depth.** (Incomplete)

- [ ] Verify negative top margins (`-mt-24` or similar) pull the poster strictly upward over the backdrop edge.
- [ ] Ensure a thick, harsh drop-shadow separates the poster drastically from the image behind it.
- [ ] Handle edge cases where the main poster image completely fails to load (display the generic dark block).
- [ ] Test width constraints to ensure standard 2:3 scaling strictly matches Library views.
- [ ] Ensure alignment correctly positions the poster adjacent to the primary title block.

**11.4 Dual Rating Display panel (TMDB Score vs. User Score).** (Incomplete)

- [ ] Verify the layout utilizes two cleanly separated blocks side-by-side or stacked vertically.
- [ ] Ensure the TMDB score prominently features a gold star and formats specifically to one decimal place.
- [ ] Handle edge cases where one score is completely absent without breaking grid alignment.
- [ ] Test UI distinction clearly identifying 'Global Rating' vs 'My Rating'.
- [ ] Ensure font sizes are extremely legible to quickly glance at numbers.

**11.5 Interactive 5-star clicking component for user ratings.** (Incomplete)

- [ ] Verify hover states exactly fill the hovered star and strictly all stars preceding it.
- [ ] Ensure clicking instantly triggers the queue to push a UI update into SQLite.
- [ ] Handle edge cases where a user rapid-clicks stars (debounce logic must prevent database locking).
- [ ] Test zero-rating functionality (e.g., clicking a filled star again completely unrates it).
- [ ] Ensure stars precisely utilize the `text-[#FF6B00]` color when active.

**11.6 Instant UI star color mutation on click (Pushes DB update via queue).** (Incomplete)

- [ ] Verify React state mutates the local array instantly without waiting for a Rust callback.
- [ ] Ensure the queue strictly processes the `UPDATE media SET user_rating` via a dedicated background thread.
- [ ] Handle edge cases where the SQLite queue completely crashes, seamlessly reverting the UI state.
- [ ] Test performance when rating 20 shows in 20 seconds.
- [ ] Ensure no full-page re-renders are triggered by this specific action.

**11.7 Overall show progress badge (Dynamic text and color: Orange/Green).** (Incomplete)

- [ ] Verify mathematical tracking dynamically identifies 'Unwatched' (Gray), 'Watching' (Orange), and 'Completed' (Green).
- [ ] Ensure the badge prominently renders directly adjacent to the main Title header.
- [ ] Handle edge cases where mathematical drift breaks the 100% logic (strictly check exact episode match).
- [ ] Test 'Archived' functionality automatically flipping this badge completely to Green.
- [ ] Ensure font tracking is extremely dense (e.g., `tracking-wide uppercase text-sm`).

**11.8 '▶ Play Next' global action button (Automatically finds the lowest SxxExx unwatched file).** (Incomplete)

- [ ] Verify Rust querying exactly mimics the complex algorithm from the Dashboard Hero.
- [ ] Ensure it completely disables if no local files exist for unwatched episodes.
- [ ] Handle edge cases where episode 3 is missing, but episode 4 exists (it should strictly play episode 4).
- [ ] Test the exact `last_position` parameter properly routing to VLC.
- [ ] Ensure the button utilizes the solid vibrant Orange fill.

**11.9 '✓ Mark All Watched' global bulk action button.** (Incomplete)

- [ ] Verify clicking opens a secondary warning modal rather than instantly destroying the database history.
- [ ] Ensure it triggers the specific `Backdate` popup rather than a blind boolean toggle.
- [ ] Handle edge cases where clicking it disables completely if the series is 100% watched.
- [ ] Test button styling ensuring it uses a secondary dark-glass tone.
- [ ] Ensure it mathematically updates all progress bars and statistics globally across the app.

**11.10 '🔄 Refresh Data' TMDB sync button.** (Incomplete)

- [ ] Verify clicking fires the asynchronous sync command directly to Rust.
- [ ] Ensure an endless spinner exactly replaces the icon while the background request runs.
- [ ] Handle edge cases where the TMDB API entirely times out (stop spinner, display red toast).
- [ ] Test rapid repeated clicks (strictly disable the button during active syncs).
- [ ] Ensure updated episode data instantly paints to the DOM on completion.

**11.11 In-place data refresh logic (Updates text/images without resetting scrollbar).** (Incomplete)

- [ ] Verify React specifically updates the mounted state `media` object rather than unmounting the entire component.
- [ ] Ensure vertical scrolling remains completely locked to its current exact pixel position.
- [ ] Handle edge cases where a refreshed image drastically changes height, slightly shifting layout.
- [ ] Test missing fields repopulating elegantly without flickering the surrounding UI.
- [ ] Ensure Framer Motion smoothly scales in any newly added Season Tabs.

**11.12 Destructive '🗑️ Remove Show' text-button.** (Incomplete)

- [ ] Verify styling uses extremely subtle text (e.g., small, silver) rather than a bright primary button to avoid accidental clicks.
- [ ] Ensure hover state gently shifts specifically to a red warning hue.
- [ ] Handle edge cases where the show is currently actively playing in VLC (the command must not fail).
- [ ] Test it strictly triggers the cascading SQLite delete without leaving orphaned files.
- [ ] Ensure the UI instantly routes back directly to the Library list upon deletion.

**11.13 Cascading red warning confirmation modal for Remove Show.** (Incomplete)

- [ ] Verify a heavy z-index dark blur immediately overtakes the entire screen to block all other actions.
- [ ] Ensure the bold red button strictly requires intentional confirmation.
- [ ] Handle edge cases where clicking the dark backdrop outside the modal safely cancels the action.
- [ ] Test text clearly explaining 'This will permanently remove history and tracking, but local files will NOT be deleted'.
- [ ] Ensure focus is automatically trapped inside the modal for keyboard safety.

**11.14 Horizontal scrolling pill-style Season Tabs.** (Incomplete)

- [ ] Verify `flex flex-row overflow-x-auto overflow-y-hidden whitespace-nowrap` perfectly layouts a single smooth line.
- [ ] Ensure hidden scrollbars (via specific plugins) completely mask the ugly browser defaults.
- [ ] Handle edge cases where a show has 45 seasons, ensuring smooth mouse wheel side-scrolling.
- [ ] Test tab clicking cleanly shifting the active state specifically without shifting adjacent tabs.
- [ ] Ensure padding perfectly aligns the first tab specifically with the episode list beneath it.

**11.15 Active Season State logic (White text, dark background transition).** (Incomplete)

- [ ] Verify inactive tabs use completely transparent backgrounds with silver text.
- [ ] Ensure active tabs use specifically `#1F222A` backing with pure white text.
- [ ] Handle edge cases where clicking an identical tab ignores re-rendering.
- [ ] Test layout shifting when swapping entirely to a season with zero episodes.
- [ ] Ensure default behavior strictly selects 'Season 1' or the earliest unwatched season automatically on page load.

**11.16 Empty state handling if TMDB fails to return episode data.** (Incomplete)

- [ ] Verify a massive, friendly 'No episodes found for this season' message centers horizontally.
- [ ] Ensure it explicitly offers a specific 'Refresh Data' button strictly beneath it.
- [ ] Handle edge cases where the database genuinely contains a 'Season 0' with zero specials currently known.
- [ ] Test rendering ensuring the blank space matches the typical height of a standard list.
- [ ] Ensure the active season tab still cleanly displays regardless.

## 📺 Part 12: Episode List & Tracking UI (Incomplete)

**12.1 Individual, rounded-corner episode row cards.** (Incomplete)

- [ ] Verify flex rows exactly constrain height to uniform dimensions.
- [ ] Ensure subtle `#1F222A` background strictly differentiates rows from the main page.
- [ ] Handle edge cases where excessive text wrapping stretches the row height unevenly.
- [ ] Test subtle hover scale/brightness specifically on individual rows.
- [ ] Ensure `mb-4` or similar gap strictly creates breathing room between rows.

**12.2 Dedicated 16:9 episode thumbnail rendering per row.** (Incomplete)

- [ ] Verify specifically forcing `aspect-video` prevents odd TMDB portrait images from ruining layout.
- [ ] Ensure standard CSS `object-cover` strictly crops central focus points.
- [ ] Handle edge cases where extremely small thumbnail files appear deeply pixelated.
- [ ] Test caching performance when strictly loading 24 thumbnails per season list simultaneously.
- [ ] Ensure corner radii specifically match the outer card.

**12.3 Advanced Fallback Image Generator: If TMDB lacks a still, applies a heavy blur to the backdrop, darkens it, and overlays 'EP X' text.** (Incomplete)

- [ ] Verify the exact show backdrop is utilized as the base layer.
- [ ] Ensure CSS `blur(12px)` and a `bg-black/60` strictly obscure the backdrop entirely.
- [ ] Handle edge cases where the show lacks a backdrop entirely (use pure dark gradient fallback).
- [ ] Test massive white, bold string strictly rendering 'EP 4' centrally on the dark blur.
- [ ] Ensure the fallback strictly calculates its dimensions matching `16:9` perfectly.

**12.4 In-place Checkmark toggle button (○ Unwatched -> ✓ Solid Green Completed).** (Incomplete)

- [ ] Verify empty circles explicitly indicate strictly unwatched status.
- [ ] Ensure clicking instantly fills the circle and swaps strictly to a Green check icon.
- [ ] Handle edge cases where rapid spam-clicking creates a race condition with DB inserts/deletes.
- [ ] Test mouse-enter hover slightly dimming the circle to indicate interactivity.
- [ ] Ensure clicking purely toggles status without accidentally launching VLC playback.

**12.5 In-place Watch Progress mathematical update (Badge increments sync with checkmark clicks).** (Incomplete)

- [ ] Verify clicking the specific row instantly increments the massive global Progress Badge located in the header.
- [ ] Ensure un-checking strictly decrements the Badge exactly in real-time.
- [ ] Handle edge cases where toggling the final episode specifically flips the entire show specifically to 'Completed'.
- [ ] Test React `useEffect` logic safely bypassing full page re-renders.
- [ ] Ensure math syncs directly to the Library poster progress bar specifically upon exiting the view.

**12.6 Clean Episode Number & Title text combination.** (Incomplete)

- [ ] Verify the formatting strictly prefixes '1. ' or 'E01 - ' distinctly before the bold title.
- [ ] Ensure standard truncating (`truncate` utility) explicitly prevents incredibly long titles breaking flexbox.
- [ ] Handle edge cases where 'TBA' or missing titles explicitly default securely to 'Episode X'.
- [ ] Test extreme font weights explicitly separating the numeric prefix from the title.
- [ ] Ensure pure white specifically contrasts sharply against the dark row.

**12.7 Runtime display string generation (e.g., '45m').** (Incomplete)

- [ ] Verify the database explicitly pulls integer lengths representing strictly exact minutes.
- [ ] Ensure strings explicitly strip hours formatting if < 60 (display '59m' not '0h 59m').
- [ ] Handle edge cases where exact runtime strictly equals 0 (do not display anything, or explicitly display '--').
- [ ] Test floating right alignment ensuring the time anchors distinctly near the Play button.
- [ ] Ensure specific silver text (`text-gray-400`) avoids distracting from titles.

**12.8 Original Air Date display string generation.** (Incomplete)

- [ ] Verify standard formatting exclusively displays human readable dates (e.g., 'Oct 12, 2018').
- [ ] Ensure dates strictly parsed from `YYYY-MM-DD` gracefully convert local OS timezone strings.
- [ ] Handle edge cases where missing air dates entirely hide the element explicitly to prevent 'Invalid Date'.
- [ ] Test dates set in the explicit future explicitly replacing the string with a bold 'Unaired' warning.
- [ ] Ensure a delicate bullet point '•' completely separates runtime from air date.

**12.9 2-line truncated episode overview text below the title.** (Incomplete)

- [ ] Verify standard CSS strictly clamps text to 2 lines (`line-clamp-2`).
- [ ] Ensure extreme vertical margins (`mt-2`) completely separate text from titles.
- [ ] Handle edge cases where synopses entirely contain Markdown formatting strictly stripping it.
- [ ] Test color contrast strictly maintaining 'silver' readability on the `#1F222A` background.
- [ ] Ensure missing synopses completely skip rendering.

**12.10 Circular Orange 'Play' icon-button strictly for episodes with linked local files.** (Incomplete)

- [ ] Verify specifically the presence of a strictly valid `local_path` entirely activates this specific button.
- [ ] Ensure standard vibrant Orange fill specifically draws immediate focus.
- [ ] Handle edge cases where hovering accurately expands the circle scaling specifically `110%`.
- [ ] Test clicking correctly passes specific SxxExx and file path strictly to the VLC Rust process.
- [ ] Ensure the button anchors specifically to the far right vertical center.

**12.11 Disabled Gray '☁️ Cloud' icon visually indicating the local file is missing/deleted.** (Incomplete)

- [ ] Verify exactly substituting the standard Play triangle exclusively with a Cloud SVG.
- [ ] Ensure standard colors strictly utilize a heavy, dim gray (`text-gray-600`) exactly indicating 'Unavailable'.
- [ ] Handle edge cases where the user clicked specifically anyway (strictly do nothing, prevent UI crashes).
- [ ] Test explicit tooltip rendering ensuring it distinctly warns 'No File Found'.
- [ ] Ensure this completely replaces the play icon cleanly without shifting row dimensions.

**12.12 Half-Watched state icon (◐) generation for paused episodes.** (Incomplete)

- [ ] Verify exactly replacing the specific circle-check specifically with a Half-Circle icon if `last_position` > 0.
- [ ] Ensure standard orange strictly colors the half-circle specifically to indicate 'In Progress'.
- [ ] Handle edge cases where specific `last_position` is perfectly 0 or specifically exceeds length.
- [ ] Test clicking strictly resumes playback automatically from that exact position.
- [ ] Ensure the UI specifically defaults to unwatched strictly if the progress is < 5%.

## 🕒 Part 13: History & "Diary" Timeline (Incomplete)

**13.1 Dedicated 'History' sidebar routing tab.** (Incomplete)

- [ ] Verify standard NavLink exactly targets strictly the `/history` path.
- [ ] Ensure standard strictly rendering instantly without full-page reloads.
- [ ] Handle edge cases where empty history specifically displays a 'Start watching!' illustration.
- [ ] Test active state strictly highlighting this tab.
- [ ] Ensure pure vertical scroll specifically captures the main window axis.

**13.2 Pagination / Lazy-loaded virtual rendering (Handles 10,000+ entries without RAM spikes).** (Incomplete)

- [ ] Verify specific React components specifically load strictly 50 rows per 'page'.
- [ ] Ensure standard intersection-observer or virtual lists explicitly append arrays instead of re-fetching strictly everything.
- [ ] Handle edge cases where extremely rapid vertical scrolling explicitly stutters the Framer Motion animation.
- [ ] Test massive array concatenation directly impacting state memory limits.
- [ ] Ensure explicit loading spinners perfectly center specifically at the timeline base.

**13.3 Vertical scrollable timeline layout architecture.** (Incomplete)

- [ ] Verify standard absolute lines specifically drawn precisely down the exact center axis.
- [ ] Ensure standard left-right alternating row structure specifically anchors dots on the exact line.
- [ ] Handle edge cases where specific mobile widths strictly force the line entirely to the left edge.
- [ ] Test standard spacing strictly padding `mb-8` exactly between date blocks.
- [ ] Ensure standard date headers exactly span entirely across the line seamlessly breaking it.

**13.4 Dynamic Date Headers grouping logic ('Today', 'Yesterday', 'Thursday, March 10th').** (Incomplete)

- [ ] Verify standard string logic strictly groups `timestamp` objects sharing identical specific 'YYYY-MM-DD'.
- [ ] Ensure standard OS relative time formatting strictly specifically uses 'Today' or 'Yesterday' explicitly for the past 48 hours.
- [ ] Handle edge cases strictly where a leap year explicitly breaks 'March 1' vs 'Feb 29'.
- [ ] Test standard string capitalization explicitly outputting 'Monday, April 5th'.
- [ ] Ensure standard sticky positioning specifically pins the header exactly to the top bar when scrolling past.

**13.5 Local OS Timezone conversion for all UTC SQLite timestamps.** (Incomplete)

- [ ] Verify standard Rust `chrono` entirely converts UTC integer strictly specifically before returning to React.
- [ ] Ensure standard standard OS strictly uses accurate local offsets explicitly even during Daylight Savings.
- [ ] Handle edge cases explicitly where the OS lacks a strictly valid timezone (fallback directly to UTC).
- [ ] Test strict formatting specifically stripping seconds strictly outputting '4:30 PM'.
- [ ] Ensure backdated explicit string logic purely ignores timezone specifically outputting 'Unknown Time'.

**13.6 Binge-Block Accordion UI: Master component for grouping sessions.** (Incomplete)

- [ ] Verify standard `session_id` logic specifically specifically wraps consecutive array objects precisely into a singular parent container.
- [ ] Ensure standard explicit summary strictly sums total exactly rendering 'Watched 5 Episodes of The Office'.
- [ ] Handle edge cases exactly where a session spans strictly exactly across midnight explicitly grouping to the prior day.
- [ ] Test click functionality explicitly toggling the boolean state of the accordion specifically.
- [ ] Ensure specific outer borders explicitly strictly visually box the entire session.

**13.7 Expandable Accordion animation (Framer Motion height transitions).** (Incomplete)

- [ ] Verify `<AnimatePresence>` standard strictly animates the specific explicit height specifically from 0 to `auto`.
- [ ] Ensure standard CSS `overflow-hidden` precisely prevents exactly specific child content specifically jutting out.
- [ ] Handle edge cases strictly where exactly expanding the accordion specifically pushes standard content explicitly past the vertical window bound.
- [ ] Test precise explicit Framer Motion specifically exactly smoothing explicitly `duration: 0.3, ease: 'easeInOut'`.
- [ ] Ensure explicit arrow icon specifically precisely exactly rotates 180 degrees specifically.

**13.8 Auto-Session Chaining Logic: If completion timestamps are < 6 hrs apart, groups them into one Binge-Block.** (Incomplete)

- [ ] Verify exactly the specific mathematical difference specifically strictly subtracts timestamps explicitly ensuring strictly < 21600 seconds.
- [ ] Ensure explicit specifically new `session_id` directly specifically applies strictly to the next show if specifically distinct.
- [ ] Handle edge cases specifically where exactly specifically 6 hours and 1 minute strictly entirely breaks the session explicitly.
- [ ] Test specifically explicit single-episode binges specifically entirely skipping the grouping explicitly.
- [ ] Ensure strictly explicit backdated history explicitly strictly ignores standard chaining explicitly entirely.

**13.9 'Live' Binge-Block generation (Allows expanding to see individual episodes).** (Incomplete)

- [ ] Verify explicit strictly child rows precisely match specific exactly specific episode list row strictly styling.
- [ ] Ensure specifically explicit sub-rows precisely exactly specifically lack specifically massive explicitly separate timeline dots.
- [ ] Handle edge cases precisely where specifically standard clicking explicitly standard unchecking exactly an episode strictly breaks the live strictly session.
- [ ] Test standard exactly specific individual precisely exactly exactly Play buttons specifically launching exactly strictly that specific file.
- [ ] Ensure exactly specific clicking standard specifically precisely routes explicitly exactly directly to the exactly specific Media View.

**13.10 Sub-episode pause timestamp tracking text (Paused at 22:15 | 11:30 PM).** (Incomplete)

- [ ] Verify text accurately extracts the specific pause timestamp formatting it perfectly for the UI.
- [ ] Ensure exact mathematical conversions explicitly output human-readable formats like `1h 22m` or `15m`.
- [ ] Handle edge cases where the UI specifically completely skips rendering this string if the file was purely marked complete instead of naturally watched.
- [ ] Test standard explicitly formatting precisely outputting the text cleanly inside the history rows.
- [ ] Ensure explicitly specifically the text contrasts properly.

**13.11 'My Watch Date vs. Original Air Date' timeline subtext comparison string.** (Incomplete)

- [ ] Verify standard math explicitly subtracts the database history timestamp specifically from the extracted air date.
- [ ] Ensure specifically the string explicitly renders 'Watched 2 years after airing' perfectly correctly.
- [ ] Handle edge cases exactly specifically where the math entirely resolves precisely to 0 days (render 'Watched on premiere day').
- [ ] Test explicitly specifically hiding this string entirely perfectly specifically if the air date is purely completely unknown.
- [ ] Ensure completely exactly specifically the specific exactly UI renders it elegantly specifically.

**13.12 Click-to-navigate routing from a History entry directly to the Media Details page.** (Incomplete)

- [ ] Verify specific exactly exactly standard React Router specifically explicitly strictly navigating specifically perfectly to the `/media/:id` page.
- [ ] Ensure specifically precisely perfectly preserving specific entirely strictly scrolling explicitly precisely exactly perfectly specifically exactly correctly entirely.
- [ ] Handle edge cases strictly exactly entirely specifically perfectly specifically precisely specifically perfectly specifically completely exactly entirely specifically.
- [ ] Test explicitly specifically clicking explicitly entirely specifically completely perfectly exactly specifically correctly specifically entirely perfectly specifically completely perfectly.
- [ ] Ensure exactly perfectly entirely completely perfectly exactly completely exactly specifically perfectly completely exactly perfectly completely exactly perfectly specifically perfectly entirely perfectly.

## 📦 Part 14: Legacy Backdating & Archiving (Scaling Solutions) (Incomplete)

**14.1 Dedicated '✓ Mark Season Watched' button injected inside specific Season Tabs.** (Incomplete)

- [ ] Verify the button correctly targets the specific season ID dynamically rendered in the active tab.
- [ ] Ensure clicking it strictly opens the Watch Log Popup instead of blindly assuming a specific state.
- [ ] Handle edge cases where the season is already 100% completed (button should be disabled or hidden).
- [ ] Test alignment ensuring the button anchors strictly to the far right side of the Season header.
- [ ] Ensure styling utilizes a subdued dark pill to avoid visually competing with the main 'Play' buttons.

**14.2 Multi-option Watch Log Popup modal.** (Incomplete)

- [ ] Verify a heavy blur overlay drops directly behind the modal to lock out background clicks.
- [ ] Ensure the modal explicitly offers exactly three distinct radio options: 'Today', 'Archive', 'Backdate'.
- [ ] Handle edge cases where the user clicks outside the modal specifically to safely close/cancel.
- [ ] Test standard keyboard accessibility specifically allowing Tab navigation and Enter selection.
- [ ] Ensure explicit warning text clearly explains what 'Archive' does vs 'Backdate'.

**14.3 'Archive' Functionality: Marks episodes completed, increments counts, but completely skips the History table INSERT (Prevents timeline spam).** (Incomplete)

- [ ] Verify episodes explicitly update their status correctly to 'Completed' in the main episode list.
- [ ] Ensure total episode count math correctly includes archived items without requiring a history timestamp.
- [ ] Handle edge cases where a user later tries to manually backdate an already archived episode.
- [ ] Test the backend query ensuring absolutely zero rows are accidentally pushed to the `History` table.
- [ ] Ensure the 'Shows Completed' statistic correctly flips to true if the entire show was specifically archived.

**14.4 'Backdate' Feature: Dual dropdown pickers for specific Year and Month.** (Incomplete)

- [ ] Verify the dropdowns dynamically generate exactly valid years (e.g., from 1950 to present year).
- [ ] Ensure months map correctly to human-readable strings ('January', 'February').
- [ ] Handle edge cases where a user accidentally selects a year completely in the future.
- [ ] Test selection logic ensuring the exact Month/Year is strictly passed down to the SQLite parsing function.
- [ ] Ensure default states strictly select the show's original `release_date` year if available.

**14.5 Natural Timeline Spreading Engine: Automatically calculates day offsets to naturally distribute a backdated season across a month.** (Incomplete)

- [ ] Verify backend logic mathematically spreads 24 episodes specifically across 30 days.
- [ ] Ensure sequential episode viewing order is strictly maintained in the distributed timestamps.
- [ ] Handle edge cases where 50 episodes must squeeze into a 28-day February.
- [ ] Test explicit `is_legacy` boolean strictly being flagged as `1` on all these mass insertions.
- [ ] Ensure the UI correctly groups these newly inserted blocks accurately under the selected Year/Month.

**14.6 Legacy Binge-Block generation (Creates static, non-expandable cards for old shows).** (Incomplete)

- [ ] Verify specifically that history rows flagged as `is_legacy` entirely bypass the standard 6-hour grouping logic.
- [ ] Ensure the UI perfectly rolls up hundreds of legacy rows strictly into a singular 'March 2015' static block.
- [ ] Handle edge cases where expanding specifically does nothing to avoid crashing the DOM with thousands of list items.
- [ ] Test visual differences specifically applying a 'Archive' icon instead of the vibrant orange play arrow.
- [ ] Ensure legacy blocks strictly sort beneath precise daily blocks in the timeline.

**14.7 'Legacy' tag UI rendering for library items without specific daily timestamps.** (Incomplete)

- [ ] Verify hovering a completely archived poster accurately renders a strictly gray 'Legacy' badge.
- [ ] Ensure the 'Last Viewed' math correctly overrides explicitly displaying 'Archived' instead of 'Unknown days ago'.
- [ ] Handle edge cases where a show contains both Legacy and modern Active history (display the most recent active).
- [ ] Test library filtering specifically specifically allowing users to hide 'Legacy' items if desired.
- [ ] Ensure standard text specifically reads 'Backdated' or 'Archived' strictly matching user expectation.

**14.8 'Archive' checkbox conditionally added to global 'Mark All Watched' popup.** (Incomplete)

- [ ] Verify clicking the primary top-level header button clearly exposes this checkbox specifically.
- [ ] Ensure default state strictly leaves the box unchecked to prevent accidental mass deletion of context.
- [ ] Handle edge cases explicitly where the entire show strictly spans thousands of episodes (e.g., One Piece).
- [ ] Test strictly disabling the specific date pickers explicitly if 'Archive' is actively checked.
- [ ] Ensure clear tooltip text warns the user that this action will permanently lack explicit timeline dates.

**14.9 'Archive' checkbox conditionally added to the 'Add to Tracker' TMDB popup.** (Incomplete)

- [ ] Verify strictly adding a totally new show directly from the TMDB Search results allows instant archiving.
- [ ] Ensure specifically checking this box instantly downloads all metadata but marks strictly all episodes complete.
- [ ] Handle edge cases where the show hasn't aired yet (the box should be completely disabled).
- [ ] Test standard queueing strictly ensuring the metadata entirely finishes downloading before the archive flip.
- [ ] Ensure the show instantly appears directly in the Library explicitly flagged as finished.

## 📥 Part 15: The Inbox (Unmatched Files Triage) (Incomplete)

**15.1 Split-pane layout architecture (Left sidebar list, Right wide action area).** (Incomplete)

- [ ] Verify the left pane visually distincts itself using a darker background shade.
- [ ] Ensure standard hidden scrollbars isolate the left list scrolling from the right action area.
- [ ] Handle edge cases on narrow widths by stacking the panes vertically instead of side-by-side.
- [ ] Test active state highlighting when a user selects a specific item in the left list.
- [ ] Ensure the right action area defaults to empty state instructions if nothing is selected.

**15.2 Intelligent grouping of unmatched files based on extracted string keys.** (Incomplete)

- [ ] Verify the regex correctly extracts just the 'Title' string for grouping purposes.
- [ ] Ensure all files sharing an identical extracted title cluster strictly under one parent node.
- [ ] Handle edge cases where capitalization mismatches (e.g., 'show' vs 'Show').
- [ ] Test extremely long title strings to ensure they wrap or truncate without breaking the layout.
- [ ] Ensure the original raw file path remains securely mapped underneath the parsed group name.

**15.3 Numerical badge count of specific files inside each group folder.** (Incomplete)

- [ ] Verify exactly rendering a vibrant orange numeric badge next to the group name.
- [ ] Ensure mathematical logic exclusively sums every individual file mapped inside the group.
- [ ] Handle edge cases where only a single file exists (still display '1').
- [ ] Test formatting to ensure massive counts (e.g., 100+ anime episodes) render cleanly.
- [ ] Ensure badge updates dynamically if files are manually re-mapped or ignored.

**15.4 Auto-filling TMDB Search Bar utilizing the parsed group name.** (Incomplete)

- [ ] Verify clicking a group instantly pushes its parsed string into the top TMDB search bar.
- [ ] Ensure the app immediately queries TMDB without requiring the user to press 'Enter'.
- [ ] Handle edge cases where the parsed string is completely illegible to TMDB.
- [ ] Test allowing the user to seamlessly edit the auto-filled string to try alternative queries.
- [ ] Ensure clear UI feedback if the auto-search returns exactly 0 results.

**15.5 '🗑️ Ignore' action button to permanently delete a group from the unmatched pool.** (Incomplete)

- [ ] Verify clicking instantly wipes the entire group from the local SQLite staging table.
- [ ] Ensure the UI dynamically updates, entirely removing the item from the left pane list.
- [ ] Handle edge cases where the user accidentally clicks (provide a brief undo toast or warning).
- [ ] Test processing multiple ignore actions consecutively to ensure DB queues don't bottleneck.
- [ ] Ensure the actual physical files on the hard drive are strictly ignored, never deleted.

**15.6 1-Click Match action: Click a TMDB search result to instantly assign all files in the group to that show.** (Incomplete)

- [ ] Verify clicking a TMDB poster instantly extracts the Media ID.
- [ ] Ensure backend logic rapidly maps every single file in the active group to that Media ID.
- [ ] Handle edge cases where the chosen TMDB show has fewer total episodes than the selected files.
- [ ] Test visual state transitions (e.g., green checkmark success) upon completion.
- [ ] Ensure the right pane instantly clears its state and prompts the user to select the next group.

**15.7 'Advanced/Manual Match' state toggle switch.** (Incomplete)

- [ ] Verify the toggle explicitly flips the UI into a granular, row-by-row mapping interface.
- [ ] Ensure the automated bulk-match '1-Click' UI specifically hides to prevent user confusion.
- [ ] Handle edge cases where the user toggles back and forth without losing unconfirmed input data.
- [ ] Test toggle animations making the UI shift feel intentional and smooth.
- [ ] Ensure the active state is clearly highlighted so the user knows they are overriding defaults.

**15.8 Manual Episode and Season integer text inputs per individual file.** (Incomplete)

- [ ] Verify input fields strictly restrict entry to pure positive integers.
- [ ] Ensure pressing 'Tab' accurately shifts focus down the list sequentially for fast data entry.
- [ ] Handle edge cases where a file belongs to a 'Special' (allow Season 0).
- [ ] Test visual validation error states if the user accidentally inputs massive, invalid numbers.
- [ ] Ensure default placeholder logic accurately guesses SxxExx based on raw regex extraction.

**15.9 Real-time file mapping list display updating as matches are confirmed.** (Incomplete)

- [ ] Verify confirming a manual match instantly removes that specific row from the UI list.
- [ ] Ensure the parent group's numeric badge precisely decrements to reflect the remaining unmatched items.
- [ ] Handle edge cases where the list hits 0 (trigger automatic UI navigation to the next group).
- [ ] Test layout shifting to ensure removing a row doesn't break the vertical alignment of adjacent elements.
- [ ] Ensure a subtle fade-out animation signifies the successful confirmation.

## 🔎 Part 16: Search, Discovery & Edge Cases (Incomplete)

**16.1 Dedicated TMDB Search full-page UI.** (Incomplete)

- [ ] Verify the search container utilizes maximum available width for expansive visual layout.
- [ ] Ensure visual parity aligns strictly with the primary Dashboard interface.
- [ ] Handle edge cases where the search page is bookmarked or refreshed directly in the browser.
- [ ] Test responsive design breaking the grid smoothly on narrow or extremely wide monitors.
- [ ] Ensure a prominent 'Back to Dashboard' or global close button exists in the upper left.

**16.2 Live search input handling with <Enter> key triggers.** (Incomplete)

- [ ] Verify pressing 'Enter' instantly fires the request to the TMDB API.
- [ ] Ensure input debouncing prevents accidental network spam if the user types rapidly.
- [ ] Handle edge cases where the user searches for purely empty spaces (should reset UI or ignore).
- [ ] Test standard clear buttons ('X' icon inside the input) rapidly resetting the view state.
- [ ] Ensure the input field is automatically focused whenever the page is initially loaded.

**16.3 API HTTP Error / Invalid Key graceful error toast catchers.** (Incomplete)

- [ ] Verify the UI explicitly renders a descriptive red toast error upon a 401 Unauthorized TMDB response.
- [ ] Ensure network timeouts trigger specific text ('Unable to reach TMDB. Check connection').
- [ ] Handle edge cases where an error occurs silently in the background (log to console instead of spamming user).
- [ ] Test toast dismissal explicitly allowing users to clear them immediately.
- [ ] Ensure failing API calls never crash the entire React application shell.

**16.4 Network disconnect / Offline mode detection and UI notification.** (Incomplete)

- [ ] Verify global connection status tracking specifically accurately detects offline states.
- [ ] Ensure a permanent visual banner or icon appears universally indicating 'Offline Mode'.
- [ ] Handle edge cases where the network rapidly drops and reconnects.
- [ ] Test disabling specific network-dependent actions (like manual syncs) explicitly when offline.
- [ ] Ensure local cached data (images, library lists) continues to render flawlessly without internet.

**16.5 Responsive grid display of fetched TMDB search results.** (Incomplete)

- [ ] Verify fetched results strictly populate identical 2:3 aspect ratio poster cards as the main Library.
- [ ] Ensure pagination triggers more results seamlessly when scrolling to the bottom.
- [ ] Handle edge cases where TMDB returns extremely low-resolution or entirely missing primary posters.
- [ ] Test filtering toggles strictly separating TV shows from Movies in the results view.
- [ ] Ensure the grid correctly handles extremely short lists of 1 or 2 items by centering them beautifully.

**16.6 Real-time cross-checking against the local DB to disable '+ Add' buttons and display green '✓ In Library' badges for already tracked media.** (Incomplete)

- [ ] Verify every remote search result explicitly checks its TMDB ID against the local SQLite store.
- [ ] Ensure matching shows render a distinct, non-clickable visual indicator (e.g., 'Added').
- [ ] Handle edge cases where a user removes a show locally, ensuring the search UI accurately reenables the Add button.
- [ ] Test performance overhead when cross-referencing hundreds of search results simultaneously.
- [ ] Ensure adding a show dynamically triggers a visual shift on the specific card without refreshing the entire list.

**16.7 Global Quick Search instant Regex filtering of the currently mounted component state.** (Incomplete)

- [ ] Verify the persistent top-bar search instantly filters whatever explicit list the user is actively viewing.
- [ ] Ensure strings matching completely ignore case sensitivity entirely.
- [ ] Handle edge cases where the search string is purely numbers (e.g., '1984').
- [ ] Test rapid rendering updates ensuring the filtered state updates continuously with every keystroke.
- [ ] Ensure pressing 'Escape' instantly clears the search state and restores the full local list.

**16.8 Dedicated Settings UI page layout.** (Complete)

- [x] Verify a clean, structured form layout cleanly separating basic logic from advanced overrides.
- [x] Ensure input fields cleanly align vertically with standardized labels on the left axis.
- [x] Handle edge cases where changes are abandoned by explicitly providing 'Save' vs 'Discard' logic.
- [x] Test tabbed navigation internal to Settings (e.g., 'General', 'Database', 'Connections').
- [x] Ensure setting modifications actively update the core `JSON` configuration file securely.

**16.9 TMDB API Key text input, validation, and persistent local storage.** (Incomplete)

- [ ] Verify text input obscures the string visually (using `type='password'`) for security.
- [ ] Ensure changing the key triggers a background validation test instantly against TMDB.
- [ ] Handle edge cases where the user accidentally copies surrounding whitespace characters.
- [ ] Test storing the value securely, overriding any previous configuration seamlessly.
- [ ] Ensure a 'Test Connection' button explicitly returns a 'Success' or 'Failure' visual cue.

**16.10 VLC Executable Path text input.** (Incomplete)

- [ ] Verify the string perfectly maps directly to the Rust backend execution parameter.
- [ ] Ensure visual error styling if the user inputs a path to a non-existent file.
- [ ] Handle edge cases specifically targeting Windows path formats vs Unix path formats.
- [ ] Test auto-detection 'Restore Default' logic dynamically finding VLC if the path is entirely deleted.
- [ ] Ensure relative paths strictly convert to absolute paths based on the application root.

**16.11 Native OS File Explorer browse window specifically for locating the VLC .exe / .app.** (Incomplete)

- [ ] Verify clicking 'Browse' strictly spawns the native OS file picker window.
- [ ] Ensure the window specifically filters only for executables (`.exe` on Windows, `.app` on Mac).
- [ ] Handle edge cases where the user cancels the OS dialog (the input value should remain untouched).
- [ ] Test inserting the returned absolute string precisely back into the React input field.
- [ ] Ensure the action does not freeze the React UI main thread.

**16.12 Image Load Error handling (Silently dropping broken TMDB URLs and defaulting to skeletons).** (Incomplete)

- [ ] Verify listening explicitly for standard `<img onError>` DOM events.
- [ ] Ensure catching the error instantly swaps the specific broken `<img src>` to the fallback placeholder.
- [ ] Handle edge cases where the fallback image itself fails to load.
- [ ] Test hiding the broken 'image not found' icon native to all specific web browsers.
- [ ] Ensure this completely avoids throwing explicit JavaScript console errors that halt execution.

**16.13 Automatic boot-up cache verification (mkdir -p equivalents in Rust to ensure the app never crashes from missing folders).** (Incomplete)

- [ ] Verify Rust strictly checks for critical directory paths specifically on every single startup sequence.
- [ ] Ensure automated recursive directory creation constructs the entire tree explicitly if deleted.
- [ ] Handle edge cases explicitly where file permission errors lock creation entirely (display fatal error UI).
- [ ] Test specifically verifying the structure required for Database, Posters, Backdrops, and Episode Stills.
- [ ] Ensure the creation specifically logs precisely to the background debugging file entirely.

## ⌨️ Part 17: Advanced Navigation & Shortcuts (UX Polish) (Incomplete)

**17.1 Ctrl+K / Cmd+K Global Shortcut: Instantly focuses the Top Search Bar from anywhere.** (Incomplete)

- [ ] Make sure pressing Ctrl+K or Cmd+K instantly places the typing cursor into the main search box.
- [ ] Stop the internet browser from doing its own normal search shortcut when these keys are pressed.
- [ ] Check that this shortcut works no matter what screen or popup the user is looking at.
- [ ] Let the user start typing right away without needing to use their mouse.
- [ ] Ignore the shortcut politely if the user is already typing inside another different text box.

**17.2 Esc Key Binding: Closes open modals, clears search inputs, or unfocuses elements.** (Incomplete)

- [ ] Close any open popup window immediately if the user taps the Escape key.
- [ ] Clear any typed words out of the search bar if no popup is open.
- [ ] Deselect whatever button or link the user is focused on if the search bar is empty.
- [ ] Prevent the app from doing all three things at the same time.
- [ ] Leave the screen looking as it was before the user accidentally clicked something.

**17.3 Arrow Key Grid Navigation: Allow users to use keyboard arrows to jump between posters in the Library.** (Incomplete)

- [ ] Move the visual highlight smoothly from one movie poster to the next when tapping the Left or Right arrow keys.
- [ ] Jump up or down to the poster in the row above or below when using the Up or Down keys.
- [ ] Scroll the page automatically so the newly highlighted poster is always visible on the screen.
- [ ] Wrap the highlight cleanly back to the start of the next row when moving past the edge of the screen.
- [ ] Stop the arrow keys from accidentally scrolling the whole page if the user hasn't selected a poster yet.

**17.4 Spacebar Playback: Pressing Space while a media card is focused instantly launches it in VLC.** (Incomplete)

- [ ] Start playing the video instantly if the user hits the Spacebar while highlighting a movie poster.
- [ ] Play the same episode that the normal orange 'Play' button would launch.
- [ ] Stop the page from accidentally jumping downward when Space is pressed.
- [ ] Do absolutely nothing if the highlighted show is missing its video file.
- [ ] Ignore the Spacebar if the user is typing a word into the search box.

**17.5 Shift+Click Multi-Select: Select multiple files at once in the Inbox for bulk ignoring/assigning.** (Incomplete)

- [ ] Remember which item the user clicked first when managing new files.
- [ ] Highlight every single item between the first click and a new Shift+Click.
- [ ] Change the background color slightly on all selected rows so they stand out clearly.
- [ ] Cancel the mass-selection if the user clicks somewhere else without holding the Shift key.
- [ ] Keep this feature disabled outside of the specific file management screen.

**17.6 Mouse Back/Forward Support: Utilize side mouse buttons to navigate back and forth between Library and Details views.** (Incomplete)

- [ ] Let users click the extra buttons on the side of their mouse to jump back to the previous screen.
- [ ] Move between the main movie library and specific show details smoothly using these physical buttons.
- [ ] Remember how far down the page the user had scrolled when they go backwards.
- [ ] Fade the previous screen back in gently instead of snapping jarringly.
- [ ] Prevent errors if the user clicks 'Back' immediately after opening the app for the first time.

**17.7 Breadcrumb Trails: E.g., Library > TV Shows > Breaking Bad > Season 2 visible at the top of the detail view.** (Incomplete)

- [ ] Show a neat, simple text trail clearly indicating where the user is inside the app.
- [ ] Separate each section cleanly with a subtle little arrow so it reads easily.
- [ ] Allow the user to click any older section of the trail to jump right back to it instantly.
- [ ] Shorten incredibly long show names neatly so they don't push the trail off the edge of the screen.
- [ ] Color the older steps slightly darker to help the current page stand out.

**17.8 Custom Context Menus: Right-clicking a poster opens a sleek, custom dark-mode menu (Play Next, Mark Watched, Edit, Remove).** (Incomplete)

- [ ] Block the standard, clunky computer menu from appearing when right-clicking a movie poster.
- [ ] Show a beautiful, dark-themed menu right next to where the mouse clicked.
- [ ] Close this menu instantly if the user simply clicks anywhere else on the page.
- [ ] Provide clear, easy buttons to instantly play the show, mark it watched, or remove it.
- [ ] Keep the menu visible on screen even if the user clicks right at the very bottom edge.

**17.9 Sticky Section Headers: As you scroll down the Library or History, the headers ("Recently Added", "Today") stick to the top of the screen until pushed up by the next header.** (Incomplete)

- [ ] Pin the date label to the top edge of the screen while the user scrolls through that section.
- [ ] Give the label a solid background so the scrolling movies don't make the text hard to read.
- [ ] Make sure the label stays clearly underneath the main search bar so they don't overlap awkwardly.
- [ ] Push the old label up and out of the way smoothly the moment the next date section arrives.
- [ ] Un-pin the label instantly when the user decides to scroll back up.

**17.10 Double-Click Play: Double-clicking a show poster bypasses the details page and instantly plays the next unwatched episode.** (Incomplete)

- [ ] Notice if a user clicks a movie poster twice rapidly instead of just once.
- [ ] Skip opening the show's page and figure out which episode they need to watch next.
- [ ] Launch the video player right away with that specific episode.
- [ ] Show a small spinning icon instantly so the user knows the video is about to start.
- [ ] Ignore the double-click if the user has already watched every single episode of that show.

**17.11 Alt+Click Catch-Up: Alt+Clicking an episode checkmark automatically marks that episode and all previous episodes in the season as watched.** (Incomplete)

- [ ] Check which episode number the user is holding the Alt key on.
- [ ] Mark that specific episode, and every single episode before it, finished all at once.
- [ ] Save this progress quietly in the background so the app doesn't freeze or slow down.
- [ ] Fill in all the visual checkmarks instantly on the screen so the user sees it worked.
- [ ] Display a brief warning if the user accidentally tries to mark 50 episodes at the same time.

**17.12 Global Command Palette: Press Cmd+P to open a quick-action menu (e.g., type ">Scan" to trigger a directory scan).** (Incomplete)

- [ ] Open a handy central menu when the user presses Cmd+P or Ctrl+P.
- [ ] Show a list of quick actions that filters down instantly as the user types letters.
- [ ] Start the chosen action immediately when the user presses Enter.
- [ ] Close the menu the exact second the action begins.
- [ ] Keep the most popular or frequently used actions right at the very top of the list.

**17.13 Touchpad Swipe Gestures: Swipe left/right on a laptop trackpad to go back/forward in the app history.** (Incomplete)

- [ ] Recognize when a user makes a strong, intentional swipe on their laptop touchpad.
- [ ] Change pages like a normal web browser does when swiping back or forward.
- [ ] Show a gentle fading arrow on the edge of the screen to confirm the swipe was noticed.
- [ ] Ignore horizontal swipes if the user is just trying to scroll through a row of movie posters.
- [ ] Let users turn off this swipe feature in the settings if they don't like it.

**17.14 Floating Action Button (FAB): A subtle "Jump to Top" arrow appears when scrolling deep into the Library.** (Incomplete)

- [ ] Notice quietly when the user has scrolled significantly far down a very long list of movies.
- [ ] Show a small, rounded button neatly in the bottom corner of the screen.
- [ ] Fade the button in smoothly instead of having it appear suddenly.
- [ ] Scroll the page back up to the very top the moment the user clicks the button.
- [ ] Keep the button hidden if the page is short enough to see everything at once.

**17.15 Shortcut Cheat Sheet: Press ? anywhere to open a modal displaying all keyboard shortcuts.** (Incomplete)

- [ ] Open a helpful popup listing every keyboard trick when the user presses the '?' key.
- [ ] Organize the list neatly so it is easy to read at a quick glance.
- [ ] Keep the popup closed if the user is just trying to type a question mark into the search bar.
- [ ] Add a simple 'Close' button to hide the list when they are done.
- [ ] Pause any video that is playing the second this cheat sheet appears.

**17.16 Scroll Memory: Pressing "Back" from a show details page restores your exact scroll position in the Library grid.** (Incomplete)

- [ ] Remember how far down the user had scrolled before they clicked on a movie.
- [ ] Jump right back to that spot smoothly when they hit the 'Back' button.
- [ ] Stop the page from awkwardly jumping to the top while the movie posters are loading.
- [ ] Forget the scroll spot if they click the home button to start a fresh search.
- [ ] Remember separate scroll spots for the TV library and the Movie library simultaneously.

**17.17 Focus Trapping: When a modal is open, pressing Tab cycles only through modal buttons, preventing the background UI from being highlighted.** (Incomplete)

- [ ] Keep the keyboard focus inside a popup window when one is open.
- [ ] Loop the focus from the last button in the popup straight back to the first button automatically.
- [ ] Stop the user from accidentally highlighting buttons on the dark screen behind the popup.
- [ ] Make sure this works backwards too, if the user holds Shift while pressing Tab.
- [ ] Give the focus back to the main search bar once the popup is finally closed.

**17.18 Auto-Focus Search: Clicking the "Search TMDB" sidebar tab instantly focuses the input cursor.** (Incomplete)

- [ ] Place the typing cursor directly into the big search box the second the user opens the search page.
- [ ] Show the blinking line immediately so the user knows they can type without clicking first.
- [ ] Keep the cursor there patiently even if the rest of the page takes a second to load.
- [ ] Stop stealing the cursor if the user clicks the search page while already trying to do something else.
- [ ] Make sure clicking the clear button puts the cursor right back into the box again.

**17.19 Inline Clear Button: A tiny x icon appears inside the search bar when typing, allowing 1-click clearing.** (Incomplete)

- [ ] Show a tiny 'X' mark neatly at the far right edge of the search box.
- [ ] Keep the 'X' hidden until the user actually types at least one letter.
- [ ] Wipe the entire search box clean the instant the 'X' is clicked.
- [ ] Put the typing cursor right back into the clean box automatically so they can try again.
- [ ] Give the 'X' a very slight highlight when hovered so it feels like a real button.

**17.20 Native Window Dragging: The entire empty space of the top navigation bar acts as a -webkit-app-region: drag zone to move the desktop window.** (Incomplete)

- [ ] Let the user click and hold any empty space at the top of the app to move the whole window.
- [ ] Stop the window from moving if they click on a button or the search box instead.
- [ ] Keep the window moving smoothly following the mouse across the screen.
- [ ] Turn off dragging if the window is already maximized to fill the whole screen.
- [ ] Let the user double-click the empty space to quickly maximize or shrink the window.

## 🎥 Part 18: Cinematic UI & Animation Details (Incomplete)

**18.1 Dynamic Background Tinting: The app extracts the dominant color from the active show's poster and applies a subtle 5% tint to the background #0D0F14.** (Incomplete)

- [ ] Pick out the main, most vibrant color directly from the show's main poster image.
- [ ] Mix this color very faintly with the dark background of the app to give each show a unique mood.
- [ ] Fade this subtle color in very smoothly when opening a new show page.
- [ ] Go back to the standard dark background immediately if the poster is totally black and white.
- [ ] Save this color in the background so the app doesn't have to figure it out again next time.

**18.2 Hero Parallax Scrolling: As you scroll down the Dashboard, the Hero Backdrop scrolls at 50% speed, creating 3D depth.** (Incomplete)

- [ ] Make the giant top image slide down slightly slower than the rest of the page when scrolling.
- [ ] Ensure this creates a beautiful, subtle 3D window effect without feeling dizzying.
- [ ] Stop the image cleanly before it scrolls too far and shows an empty gap.
- [ ] Turn this effect off on phones or older computers to keep the app running fast.
- [ ] Keep the dark shadow over the image still so the title text stays easy to read.

**18.3 Cinematic Film Grain: A very faint, CSS-based animated film grain overlays the background for texture.** (Incomplete)

- [ ] Place a nearly invisible layer of static noise over the entire background of the app.
- [ ] Keep it incredibly faint so it adds a subtle movie-theater feel without making the screen look dirty.
- [ ] Make the static move slightly so it feels like real, classic film texture.
- [ ] Turn the moving static off if the user has requested fewer animations on their computer.
- [ ] Ensure this static never accidentally blocks a user from clicking a button underneath it.

**18.4 Active Show Shimmer: The poster of a show you are currently watching has a very subtle, slow-pulsing glowing border.** (Incomplete)

- [ ] Find which shows the user is currently in the middle of watching.
- [ ] Draw a very thin, glowing orange line around the edge of those specific posters.
- [ ] Make the glow pulse very slowly and smoothly so it isn't distracting.
- [ ] Turn the glow off the exact moment the user finishes the last episode.
- [ ] Keep the effect subtle enough that a whole row of active shows doesn't look like a neon sign.

**18.5 Aspect-Ratio Skeleton Loaders: Loading placeholders match the 2:3 ratio of posters and 16:9 ratio of episodes.** (Incomplete)

- [ ] Make sure the gray loading boxes are the same shape as the movie posters will be.
- [ ] Make the episode loading boxes widescreen so they don't look awkwardly tall.
- [ ] Shrink or grow these boxes smoothly to fit any screen size just like real images would.
- [ ] Round the corners of the loading boxes so they match the final polished look.
- [ ] Put a tiny, faint play button shape in the middle of the episode loading boxes.

**18.6 Shimmering Skeletons: Loading boxes use a smooth, left-to-right CSS gradient animation.** (Incomplete)

- [ ] Add a shiny, moving highlight across all the gray loading boxes.
- [ ] Make the highlight sweep smoothly from left to right like light reflecting off glass.
- [ ] Keep all the boxes shimmering together at the exact same time.
- [ ] Make the highlight loop endlessly without any jarring jumps or stutters.
- [ ] Keep the shiny effect neatly inside the rounded corners of the loading boxes.

**18.7 Staggered Grid Intro: When loading the Library, posters fade-in-up one by one in a rapid wave sequence, rather than flashing on screen simultaneously.** (Incomplete)

- [ ] Bring the movie posters onto the screen one after another in a quick, flowing wave.
- [ ] Make them fade in gently while sliding up slightly into place.
- [ ] Show this beautiful wave only once when the page loads, not every single time the user clicks a button.
- [ ] Stop the wave on posters that are way down out of sight to save computer power.
- [ ] Show the posters instantly without the wave if a user is just typing a fast search.

**18.8 Custom Themed Tooltips: Native OS tooltips are replaced by instant, styled #1F222A glassy popups.** (Incomplete)

- [ ] Remove the ugly, standard white hover-text boxes that computers normally show.
- [ ] Replace them with beautiful, dark, slightly see-through popup boxes.
- [ ] Make sure these boxes always stay on the screen and never get cut off at the edges.
- [ ] Wait just a tiny fraction of a second before showing them so they don't flash annoyingly.
- [ ] Add a tiny, sharp pointer connecting the box directly to the button the user is hovering over.

**18.9 Hero Text Shadowing: Ensures pure white text is readable even if the movie backdrop is a bright daytime scene.** (Incomplete)

- [ ] Add a very tight, dark shadow behind the big show title at the top of the page.
- [ ] Keep the shadow incredibly clean so the letters don't look blurry or messy.
- [ ] Put a much softer, larger dark cloud behind all the text to dim the bright image behind it.
- [ ] Blend this with the dark bottom edge so the transition looks natural.
- [ ] Test the text against a pure white image to guarantee it can always be read.

**18.10 Search Match Highlighting: When using Quick Search, the matching letters in the title are highlighted in orange.** (Incomplete)

- [ ] Find the specific letters the user typed directly inside the movie title.
- [ ] Change those matching letters to a bright orange color so they stand out.
- [ ] Keep the original capital or lowercase letters the same, even if the user typed them differently.
- [ ] Handle it gracefully if the user types a word that shows up twice in the very same title.
- [ ] Make the orange letters slightly bolder to catch the user's eye instantly.

**18.11 Hero Crossfade Transitions: Changing the featured Hero show performs a smooth 1-second image crossfade.** (Incomplete)

- [ ] Load the new background image silently before showing it.
- [ ] Fade the old image away smoothly over one full second.
- [ ] Stop the app from slowing down if the user clicks through five shows extremely fast.
- [ ] Fade the title text out and swap it to the new title in the middle of the transition.
- [ ] Handle it beautifully if the user clicks back to the main menu without any background image at all.

**18.12 "Ken Burns" Hero Effect: The Dashboard backdrop slowly scales up (1.00 to 1.05) over 30 seconds for subtle life.** (Incomplete)

- [ ] Make the giant background image grow incredibly slowly over a long period of time.
- [ ] Have it slowly zoom in and then zoom back out in an endless, gentle loop.
- [ ] Keep the very center of the image in the middle so it doesn't drift off to the side.
- [ ] Ensure the movement is smooth without any tiny visual jitters or steps.
- [ ] Pause the growing the second the user scrolls away so the computer doesn't waste energy.

**18.13 Glowing Progress Tails: The active end of the orange progress bar features a subtle blur/glow drop-shadow.** (Incomplete)

- [ ] Add a bright, glowing orange dot at the very tip of the progress bar.
- [ ] Let the glow spill slightly forward into the empty part of the bar like a real light.
- [ ] Keep the color of the glow matching the vibrant orange of the filled bar.
- [ ] Turn the glow off if the bar is full and turns green.
- [ ] Hide the glowing dot if the user hasn't started the episode yet.

**18.14 Truncation Fade: Long titles use a mask-image: linear-gradient to fade out softly on the right edge instead of hard... cuts.** (Incomplete)

- [ ] Fade the very end of incredibly long titles out smoothly into the background.
- [ ] Stop the text from wrapping awkwardly onto a second line.
- [ ] Hide the standard, ugly three dots (...) when this smooth fade is used.
- [ ] Only use this smooth fade on titles that are actually too long to fit.
- [ ] Keep short titles looking normal without any fading at all.

**18.15 Golden Completion Badge: Shows with 100% completion get a special gold laurel-wreath icon instead of the standard checkmark.** (Incomplete)

- [ ] Check if a user has watched every single available episode of a show.
- [ ] Swap the normal green checkmark for a beautiful, detailed gold wreath icon.
- [ ] Make the gold color vibrant and slightly shiny so it feels like a real reward.
- [ ] Place the wreath in the top corner of the movie poster where it is easy to see.
- [ ] Remove the gold wreath instantly if the user decides to mark an episode as unwatched later.

**18.16 Dynamic Border-Radii: Posters have an 8px radius, but when hovered/scaled, the radius adjusts slightly to maintain optical perfection.** (Incomplete)

- [ ] Change the roundness of the poster corners very slightly when the poster grows on hover.
- [ ] Keep the corners looking smooth without becoming awkwardly sharp during the animation.
- [ ] Time the corner change flawlessly to match the speed of the poster growing.
- [ ] Keep the dark shadow behind the poster matched to the new corner shape.
- [ ] Return the corners to their normal shape instantly when the user moves the mouse away.

**18.17 Glass Reflection Animation: Hovering a card triggers a fast, 45-degree white light reflection sweep across the surface.** (Incomplete)

- [ ] Create a shiny, see-through streak of white light across the movie poster.
- [ ] Hide the light off to the side before the user hovers over the poster.
- [ ] Sweep the light rapidly from the top corner to the bottom corner when the mouse touches it.
- [ ] Keep the light inside the edges of the poster so it doesn't spill onto the background.
- [ ] Only show the light sweep once per hover so it doesn't loop forever and become annoying.

**18.18 Variable Opacity Stars: Unfilled stars in the rating widget are 20% opacity white, not just gray.** (Incomplete)

- [ ] Make the empty rating stars a very faint, see-through white color instead of dull gray.
- [ ] Turn them solid and bright immediately when the user hovers over them.
- [ ] Keep the filled, orange stars solid so they stand out.
- [ ] Ensure the empty, faint stars are still easy to see against a bright background image.
- [ ] Color the stars instantly when clicked without waiting for a server to respond.

**18.19 Star "Pop" Animation: Clicking a star triggers a micro-scaling "bounce" effect.** (Incomplete)

- [ ] Make the star jump slightly larger for a tiny fraction of a second when clicked.
- [ ] Snap the star back down to normal size quickly to feel like a real, tactile button.
- [ ] Make all the stars before the clicked one jump at the same time.
- [ ] Keep the jump incredibly short so it feels snappy and responsive.
- [ ] Ignore the jump effect if the user is merely sliding their mouse over the stars.

**18.20 Smooth Accordion Heights: Expanding a Binge-Block animates the height dynamically rather than instantly snapping the layout down.** (Incomplete)

- [ ] Make the history section slide open smoothly to reveal the episodes hidden inside.
- [ ] Let the area grow as tall as it needs to be to fit the new content.
- [ ] Give the sliding motion a very slight, natural bounce so it feels organic.
- [ ] Stop the text inside from looking awkwardly squished while the section is opening.
- [ ] Slide the section back shut smoothly when the user clicks to close it.

## 📡 Part 19: VLC Engine & Playback Polish (Incomplete)

**19.1 Auto-Fullscreen Flag: Option in Settings to append --fullscreen to the VLC launch command.** (Incomplete)

- [ ] Provide a simple toggle switch in the settings menu allowing users to choose if videos should always open in fullscreen mode.
- [ ] Save this choice permanently so it is remembered every time the user opens the application.
- [ ] Apply the fullscreen behavior flawlessly every time a video is played.
- [ ] Allow the user to press the Escape key once the video is playing to return to a standard window without breaking the connection.
- [ ] Prevent any conflict if the user happens to have standard fullscreen preferences already set up inside their own video player.

**19.2 Launch Muted Flag: Option to launch VLC silently (--volume=0).** (Incomplete)

- [ ] Include a straightforward checkbox in the preferences to start all new media muted by default.
- [ ] Open the video player with zero volume immediately when this setting is turned on.
- [ ] Ensure this feature only affects the video player itself, leaving the main computer volume untouched.
- [ ] Display a helpful visual hint on the dashboard letting the user know the audio is intentionally silenced.
- [ ] Permit the user to freely turn the volume back up manually at any point while watching.

**19.3 Preferred Audio Track: Save an integer preference (e.g., Track 2 for Japanese audio) and pass to VLC.** (Incomplete)

- [ ] Allow users to pick their favorite audio language (like 'Track 1' or 'Track 2') from a simple dropdown menu.
- [ ] Remember this choice automatically for the specific TV show so they don't have to select it every episode.
- [ ] Start the video player smoothly with the chosen audio track already active.
- [ ] Fall back quietly to the default audio track if the requested one is missing from the file.
- [ ] Add a small tooltip explaining that track numbers are based on how the video file was created.

**19.4 Preferred Subtitle Track: Save and pass subtitle track preference to VLC (--sub-track).** (Incomplete)

- [ ] Give users the ability to lock in a specific subtitle track number to match their language preference.
- [ ] Launch the media player with the correct subtitles displaying instantly on screen.
- [ ] Offer a clear 'Off' or 'Disabled' option for users who never want subtitles to appear automatically.
- [ ] Handle situations gracefully where a video has no subtitles at all, ensuring playback still starts normally.
- [ ] Recognize and load standalone subtitle files automatically if they are placed next to the video file.

**19.5 "Memory Jogger" Rewind: Auto-resume rewinds 5 seconds from the exact last_position to refresh the user's memory of the scene.** (Incomplete)

- [ ] Calculate the starting position 5 seconds prior to where the user previously paused the video.
- [ ] Start the video at the very beginning (0 seconds) if they paused less than 5 seconds into the clip.
- [ ] Skip this rewind if the user is launching a brand new, never-before-seen episode.
- [ ] Provide a simple slider in the settings to let users change this rewind length from 0 to 15 seconds.
- [ ] Update the progress bar visually on the dashboard so it correctly reflects the slightly rewound starting point.

**19.6 VLC Instance Management: Detect if VLC is already running and cleanly enqueue/replace media without spawning multiple windows.** (Incomplete)

- [ ] Check quietly in the background if the video player is already open before trying to launch a new video.
- [ ] Swap the media cleanly inside the existing player window instead of cluttering the screen with a second window.
- [ ] Bring the existing player window directly to the front so the user knows the new video has started.
- [ ] Reconnect the background tracker seamlessly so the new video's progress is saved correctly.
- [ ] Show a gentle warning message if the user clicks 'Play' on five different episodes at the exact same time.

**19.7 Zombie Process Recovery: If WatchMark crashes and restarts while VLC is open, it automatically reconnects to the running VLC HTTP heartbeat.** (Incomplete)

- [ ] Look for an orphaned video player window immediately when the tracking app is opened.
- [ ] Re-establish the connection to the video player quietly without interrupting the movie that is currently playing.
- [ ] Update the app's dashboard instantly to show what is currently playing and how much time is left.
- [ ] Continue tracking the progress seamlessly as if the tracking app had never closed.
- [ ] Ignore any video players that the user opened manually themselves, focusing only on ones the app controls.

**19.8 Reset Progress Action: A specific UI button to clear last_position to 0 without marking as unwatched.** (Incomplete)

- [ ] Place a small, clear 'Reset' icon right beside the progress bar on the show's detail page.
- [ ] Clear out the saved pause time without deleting the history of when they watched it.
- [ ] Update the page immediately to show that the episode is back to 0% progress.
- [ ] Ask for a quick confirmation or require a double-click so the user doesn't accidentally wipe their progress.
- [ ] Keep the show comfortably in the 'Continue Watching' row if they still have other episodes left to finish.

**19.9 Missing File "Locate" Button: If a file is moved, the ☁️ icon turns into a magnifying glass to manually re-link the new file path.** (Incomplete)

- [ ] Change the missing file icon into an interactive magnifying glass to let the user find the missing video.
- [ ] Open a standard file browser window when clicked, filtering specifically for video files.
- [ ] Update the missing link instantly once the user selects the new correct file location.
- [ ] Swap the magnifying glass right back to the standard 'Play' button immediately after it's fixed.
- [ ] Verify the newly chosen file actually works before saving the changes to prevent further confusion.

**19.10 OS Title Bar Injection: Pass TMDB episode title to VLC (--meta-title) so the Windows Taskbar reads "Breaking Bad - S01E01" instead of "file_xyz.mkv".** (Incomplete)

- [ ] Build a clean, readable name using the show title, season, episode number, and episode name.
- [ ] Send this beautiful name directly to the video player so it displays correctly at the top of the window.
- [ ] Handle special characters like quotes or emojis safely so they don't break the title.
- [ ] Shorten the title elegantly with an ellipsis if it is incredibly long and won't fit in the taskbar.
- [ ] Make sure this custom title also appears nicely in the standard volume control popups on the user's computer.

**19.11 OS Media Art Injection: Pass local cached poster path to VLC (--meta-art) for Windows/Mac media control overlays.** (Incomplete)

- [ ] Find the exact location of the downloaded poster image on the user's computer.
- [ ] Send this image path to the video player when the episode starts.
- [ ] Skip this feature without causing an error if the poster image hasn't finished downloading yet.
- [ ] Check that the image displays correctly on the computer's lock screen when the user pauses the video and walks away.
- [ ] Handle folder names with spaces or special characters safely so the image always loads.

**19.12 VLC Crash Catching: If the VLC.exe exits with a crash code, WatchMark displays a specific error toast.** (Incomplete)

- [ ] Watch the video player closely in the background to see if it closes normally or if it unexpectedly crashes.
- [ ] Show a helpful, bright red warning notification on the screen if a crash is detected.
- [ ] Stop the app from saving corrupted pause times if the player crashed while the user was skipping forward.
- [ ] Provide a small button on the notification to let advanced users view what went wrong.
- [ ] Ensure the main tracking app remains stable and usable even if the video player fails.

**19.13 "Test VLC Connection" Button: A button in Settings to verify the path and HTTP port are accessible.** (Incomplete)

- [ ] Add a prominent 'Test Connection' button right below where the user types in their video player folder path.
- [ ] Open a hidden version of the player briefly just to make sure the app can talk to it successfully.
- [ ] Close the hidden player instantly once the test is finished.
- [ ] Show a cheerful green 'Success' badge if everything is working.
- [ ] Display clear troubleshooting advice if the test fails or times out.

**19.14 Configurable Heartbeat: Slider in settings to change polling from 5s to 1s (high precision) or 10s (low CPU).** (Incomplete)

- [ ] Provide an easy-to-use slider letting users choose how often the app checks the video progress.
- [ ] Update the settings instantly as the user drags the slider, without needing to click a save button.
- [ ] Apply the new checking speed immediately, even if a video is currently playing.
- [ ] Add simple text labels explaining that faster checking uses slightly more battery power.
- [ ] Set the default value to 5 seconds to provide a great balance for most users right out of the box.

**19.15 Minimization Auto-Pause: Optional setting: When WatchMark is minimized, send an HTTP command to pause VLC.** (Incomplete)

- [ ] Detect when the user minimizes the main tracking app window.
- [ ] Send a quick command to pause the video player the second the app drops out of view.
- [ ] Automatically un-pause the movie the moment the user brings the tracking app back onto their screen.
- [ ] Make this feature optional so users with two monitors can keep watching while using other apps.
- [ ] Ignore the pause command if the video is already paused to prevent accidentally un-pausing it.

**19.16 "Skip Intro" Manual Offset: Set a global offset per-show (e.g., "Always start this anime at 01:30").** (Incomplete)

- [ ] Create a small text box on the show's page where the user can type in a specific amount of seconds to skip.
- [ ] Start every new, unwatched episode of that show at that requested timestamp automatically.
- [ ] Keep standard pause-and-resume behavior fully intact if the user has already watched past the intro.
- [ ] Show a brief 'Skipped Intro' message on the screen when the video starts so the user knows it worked.
- [ ] Allow the user to easily delete the number to return the show back to normal playback.

**19.17 Live Playback HUD: A tiny "Now Playing" widget in WatchMark's sidebar updating live while VLC is open.** (Incomplete)

- [ ] Dedicate a small space at the bottom of the navigation menu specifically for a live status card.
- [ ] Show the title of the current episode and a tiny progress bar that fills up while they watch.
- [ ] Slide the widget onto the screen smoothly the moment the video player opens.
- [ ] Hide the widget when the movie is over so the screen stays clean and uncluttered.
- [ ] Ensure clicking the widget takes the user straight to that show's detail page.

**19.18 Remote "Stop" Button: A button inside WatchMark that kills the VLC process without switching windows.** (Incomplete)

- [ ] Add a clear, red 'Stop' icon directly onto the live 'Now Playing' sidebar widget.
- [ ] Close the video player instantly when clicked without forcing the user to switch over to the video window.
- [ ] Wait for the video player to fully close before updating the main app screen.
- [ ] Make absolutely sure the final watch progress is saved securely right before the video window disappears.
- [ ] Return the user automatically to the episode list once the movie stops.

**19.19 VLC Version Logging: Displays detected VLC version in Settings for debugging.** (Incomplete)

- [ ] Ask the video player quietly in the background what version it is currently running.
- [ ] Read the response cleanly to extract just the version number (like '3.0.18').
- [ ] Show this number in small, faint text at the bottom of the settings page for easy troubleshooting.
- [ ] Handle things calmly without crashing if the video player refuses to share its version number.
- [ ] Update this version number automatically if the user points the app to a different video player folder.

**19.20 Portable VLC Support: Resolves relative paths (./VLC/vlc.exe) for users running off USB drives.** (Incomplete)

- [ ] Notice if the user types a folder path that looks like it belongs on a portable USB drive.
- [ ] Combine that portable path intelligently with wherever the tracking app is currently located.
- [ ] Check to make sure the video player actually exists in that folder before trying to open it.
- [ ] Save the path specifically in its portable format so the app still works if the USB drive gets a different letter next time.
- [ ] Give the user a friendly heads-up if they accidentally type the folder name wrong.

## 🎭 Part 20: Deep Metadata & Rich Details (Incomplete)

**20.1 Cast & Crew Fetching: Pull the top 5 billed actors from TMDB.** (Incomplete)

- [ ] Download specifically the primary five actors assigned to the movie or television show.
- [ ] Display their real names prominently right below the main description paragraph.
- [ ] Handle situations calmly where the database only knows about two or three actors for older or obscure media.
- [ ] Ignore massive lists of hundreds of background extras to keep the page clean and readable.
- [ ] Update the actor list seamlessly if the user clicks the refresh button to grab newer information.

**20.2 Actor Profile UI: Display actors in a small horizontal row of circular profile pictures.** (Incomplete)

- [ ] Show tiny, round portrait photos for each of the top billed actors.
- [ ] Provide a neat, dark placeholder silhouette if the actor doesn't have a photo available.
- [ ] Put the actor's real name and the name of the character they play in small text directly under their photo.
- [ ] Make sure the row of pictures fits on the screen without forcing the user to scroll sideways.
- [ ] Give each photo a very slight, beautiful glow when the user hovers their mouse over it.

**20.3 Network/Studio Logos: Fetch and display transparent white logos for HBO, Netflix, Apple TV+, etc.** (Incomplete)

- [ ] Look up which company originally created or aired the television show.
- [ ] Find a clean, high-quality version of their logo that has no solid background color behind it.
- [ ] Display the logo neatly in pure white so it matches the dark cinematic theme of the app.
- [ ] Shrink incredibly wide logos down appropriately so they don't dominate the top of the screen.
- [ ] Skip showing a logo if the database doesn't have a clean, high-quality version available.

**20.4 Content Rating Badges: Display official maturity ratings (TV-MA, R, PG-13) in visually distinct pill borders.** (Incomplete)

- [ ] Place a crisp, easy-to-read rating box right next to the show's release year and length.
- [ ] Draw a sharp white outline around the letters to mimic standard television rating symbols.
- [ ] Show absolutely nothing if the show or movie has never been officially rated by a board.
- [ ] Choose the correct country's rating system automatically based on the user's computer settings.
- [ ] Keep the badge small but bold so it provides information quickly without being distracting.

**20.5 Creator/Director Tags: "Created by Vince Gilligan" text block under the synopsis.** (Incomplete)

- [ ] Identify who directed the movie or created the television series.
- [ ] Write their name clearly in a slightly lighter, silver color beneath the main story summary.
- [ ] Add the correct title prefix, like 'Directed by' for movies or 'Created by' for shows.
- [ ] Combine names neatly with an ampersand if two or more people share the exact same role.
- [ ] Hide this line if the original creator information is missing from the database.

**20.6 Trailer Integration: A "Trailer" button that fetches the TMDB YouTube key and opens a sleek iframe modal.** (Incomplete)

- [ ] Add a prominent, beautiful button that says 'Watch Trailer' right next to the main 'Play' button.
- [ ] Open a dark, centered popup window immediately when the user clicks the button.
- [ ] Play the official YouTube trailer automatically inside the popup without sending the user to a different website.
- [ ] Give the user an easy, obvious way to close the trailer and return to what they were doing.
- [ ] Hide the trailer button if no video link can be found for the show.

**20.7 Next Episode Countdown: For currently airing shows, displays: "Next Episode Airs in: 3 days, 4 hours."** (Incomplete)

- [ ] Check when the very next brand new episode is scheduled to appear on television.
- [ ] Calculate the exact amount of time left between right now and that specific future date.
- [ ] Show a highly visible, live countdown clock near the top of the show's page.
- [ ] Update the words intelligently so it says 'Airs Today' if the episode comes out in less than 24 hours.
- [ ] Remove the countdown once the episode officially airs or if the show is permanently finished.

**20.8 Poster Toggle: A switch to view the unique "Season Poster" instead of the primary "Show Poster" when browsing season tabs.** (Incomplete)

- [ ] Let users flip a simple switch to see the unique artwork created just for that specific season.
- [ ] Swap the giant main poster on the left side of the screen instantly when they change seasons.
- [ ] Fade smoothly between the two different images so it feels polished and high-end.
- [ ] Stick to the main show poster automatically if the specific season doesn't have its own unique artwork.
- [ ] Remember their choice specifically so they don't have to flip the switch every single time.

**20.9 Guest Star Data: Dropdown arrow on episodes to see notable guest stars.** (Incomplete)

- [ ] Add a very small, subtle arrow icon onto individual episode rows.
- [ ] Slide the row open gently when clicked to reveal a neat list of special guest actors for that episode.
- [ ] Show the actor's real name and the name of their temporary character.
- [ ] Hide the drop-down arrow if the episode features absolutely no special guests.
- [ ] Close the list smoothly if the user clicks the arrow a second time or opens a different episode.

**20.10 Season-Level Synopsis: Display the unique text overview for a specific season above the episode list.** (Incomplete)

- [ ] Find the special paragraph that summarizes what happens during this specific season of the show.
- [ ] Display this text cleanly above the first episode in the list.
- [ ] Provide a 'Read More' button if the summary is incredibly long so it doesn't push the episodes off the screen.
- [ ] Skip showing anything if the database only has a summary for the whole show and not this exact season.
- [ ] Update the text instantly the exact moment the user clicks a different season tab.

**20.11 Interactive Genre Tags: Clicking a genre pill (e.g., "Sci-Fi") instantly routes to the Library pre-filtered for that genre.** (Incomplete)

- [ ] Turn the tiny genre labels, like 'Action' or 'Comedy', into fully clickable buttons.
- [ ] Jump the user straight back to their main movie library the exact second they click one.
- [ ] Filter the entire library automatically so they only see other movies that share that same genre.
- [ ] Show a clear, friendly message confirming which genre they are currently looking at.
- [ ] Let them easily cancel the filter with one click to see their entire collection again.

**20.12 Similar/Recommended Row: At the very bottom of the details page, show 5 dynamic posters of "If you liked this..."** (Incomplete)

- [ ] Ask the database which other shows or movies are remarkably similar to the one they are looking at.
- [ ] Show a beautiful horizontal row of five movie posters at the very bottom of the page.
- [ ] Check to see if the user already has any of these recommended shows in their own library.
- [ ] Put a tiny, helpful green checkmark on the posters of the shows they already own.
- [ ] Let them click any of the posters to instantly open the details page for that new recommendation.

**20.13 Micro-Refresh: Option to right-click and "Refresh Data" for a single episode rather than the whole show.** (Incomplete)

- [ ] Give the user an option to update the information specifically for just one single episode.
- [ ] Add this choice neatly to the beautiful custom right-click menu on the episode row.
- [ ] Update the title, description, and thumbnail picture instantly without reloading the rest of the page.
- [ ] Show a tiny spinning circle on just that one row so the user knows it is thinking.
- [ ] Handle it gracefully if the database still doesn't have any new information to provide.

**20.14 Precise Runtime Formatting: Format 135 minutes as 2h 15m instead of just 135m.** (Incomplete)

- [ ] Take the total number of minutes a movie lasts and break it down into exact hours and minutes.
- [ ] Write the new time beautifully, like '2h 15m', so it is instantly easier for a human to read.
- [ ] Drop the hour if the video is extremely short, like a 22-minute television episode.
- [ ] Drop the minutes if the movie happens to be two hours long.
- [ ] Show 'Unknown' if the file length is missing or broken.

**20.15 Local File Size UI: Extract and display file size (e.g., 1.2 GB) from the OS.** (Incomplete)

- [ ] Ask the user's computer how much hard drive space the video file is taking up.
- [ ] Round the number beautifully so it reads like '1.2 GB' instead of a massive string of random numbers.
- [ ] Place this information neatly in a small, dim font right next to the file path on the screen.
- [ ] Update the number immediately if the user replaces the old file with a much larger, higher-quality version.
- [ ] Show 'File Missing' clearly if the app fails to find the video on the hard drive.

**20.16 Resolution Tagging: Extract and display 1080p or 4K from the local filename string.** (Incomplete)

- [ ] Look specifically at the name of the video file to see if it mentions how high-quality the video is.
- [ ] Look for common keywords like '1080p', '720p', or '4K' right in the file name.
- [ ] Create a tiny, bright badge next to the episode title displaying this video quality.
- [ ] Ensure the badge looks distinct from standard maturity ratings or genre tags.
- [ ] Hide the badge if the file name gives absolutely no hints about the video quality.

**20.17 Audio Codec Tagging: Extract AAC or 5.1 from the local filename string.** (Incomplete)

- [ ] Read the specific file name again to see if it mentions anything related to the sound quality.
- [ ] Find very specific audio keywords like '5.1', '7.1', or 'AAC' neatly hidden in the text.
- [ ] Add a second tiny badge right next to the video quality badge to show off the audio format.
- [ ] Make sure this little badge doesn't push the episode title awkwardly off the edge of the screen.
- [ ] Don't show anything at all if the file name is silent on audio details.

**20.18 "Copy Path" Quick Action: Right-click an episode to copy the raw C:\... path to clipboard.** (Incomplete)

- [ ] Add a brand new 'Copy File Path' button to the beautiful custom right-click menu.
- [ ] Save the exact, complete folder path of the video directly to the user's invisible computer clipboard when clicked.
- [ ] Show a quick, tiny popup message confirming 'Copied to Clipboard!' so they know it worked.
- [ ] Ensure the path includes the exact drive letter and every single subfolder.
- [ ] Disable the button if the file is currently marked as missing or deleted.

**20.19 "Show in Explorer" Action: Right-click to open native OS file manager with the file highlighted.** (Incomplete)

- [ ] Add a handy 'Open Folder' button right next to the copy button in the right-click menu.
- [ ] Open the user's actual computer file browser to the folder where the video is hiding.
- [ ] Highlight the exact video file automatically so the user doesn't have to search for it among hundreds of other files.
- [ ] Make sure this works flawlessly on both Windows computers and Mac computers.
- [ ] Show a polite error explaining the issue if the folder has been renamed or moved.

**20.20 Metadata Warning Icon: A tiny yellow ! if an episode exists but TMDB returned absolutely zero data for it.** (Incomplete)

- [ ] Notice if an episode has a video file ready to play but zero information from the internet database.
- [ ] Place a very small, bright yellow warning triangle next to the episode title.
- [ ] Show a small text box explaining 'Information Missing' when the user hovers over the triangle.
- [ ] Remove the warning triangle instantly if the user successfully uses the refresh button to find the missing details.
- [ ] Keep the play button functional so the user can still watch their show even without a summary.

## 📚 Part 21: Advanced Library Organization (Incomplete)

**21.1 Compact List Toggle: A button to switch the Poster Grid into a dense, text-heavy data table view.** (Incomplete)

- [ ] Add an icon button in the top right corner to flip the library layout.
- [ ] Hide the movie posters and switch to an organized list of text rows.
- [ ] Show important details like release year, rating, and watch progress next to each title.
- [ ] Keep the list easy to read by adding alternating background colors to every other row.
- [ ] Remember the user's layout choice so they don't have to click the button every time.

**21.2 Adjustable Poster Sizing: A slider in the top bar to scale the grid (Small, Medium, Large posters).** (Incomplete)

- [ ] Place a subtle sliding bar next to the layout button to control picture size.
- [ ] Shrink or grow all movie posters in real time as the user drags the slider.
- [ ] Rearrange the posters automatically to fit the new sizes onto the screen.
- [ ] Set strict limits so the posters never become impossibly tiny or huge.
- [ ] Save this size preference silently so it stays consistent the next time they log in.

**21.3 Genre Filtering Dropdown: Multi-select checkboxes to filter library by "Action AND Comedy".** (Incomplete)

- [ ] Build a dropdown menu filled with every movie and television genre.
- [ ] Let users check multiple boxes at the same time to mix and match their search.
- [ ] Filter the movie posters instantly the second a new box is checked or unchecked.
- [ ] Show a polite, friendly message if the chosen combination results in zero movies.
- [ ] Add a quick 'Clear' button inside the menu to instantly uncheck every box.

**21.4 Status Filtering: Filter library by "Ended" or "Returning Series".** (Incomplete)

- [ ] Add a simple filter option letting users look only at television shows that are finished.
- [ ] Provide another option for shows that are actively still airing new episodes.
- [ ] Hide movies from the screen whenever these television-specific filters are turned on.
- [ ] Combine this filter seamlessly with other choices, like finding a 'Finished' show that is also a 'Comedy'.
- [ ] Handle situations gracefully where a show's status is unknown.

**21.5 Missing Files Filter: View tracked shows where local files have been deleted/moved.** (Incomplete)

- [ ] Create a specific filter button dedicated to finding broken or missing video files.
- [ ] Show only the movie posters for episodes that the app can no longer find on the computer.
- [ ] Make it incredibly easy for the user to select these broken shows and remove them or fix them.
- [ ] Hide the standard 'Play' buttons on these posters to prevent annoying errors.
- [ ] Empty this filter list instantly the second the user plugs their external hard drive back in.

**21.6 Favorites Filter: 1-click quick filter to only show 5-star rated media.** (Incomplete)

- [ ] Place a shiny star icon button prominently next to the main search bar.
- [ ] Click the button to instantly hide absolutely everything that doesn't have a perfect five-star rating.
- [ ] Keep the user's sorting choices the same while the filter is active.
- [ ] Turn the star button brightly orange while the filter is turned on so it is obvious.
- [ ] Let the user click the bright star a second time to instantly return to their normal library.

**21.7 Decade Filter: Group shows by 80s, 90s, 2000s, 2010s, 2020s.** (Incomplete)

- [ ] Add a fun, simple dropdown letting users jump back in time to their favorite movie eras.
- [ ] Find the release year for every movie and sort them into these ten-year buckets.
- [ ] Show only the posters that fit into the chosen decade.
- [ ] Make sure television shows are sorted based on the year their very first episode aired.
- [ ] Handle incredibly old classic movies without breaking the layout or the math.

**21.8 "Pick a Show for Me" Button: A dice icon that randomly selects an unwatched/in-progress show.** (Incomplete)

- [ ] Put a neat little dice icon at the very top of the main screen for indecisive viewers.
- [ ] Pick one random movie or television show from their personal collection when clicked.
- [ ] Ignore shows that the user has already finished to keep the choice fresh.
- [ ] Open the detailed page for the chosen show instantly so they can start watching.
- [ ] Ensure the app genuinely picks randomly every time instead of getting stuck on one show.

**21.9 "Play Random Episode": A specific button for Sitcoms to launch a random completed episode (e.g., The Office).** (Incomplete)

- [ ] Add a special 'Shuffle' button to the details page of television shows.
- [ ] Pick one random episode from the list of episodes the user has already watched.
- [ ] Launch the video player instantly with that specific episode the second the button is clicked.
- [ ] Skip over any episodes that are missing their video file so the user doesn't hit an error.
- [ ] Hide this button on movies, since shuffling a single movie doesn't make sense.

**21.10 A-Z Index Dividers: When sorted alphabetically, visual horizontal dividers separate the 'A's from the 'B's.** (Incomplete)

- [ ] Draw a beautiful, clean line across the screen where the starting letters change.
- [ ] Place a giant, elegant letter on the left side of the line, like a bold 'A' or 'B'.
- [ ] Make sure these dividers appear only when the user is sorting their library by name.
- [ ] Group numbers or special characters together under one single '#' divider at the very top.
- [ ] Keep the posters directly underneath the dividers aligned in their normal grid shape.

**21.11 Year Index Dividers: When sorted by Release Date, dividers separate 2024 from 2023.** (Incomplete)

- [ ] Draw the same beautiful dividing lines when the user sorts their movies from newest to oldest.
- [ ] Put the four-digit year prominently right on the dividing line so it is easy to read.
- [ ] Hide these dividers if the user switches to a different sorting method like highest rated.
- [ ] Handle movies that are missing release years by grouping them neatly under an 'Unknown' divider at the bottom.
- [ ] Stop the dividers from appearing if the user has filtered the list down to only a handful of posters.

**21.12 Persistent View State: The app remembers if you sorted Movies by Rating and TV by Added, saving it to LocalStorage.** (Incomplete)

- [ ] Save the user's sorting choice in the background every time they change it.
- [ ] Remember the settings for the Television library separate from the Movie library.
- [ ] Load these settings silently the very next time the user opens the application.
- [ ] Apply the same memory to whether the user prefers the grid view or the compact list view.
- [ ] Make sure this invisible memory feature never accidentally slows down or breaks the app's loading speed.

**21.13 "Clear All Filters" Pill: A floating action button that appears when any complex filters are active.** (Incomplete)

- [ ] Watch carefully to see if the user has turned on more than one tricky filter at the exact same time.
- [ ] Pop a beautiful, bright button onto the top of the screen offering to clear everything.
- [ ] Wipe every single filter away instantly when the user clicks the button.
- [ ] Return the movie library flawlessly back to its normal, unfiltered state.
- [ ] Hide the bright button instantly the second the library goes back to normal.

**21.14 Results Counter: Subtle text stating "Showing 42 of 150 items".** (Incomplete)

- [ ] Place a very tiny, dim line of text at the top corner of the movie library.
- [ ] Update the numbers instantly every time a new filter is clicked or a search is typed.
- [ ] Count precisely how many total movies exist compared to what is currently visible.
- [ ] Keep the text silent and invisible if the user is simply looking at their entire collection.
- [ ] Make sure the numbers never accidentally overlap over any important buttons or posters.

**21.15 Intersection Observer Rendering: Posters off-screen are replaced by empty divs to conserve DOM memory.** (Incomplete)

- [ ] Watch silently to see which movie posters disappear off the top or bottom of the screen.
- [ ] Remove the heavy picture from the computer's memory to keep the app running fast.
- [ ] Leave a totally invisible, sized empty box in its place so the scrolling doesn't jump or break.
- [ ] Put the picture right back into the box instantly before the user scrolls back to it.
- [ ] Make sure the user never accidentally sees the blank boxes while scrolling normally.

**21.16 Image Retry Logic: If a local cached image is corrupted, automatically attempt to re-download it from TMDB.** (Incomplete)

- [ ] Notice instantly if a movie poster picture file is totally broken or won't load properly.
- [ ] Try quietly in the background to download a fresh copy of the picture from the internet.
- [ ] Replace the broken picture instantly on the screen the exact second the new one finishes downloading.
- [ ] Stop trying permanently if the internet is down so the app doesn't freeze or crash.
- [ ] Show the neat, beautiful gray placeholder box while the app is silently fixing the picture.

**21.17 Type Iconography: In "All Search Results", overlay a tiny Movie clapperboard or TV icon to distinguish media types.** (Incomplete)

- [ ] Look closely at mixed search results to see what kind of media they are.
- [ ] Place a tiny, incredibly cute television icon in the corner of all the television shows.
- [ ] Place a tiny movie clapperboard icon in the corner of all the standard feature films.
- [ ] Keep these icons small so they don't cover up the actual title of the movie.
- [ ] Hide the icons if the user is already specifically browsing just their TV or Movie libraries.

**21.18 "Unwatched Only" Filter: Distinct from "Hide Completed"—this specifically hides anything you've started.** (Incomplete)

- [ ] Add a brand new checkbox separate from the standard 'Hide Completed' button.
- [ ] Hide every single movie or show that the user has even watched five minutes of.
- [ ] Leave only the totally fresh, absolutely untouched movies on the screen.
- [ ] Help the user easily find something brand new they haven't started yet.
- [ ] Keep this filter compatible with all the other genre or sorting tools.

**21.19 Library Multi-Select State: Allow users to Ctrl+Click multiple posters to Bulk Remove.** (Incomplete)

- [ ] Let users hold down the Ctrl or Cmd key and click on several different movie posters.
- [ ] Highlight every single poster they click so it is obvious they are selected.
- [ ] Pop a new menu at the top of the screen offering a massive 'Delete All' button.
- [ ] Remove every single chosen movie the instant the big button is pressed.
- [ ] Un-highlight everything the exact second the user clicks anywhere else on the page.

**21.20 "Pin to Top" Feature: Right-click a show to pin it, ensuring it stays at the top of the grid regardless of sort order.** (Incomplete)

- [ ] Add a neat 'Pin to Top' button directly inside the custom right-click menu.
- [ ] Move that specific movie poster to the absolute very first spot in the entire library.
- [ ] Keep it locked in that number one spot even if the user changes how the list is sorted.
- [ ] Add a tiny little thumbtack icon to the corner of the poster so it is obvious.
- [ ] Un-pin the poster the second the user clicks the button again.

## 🕵️ Part 22: Scanner & Parsing Intelligence (Incomplete)

**22.1 Multi-Root Directories: Support adding multiple target folders (e.g., D:\TV Shows and E:\Anime).** (Incomplete)

- [ ] Create a neat, organized list in the settings showing every single folder the app currently watches.
- [ ] Add a prominent button letting users easily pick a new folder from any hard drive on their computer.
- [ ] Let users quickly and safely remove a folder from the list without permanently deleting any of their history.
- [ ] Scan flawlessly through all the listed folders simultaneously without mixing up the movies inside them.
- [ ] Show a polite warning if a user accidentally tries to add the exact same folder twice.

**22.2 .watchmarkignore Support: Place this file in a folder to tell the scanner to skip it entirely.** (Incomplete)

- [ ] Teach the app to quietly look for a tiny, hidden file named `.watchmarkignore` inside any folder.
- [ ] Skip the entire folder and absolutely everything inside it instantly if that special file is found.
- [ ] Ignore any sub-folders hiding inside the ignored folder.
- [ ] Help users easily hide personal home videos or private collections without moving them off the hard drive.
- [ ] Keep the app fast and silent while it skips over these ignored sections.

**22.3 Sample File Exclusion: Automatically ignore video files under 50MB (bypasses trailers/samples).** (Incomplete)

- [ ] Check how large every single video file is before trying to identify it.
- [ ] Skip over tiny files that are just short preview clips or downloading errors.
- [ ] Put a simple slider directly in the settings so the user can change what 'tiny' means to them.
- [ ] Keep the main movie library clean by keeping these junk files out.
- [ ] Process the large, actual movie files normally even if they sit right next to a tiny preview file.

**22.4 Absolute Number Parsing: Regex logic to understand Anime formatting (e.g., Naruto 105.mkv -> Season 1, Ep 105).** (Incomplete)

- [ ] Teach the app to recognize when a video file just has one giant number, like '105'.
- [ ] Map these massive numbers back to 'Season 1' so the television library doesn't break.
- [ ] Stop the app from accidentally thinking the number '105' means 'Season 1, Episode 5'.
- [ ] Identify the show title correctly even when it sits right next to these strange numbering formats.
- [ ] Match the episode with the correct internet summary information even if the numbering is weird.

**22.5 Date-Based Parsing: Regex for daily shows (e.g., Late Show 2024-03-10.mkv).** (Incomplete)

- [ ] Recognize when a file name uses a full calendar date instead of normal season numbers.
- [ ] Pull the exact year, month, and day out of the file name cleanly.
- [ ] Search the internet database using that exact date to find the specific daily talk show episode.
- [ ] Keep the show title separate from the date so the app knows what to search for.
- [ ] Organize these daily episodes inside the app in the correct calendar order.

**22.6 Boot-Up Auto-Scan: Optional setting to run a silent background scan every time WatchMark opens.** (Incomplete)

- [ ] Add a simple checkbox in the settings offering to look for new movies the second the app starts.
- [ ] Run the search silently in the background so the user can start using the app instantly.
- [ ] Show a tiny, subtle spinning icon out of the way so the user knows the app is thinking.
- [ ] Pop a polite notification when the invisible search finishes, if it found new things.
- [ ] Let users turn this off if they prefer to control when the app searches their computer.

**22.7 File System Watcher (Rust notify): Hooks into OS events to instantly recognize when a file is dragged into your watched folder without manual scanning.** (Incomplete)

- [ ] Listen quietly to the user's computer to hear when a brand new file is created.
- [ ] Add the brand new movie instantly to the app the second it finishes downloading or copying.
- [ ] Prevent the app from trying to read the file before it is finished copying over.
- [ ] Notice if the user decides to suddenly delete a movie or move it to a different folder.
- [ ] Keep the main library up-to-date instantly without the user ever needing to click a refresh button.

**22.8 Detailed Scan Log Modal: Shows what Regex matched what string and what TMDB ID was assigned.** (Incomplete)

- [ ] Create a special, detailed history page showing what the app did during its last search.
- [ ] List every single file it found and explain how it guessed the title.
- [ ] Show specifically which internet movie ID it decided to attach to the file.
- [ ] Highlight which files the app failed to understand so the user can fix them.
- [ ] Keep this detailed screen hidden inside the settings so it doesn't confuse normal users.

**22.9 "Dry Run" Scan: Simulates a scan and shows you what it would match without writing to the database.** (Incomplete)

- [ ] Build a safe, practice search button that doesn't save anything permanently.
- [ ] Show the user a detailed list predicting what the app would do.
- [ ] Let the user check to see if their complicated folder names will confuse the app.
- [ ] Keep the actual, permanent movie library untouched during this practice test.
- [ ] Give the user a big, obvious button to make the changes permanent if they like the results.

**22.10 Duplicate Resolution: If S01E01 is found twice, UI allows user to pick between the 1080p and 720p version.** (Incomplete)

- [ ] Notice if the exact same episode is hiding twice inside the user's folders.
- [ ] Pause the automatic process and pop a helpful little message asking the user for help.
- [ ] Show the user both file names so they can clearly see the difference in quality.
- [ ] Let the user click which version they want to keep permanently in their library.
- [ ] Remember their choice so the app doesn't bother them about those specific files again.

**22.11 Incomplete Download Skipping: Ignores.part,.crdownload, and.!qB files instantly.** (Incomplete)

- [ ] Ignore any file that has an ending matching common downloading programs.
- [ ] Skip over them silently so the user's library doesn't fill up with broken, unplayable junk.
- [ ] Notice the second the downloading program finishes and renames the file normally.
- [ ] Add the finally completed file to the library instantly once it is ready.
- [ ] Prevent the app from crashing if it tries to read a half-finished file.

**22.12 Auto-Cleanup: If a show is removed from Tracker, optionally prompt "Delete empty parent folders?".** (Incomplete)

- [ ] Notice when the user removes an entire show from the app.
- [ ] Check if the folder that used to hold that show is now totally, empty.
- [ ] Pop a polite, optional message asking if the user wants to delete the empty folder too.
- [ ] Leave the folder alone if the user clicks 'No'.
- [ ] Keep the user's computer clean without them having to do it themselves.

**22.13 Split-Movie Merging: Detects Movie-CD1.avi and Movie-CD2.avi and handles them logically.** (Incomplete)

- [ ] Recognize when a single long movie is broken into two separate video files.
- [ ] Group both parts under one single, beautiful poster in the main library.
- [ ] Start playing the second part automatically the second the first part finishes.
- [ ] Remember where the user paused even if the pause happens across the split.
- [ ] Keep the library clean by hiding the confusing duplicate files from the main screen.

**22.14 Archive Detection: Flags.rar or.zip files containing video and notifies the user to extract them.** (Incomplete)

- [ ] Notice when a massive video file is hidden inside a compressed folder.
- [ ] Pop a helpful warning letting the user know they need to unzip it.
- [ ] Prevent the app from trying to force the video player to play a zipped file.
- [ ] Stop bothering the user about the zip file once they successfully extract the video.
- [ ] Keep the app fast by not trying to scan inside every single zip file on the computer.

**22.15 Fallback Folder Parsing: If a file is named 1.mkv, the parser crawls up to read the parent directory name Breaking Bad.** (Incomplete)

- [ ] Notice when a video file name is too short to be a real title.
- [ ] Look at the name of the folder holding the file to find the real show name.
- [ ] Combine the folder name with the file number to figure out what the episode is.
- [ ] Add the episode to the library as if the file name had been normal.
- [ ] Handle situations where the folder name is also useless or confusing.

**22.16 Inbox Sort by Date: Sort unmatched files by the OS "Date Modified" attribute.** (Incomplete)

- [ ] Give the user a simple button to sort their confusing files by age.
- [ ] Put the absolutely newest, most recently downloaded files at the top of the list.
- [ ] Help the user figure out what a file is by remembering when they downloaded it.
- [ ] Refresh the list instantly when the button is clicked.
- [ ] Keep the list organized even if the user has thousands of confusing files.

**22.17 Inbox Sort by Size: Sort unmatched files by byte size.** (Incomplete)

- [ ] Add a button to sort the confusing files by how massive they are.
- [ ] Group massive 4K movies together at one end of the list.
- [ ] Group tiny preview clips at the other end.
- [ ] Help the user identify what a file is by looking at its size.
- [ ] Keep the sorting fast even if the user has a massive hard drive.

**22.18 Inbox Fuzzy Matching: Groups strings with minor typos together (e.g., "The Wrie" and "The Wire").** (Incomplete)

- [ ] Notice when two file names are almost identical.
- [ ] Group them together in the confusing files list even if one has a small typo.
- [ ] Help the user fix both files at the exact same time.
- [ ] Stop grouping files together if they are actually totally different shows.
- [ ] Make the confusing files list much smaller and easier to manage.

**22.19 Inbox Inline Editing: Allow the user to manually edit the extracted string in the UI before hitting "Search TMDB".** (Incomplete)

- [ ] Let the user click right on the guessed title in the confusing files list.
- [ ] Turn the text into a totally normal typing box so they can fix spelling mistakes.
- [ ] Search the internet the second they finish typing the fixed name.
- [ ] Save the user from opening a massive separate popup window.
- [ ] Keep the typing box explicit simple and easy to use.

**22.20 Inbox Type Override: A quick toggle to force TMDB to search for a "Movie" if the parser incorrectly guessed "TV".** (Incomplete)

- [ ] Add a tiny, perfect simple button next to the confusing file.
- [ ] Let the user flip between 'Television' and 'Movie'.
- [ ] Search the internet again instantly using the brand new category.
- [ ] Help the user fix mistakes when a movie happens to have a number in its name.
- [ ] Keep the button explicit out of the way unless the user needs it.

## 📊 Part 23: Dashboard Insights & Statistics (Incomplete)

**23.1 "Finish the Season" Prompt: If 1 episode is left in a season, the Hero text changes to highlight it.** (Incomplete)

- [ ] Calculate precisely how many unwatched episodes remain in the active season for the featured show.
- [ ] Change the standard subtitle text to a bold, encouraging message when only one episode is left.
- [ ] Highlight this special text in a distinct color to grab the user's attention.
- [ ] Revert immediately to standard text formatting if the user watches the final episode or starts a different show.
- [ ] Make sure this prompt ignores special episodes or trailers that aren't part of the main story.

**23.2 "New Season Premiered" Alert: If an archived show is updated on TMDB with a new season, a temporary alert card appears on the Dashboard.** (Incomplete)

- [ ] Notice quietly in the background if a television series the user finished suddenly receives brand new episodes.
- [ ] Display a prominent, exciting notification card right at the top of the main screen.
- [ ] Show the title of the series and how many new episodes are now available to watch.
- [ ] Let the user dismiss this alert permanently with a single click if they aren't interested.
- [ ] Automatically remove the card the second the user actually starts watching one of the new episodes.

**23.3 Recently Finished Row: A dynamic row showing the posters of the last 5 shows you hit 100% on.** (Incomplete)

- [ ] Track when a user watches the absolute final episode of an entire series.
- [ ] Build a sleek, horizontal carousel right on the main screen displaying these completed accomplishments.
- [ ] Limit the list to only the five most recent shows to keep the screen from getting cluttered.
- [ ] Hide this row if the user has never finished a television show.
- [ ] Make sure adding a new season to a finished show drops it off this list until they finish it again.

**23.4 "On This Day" Memory: Shows a card highlighting what you binged 1, 2, or 5 years ago today.** (Incomplete)

- [ ] Look back through the user's personal viewing history to find what they watched on this exact calendar date in previous years.
- [ ] Create a fun, nostalgic memory card displaying the movie poster and how many years ago it was watched.
- [ ] Skip this feature if the user hasn't been using the app long enough to have any yearly memories.
- [ ] Prioritize massive binge sessions or highly-rated movies if multiple things were watched on that specific day.
- [ ] Let the user click the memory card to jump straight to that show's detail page.

**23.5 Time Saved Statistic: Math calculation showing hours saved if you consistently used the "Skip Intro" manual offset.** (Incomplete)

- [ ] Add up all the seconds the user has automatically skipped across every single show they watch.
- [ ] Convert that massive number of seconds into an easy-to-read format, like '12 Hours Saved'.
- [ ] Display this fun statistic proudly in the main insights grid on the dashboard.
- [ ] Keep the number hidden until the user has actually saved at least one full hour of time.
- [ ] Update the math accurately in real time every time an episode finishes playing.

**23.6 Time-Aware Greetings: Dashboard top text reads "Good morning" or "Late night binge?" based on local OS clock.** (Incomplete)

- [ ] Read the current time directly from the user's computer clock.
- [ ] Change the main welcome text dynamically depending on whether it is morning, afternoon, evening, or past midnight.
- [ ] Make the late-night greeting playful to acknowledge that the user is staying up late watching movies.
- [ ] Ensure the greeting updates seamlessly if the user leaves the app open across different times of the day.
- [ ] Keep the font elegant and subtle so it doesn't distract from the giant movie posters.

**23.7 Dynamic Queue Hiding: If you have 0 shows in progress, the "Up Next" row collapses rather than showing empty space.** (Incomplete)

- [ ] Notice immediately if the user has absolutely nothing currently paused or halfway finished.
- [ ] Remove the entire 'Continue Watching' row from the screen so it doesn't waste space.
- [ ] Slide all the other dashboard sections up smoothly to fill in the empty gap.
- [ ] Bring the row back instantly the second the user starts a brand new movie.
- [ ] Avoid showing ugly 'Nothing to watch' text boxes in this specific row.

**23.8 Interactive Stat Widgets: Clicking the "Shows Completed" widget automatically navigates to the Library with the 'Completed' filter applied.** (Incomplete)

- [ ] Turn the big, numbered statistic boxes on the dashboard into clickable buttons.
- [ ] Jump the user straight to their movie collection when they click one of the numbers.
- [ ] Automatically turn on the correct filters so the list matches the statistic they clicked.
- [ ] Add a very slight visual highlight when the mouse hovers over the boxes so they feel interactive.
- [ ] Make sure the 'Back' button works properly to return them to the dashboard after clicking.

**23.9 Hero Progress Color Shift: The progress bar turns solid Green when over 90%, distinguishing it from Orange (in-progress).** (Incomplete)

- [ ] Watch the progress bar percentage closely while the user is watching an episode.
- [ ] Change the color of the bar instantly from orange to bright green the second they pass the 90 percent mark.
- [ ] Make this color shift extremely smooth so it looks like a natural transition.
- [ ] Keep the bar green permanently for that specific episode to indicate it is considered finished.
- [ ] Ensure this color logic matches the math used for the app's history tracking.

**23.10 Hover-Timestamp Reveal: Hovering the Hero progress bar reveals the exact string: 45:12 / 50:00.** (Incomplete)

- [ ] Hide the exact minute and second numbers normally to keep the screen looking clean and cinematic.
- [ ] Pop up a tiny, dark tooltip showing the precise time remaining only when the user places their mouse over the bar.
- [ ] Format the numbers cleanly like a standard digital clock so they are incredibly easy to read at a glance.
- [ ] Make the tooltip follow the mouse left and right along the progress bar.
- [ ] Hide the numbers instantly again when the mouse moves away.

**23.11 "Dismiss from Queue" Action: An X button on Continue Watching cards to hide a show you got bored of but don't want to delete.** (Incomplete)

- [ ] Add a small, faint 'X' icon to the corner of the shows listed in the 'Continue Watching' row.
- [ ] Let the user click it to instantly remove that specific show from their active list.
- [ ] Ensure the show remains safely in their library and history, but stops bothering them on the main screen.
- [ ] Ask for a quick confirmation so they don't accidentally hide a show they actually wanted to watch.
- [ ] Provide an easy way inside the show's detail page to put it back into the queue if they change their mind.

**23.12 "Dropped" Database Status: A 4th status for shows you quit, preventing them from showing up in queues.** (Incomplete)

- [ ] Create a brand new category specifically for television series the user has decided to stop watching forever.
- [ ] Stop these abandoned shows from appearing in any 'Up Next' or 'Continue Watching' lists.
- [ ] Add a specific 'Mark as Dropped' button to the show's detail page.
- [ ] Color-code this status differently, perhaps with a subtle gray badge, so it is obvious the show was abandoned.
- [ ] Keep the show's previous history totally intact so the user still has a record of what they did watch.

**23.13 Live Playback Pulse: The card of the episode currently playing in VLC glows rhythmically on the Dashboard.** (Incomplete)

- [ ] Notice which episode is actively playing in the external video player right now.
- [ ] Add a very slow, gentle breathing animation to that specific episode's card on the dashboard.
- [ ] Use the primary accent color for the glow so it looks intentional and stylish.
- [ ] Stop the pulsing animation instantly the second the user hits pause or closes the video player.
- [ ] Ensure this animation is smooth and subtle enough not to distract the user if they are looking at other things.

**23.14 Total Local Storage Stat: Calculates the GB/TB size of all tracked media files combined.** (Incomplete)

- [ ] Add up the exact file size of every single movie and television episode the app is currently watching.
- [ ] Convert this massive number neatly into Gigabytes or Terabytes so it is easy to understand.
- [ ] Display this impressive number proudly in the main statistics grid on the dashboard.
- [ ] Update the math silently in the background whenever the user adds or deletes a video file.
- [ ] Handle missing files properly by not including them in the final total weight.

**23.15 Most Watched Genre Widget: Analyzes DB and shows a pie chart or text of your top genre.** (Incomplete)

- [ ] Look through every movie and show the user has ever finished to figure out their favorite type of story.
- [ ] Display the winning category, like 'Sci-Fi' or 'Comedy', prominently in a small dashboard box.
- [ ] Include a tiny, colorful visual chart showing how their other favorite genres stack up.
- [ ] Update this favorite category automatically as their viewing habits change over the years.
- [ ] Hide this box if the user hasn't watched enough things to establish a real pattern.

**23.16 Average Watch Time Widget: Calculates average daily media consumption in minutes.** (Incomplete)

- [ ] Figure out how many minutes of video the user watches on a typical day.
- [ ] Display this daily average cleanly on the dashboard alongside the other fun statistics.
- [ ] Format the number beautifully, like '1h 45m per day', instead of a confusing raw number of seconds.
- [ ] Ensure the math only looks at actual time spent watching, ignoring time where a video was just paused in the background.
- [ ] Don't let days where the user watched absolutely nothing ruin the math unfairly.

**23.17 Modular Dashboard Layout: Settings toggle to re-order dashboard rows (e.g., move Stats above Recently Added).** (Incomplete)

- [ ] Give the user a simple list in the settings menu showing every row on their main dashboard.
- [ ] Let them click and drag these rows up and down to change what order they appear in.
- [ ] Apply the new layout instantly the second they return to the main screen.
- [ ] Remember this custom order permanently so their dashboard always looks how they like it.
- [ ] Ensure the giant Hero banner is locked at the very top and cannot be accidentally moved down.

**23.18 "Upcoming Airing" Row: A row for tracked shows that have episodes airing in the next 7 days.** (Incomplete)

- [ ] Check the calendar to see if any television shows the user watches have brand new episodes coming out this week.
- [ ] Build a special, temporary row on the dashboard exclusively for these upcoming premieres.
- [ ] Show the movie poster and which day of the week the episode will be available.
- [ ] Hide this row if nothing the user watches is scheduled to air in the next seven days.
- [ ] Remove an episode from this row automatically the exact second the air date actually passes.

**23.19 Collapsible Dashboard Sections: Chevron icons to minimize rows you don't care about.** (Incomplete)

- [ ] Add a tiny, subtle arrow icon to the title text of every single row on the dashboard.
- [ ] Let the user click the arrow to hide all the movie posters inside that specific row instantly.
- [ ] Slide the rest of the page up smoothly to fill in the space where the posters used to be.
- [ ] Remember which rows are hidden permanently so they stay closed the next time the app opens.
- [ ] Flip the arrow upside down so it is obvious the user can click it again to bring the posters back.

## 📅 Part 24: Diary & History Refinements (Incomplete)

**24.1 Inline Date Editing: A pencil icon on history entries to manually correct the day/time.** (Incomplete)

- [ ] Add a small pencil icon next to the time on every single history row.
- [ ] Change the text into a simple date and time picker when the pencil is clicked.
- [ ] Update the timeline instantly to move the row to its new proper date section.
- [ ] Stop the user from setting a date that is in the future.
- [ ] Remember to update any binge blocks if the new time separates it from the group.

**24.2 Non-Destructive Deletion: An X button to delete a specific watch event from history without altering the global "Completed" status.** (Incomplete)

- [ ] Add a tiny trash can or 'X' icon specifically to history rows.
- [ ] Remove the row from the timeline when clicked so it is gone.
- [ ] Keep the episode marked as finished in the main library even after deleting the row.
- [ ] Ask for a quick confirmation so the user doesn't delete a memory by accident.
- [ ] Update the daily watch time numbers to reflect that the episode is gone.

**24.3 Daily Total Calculations: Next to the "Today" header, show (3h 45m) representing total time watched that specific day.** (Incomplete)

- [ ] Add up the runtimes of every single episode watched on a specific date.
- [ ] Display this total time neatly right next to the date header, like 'Monday (2h 15m)'.
- [ ] Skip showing the time if it adds up to zero.
- [ ] Use the actual lengths of the video files instead of estimates when possible.
- [ ] Update the number immediately if the user deletes a history row from that day.

**24.4 Monthly Wrap-Up: At the top of a Monthly group, display "24 Episodes, 3 Movies Watched".** (Incomplete)

- [ ] Group the history neatly by month when the user scrolls far back enough in time.
- [ ] Count exactly how many movies and television episodes were finished in that month.
- [ ] Display a beautiful summary header at the start of the month, like 'October 2023 Summary'.
- [ ] Keep the summary hidden if the user only watched one or two things that month.
- [ ] Separate the movie count from the television count so the numbers are clear.

**24.5 Calendar Picker Filter: A date-range picker input to filter history between specific weeks.** (Incomplete)

- [ ] Add a calendar button to the top of the history page to open a date selector.
- [ ] Let the user pick a specific start date and a specific end date.
- [ ] Hide all history rows that fall outside of that chosen window of time.
- [ ] Provide quick preset buttons like 'Last Week' or 'Last Month'.
- [ ] Give the user an easy way to clear the filter and see everything again.

**24.6 Activity Heatmap: A GitHub-style contribution grid visualization of your watching habits over the year.** (Incomplete)

- [ ] Build a small grid of boxes where each box represents one day of the year.
- [ ] Color the boxes darker green or orange based on how many hours were watched that day.
- [ ] Let the user hover over a box to see the exact date and exactly what they watched.
- [ ] Place this heatmap at the very top of the history page or in the settings.
- [ ] Provide buttons to flip back and see heatmaps from previous years.

**24.7 Episode-Specific Search: The History search bar queries exact episode names ("The Red Wedding") not just show titles.** (Incomplete)

- [ ] Make the search bar on the history page look incredibly deep into the data.
- [ ] Find matches even if the user just types the specific title of one episode.
- [ ] Show the exact date they watched that specific episode in the search results.
- [ ] Ignore uppercase and lowercase letters so the search is easy to use.
- [ ] Keep the search fast even if the user has watched thousands of episodes.

**24.8 History Pagination UI: Visual indicators at the bottom indicating Page 1 of 50.** (Incomplete)

- [ ] Break the massive history list into smaller pages to keep the app running fast.
- [ ] Show a neat row of numbers at the bottom to let the user jump between pages.
- [ ] Highlight the current page number so they know exactly where they are.
- [ ] Provide simple 'Next' and 'Previous' buttons for easy reading.
- [ ] Ensure scrolling to the bottom naturally loads the next page without making them click.

**24.9 Binge Duration Math: On a Binge-Block, explicitly state "Binge Duration: 6h 15m".** (Incomplete)

- [ ] Add up the exact running time of every episode tucked inside a binge block.
- [ ] Display this impressive total time prominently right on the closed block.
- [ ] Update the math if the user removes an episode from the block.
- [ ] Make sure the time format matches the rest of the app, like '6h 15m'.
- [ ] Hide this string if the block only contains two very short episodes.

**24.10 Clipboard Sharing: A button to copy a binge log to clipboard formatted for Discord/Reddit (Finished Breaking Bad S1 - 8/10!).** (Incomplete)

- [ ] Add a small 'Share' icon right next to the binge duration text.
- [ ] Generate a clean, readable text summary when the user clicks the icon.
- [ ] Include the show name, the season number, and the user's personal star rating in the text.
- [ ] Save the text directly to their computer clipboard instantly.
- [ ] Show a quick 'Copied!' message so they know it worked.

**24.11 Rewatch Flagging: If watch_count > 1, history entries get a subtle circular arrow icon indicating a rewatch.** (Incomplete)

- [ ] Check if the user has watched the exact same episode before on an older date.
- [ ] Add a small, elegant looping arrow icon next to the newest history row.
- [ ] Help the user easily see at a glance which shows they enjoy repeating.
- [ ] Provide a small tooltip explaining 'Rewatch' when they hover over the icon.
- [ ] Ensure the first time they watched it stays normal without the special icon.

**24.12 Rewatch Filter: Toggle history to only show items you have watched multiple times.** (Incomplete)

- [ ] Add a simple toggle switch near the search bar on the history page.
- [ ] Hide everything except the rows that have the special rewatch arrow icon.
- [ ] Let the user easily see all their favorite, highly-repeated shows in one place.
- [ ] Keep the dates and timeline layout intact while this filter is on.
- [ ] Show a friendly empty state if they have never rewatched anything.

**24.13 CSV Export: Fully format and export the SQLite History table to a portable .csv file.** (Incomplete)

- [ ] Create a big 'Export History' button in the advanced settings menu.
- [ ] Build a standard spreadsheet file containing every single watch date, show title, and rating.
- [ ] Let the user pick exactly where on their computer they want to save the file.
- [ ] Format the dates cleanly so they work properly in Excel or other spreadsheet programs.
- [ ] Show a clear progress bar if the export takes a few seconds to build.

**24.14 Generic CSV Import: Import logic to parse standard Letterboxd/Trakt export formats to backfill the database.** (Incomplete)

- [ ] Add an 'Import History' button right next to the export option.
- [ ] Let the user select a spreadsheet file they downloaded from other movie websites.
- [ ] Read the file and match the old titles to the correct internet database IDs.
- [ ] Add all the old dates into the app's history without breaking the timeline.
- [ ] Show a summary of how many shows were successfully added when it finishes.

**24.15 Timeline Fast-Scroller: A tiny alphabet/year vertical index on the right edge of the screen to jump instantly to 2018.** (Incomplete)

- [ ] Draw a very thin vertical list of years on the far right side of the history screen.
- [ ] Let the user click '2018' to instantly jump all the way down the page to that year.
- [ ] Make the list of years update automatically based on how far back their history goes.
- [ ] Keep the list stuck to the screen even while the user scrolls normally.
- [ ] Hide this fast-scroller on narrow mobile screens so it doesn't block the text.

**24.16 Watch Gap String: "Watched 5 years after airing" dynamically calculated string on history rows.** (Incomplete)

- [ ] Compare the date the user watched the episode to the date it originally aired on television.
- [ ] Calculate the exact number of years or months between those two dates.
- [ ] Display a fun little string like 'Watched 2 years later' right inside the history row.
- [ ] Hide the string if they watched it on the exact same day it premiered.
- [ ] Hide the string if the internet database doesn't know when the episode aired.

**24.17 Accordion Auto-Collapse: Expanding a Binge-Block automatically closes previously opened ones to keep the view clean.** (Incomplete)

- [ ] Notice when the user clicks to open a large group of watched episodes.
- [ ] Find any other groups that are currently open on the screen and close them smoothly.
- [ ] Prevent the page from becoming miles long and impossible to navigate.
- [ ] Keep the scroll position steady so the screen doesn't jump wildly when things close.
- [ ] Let the user turn this feature off in the settings if they prefer leaving everything open.

**24.18 Midnight Crossover Icon: A tiny moon icon if a single Binge-Block spans across two calendar days.** (Incomplete)

- [ ] Notice if a single continuous viewing session starts before midnight and ends after midnight.
- [ ] Add a small, elegant crescent moon icon next to the binge duration time.
- [ ] Let the user know they stayed up incredibly late without being judgmental.
- [ ] Provide a tooltip explaining 'Spans across midnight' when hovered.
- [ ] Keep the block safely grouped under the day the viewing session started.

**24.19 "Marathon" Tier Badge: Binge-Blocks exceeding 12 hours receive a special red/gold flame badge.** (Incomplete)

- [ ] Check if the total running time of a single group of episodes goes over twelve hours.
- [ ] Add an exciting, colorful flame icon to the top of that specific group.
- [ ] Reward the user for their massive dedication to finishing a story.
- [ ] Ensure the badge is visually distinct from the standard completion checkmarks.
- [ ] Hide the badge if the user manually changes the dates and breaks the marathon.

## ⚙️ Part 25: Settings, Safety & Edge Cases (Incomplete)

**25.1 Automated SQLite Backups: The Rust backend copies watchmark.db to watchmark.bak every 24 hours.** (Incomplete)

- [x] Build a silent background task that runs once a day while the app is open.
- [x] Create a safe copy of the main database file and store it in the same hidden folder.
- [x] Keep only the three most recent backup files to save hard drive space.
- [x] Perform this action quietly without interrupting the user or slowing down the app.
- [x] Show a small text note in the settings menu detailing exactly when the last backup happened.

**25.2 Manual DB Backup Action: A button opening a native OS dialog to save a copy of the database to Documents.** (Incomplete)

- [ ] Wire up the 'Backup DB' button in the System settings tab.
- [ ] Open a standard folder selection window when the user clicks the button.
- [ ] Suggest a clear file name automatically, like 'WatchMark-Backup-2024.db'.
- [ ] Copy the database over to their chosen folder safely and instantly.
- [ ] Display a cheerful success message once the file finishes saving.

**25.3 Manual DB Restore Action: Safely overwrite the active DB from a backup file with a restart prompt.** (Incomplete)

- [ ] Add and wire up an 'Import Backup' button in the System settings tab.
- [ ] Let the user pick a backup file from their computer using a standard file window.
- [ ] Show a strong warning explaining that restoring a backup will erase their current progress.
- [ ] Close the app automatically and swap the database files quietly in the background.
- [ ] Restart the app fresh with all the restored data intact and ready to use.

**25.4 VACUUM Optimizer: A "Clean Database" button that runs SQLite vacuum and analyze routines to shrink file size.** (Incomplete)

- [ ] Wire up the 'Vacuum DB' button in the System settings tab.
- [ ] Run a deep optimization process to delete hidden leftover data and shrink the file size.
- [ ] Show a spinning loading icon while the cleaning process runs so the user knows it is working.
- [ ] Lock the rest of the app briefly so nothing breaks while the files are being organized.
- [ ] Tell the user exactly how much hard drive space was saved when the process finishes.

**25.5 Color Theme Customizer: Settings to swap the Accent Color (Orange -> Red, Neon Blue, Emerald Green).** (Incomplete)

- [ ] Build a row of colorful circles in the settings menu letting the user pick their favorite color.
- [ ] Change every single orange play button, progress bar, and active link to the new chosen color instantly.
- [ ] Remember this custom color choice permanently every time the app opens.
- [ ] Ensure all the new color options remain bright and easy to read against the dark background.
- [ ] Provide an easy 'Restore Default' button to go back to the classic vibrant orange.

**25.6 Light Mode Toggle: Full color inversion logic for users who prefer bright interfaces.** (Incomplete)

- [ ] Add a simple switch letting users change the entire app from dark mode to light mode.
- [ ] Swap the dark background for a clean, bright white or light gray tone.
- [ ] Turn all the white text black so it remains easy to read.
- [ ] Keep the colorful movie posters and accent buttons exactly the same.
- [ ] Let the app automatically match the light or dark setting of the user's actual computer OS.

**25.7 UI Zoom Scaling: A slider mapping CSS variables to scale the entire UI from 80% to 120%.** (Incomplete)

- [ ] Place a simple slider in the accessibility settings to change the size of the whole app.
- [ ] Make the text, buttons, and posters grow or shrink smoothly as the slider moves.
- [ ] Ensure the grid layout adapts intelligently so posters don't get pushed off the edge of the screen.
- [ ] Help users with large monitors or poor vision make the text comfortable to read.
- [ ] Keep the default size set to 100 percent for a standard, expected look.

**25.8 Hardware Acceleration Toggle: Exposes Tauri's WebView GPU acceleration settings for low-end machines.** (Incomplete)

- [ ] Add a deep settings switch to turn off heavy graphical features if the app runs slowly.
- [ ] Require a quick app restart for this specific setting change to take effect.
- [ ] Warn the user that turning this off might make animations feel slightly less smooth.
- [ ] Default this setting to 'On' so modern computers get the best possible visual experience.
- [ ] Help old laptops run the app without overheating or freezing.

**25.9 Minimize to Tray: Option to hide the app to the Windows System Tray / Mac Menu Bar instead of taskbar.** (Incomplete)

- [ ] Give the user an option to keep the app running hidden down by their computer clock.
- [ ] Remove the app from the main taskbar entirely when they click the minimize button.
- [ ] Let the user bring the app back instantly by clicking the tiny tray icon.
- [ ] Keep background tracking and downloading active even when the app is tucked away.
- [ ] Add a simple right-click menu to the tray icon with an option to close the app for good.

**25.10 Start with OS: Rust hook to add the application to OS startup items automatically.** (Incomplete)

- [ ] Add a checkbox asking if the app should open automatically when the computer turns on.
- [ ] Tell the computer's operating system to add the app to its official startup list.
- [ ] Remove the app from the startup list instantly if the user unchecks the box later.
- [ ] Start the app quietly in the system tray so it doesn't bother the user with a giant window right away.
- [ ] Help users ensure their background tracking is always active without needing to remember to open the app.

**25.11 Close = Minimize Toggle: Overrides the Window X button to hide to tray rather than killing the process.** (Incomplete)

- [ ] Provide an option to change what the main red 'X' close button does.
- [ ] Make the 'X' button hide the app in the system tray instead of shutting it down.
- [ ] Prevent users from accidentally stopping a download or breaking a watch session by closing the window.
- [ ] Make sure the 'Quit' button inside the actual menus still closes the app permanently.
- [ ] Show a one-time helpful hint explaining where the app went the first time they click the 'X'.

**25.12 GitHub Release Updater: Built-in Tauri updater checking the repo for new versions.** (Incomplete)

- [ ] Ask the internet quietly in the background if a brand new version of the app is available.
- [ ] Show a friendly notification badge on the settings gear icon if an update is found.
- [ ] Tell the user exactly what new features are included in the new version.
- [ ] Provide a simple button to start downloading the update right there inside the app.
- [ ] Ensure the update process is safe and doesn't delete their history or settings.

**25.13 Silent Auto-Update: Downloads updates in the background and prompts for a quick restart.** (Incomplete)

- [ ] Download the new version automatically without making the user click anything.
- [ ] Wait until the download is totally finished before showing any messages.
- [ ] Pop a gentle toast message asking the user to restart the app to apply the new features.
- [ ] Apply the new version instantly the next time they open the app naturally.
- [ ] Let users turn off silent updates if they prefer to manage versions manually.

**25.14 Factory Reset Settings: Restores window sizes and preferences to default without touching the database.** (Incomplete)

- [ ] Add a safety button at the very bottom of the settings page to fix broken layouts.
- [ ] Wipe out custom colors, window sizes, and slider preferences instantly.
- [ ] Keep the actual movie history, lists, and library safe from being deleted.
- [ ] Return the app visually to exactly how it looked the very first day it was installed.
- [ ] Ask for a quick confirmation so the user doesn't reset things by accident.

**25.15 Nuclear Wipe Action: Deletes database and all cache files (Requires typing the word DELETE to confirm).** (Incomplete)

- [ ] Create a massive, red danger zone button for users who want to start totally fresh.
- [ ] Open a very serious warning popup explaining that all history and tracking will be gone forever.
- [ ] Force the user to literally type the word 'DELETE' into a text box to prove they mean it.
- [ ] Wipe the database, all the downloaded posters, and all settings files clean off the hard drive.
- [ ] Restart the app immediately so it looks like a brand new installation.

**25.16 Global Tooltips Toggle: A master switch to turn off all hover helper-text for expert users.** (Incomplete)

- [ ] Add a simple switch in the settings to disable every single helpful popup box.
- [ ] Stop the small descriptive text from appearing when hovering over buttons or icons.
- [ ] Keep the screen looking incredibly clean for users who already know what every button does.
- [ ] Leave critical warning messages or error popups active so the app remains safe to use.
- [ ] Turn the tooltips back on instantly if the user changes their mind.

**25.17 Configurable Binge Threshold: Slider to change the Binge definition from 6 hours up to 12 hours.** (Incomplete)

- [ ] Give the user a slider to define how long a break they can take before a binge session ends.
- [ ] Let them stretch the timer up to 12 hours if they take long breaks between episodes.
- [ ] Group their history rows together intelligently based on this new custom timeline.
- [ ] Update the history page immediately to reflect the new groupings without deleting any data.
- [ ] Keep the default set to 6 hours for a standard, expected experience.

**25.18 Configurable Completion Threshold: Slider to adjust auto-completion from 90% to 85% or 95%.** (Incomplete)

- [ ] Let the user decide exactly what percentage of a video counts as 'finished'.
- [ ] Provide a slider ranging from 80 percent up to 99 percent.
- [ ] Mark the episode with a green checkmark automatically when they pass this specific custom mark.
- [ ] Help users who watch shows with incredibly long end-credits scenes.
- [ ] Apply this new rule to all future watching sessions automatically.

**25.19 Disable Auto-Complete: A master switch to turn off VLC auto-completion entirely for manual purists.** (Incomplete)

- [ ] Add a checkbox allowing users to stop the app from ever marking things finished on its own.
- [ ] Leave the episode marked as 'Watching' even if they reach the very last second of the video.
- [ ] Force the user to click the green checkmark button themselves when they are done.
- [ ] Continue to track and save their exact pause time accurately.
- [ ] Help users who want absolute, total manual control over their own history diary.

**25.20 Rust Debug Log View: An in-app terminal window in Settings showing live Rust stdout and database query speeds.** (Incomplete)

- [ ] Build a small, dark text box hidden deep in the advanced settings page.
- [ ] Show a live stream of text explaining exactly what the background code is doing.
- [ ] Print out how many milliseconds it takes to search the database so users can check performance.
- [ ] Add a 'Copy Log' button so users can easily share errors with the developer for help.
- [ ] Keep this text box hidden from normal users so it doesn't look confusing or scary.

**25.21 0-Episode Handling: Graceful UI states for TMDB entries that exist but have no seasons/episodes added yet.** (Incomplete)

- [ ] Notice if a brand new show is added to the library but the internet database is empty.
- [ ] Show a friendly, beautiful message saying 'No episodes have been announced yet' on the show page.
- [ ] Hide the play buttons and progress bars since there is nothing to track or watch.
- [ ] Keep the 'Refresh Data' button highly visible so they can check for updates easily later.
- [ ] Ensure the app doesn't crash or show ugly errors when the episode list comes back empty.

**25.22 Debounced Search Input: React useDebounce waits 300ms after typing stops before querying the DB to prevent lag.** (Incomplete)

- [ ] Watch the user as they type letters into any search box in the app.
- [ ] Wait a tiny fraction of a second after they stop typing before actually running the search.
- [ ] Prevent the app from freezing by not searching the massive database for every single letter.
- [ ] Ensure the search still feels incredibly fast and responsive to the user.
- [ ] Ignore this delay if the user hits the Enter key to search immediately.

**25.23 Double-Click Prevention: Disables the "Play" button for 3 seconds after clicking to prevent spawning 5 VLCs.** (Incomplete)

- [ ] Notice the instant a user clicks a bright orange play button on any movie poster.
- [ ] Turn the button gray and make it unclickable for a few seconds.
- [ ] Stop the computer from accidentally opening five different video player windows at once.
- [ ] Return the button to normal automatically once the video player actually opens successfully.
- [ ] Keep the rest of the app functional so they can still scroll while the video loads.

**25.24 Optimistic Destructive Updates: Clicking "Remove Show" instantly hides the card in React while Rust processes the cascading delete.** (Incomplete)

- [ ] Hide the movie poster from the library screen the exact millisecond the user clicks delete.
- [ ] Let the background code do the heavy work of actually erasing the files quietly.
- [ ] Make the app feel incredibly fast and snappy by not making the user wait for the hard drive.
- [ ] Put the poster back quietly if the background deletion process accidentally fails.
- [ ] Ensure the numbers on the dashboard update instantly to reflect the removed show.

**25.25 Custom 404 Route: A stylized "Page Not Found" component for internal React Router errors.** (Incomplete)

- [ ] Catch the user if they somehow click a broken link or navigate to a weird page.
- [ ] Show a beautiful, dark-themed error screen instead of a broken, blank white page.
- [ ] Include a massive, friendly 'Take me back home' button in the center of the screen.
- [ ] Add a fun, movie-themed illustration or joke to make the error less annoying.
- [ ] Keep the top navigation bar visible so they can easily click their way out.

**25.26 CSS Text-Select Disabling: user-select-none applied globally to UI, but enabled specifically for Synopsis text so users can copy descriptions.** (Incomplete)

- [ ] Stop the user from accidentally highlighting buttons, headers, and menus when clicking around.
- [ ] Keep the app feeling like a solid, native desktop program instead of a messy website.
- [ ] Leave the highlighting feature turned on for the main story descriptions and titles.
- [ ] Allow users to easily highlight and copy a show's summary to share with a friend.
- [ ] Ensure the text cursor only appears when hovering over areas they are actually allowed to copy.

**25.27 Custom Window Drag Regions: Applying Tauri's data-tauri-drag-region exclusively to empty top-bar space so buttons remain clickable.** (Incomplete)

- [ ] Let the user click and drag the very top edge of the app to move the window around.
- [ ] Stop the dragging feature from covering up the search bar or the back button.
- [ ] Ensure clicks on the search bar always open the typing cursor instead of moving the window.
- [ ] Make the drag area wide enough that the user doesn't have to hunt for a safe spot to click.
- [ ] Disable dragging entirely when the app is maximized to fill the whole screen.

**25.28 Accurate Maximize Icons: The top-right square icon changes to "restore down" overlapping squares when window is maximized.** (Incomplete)

- [ ] Watch the window size to see if it is currently taking up the entire computer monitor.
- [ ] Change the square icon in the top right corner to look like two smaller squares.
- [ ] Let the user know clicking it will shrink the window back down to a normal size.
- [ ] Change the icon back to a single square when the window is shrunk down again.
- [ ] Ensure the icons match the standard look and feel of the user's specific operating system.

**25.29 Modal Hover Persistence: Ensures hover overlays on posters disappear instantly if a Settings modal is opened over them.** (Incomplete)

- [ ] Notice when a giant popup window, like the settings menu, opens over the main library.
- [ ] Force any dark hover boxes over the movie posters to vanish immediately.
- [ ] Keep the screen clean so the hover boxes don't bleed through the blurred popup background.
- [ ] Prevent the user from accidentally clicking hidden play buttons behind the popup window.
- [ ] Let the hover boxes work normally again the second the popup is closed.

**25.30 Inbox Scanning Skeletons: Animated placeholder rows in the Inbox while the background Rust scanner is running.** (Incomplete)

- [ ] Show gray, shimmering placeholder rows in the file inbox when a massive folder scan starts.
- [ ] Let the user know the app is actively working on finding their confusing video files.
- [ ] Replace the placeholder rows with real file names one by one as the scan finishes them.
- [ ] Keep the rest of the inbox usable so the user can organize old files while the scan runs.
- [ ] Remove the placeholders cleanly if the scan finishes and finds absolutely nothing new.

**25.31 Path Wrapping: Force CSS break-words on long file paths in the Inbox so they don't break the flex layout.** (Incomplete)

- [ ] Identify incredibly long file paths that try to stretch past the edge of the screen.
- [ ] Force the long text to break cleanly and wrap onto a second or third line.
- [ ] Keep the buttons and layout rigid so the long text doesn't ruin the shape of the inbox.
- [ ] Ensure the text breaks at logical points, like slashes, instead of chopping words in half.
- [ ] Keep the text small and dim so it remains readable without dominating the entire row.

**25.32 Cross-View Scroll Reset: Navigating from "TV Shows" to "Movies" automatically resets the window scroll to the top.** (Incomplete)

- [ ] Notice when the user clicks a major section link in the left sidebar menu.
- [ ] Scroll the page instantly to the very top before showing the new list of posters.
- [ ] Prevent the user from arriving halfway down the page on a brand new screen.
- [ ] Keep this logic separate from the specific 'Back' button memory feature.
- [ ] Ensure the scroll jump happens smoothly during the page fade animation.

**25.33 Search Clear Focus Retention: Clicking the X to clear a search keeps the blinking cursor in the input box.** (Incomplete)

- [ ] Notice when the user clicks the tiny clear button inside the main search bar.
- [ ] Wipe out the text but keep the blinking cursor locked inside the empty box.
- [ ] Let the user immediately start typing a brand new search without having to click the box again.
- [ ] Prevent the page from losing focus and forcing the user to use their mouse a second time.
- [ ] Ensure standard keyboard navigation still works correctly after the box is cleared.

**25.34 / Global Hotkey: Pressing forward-slash instantly selects the quick search bar (Standard web UX).** (Incomplete)

- [ ] Listen for the forward-slash key on the keyboard no matter what page the user is on.
- [ ] Jump the typing cursor instantly into the top search bar when the key is pressed.
- [ ] Help power users navigate the app incredibly fast using standard internet shortcuts.
- [ ] Ignore the shortcut if the user is already typing inside a different text box.
- [ ] Prevent the forward-slash character from actually being typed into the search box.

**25.35 Global Processing Cursor: Changes cursor to wait (hourglass) during heavy synchronous DB operations.** (Incomplete)

- [ ] Notice when the app asks the database to do a massive job, like importing thousands of files.
- [ ] Change the mouse pointer into a spinning circle or hourglass icon.
- [ ] Let the user know the app is thinking and hasn't frozen or crashed.
- [ ] Prevent the user from clicking important buttons while the database is locked.
- [ ] Return the mouse pointer to normal the exact millisecond the heavy job finishes.

**25.36 Real-time File Deletion Catching: If a file is deleted via Windows Explorer while WatchMark is open, the app catches the OS FileNotFound error upon clicking Play and updates the icon to ☁️.** (Incomplete)

- [ ] Try to launch the video player when the user clicks the orange play button.
- [ ] Catch the error quietly if the computer says the file no longer exists.
- [ ] Swap the play button instantly to a gray cloud icon so the user knows the file is missing.
- [ ] Show a polite little message explaining the file was moved or deleted outside the app.
- [ ] Prevent the app from crashing or showing terrifying code errors to the user.

**25.37 Layout Snap Prevention: Uses AnimatePresence mode="popLayout" to ensure when a Binge-Block expands, the elements below it slide smoothly instead of jumping.** (Incomplete)

- [ ] Tell the animation engine to handle layout changes smoothly when things appear or disappear.
- [ ] Make the history items below a binge block slide down naturally when it opens.
- [ ] Stop the items from instantly teleporting or snapping jarringly down the screen.
- [ ] Keep the visual tracking clean so the user's eye can follow the movement.
- [ ] Ensure the closing animation is just as smooth and pushes things back up naturally.

**25.38 VLC Spawn Loading State: The Orange play button changes to an animated spinner for 0.5s while the VLC .exe boots up.** (Incomplete)

- [ ] Hide the play triangle icon the moment the user clicks the button.
- [ ] Show a tiny, spinning circle in its place to indicate the app is working.
- [ ] Keep the spinner visible just long enough to cover the time it takes the video player to open.
- [ ] Put the play triangle back once the video player confirms it is running.
- [ ] Make the button unclickable while it is spinning to prevent accidental double-launches.

**25.39 Season Tab Overflow Handling: If a show has 30 seasons, the horizontal season-pill row becomes mouse-draggable.** (Incomplete)

- [ ] Notice if a show has too many seasons to fit neatly on one single line across the screen.
- [ ] Let the user click and drag the row of season buttons left and right with their mouse.
- [ ] Hide the ugly scrollbars that computers normally put on sideways lists.
- [ ] Ensure clicking a tab still changes the season instead of just dragging the row.
- [ ] Add a subtle shadow on the edge of the screen hinting that there are more seasons hiding over there.

**25.40 Conditional Back-to-Top Button: Appears in the History tab only when scrollY > 2000px.** (Incomplete)

- [ ] Measure exactly how far down the page the user has scrolled on long library lists.
- [ ] Keep the helpful 'Jump to Top' button completely hidden at the start.
- [ ] Fade the button into the corner of the screen only when they scroll very deep into the list.
- [ ] Scroll the page back to the top beautifully and smoothly when the button is clicked.
- [ ] Fade the button back out once they reach the top of the page again.

**25.41 TMDB API Key Verification Spinner: When saving settings, shows a loader while pinging TMDB to verify the key is actually valid.** (Incomplete)

- [ ] Show a spinning loading icon next to the save button when the user enters a new secret key.
- [ ] Send a quick, silent test message to the internet database to make sure the key works.
- [ ] Swap the spinner for a bright green checkmark if the database accepts the key.
- [ ] Swap the spinner for a red warning icon if the database rejects the key or is offline.
- [ ] Stop the user from trying to download movie posters if the key is proven to be broken.

**25.42 Empty Season Handling: If a season exists in TMDB but has 0 episodes, the Season Tab is disabled/grayed out in the UI.** (Incomplete)

- [ ] Notice when the internet database lists a season that hasn't actually aired any episodes yet.
- [ ] Show the button for that season in the list, but color it a dim, inactive gray.
- [ ] Prevent the user from clicking the button since there is nothing inside it to see.
- [ ] Add a helpful tooltip explaining 'No episodes available' when they hover over it.
- [ ] Turn the button bright and clickable automatically the second a new episode is added.

**25.43 Offline Placeholder Avatars: If actor headshots fail to load due to network drops, use a stylized SVGs matching the dark theme.** (Incomplete)

- [ ] Catch the error quietly if the app tries to download an actor's picture but the internet is broken.
- [ ] Swap the broken image box for a beautiful, dark silhouette icon.
- [ ] Keep the screen looking polished and professional instead of showing ugly 'image missing' errors.
- [ ] Match the silhouette style perfectly to the rest of the dark cinematic theme.
- [ ] Ensure the actor's name underneath remains clearly readable.

**25.44 Watch Time Extrapolation: If a show has no runtime data on TMDB, app calculates average runtime from local video files to estimate "Hours Watched".** (Incomplete)

- [ ] Notice if the internet database completely forgets to list how long a television show's episodes are.
- [ ] Look at the actual video files on the user's hard drive to see how long they usually run.
- [ ] Create a smart guess, like '45 minutes', based on those real video files.
- [ ] Use this smart guess to keep the fun dashboard statistics accurate and unbroken.
- [ ] Update the guess automatically if the user downloads longer or shorter episodes later.

**25.45 Dynamic "Missing API Key" State: Instead of throwing alerts, the entire "Search" tab shows a beautiful full-screen prompt explaining how to get a free API key with a direct link.** (Incomplete)

- [ ] Check to see if the user has forgotten to enter their secret database key in the settings.
- [ ] Replace the empty search page with a friendly, welcoming instruction screen.
- [ ] Explain simply why the key is needed to download movie posters and descriptions.
- [ ] Provide a giant, clickable link directly to the website where they can sign up for free.
- [ ] Remove this instruction screen instantly the second a valid key is saved in the settings.

**25.46 Easter Egg / Konami Code: Typing a secret sequence triggers a playful CSS animation or unlocks a secret color theme for fun.** (Incomplete)

- [ ] Listen quietly for a very specific, secret pattern of keyboard presses.
- [ ] Make sure the sequence doesn't accidentally trigger while the user is typing a real search.
- [ ] Unlock a silly visual joke, like making the movie posters spin around, when the code is entered.
- [ ] Add a hidden 'Neon Pink' color theme to the settings menu as a permanent reward.
- [ ] Keep the easter egg totally harmless so it doesn't break any real app features or data.

## 📥 Part 26: Automated Sourcing & Telegram Download Engine (Incomplete)

**26.1 Missing File Action Trigger: The disabled ☁️ (Cloud) icon on missing episodes is transformed into an interactive, glowing blue 📥 (Download) button.** (Incomplete)

- [ ] Change the gray cloud icon into a bright blue download symbol for any episode missing its video file.
- [ ] Make the new button glow slightly so the user knows they can click it to fix the missing file.
- [ ] Open a small confirmation menu when the button is clicked to make sure they want to start downloading.
- [ ] Provide a tooltip explaining that clicking the button will search the internet for the missing episode.
- [ ] Keep the button hidden if the user has disabled downloading features in their main settings.

**26.2 "Fetch Entire Season" Bulk Action: A global button added to the Season UI tab that queues the automated search and download process for all missing episodes in that specific season.** (Incomplete)

- [ ] Place a massive 'Download Season' button right at the top of the season view page.
- [ ] Find every single episode in that specific season that currently lacks a local video file.
- [ ] Add all of those missing episodes to a download waiting list automatically with one click.
- [ ] Gray the button out if the user already has every single episode downloaded for that season.
- [ ] Show a polite warning if the season contains more than 50 missing episodes to prevent accidental mass downloads.

**26.3 Automated Web-Scraping Engine: Rust backend silently issues HTTP requests to pre-configured indexing sites (e.g., ibox-tv) using the exact TMDB Title and Year to ensure precise search matches.** (Incomplete)

- [ ] Start a silent background search using the show's name and release year to find the correct files.
- [ ] Wait patiently for the internet search to finish without freezing the main app screen.
- [ ] Ignore search results that belong to different shows that just happen to share a similar name.
- [ ] Provide a clear error message on the screen if the specific indexing website is currently offline.
- [ ] Let the user continue browsing their library normally while this search happens invisibly.

**26.4 Headless Result Parsing: The backend parses the HTML DOM of the search results, identifies the correct show page, and navigates to the details page without opening a visible browser window.** (Incomplete)

- [ ] Read the hidden website code to find the best match for the specific television show.
- [ ] Follow the hidden website links to reach the final download page without opening a real web browser.
- [ ] Stop the search safely if the website changes its layout and the app can no longer read it.
- [ ] Pick the highest quality version available if the website offers multiple different choices.
- [ ] Keep the user's computer clean by not leaving hidden browser windows running in the background.

**26.5 Telegram Deep-Link Extraction: Scrapes the target page specifically looking for t.me/ join links or specific Telegram file IDs hidden in the "Download Now" buttons.** (Incomplete)

- [ ] Search the final webpage to find the specific hidden link needed to get the video file.
- [ ] Grab the unique file identification number out of the website button.
- [ ] Stop the process gently if the webpage promises a download but doesn't actually contain a valid link.
- [ ] Handle situations safely where the website requires solving a puzzle before showing the link.
- [ ] Secure the extracted link so it can be passed to the downloading engine safely.

**26.6 Native MTProto Telegram Integration: WatchMark acts as a headless Telegram client (via API ID and Hash), bypassing the need to physically open the Telegram Desktop GUI.** (Incomplete)

- [ ] Connect the app straight to the chat servers without making the user open a separate chat program.
- [ ] Keep the connection totally invisible so it feels like a native feature of the movie app.
- [ ] Ensure the user's personal chat messages remain totally private and untouched.
- [ ] Handle network drops by pausing the connection and trying again when the internet returns.
- [ ] Close the connection safely the moment the app finishes pulling the requested files.

**26.7 Automated Channel Joining: The backend securely passes the extracted t.me invite hash to the Telegram servers, automatically joining the required distribution channel on the user's behalf.** (Incomplete)

- [ ] Use the hidden link from the webpage to join the specific chat room where the files live.
- [ ] Wait for the server to confirm the user was successfully added to the room before searching.
- [ ] Show an error if the specific chat room was banned or deleted by the server administrators.
- [ ] Bypass any welcome messages or group rules without forcing the user to read them.
- [ ] Keep the app from joining the same room twice if the user downloads another episode later.

**26.8 Smart Message Filtering: Once in the channel, the backend scans the message history, using your existing regex engine to identify only the specific .mkv or .mp4 files that match the missing SxxExx numbers.** (Incomplete)

- [ ] Look through the history of the chat room to find the exact video file needed.
- [ ] Use the show's season and episode number to skip over files that belong to different episodes.
- [ ] Ignore small files like pictures or text documents that might be mixed in with the videos.
- [ ] Handle situations where the file name is slightly misspelled but still obviously the correct episode.
- [ ] Stop searching once the correct file is found so the app doesn't waste time reading the whole room.

**26.9 In-App Download Manager (UI): A new sliding drawer or modal in WatchMark displaying active background downloads, featuring real-time progress bars, MB/s speed metrics, and ETAs.** (Incomplete)

- [ ] Build a sleek menu that slides out from the side of the screen to show what is downloading.
- [ ] Show a moving progress bar for every single file currently being pulled from the internet.
- [ ] Display numbers telling the user exactly how fast the download is moving.
- [ ] Guess how many minutes are left until the download finishes and display it clearly.
- [ ] Provide a simple 'Cancel' button next to each file in case the user changes their mind.

**26.10 Direct-to-Library Routing: Files are not dumped into the OS "Downloads" folder; they are streamed directly into the user's configured WatchMark media root directory (e.g., D:\TV Shows\[Show Name]\Season 1\).** (Incomplete)

- [ ] Create the correct show and season folders automatically on the user's hard drive if they don't exist.
- [ ] Save the downloading video file straight into those specific folders instead of a messy general pile.
- [ ] Name the new file cleanly using the standard season and episode numbering format.
- [ ] Avoid leaving broken, half-finished files sitting in the folders if the download is cancelled.
- [ ] Make sure the app has the right computer permissions to save files in that specific location.

**26.11 Auto-Link & UI Refresh: The exact millisecond a download hits 100%, the backend updates the Local_Files database table and instantly transitions the UI icon from the blue downloading spinner to the solid orange ▶ Play button.** (Incomplete)

- [ ] Tell the database exactly where the new video file is located the second the download finishes.
- [ ] Change the blue download icon into a bright orange play button instantly on the screen.
- [ ] Let the user click play to start watching the video without needing to refresh the page.
- [ ] Update the dashboard statistics and continuing watching rows to include the brand new episode.
- [ ] Remove the finished item from the active download list so the menu stays clean.

**26.12 Telegram Auth Setup (Settings): A secure panel in the Settings menu where users input their Telegram Phone Number and API credentials to authenticate the background downloader via a one-time SMS verification code.** (Incomplete)

- [ ] Create a special section in the settings menu dedicated to setting up the chat connection.
- [ ] Provide clear text boxes for the user to type in their phone number and secret credentials.
- [ ] Let the user type in the verification code they receive on their phone to prove who they are.
- [ ] Hide all the typed secret numbers behind stars or dots so people walking by can't see them.
- [ ] Show a friendly green checkmark when the connection is tested and proven to work.

**26.13 Auto-Leave Channel (Cleanup): An optional setting to automatically leave the Telegram channel the moment the required files finish downloading, preventing the user's personal Telegram chat list from becoming cluttered with hundreds of file channels.** (Incomplete)

- [ ] Add a simple toggle switch in the settings asking if the app should clean up after itself.
- [ ] Remove the user from the chat room the second the video file finishes saving.
- [ ] Stop the user's personal phone app from filling up with hundreds of random movie groups.
- [ ] Leave the user in the room if they turn this setting off so they can easily find more files later.
- [ ] Wait until every single queued episode from that specific room is finished before leaving.

**26.14 Multi-Threaded Downloading: Support for concurrent chunk-downloading from Telegram servers to maximize bandwidth utilization and speed up large season packs.** (Incomplete)

- [ ] Break massive video files into tiny pieces and download several pieces at the same time.
- [ ] Put the tiny pieces back together seamlessly on the hard drive so the video plays smoothly.
- [ ] Pull down multiple different episodes at the exact same time if the user queued an entire season.
- [ ] Ensure this complex downloading process doesn't cause the main app screen to stutter or freeze.
- [ ] Handle situations gracefully where one tiny piece fails to download by trying it again.

**26.15 Download Failure Fallback: If the Telegram link is expired or the file is missing from the channel, the UI gracefully falls back, throwing a toast notification: "Source unavailable. Manual search required."** (Incomplete)

- [ ] Notice if the hidden link is old and the chat servers refuse to let the app inside the room.
- [ ] Show a polite popup notification letting the user know the file could not be found automatically.
- [ ] Change the spinning download icon back into the original gray cloud icon.
- [ ] Keep the app from crashing or locking up if it hits a dead end during the invisible search.
- [ ] Remove the broken item from the active download list so it doesn't get stuck there forever.

**26.16 Bandwidth Throttling: A slider in Settings to cap the Telegram download speed so background fetching doesn't ruin the user's ping while gaming or browsing.** (Incomplete)

- [ ] Add a slider to the download settings letting the user set a strict speed limit.
- [ ] Force the invisible downloader to slow down so it never crosses that specific limit.
- [ ] Help users keep their home internet fast enough for gaming or video calls while movies download.
- [ ] Allow the user to uncap the limit entirely if they want their movies to finish as fast as possible.
- [ ] Update the estimated finish time in the download menu to reflect the slower speed limit.
