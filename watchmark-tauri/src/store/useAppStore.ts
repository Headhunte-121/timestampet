import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface AppState {
  isCinemaMode: boolean;
  initialized: boolean;
  setCinemaMode: (mode: boolean) => Promise<void>;
  initializeSettings: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  isCinemaMode: true,
  initialized: false,
  setCinemaMode: async (mode: boolean) => {
    set({ isCinemaMode: mode });
    // Note: We intentionally do not auto-save to the backend here.
    // The SettingsView component handles batching and saving via the user's explicit action.
  },
  initializeSettings: async () => {
    if (get().initialized) return;
    try {
      const settings: any = await invoke('get_settings');
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
      console.error("Failed to initialize settings:", e);
      set({ initialized: true });
    }
  }
}));
