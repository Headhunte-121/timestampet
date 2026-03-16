import { invoke } from '@tauri-apps/api/core';

/**
 * WatchMark Frontend Plain-English Logging Blueprint
 *
 * Provides a "Cinema-Grade" script of exactly what the user and application are doing,
 * categorized by context and prefixed with distinct emojis.
 * Also pipes all logs to the Rust backend so they appear centrally in the terminal.
 */

// Helper to push logs to backend
const logToBackend = (level: string, message: string, context?: any) => {
    const isBrowserOnly = !(window as any).__TAURI_INTERNALS__ && !(window as any).__TAURI_IPC__;
    if (isBrowserOnly) return; // Running in simple browser mode, backend invoke won't work

    let contextStr: string | undefined = undefined;
    if (context !== undefined) {
        if (context instanceof Error) {
            contextStr = context.message;
        } else if (typeof context === 'object') {
            try {
                contextStr = JSON.stringify(context);
            } catch (e) {
                contextStr = String(context);
            }
        } else {
            contextStr = String(context);
        }
    }

    invoke('frontend_log', { level, message, context: contextStr }).catch((e) => {
        console.error("Failed to pipe log to backend:", e);
    });
};

export const logger = {
    // 1. Navigation & App Lifecycle
    app: (msg: string) => {
        const text = `[APP] 🚀 ${msg}`;
        console.log(text);
        logToBackend("info", text);
    },
    settings: (msg: string) => {
        const text = `[APP] ⚙️ ${msg}`;
        console.log(text);
        logToBackend("info", text);
    },
    navTo: (msg: string) => {
        const text = `[NAV] ➡️ ${msg}`;
        console.info(text);
        logToBackend("info", text);
    },
    navBack: (msg: string) => {
        const text = `[NAV] ⬅️ ${msg}`;
        console.info(text);
        logToBackend("info", text);
    },

    // 2. User Actions (Clicks & Inputs)
    click: (msg: string) => {
        const text = `[ACTION] 🖱️ User clicked ${msg}.`;
        console.info(text);
        logToBackend("info", text);
    },
    input: (msg: string) => {
        const text = `[ACTION] ⌨️ User typing in ${msg}...`;
        console.info(text);
        logToBackend("info", text);
    },
    action: (msg: string) => {
        const text = `[ACTION] 🖱️ ${msg}`;
        console.info(text);
        logToBackend("info", text);
    },

    // 3. Backend Communication (IPC Requests)
    ipcSend: (cmd: string, context?: any, id?: string) => {
        let actionStr = cmd;
        let detailsStr = "";

        // Map technical command names to plain English
        switch (cmd) {
            case "get_dashboard_data":
                actionStr = "dashboard data";
                break;
            case "get_library_data":
                actionStr = "Library Data";
                if (context) {
                    const page = context.page !== undefined ? ` (Page ${context.page + 1})` : "";
                    const mediaType = context.mediaType || "items";
                    detailsStr = ` for ${mediaType} sorted by '${context.sortBy}'${page}`;
                }
                break;
            case "perform_tmdb_search":
                actionStr = "TMDB Search";
                if (context && context.query) {
                    detailsStr = ` for query: '${context.query}'`;
                }
                break;
            case "get_media_details_db":
                actionStr = "Media Details";
                if (context && context.mediaId) {
                    detailsStr = ` for ID: ${context.mediaId}`;
                }
                break;
            case "toggle_episode_status":
                actionStr = "status toggle";
                if (context && context.episode_id !== undefined) {
                    detailsStr = ` for Episode ${context.episode_id}`;
                }
                break;
            default:
                if (context) {
                    // Fallback to JSON string if it's an unrecognized command,
                    // but limit length so it doesn't flood the console.
                    let cStr = "";
                    try { cStr = JSON.stringify(context); } catch(e) {}
                    detailsStr = ` with args: ${cStr.substring(0, 40)}${cStr.length > 40 ? '...' : ''}`;
                }
                break;
        }

        let text = `[IPC: SEND] 📡 Requesting ${actionStr}${detailsStr}`;
        if (id) {
            // Shorten UUIDs so they don't look ugly
            const shortId = id.split('-')[0];
            text += ` (ID: req-${shortId})`;
        }
        text += '.';
        console.log(text);
        logToBackend("info", text);
    },
    ipcCancel: (id: string, reason: string) => {
        const text = `[IPC: CANCEL] 🛑 ${reason}. Cancelling active background task (ID: ${id}).`;
        console.log(text);
        logToBackend("warn", text);
    },

    // 4. Success States & Data Arrival
    ipcSuccess: (msg: string) => {
        const text = `[IPC: SUCCESS] ✨ ${msg}`;
        console.log(text);
        logToBackend("info", text);
    },
    state: (msg: string) => {
        const text = `[STATE] 🔄 ${msg}`;
        console.log(text);
        logToBackend("info", text);
    },

    // 5. Errors & Edge Cases
    error: (context: string, error?: any) => {
        const text = `[ERROR] 🚨 ${context}`;
        if (error !== undefined) {
            console.error(text, error);
            logToBackend("error", text, error);
        } else {
            console.error(text);
            logToBackend("error", text);
        }
    },
    warn: (msg: string) => {
        const text = `[WARNING] ⚠️ ${msg}`;
        console.warn(text);
        logToBackend("warn", text);
    },

    // 6. The "Unmatched Inbox" Flow
    inboxGroup: (msg: string) => {
        const text = `[INBOX] 📁 User selected the unmatched group: '${msg}'.`;
        console.log(text);
        logToBackend("info", text);
    },
    inboxEdit: (oldQuery: string, newQuery: string) => {
        const text = `[INBOX] ⌨️ User manually edited the search query from '${oldQuery}' to '${newQuery}'.`;
        console.log(text);
        logToBackend("info", text);
    },
    inboxMatch: (tmdbId: string | number) => {
        const text = `[INBOX] 🖱️ User clicked '1-Click Match' for TMDB ID: ${tmdbId}.`;
        console.log(text);
        logToBackend("info", text);
    },
    inboxSuccess: (count: number, showName: string) => {
        const text = `[INBOX] ✅ Successfully linked ${count} local files to '${showName}'. Removing from Inbox.`;
        console.log(text);
        logToBackend("info", text);
    },
    inboxIgnore: (msg: string) => {
        const text = `[INBOX] 🗑️ User clicked 'Ignore'. Blacklisting folder path: ${msg}.`;
        console.log(text);
        logToBackend("warn", text);
    },

    // 7. General Debug
    debug: (msg: string) => {
        const text = `[DEBUG] 🐛 ${msg}`;
        console.log(text);
        logToBackend("debug", text);
    },

    // 8. Toasts
    toast: (msg: string, type: 'success' | 'error' | 'info' = 'info') => {
        let emoji = '🍞';
        if (type === 'success') emoji = '✅';
        if (type === 'error') emoji = '🚨';
        const text = `[UI] ${emoji} Displaying ${type} toast: "${msg}"`;
        if (type === 'error') {
            console.error(text);
            logToBackend("error", text);
        } else {
            console.info(text);
            logToBackend("info", text);
        }
    }
};
