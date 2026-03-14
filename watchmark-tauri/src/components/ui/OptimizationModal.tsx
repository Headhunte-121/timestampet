import { motion, AnimatePresence } from "framer-motion";
import { useTaskStore } from "../../store/useTaskStore";
import { Play } from "lucide-react";

export function OptimizationModal() {
  const { isOptimizing } = useTaskStore();

  return (
    <AnimatePresence>
      {isOptimizing && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-[300] flex items-center justify-center bg-[#0D0F14]/80 backdrop-blur-xl pointer-events-auto"
        >
          <div className="flex flex-col items-center">
            {/* Pulsing Logo Centerpiece */}
            <motion.div
              animate={{
                scale: [1, 1.1, 1],
                boxShadow: [
                  "0 0 0px rgba(255, 107, 0, 0)",
                  "0 0 40px rgba(255, 107, 0, 0.4)",
                  "0 0 0px rgba(255, 107, 0, 0)"
                ]
              }}
              transition={{
                duration: 2,
                repeat: Infinity,
                ease: "easeInOut"
              }}
              className="w-32 h-32 rounded-full flex items-center justify-center bg-[#1F222A] mb-8"
            >
              <Play className="w-16 h-16 text-[#FF6B00] ml-2" fill="currentColor" />
            </motion.div>

            <motion.h2
              initial={{ y: 20, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              transition={{ delay: 0.2 }}
              className="text-2xl font-bold text-gray-300 tracking-wider mb-2"
            >
              Defragmenting Library & Optimizing Indices...
            </motion.h2>
            <motion.p
              initial={{ y: 20, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              transition={{ delay: 0.4 }}
              className="text-sm text-gray-500"
            >
              Please wait while WatchMark performs maintenance.
            </motion.p>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
