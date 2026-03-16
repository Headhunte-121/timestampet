import { create } from 'zustand';

interface TaskState {
  isImporting: boolean;
  isScanning: boolean;
  isScanPaused: boolean;
  isOptimizing: boolean;
  progress: number;
  total: number;
  activeSyncs: Record<string, number>;
  setProgress: (progress: number, total: number, isImporting: boolean) => void;
  setScanning: (isScanning: boolean) => void;
  setScanPaused: (isScanPaused: boolean) => void;
  setOptimizing: (isOptimizing: boolean) => void;
  updateSyncProgress: (id: string, current: number, total: number) => void;
}

export const useTaskStore = create<TaskState>((set) => ({
  isImporting: false,
  isScanning: false,
  isScanPaused: false,
  isOptimizing: false,
  progress: 0,
  total: 0,
  activeSyncs: {},
  setProgress: (progress, total, isImporting) => set({ progress, total, isImporting }),
  setScanning: (isScanning: boolean) => set({ isScanning }),
  setScanPaused: (isScanPaused: boolean) => set({ isScanPaused }),
  setOptimizing: (isOptimizing: boolean) => set({ isOptimizing }),
  updateSyncProgress: (id: string, current: number, total: number) => set((state) => {
    const newSyncs = { ...state.activeSyncs };
    if (current > total) {
      delete newSyncs[id];
    } else {
      newSyncs[id] = (current / total) * 100;
    }
    return { activeSyncs: newSyncs };
  }),
}));
