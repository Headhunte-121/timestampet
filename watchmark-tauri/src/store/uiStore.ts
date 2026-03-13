import { create } from 'zustand';

export type ModalType = 'alert' | 'confirm' | 'prompt' | null;

interface ModalState {
  isOpen: boolean;
  type: ModalType;
  title: string;
  message: string;
  defaultValue: string;
  resolve: ((value: any) => void) | null;
  reject: ((reason?: any) => void) | null;
}

interface UiState {
  modal: ModalState;
  showAlert: (title: string, message: string) => Promise<void>;
  showConfirm: (title: string, message: string) => Promise<boolean>;
  showPrompt: (title: string, message: string, defaultValue?: string) => Promise<string | null>;
  closeModal: (value?: any) => void;
}

export const useUiStore = create<UiState>((set, get) => ({
  modal: {
    isOpen: false,
    type: null,
    title: '',
    message: '',
    defaultValue: '',
    resolve: null,
    reject: null,
  },

  showAlert: (title, message) => {
    return new Promise<void>((resolve) => {
      set({
        modal: {
          isOpen: true,
          type: 'alert',
          title,
          message,
          defaultValue: '',
          resolve: () => resolve(),
          reject: null,
        }
      });
    });
  },

  showConfirm: (title, message) => {
    return new Promise<boolean>((resolve) => {
      set({
        modal: {
          isOpen: true,
          type: 'confirm',
          title,
          message,
          defaultValue: '',
          resolve,
          reject: null,
        }
      });
    });
  },

  showPrompt: (title, message, defaultValue = '') => {
    return new Promise<string | null>((resolve) => {
      set({
        modal: {
          isOpen: true,
          type: 'prompt',
          title,
          message,
          defaultValue,
          resolve,
          reject: null,
        }
      });
    });
  },

  closeModal: (value) => {
    const { resolve } = get().modal;
    if (resolve) {
      resolve(value);
    }
    set({
      modal: {
        isOpen: false,
        type: null,
        title: '',
        message: '',
        defaultValue: '',
        resolve: null,
        reject: null,
      }
    });
  }
}));
