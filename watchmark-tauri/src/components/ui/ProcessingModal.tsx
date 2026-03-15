import { motion, AnimatePresence } from "framer-motion";
import { useUiStore } from "../../store/uiStore";

export function ProcessingModal() {
  const { isProcessing, processingMessage } = useUiStore();

  return (
    <AnimatePresence>
      {isProcessing && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 backdrop-blur-xl"
        >
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.95, opacity: 0 }}
            className="bg-[#1F222A] p-8 rounded-2xl border border-white/10 shadow-2xl flex flex-col items-center max-w-sm w-full text-center"
          >
            <div className="w-16 h-16 border-4 border-[#FF6B00] border-t-transparent rounded-full animate-spin mb-6" />
            <h2 className="text-xl font-bold text-white mb-2">Processing...</h2>
            <p className="text-muted">{processingMessage}</p>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
