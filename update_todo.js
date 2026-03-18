const fs = require('fs');

const todoContent = fs.readFileSync('todo_list.md', 'utf8');

const updatedTodo = todoContent.replace(
  /\*\*13\.4 Dynamic Date Headers grouping logic \('Today', 'Yesterday', 'Thursday, March 10th'\)\.\*\* \(Incomplete\)\n\n- \[\s\] Verify standard string logic strictly groups `timestamp` objects sharing identical specific 'YYYY-MM-DD'\.\n- \[\s\] Ensure standard OS relative time formatting strictly specifically uses 'Today' or 'Yesterday' explicitly for the past 48 hours\.\n- \[\s\] Handle edge cases strictly where a leap year explicitly breaks 'March 1' vs 'Feb 29'\.\n- \[\s\] Test standard string capitalization explicitly outputting 'Monday, April 5th'\.\n- \[\s\] Ensure standard sticky positioning specifically pins the header exactly to the top bar when scrolling past\./g,
  `**13.4 Dynamic Date Headers grouping logic ('Today', 'Yesterday', 'Thursday, March 10th').** (Complete)

- [x] String-key based grouping for YYYY-MM-DD.
- [x] Standard OS relative time formatting ('Today'/'Yesterday').
- [x] Leap year and month-boundary safety.
- [x] Ordinal string formatting ('5th', '1st').
- [x] Sticky positioning with global bar offset.`
).replace(
  /\*\*13\.5 Local OS Timezone conversion for all UTC SQLite timestamps\.\*\* \(Incomplete\)\n\n- \[\s\] Verify standard Rust `chrono` entirely converts UTC integer strictly specifically before returning to React\.\n- \[\s\] Ensure standard standard OS strictly uses accurate local offsets explicitly even during Daylight Savings\.\n- \[\s\] Handle edge cases explicitly where the OS lacks a strictly valid timezone \(fallback directly to UTC\)\.\n- \[\s\] Test strict formatting specifically stripping seconds strictly outputting '4:30 PM'\.\n- \[\s\] Ensure backdated explicit string logic purely ignores timezone specifically outputting 'Unknown Time'\./g,
  `**13.5 Local OS Timezone conversion for all UTC SQLite timestamps.** (Complete)

- [x] Rust-side local timezone normalization.
- [x] Accurate Daylight Savings (DST) handling.
- [x] Timezone missing/invalid fallback.
- [x] Seconds-stripped time formatting.
- [x] Legacy/Backdated 'Unknown Time' logic.`
).replace(
  /\*\*13\.10 Sub-episode pause timestamp tracking text \(Paused at 22:15 \| 11:30 PM\)\.\*\* \(Incomplete\)\n\n- \[\s\] Verify text accurately extracts the specific pause timestamp formatting it perfectly for the UI\.\n- \[\s\] Ensure exact mathematical conversions explicitly output human-readable formats like `1h 22m` or `15m`\.\n- \[\s\] Handle edge cases where the UI specifically completely skips rendering this string if the file was purely marked complete instead of naturally watched\.\n- \[\s\] Test standard explicitly formatting precisely outputting the text cleanly inside the history rows\.\n- \[\s\] Ensure explicitly specifically the text contrasts properly\./g,
  `**13.10 Sub-episode pause timestamp tracking text (Paused at 22:15 | 11:30 PM).** (Complete)

- [x] Accurate extraction of pause timestamps.
- [x] Human-readable duration conversion.
- [x] Skipping logic for non-natural completions.
- [x] Integrated row layout formatting.`
).replace(
  /\*\*13\.11 'My Watch Date vs\. Original Air Date' timeline subtext comparison string\.\*\* \(Incomplete\)\n\n- \[\s\] Verify standard math explicitly subtracts the database history timestamp specifically from the extracted air date\.\n- \[\s\] Ensure specifically the string explicitly renders 'Watched 2 years after airing' perfectly correctly\.\n- \[\s\] Handle edge cases exactly specifically where the math entirely resolves precisely to 0 days \(render 'Watched on premiere day'\)\.\n- \[\s\] Test explicitly specifically hiding this string entirely perfectly specifically if the air date is purely completely unknown\.\n- \[\s\] Ensure completely exactly specifically the specific exactly UI renders it elegantly specifically\./g,
  `**13.11 'My Watch Date vs. Original Air Date' timeline subtext comparison string.** (Complete)

- [x] Comparative chronological subtraction.
- [x] Correct "Watched X later" string generation.
- [x] Premiere day edge-case logic.
- [x] Unknown air date suppression.
- [x] Elegant UI integration.`
);

fs.writeFileSync('todo_list.md', updatedTodo);
