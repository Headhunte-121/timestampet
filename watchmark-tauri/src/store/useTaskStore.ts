import { create } from 'zustand';

interface TaskState {
  isImporting: boolean;
  isScanning: boolean;
  isOptimizing: boolean;
  progress: number;
  total: number;
  setProgress: (progress: number, total: number, isImporting: boolean) => void;
  setScanning: (isScanning: boolean) => void;
  setOptimizing: (isOptimizing: boolean) => void;
}

export const useTaskStore = create<TaskState>((set) => ({
  isImporting: false,
  isScanning: false,
  isOptimizing: false,
  progress: 0,
  total: 0,
  setProgress: (progress, total, isImporting) => set({ progress, total, isImporting }),
  setScanning: (isScanning: boolean) => set({ isScanning }),
  setOptimizing: (isOptimizing: boolean) => set({ isOptimizing }),
}));
