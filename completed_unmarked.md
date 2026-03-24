# Completed but Unmarked Tasks

Based on a review of the codebase against `todo_list.md`, the following micro-features are already fully implemented but are not marked as `(Complete)` or checked off `[x]` in the list.

## Micro-Feature 39: Media Granular Data
- **20.14 Precise Runtime Formatting: Format 135 minutes as 2h 15m instead of just 135m.**
  - Implemented in `watchmark-tauri/src/utils/dateFormatter.ts` (`formatRuntime`) and used in `MediaDetails.tsx`.

## Micro-Feature 50: Advanced Nav UX
- **17.19 Inline Clear Button: A tiny x icon appears inside the search bar when typing, allowing 1-click clearing.**
  - Implemented in `App.tsx` global search input using `XCircle` from lucide-react.

## Micro-Feature 61: Form & Input UX Polish
- **25.31 Path Wrapping: Force CSS break-words on long file paths in the Inbox so they don't break the flex layout.**
  - Implemented in `InboxView.tsx` where file paths use `break-all`.
- **25.32 Cross-View Scroll Reset: Navigating from "TV Shows" to "Movies" automatically resets the window scroll to the top.**
  - Implemented in `App.tsx` (`handleNav` via `document.querySelector('main > div.flex-1')?.scrollTo({ top: 0, behavior: 'smooth' });`).
- **25.33 Search Clear Focus Retention: Clicking the X to clear a search keeps the blinking cursor in the input box.**
  - Implemented in `App.tsx` where the 'X' button `onClick` calls `input.focus()`.

## Micro-Feature 34: Global Stats Widgets
- **8.15 Global Personal Stats grid layout containing 3 glassy widgets.**
  - Implemented in `Dashboard.tsx` (`<div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">...`).
- **8.16 Real-time 'Total Episodes Watched' mathematical calculation.**
  - Implemented in `Dashboard.tsx` (`data.stats.shows_completed`).
- **8.17 Real-time 'Hours Watched' calculation (Summing completed episode runtimes).**
  - Implemented in `Dashboard.tsx` (`data.stats.hrs_watched`).
- **8.18 Real-time 'Shows Completed' mathematical calculation.**
  - Implemented in `Dashboard.tsx` (`data.stats.shows_completed`).

*Note: Some features like 13.6 Binge-Block Accordion UI and 14.2 Multi-option Watch Log Popup modal have partial implementations or mockups but lack the complete functional integration per the detailed checkboxes.*
