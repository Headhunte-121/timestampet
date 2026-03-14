import { create } from 'zustand';

interface TaskState {
  isImporting: boolean;
  progress: number;
  total: number;
  setProgress: (progress: number, total: number, isImporting: boolean) => void;
}

export const useTaskStore = create<TaskState>((set) => ({
  isImporting: false,
  progress: 0,
  total: 0,
  setProgress: (progress, total, isImporting) => set({ progress, total, isImporting }),
}));
