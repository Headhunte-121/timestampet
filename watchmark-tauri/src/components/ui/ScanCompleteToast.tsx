import { motion } from "framer-motion";
import { CheckCircle2, FolderSearch } from "lucide-react";
import { useEffect, useState } from "react";
import { toast as sonnerToast } from "sonner";

export const ScanCompleteToast = ({ result, t }: { result: any; t: string | number }) => {
    const [paused, setPaused] = useState(false);
    const [progress, setProgress] = useState(100);

    useEffect(() => {
        let interval: NodeJS.Timeout;
        if (!paused) {
            interval = setInterval(() => {
                setProgress((prev) => {
                    const next = prev - (100 / 50); // 50 steps = 5000ms
                    if (next <= 0) {
                        clearInterval(interval);
                        sonnerToast.dismiss(t);
                        return 0;
                    }
                    return next;
                });
            }, 100);
        }
        return () => clearInterval(interval);
    }, [paused, t]);

    return (
        <div
            className="relative w-full max-w-[350px] bg-[#1F222A] rounded-xl border border-white/10 shadow-2xl overflow-hidden cursor-default group pointer-events-auto"
            onMouseEnter={() => setPaused(true)}
            onMouseLeave={() => setPaused(false)}
        >
            <div className="p-4 flex items-start gap-4">
                <div className="w-10 h-10 rounded-full bg-[#FF6B00]/10 flex items-center justify-center shrink-0">
                    <CheckCircle2 className="w-5 h-5 text-[#FF6B00]" />
                </div>

                <div className="flex-1">
                    <h3 className="text-white font-bold text-base mb-1">Scan Complete</h3>

                    <div className="flex gap-4 mt-3">
                        <div className="flex flex-col">
                            <span className="text-3xl font-black text-white">{result.auto_matched_count || 0}</span>
                            <span className="text-xs font-bold text-gray-500 uppercase tracking-wider">Matched</span>
                        </div>

                        <div className="w-px bg-white/10 self-stretch" />

                        <div className="flex flex-col">
                            <span className="text-3xl font-black text-[#FF6B00]">{result.unmatched_count || 0}</span>
                            <span className="text-xs font-bold text-gray-500 uppercase tracking-wider">To Review</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Progress Border */}
            <div className="h-1 w-full bg-black/50 absolute bottom-0 left-0">
                <motion.div
                    className="h-full bg-[#FF6B00]"
                    animate={{ width: `${progress}%` }}
                    transition={{ ease: "linear", duration: 0.1 }}
                />
            </div>
        </div>
    );
};
