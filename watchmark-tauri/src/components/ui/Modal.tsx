import React, { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useUiStore } from '../../store/uiStore';

export function Modal() {
  const { modal, closeModal } = useUiStore();
  const [inputValue, setInputValue] = useState(modal.defaultValue);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (modal.isOpen && modal.type === 'prompt') {
      setInputValue(modal.defaultValue);
      setTimeout(() => {
        inputRef.current?.focus();
        inputRef.current?.select();
      }, 50);
    }
  }, [modal.isOpen, modal.type, modal.defaultValue]);

  const handleConfirm = () => {
    if (modal.type === 'prompt') {
      closeModal(inputValue);
    } else if (modal.type === 'confirm') {
      closeModal(true);
    } else {
      closeModal();
    }
  };

  const handleCancel = () => {
    if (modal.type === 'prompt') {
      closeModal(null);
    } else if (modal.type === 'confirm') {
      closeModal(false);
    } else {
      closeModal();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleConfirm();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleCancel();
    }
  };

  return (
    <AnimatePresence>
      {modal.isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0"
            onClick={handleCancel}
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            transition={{ type: 'spring', damping: 25, stiffness: 300 }}
            className="relative w-full max-w-md bg-[#1A1C23] border border-white/10 rounded-xl shadow-2xl p-6 overflow-hidden flex flex-col gap-4"
          >
            <div>
              <h2 className="text-xl font-bold text-white mb-2">{modal.title}</h2>
              {modal.message && (
                <p className="text-white/70 text-sm whitespace-pre-wrap">{modal.message}</p>
              )}
            </div>

            {modal.type === 'prompt' && (
              <input
                ref={inputRef}
                type="text"
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
                onKeyDown={handleKeyDown}
                className="w-full bg-[#0D0F14] border border-white/10 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-[#FF6B00] transition-colors"
              />
            )}

            <div className="flex justify-end gap-3 mt-4">
              {(modal.type === 'confirm' || modal.type === 'prompt') && (
                <button
                  onClick={handleCancel}
                  className="px-4 py-2 rounded-lg text-white/70 hover:text-white hover:bg-white/5 transition-colors font-medium"
                >
                  Cancel
                </button>
              )}
              <button
                onClick={handleConfirm}
                autoFocus={modal.type !== 'prompt'}
                className="px-6 py-2 rounded-lg bg-[#FF6B00] text-white hover:bg-[#FF8533] transition-colors font-medium"
              >
                {modal.type === 'alert' ? 'OK' : 'Confirm'}
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
