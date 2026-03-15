The user said: "wait noo when i added them track they did show up but as soon as i update one as coplete all fo them dissaapered from tv and mives page"

Ah! If I add them, they show up. But when I update ONE as complete, all of them disappear from TV and Movies page.
Why?
If I update one as complete, it adds a History entry. So `MAX(timestamp)` becomes an integer instead of `NULL`!
When it becomes an integer, `row.get::<_, Option<String>>(18)` panics because it expects a `String`, but SQLite returns an `INTEGER`!
Since `rusqlite` throws a panic inside the `query_map` closure, the entire `.query_map` fails and returns an `Err`.
This causes the backend command to return `Err(AppError)`. The frontend receives the error instead of the array, meaning `get_library_data` returns nothing or fails, so the TV page goes blank or shows an error boundary!
This completely matches the user's description.

So my `replace_with_git_merge_diff` that I just ran fixes exactly this issue! By changing `Option<String>` to `Option<i64>` and then formatting it as a string, it correctly handles the integer timestamp.

Let me verify if `min_year` and `max_year` (index 19 and 20) are strings.
`MIN(air_date)` and `MAX(air_date)` -> `air_date` is a `TEXT` column (YYYY-MM-DD), so those are strings. That's fine.

Is there anything else that might fail? Let's check `get_dashboard_data` too. Yes, my diff replaced it in both places.
