import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Loader2, TriangleAlert } from "lucide-react";
import { cn } from "../../App";

interface RestoreConfirmationModalProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  isRestoring: boolean;
}

export function RestoreConfirmationModal({
  isOpen,
  onConfirm,
  onCancel,
  isRestoring
}: RestoreConfirmationModalProps) {
  const [confirmText, setConfirmText] = useState("");

  const handleCancel = () => {
    if (isRestoring) return;
    setConfirmText("");
    onCancel();
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        handleCancel();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, isRestoring, onCancel]);

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-[9999] flex items-center justify-center pointer-events-auto">
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-black/80 backdrop-blur-xl"
            onClick={handleCancel}
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            className="relative w-full max-w-lg bg-[#1a1b26] border border-white/10 rounded-3xl p-8 shadow-2xl flex flex-col items-center text-center overflow-hidden"
            onClick={e => e.stopPropagation()}
          >
            <div className="w-20 h-20 bg-[#EF4444]/10 rounded-full flex items-center justify-center mb-6">
              <TriangleAlert className="w-10 h-10 text-[#EF4444]" />
            </div>

            <h2 className="text-3xl font-bold text-white mb-4 flex items-center gap-2">
              <span className="text-[#EF4444]">⚠️</span> Overwrite Current Library?
            </h2>

            <p className="text-muted mb-8 text-lg">
              This will permanently delete your current watch history and ratings. This action cannot be undone.
            </p>

            <div className="w-full bg-black/40 rounded-xl p-6 border border-[#EF4444]/20 mb-8">
              <label className="block text-sm font-bold text-gray-300 mb-2">
                Type <span className="text-[#EF4444] select-all">RESTORE</span> to confirm
              </label>
              <input
                type="text"
                value={confirmText}
                onChange={(e) => setConfirmText(e.target.value)}
                disabled={isRestoring}
                className="w-full bg-black/60 text-white px-4 py-3 rounded-lg border border-white/10 focus:border-[#EF4444] outline-none text-center font-mono text-xl tracking-widest uppercase"
                placeholder="RESTORE"
                autoComplete="off"
                spellCheck="false"
              />
            </div>

            <div className="flex gap-4 w-full">
              <button
                onClick={handleCancel}
                disabled={isRestoring}
                className="flex-1 px-6 py-4 bg-white/5 hover:bg-white/10 text-white font-bold rounded-xl transition-colors disabled:opacity-50"
              >
                Cancel
              </button>

              <button
                onClick={onConfirm}
                disabled={confirmText !== "RESTORE" || isRestoring}
                className={cn(
                  "flex-1 px-6 py-4 font-bold rounded-xl transition-all flex items-center justify-center gap-2 relative overflow-hidden",
                  confirmText === "RESTORE"
                    ? "bg-[#EF4444] text-white hover:bg-[#DC2626] shadow-[0_0_20px_rgba(239,68,68,0.4)]"
                    : "bg-white/5 text-gray-500 cursor-not-allowed"
                )}
              >
                {isRestoring && (
                  <div className="absolute inset-0 w-full h-full bg-gradient-to-r from-transparent via-white/20 to-transparent animate-[shimmer_1.5s_infinite]" />
                )}
                {isRestoring ? (
                  <>
                    <Loader2 className="w-5 h-5 animate-spin" />
                    Restoring...
                  </>
                ) : (
                  "Confirm and Restart"
                )}
              </button>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
