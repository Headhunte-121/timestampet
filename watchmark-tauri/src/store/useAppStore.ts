import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { logger } from '../utils/logger';

interface AppState {
  isCinemaMode: boolean;
  initialized: boolean;
  isApiAuthorized: boolean;
  isOffline: boolean;
  fastHistory: any[];
  setCinemaMode: (mode: boolean) => Promise<void>;
  setApiAuthorized: (authorized: boolean) => void;
  setOffline: (offline: boolean) => void;
  initializeSettings: () => Promise<void>;
  fetchFastHistory: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  isCinemaMode: true,
  initialized: false,
  isApiAuthorized: true,
  isOffline: false,
  fastHistory: [],
  setApiAuthorized: (authorized: boolean) => set({ isApiAuthorized: authorized }),
  setOffline: (offline: boolean) => set({ isOffline: offline }),
  setCinemaMode: async (mode: boolean) => {
    set({ isCinemaMode: mode });
    // Note: We intentionally do not auto-save to the backend here.
    // The SettingsView component handles batching and saving via the user's explicit action.
  },
  fetchFastHistory: async () => {
    try {
      const history = await invoke('fetch_history', { requestId: "fast_history_init", page: 0, pageSize: 10 });
      set({ fastHistory: history as any[] });
    } catch (e) {
      logger.error("Failed to load fast history", e);
    }
  },
  initializeSettings: async () => {
    if (get().initialized) return;
    try {
      const settings: any = await invoke('get_settings');
      logger.settings("Loaded user settings from backend.");
      // If the backend has a value, use it. If not, fallback to true but check reduced motion.
      let mode = true;
      if (settings && typeof settings.cinema_mode === 'boolean') {
        mode = settings.cinema_mode;
      } else {
        const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        mode = !prefersReducedMotion;
        // Optionally save this back if it's the first run
      }
      set({ isCinemaMode: mode, initialized: true });
    } catch (e) {
      logger.error("Failed to initialize settings", e);
      set({ initialized: true });
    }
  }
}));
