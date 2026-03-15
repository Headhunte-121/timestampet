# Jules Agent Standard Operating Procedure (SOP)

As the Jules Agent, you must strictly follow this post-task workflow at the completion of **every** feature, task, or commit. 

Whenever you finish implementing a specific task (referred to dynamically as `[TASK_ID]`), you must execute the following testing and documentation routine:

### 1. Write and Execute Real Test Cases
* Carefully review the user's prompt for any "Edge Cases" or "Test Cases".
* You must **write actual, executable test files or test functions** (e.g., `#[cfg(test)]` unit tests in Rust, or Jest/Vitest tests in React) for EVERY edge case provided they should have nameing convensuion such that its easy to know which update they are from.
* Do NOT just manually simulate or assume "pass/fail". You must run the tests you wrote and ensure they successfully pass before finalizing the code.

### 2. Update the Master To-Do List
* Locate the current `[TASK_ID]` in the master To-Do list.
* Mark all completed sub-tasks as done (change `[ ]` to `[x]`).
* only update the metiond todo not others

### 3. Update the Progress Log (`update.md`)
* Open `update.md`.
* Add a new section for the current `[TASK_ID]` and summarize the updates, changes, and implementations made during this commit at the bottom.

### 4. Record Test Results (`TestResult.md`)
* Open `TestResult.md`.
* Scroll to the absolute bottom of the document.
* Add a new heading in the format: `## TODO [TASK_ID]`
* Document the results of the automated tests you wrote and executed in Step 1 using the exact markdown table format below:

| Test Case | Method | Result |
| :--- | :--- | :--- |
| *Brief description of what was tested* | *E.g., Rust Unit Test (src/db/tests.rs)* | *Pass/Fail/Notes* |

---

### 5. Working Directory & Architecture Rules
* **Rust Backend:** All Rust code, `Cargo.toml`, and database schemas reside inside `watchmark-tauri/src-tauri`. You must navigate to this directory before running any `cargo` commands.
* **React Frontend:** All UI, Tailwind, and React code resides inside `watchmark-tauri/src`.

### 6. Verification Rules (Speed Optimization)
* For small logic changes, do NOT run a full `cargo build` as it wastes time.
* Only run `cargo check` or `cargo test` to verify types and tests.
* If testing is required, only run the specific test related to the change (e.g., `cargo test test_name`).


### 7. Requirement Reconciliation & Integrity Audit
Before submitting a Pull Request or marking a task as complete, you must perform a mandatory "Integrity Audit" to ensure zero feature loss and total compliance:

*   **Cross-Reference Requirements:** Re-read the original user prompt line-by-line. Cross-reference every explicit instruction, "edge case," and "detailed plan" item against your implemented code. You are strictly prohibited from omitting any requested logic or skipping verification steps.
*   **Audit for Regression:** Review all changes to ensure that new logic does not inadvertently remove or break features established in previous commits. If you refactor a function, you must carry over all existing functionality into the new implementation.
*   **Enforce Project Standards:** Ensure all new and modified logic adheres to the established architectural patterns of the project (e.g., specific argument requirements, naming conventions, and error handling protocols).
*   **Self-Correction:** If the audit reveals any missing requirements or inconsistent patterns, you must resolve them immediately before finalizing the documentation in `update.md` or `TestResult.md`.


**Execution Trigger:** 
Always perform these steps automatically before finalizing a commit or marking a prompt as fully complete. Do not ask for permission to do this; it is mandatory for every update.
