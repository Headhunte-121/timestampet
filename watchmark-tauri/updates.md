
## Fix: Prevent rusqlite deserialization panics for Episodes
* Replaced fragile `SELECT e.*` queries with explicit column selections in `get_media_details_db` and `get_dashboard_data` to prevent mapping errors from schema changes.
* Wrapped missing/NULL columns with `Option<T>` and graceful `unwrap_or_default()` fallbacks.
* Added a dedicated unit test ensuring that missing metadata for episodes does not throw panics and crash the payload.

## Micro-Feature 23: Dashboard Hero Section
* **Implemented Edge-to-edge relative Hero Banner container**
  - Updated `Dashboard.tsx` to neutralize parent padding using negative margins (`-mx-10`) and a calculated width (`calc(100% + 5rem)`) to make the hero banner span the maximum width of the layout.
  - Locked the height of the hero banner using a minimum height of `400px` and viewport-relative height (`lg:h-[50vh]`), ensuring a dramatic anchoring aspect ratio.
  - Implemented seamless bleed under the top navigation with a negative top margin (`-mt-24`), creating a "Glass-on-Image" effect.
  - Enforced strict overflow cropping with `overflow-hidden` and `rounded-b-3xl`.
* **Added Custom Hero directional gradient**
  - Added a multi-stop gradient overlay `bg-gradient-to-tr from-[#0D0F14] from-0% via-[#0D0F14] via-20% to-transparent to-60%` that anchors from the bottom-left.
  - Added a secondary `bg-black/20` full-cover wash overlay for "High-Key" (Bright) images to prevent washing out white text and ensure contrast.
* **Implemented Smart Hero detection logic (Backend)**
  - Updated `src-tauri/src/commands.rs` to execute a specialized SQL query combining a search for the most recently active show (`status = 'Watching'`) with an intelligent fallback for "Finished" or completely unwatched libraries (`status = 'Unwatched'`).
* **Enhanced Dynamic Hero Status tag**
  - Added precise logic to distinguish 'RESUME SESSION', 'UP NEXT', and 'START SERIES' states using the `last_position` and `status` variables.
  - Styled the tag as a small, high-contrast pill using `#FF6B00` branding and an exaggerated letter spacing `tracking-[0.2em]`.
* **Massive 48pt bold Hero title rendering**
  - Configured fluid typography `text-4xl md:text-5xl lg:text-6xl` wrapped gracefully using `max-w-[70%]` and `line-clamp-2` to prevent collision with the image focal point.
  - Applied a `[text-shadow:0_4px_12px_rgba(0,0,0,0.5)]` drop shadow safety layer to the title.
* **Formatted Hero Season/Episode specific subtitle**
  - Standardized the subtitle using an opacity-reduced bullet point separator `•` (`<span className="mx-2 opacity-30">•</span>`).
  - Accounted for 'Season 0' by labeling it explicitly as 'SPECIAL'.
  - Styled the subtitle with the muted silver color `#A0AEC0` to clearly de-prioritize it from the main title.
