import { create } from 'zustand';

interface TaskState {
  isImporting: boolean;
  isScanning: boolean;
  progress: number;
  total: number;
  setProgress: (progress: number, total: number, isImporting: boolean) => void;
  setScanning: (isScanning: boolean) => void;
}

export const useTaskStore = create<TaskState>((set) => ({
  isImporting: false,
  isScanning: false,
  progress: 0,
  total: 0,
  setProgress: (progress, total, isImporting) => set({ progress, total, isImporting }),
  setScanning: (isScanning: boolean) => set({ isScanning }),
}));
