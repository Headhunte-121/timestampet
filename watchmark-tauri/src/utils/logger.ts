/**
 * WatchMark Frontend Plain-English Logging Blueprint
 *
 * Provides a "Cinema-Grade" script of exactly what the user and application are doing,
 * categorized by context and prefixed with distinct emojis.
 */

export const logger = {
    // 1. Navigation & App Lifecycle
    app: (msg: string) => console.log(`[APP] 🚀 ${msg}`),
    settings: (msg: string) => console.log(`[APP] ⚙️ ${msg}`),
    navTo: (msg: string) => console.info(`[NAV] ➡️ ${msg}`),
    navBack: (msg: string) => console.info(`[NAV] ⬅️ ${msg}`),

    // 2. User Actions (Clicks & Inputs)
    click: (msg: string) => console.info(`[ACTION] 🖱️ User clicked ${msg}.`),
    input: (msg: string) => console.info(`[ACTION] ⌨️ User typing in ${msg}...`),
    action: (msg: string) => console.info(`[ACTION] 🖱️ ${msg}`),

    // 3. Backend Communication (IPC Requests)
    ipcSend: (cmd: string, context?: string, id?: string) => {
        let msg = `[IPC: SEND] 📡 Requesting '${cmd}'`;
        if (context) msg += ` for ${context}`;
        if (id) msg += ` (ID: ${id})`;
        console.log(msg + '.');
    },
    ipcCancel: (id: string, reason: string) => console.log(`[IPC: CANCEL] 🛑 ${reason}. Cancelling active background task (ID: ${id}).`),

    // 4. Success States & Data Arrival
    ipcSuccess: (msg: string) => console.log(`[IPC: SUCCESS] ✨ ${msg}`),
    state: (msg: string) => console.log(`[STATE] 🔄 ${msg}`),

    // 5. Errors & Edge Cases
    error: (context: string, error?: any) => {
        if (error !== undefined) {
            console.error(`[ERROR] 🚨 ${context}`, error);
        } else {
            console.error(`[ERROR] 🚨 ${context}`);
        }
    },
    warn: (msg: string) => console.warn(`[WARNING] ⚠️ ${msg}`),

    // 6. The "Unmatched Inbox" Flow
    inboxGroup: (msg: string) => console.log(`[INBOX] 📁 User selected the unmatched group: '${msg}'.`),
    inboxEdit: (oldQuery: string, newQuery: string) => console.log(`[INBOX] ⌨️ User manually edited the search query from '${oldQuery}' to '${newQuery}'.`),
    inboxMatch: (tmdbId: string | number) => console.log(`[INBOX] 🖱️ User clicked '1-Click Match' for TMDB ID: ${tmdbId}.`),
    inboxSuccess: (count: number, showName: string) => console.log(`[INBOX] ✅ Successfully linked ${count} local files to '${showName}'. Removing from Inbox.`),
    inboxIgnore: (msg: string) => console.log(`[INBOX] 🗑️ User clicked 'Ignore'. Blacklisting folder path: ${msg}.`),

    // 7. General Debug
    debug: (msg: string) => console.log(`[DEBUG] 🐛 ${msg}`)
};
