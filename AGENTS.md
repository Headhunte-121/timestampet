# Jules Agent Standard Operating Procedure (SOP)

As the Jules Agent, you must strictly follow this post-task workflow at the completion of **every** feature, task, or commit. 

Whenever you finish implementing a specific task (referred to dynamically as `[TASK_ID]`), you must execute the following documentation and verification routine:

### 1. Professional Version Control & Clean Commit Messages
* Write clear, conventional commit messages (`feat: ...`, `fix: ...`, `chore: ...`).
* Never include internal scratchpad notes, debug logs, or references to deprecated experimental features in public commit messages.
* Keep the git history clean, professional, and presentable for enterprise portfolios and resumes.

### 3. Working Directory & Architecture Rules
* **Rust Backend:** All Rust code, `Cargo.toml`, and database schemas reside inside `watchmark-tauri/src-tauri`. You must navigate to this directory before running any `cargo` commands.
* **React Frontend:** All UI, Tailwind, and React code resides inside `watchmark-tauri/src`.

### 4. Verification Rules (High-Velocity Mode)
* **DO NOT** write automated test files, unit tests, or test functions. 
* **DO NOT** update or write to `TestResult.md`.
* To verify your work, simply run `cargo check` (for Rust) or the equivalent linter (for React) exactly **ONCE** at the end of your implementation.
* Your only verification goal is to ensure the project successfully compiles without syntax, lifetime, or type errors. 

### 5. Requirement Reconciliation & Integrity Audit
Before submitting a Pull Request or marking a task as complete, you must perform a mandatory "Integrity Audit" to ensure zero feature loss and total compliance:
*   **Cross-Reference Requirements:** Re-read the original user prompt line-by-line. Cross-reference every explicit instruction and "edge case" against your implemented code. Ensure the logic handles the edge cases, even without formal test files.
*   **Audit for Regression:** Review all changes to ensure that new logic does not inadvertently remove or break features established in previous commits. If you refactor a function, you must carry over all existing functionality.
*   **Enforce Project Standards:** Ensure all new and modified logic adheres to the established architectural patterns (e.g., `useAsyncInvoke`, `requestId` for commands).
*   **Self-Correction:** If the audit reveals any missing requirements or inconsistent patterns, you must resolve them immediately before finalizing your commit.
*   cleanup additional files before PR



**Execution Trigger:** 
Always perform these steps automatically before finalizing a commit or marking a prompt as fully complete. Do not ask for permission to do this; it is mandatory for every update.
