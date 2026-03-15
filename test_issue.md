User says dashboard works fine but tv page is broken again.
The TV page is likely `Library.tsx` (currentView === "tv").
In `Library.tsx`, it calls `get_library_data`.
Let me check if `get_library_data` was modified or if it uses `useTaskStore.getState().activeSyncs`.
Also, check browser console output from before:
"An error occurred in the <Dashboard> component. Consider adding an error boundary..."
Wait, the error was in Dashboard earlier, but now the user says Dashboard is fine and TV page is broken.
Let me reproduce the issue.
