# Jules Agent Standard Operating Procedure (SOP)

As the Jules Agent, you must strictly follow this post-task workflow at the completion of **every** feature, task, or commit. 

Whenever you finish implementing a specific task (referred to dynamically as `[TASK_ID]`), you must execute the following documentation and testing routine:

### 1. Update the Master To-Do List
* Locate the current `[TASK_ID]` in the master To-Do list.
* Mark all completed sub-tasks as done (change `[ ]` to `[x]`).

### 2. Update the Progress Log (`update.md`)
* Open `update.md`.
* Add a new section for the current `[TASK_ID]` and summarize the updates, changes, and implementations made during this commit.

### 3. Verify and Record Test Cases (`TestResult.md`)
* Verify all test cases related to the current `[TASK_ID]`.
* Open `TestResult.md`.
* Scroll to the absolute bottom of the document.
* Add a new heading in the format: `## TODO [TASK_ID]`
* Document the results of your verifications using the exact markdown table format below to match existing entries:

| Test Case | Method | Result |
| :--- | :--- | :--- |
| *Brief description of what was tested* | *How it was tested (e.g., Unit Test, Manual UI, Mock)* | *Pass/Fail/Notes* |
| *...* | *...* | *...* |

**Execution Trigger:** 
Always perform these 3 steps automatically before finalizing a commit or marking a prompt as fully complete. Do not ask for permission to do this; it is mandatory for every update.

### Verification Rules
- For small logic changes, do NOT run a full `cargo build`.
- Only run `cargo check` to verify types.
- If testing is required, only run the specific test related to the change (e.g., `cargo test test_name`).
