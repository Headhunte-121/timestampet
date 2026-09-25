import React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Sparkles, X, ChevronRight, Tv } from "lucide-react";
import { SafeImage } from "./ui/SafeImage";
import { useAppStore } from "../store/useAppStore";

export interface NewSeasonAlert {
  media_id: number;
  season_num: number;
  show_title: string;
  poster_path: string;
  backdrop_path: string;
  episode_count: number;
}

interface NewSeasonAlertBannerProps {
  alerts: NewSeasonAlert[];
  onDismiss: (mediaId: number) => void;
  onSelectMedia: (mediaId: number) => void;
}

export const NewSeasonAlertBanner: React.FC<NewSeasonAlertBannerProps> = ({
  alerts,
  onDismiss,
  onSelectMedia,
}) => {
  const { isCinemaMode } = useAppStore();

  if (!alerts || alerts.length === 0) return null;

  return (
    <div className="w-full mb-8 space-y-3">
      <AnimatePresence mode="popLayout">
        {alerts.map((alert) => (
          <motion.div
            key={`${alert.media_id}-${alert.season_num}`}
            layout
            initial={isCinemaMode ? { opacity: 0, y: -20, scale: 0.98 } : { opacity: 1, y: 0 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={isCinemaMode ? { opacity: 0, height: 0, marginBottom: 0, scale: 0.95 } : { opacity: 0 }}
            transition={isCinemaMode ? { type: "spring", stiffness: 350, damping: 25 } : { duration: 0.2 }}
            className="relative overflow-hidden rounded-2xl border border-[#FF6B00]/40 bg-gradient-to-r from-[#1F222A]/90 via-[#27201c]/80 to-[#1F222A]/90 backdrop-blur-xl p-4 md:p-5 shadow-2xl shadow-[#FF6B00]/10 flex flex-col sm:flex-row items-center justify-between gap-4 group"
          >
            {/* Ambient Background Glow */}
            <div className="absolute -left-20 -top-20 w-48 h-48 bg-[#FF6B00]/15 rounded-full blur-3xl pointer-events-none" />

            {/* Left Content with Poster Thumbnail and Info */}
            <div className="flex items-center gap-4 w-full sm:w-auto z-10">
              <div className="relative w-14 h-20 md:w-16 md:h-24 rounded-xl overflow-hidden flex-shrink-0 bg-[#0D0F14] shadow-md border border-white/10 group-hover:scale-105 transition-transform duration-300">
                {alert.poster_path ? (
                  <SafeImage
                    srcPath={alert.poster_path}
                    type="poster"
                    altText={alert.show_title}
                    title={alert.show_title}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center text-muted">
                    <Tv className="w-6 h-6 text-[#FF6B00]/60" />
                  </div>
                )}
              </div>

              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1.5 flex-wrap">
                  <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-black uppercase tracking-wider bg-[#FF6B00]/20 text-[#FF8533] border border-[#FF6B00]/30 animate-pulse">
                    <Sparkles className="w-3.5 h-3.5 fill-[#FF6B00] text-[#FF6B00]" />
                    New Season Premiered
                  </span>
                  <span className="text-xs text-white/50 font-medium">
                    Season {alert.season_num}
                  </span>
                </div>
                <h3 className="text-lg md:text-xl font-bold text-white tracking-tight truncate">
                  {alert.show_title}
                </h3>
                <p className="text-xs md:text-sm text-[#A0AEC0] mt-0.5">
                  {alert.episode_count > 0
                    ? `${alert.episode_count} new episodes now available to watch.`
                    : "New season announced and ready in your library."}
                </p>
              </div>
            </div>

            {/* Right Action Buttons */}
            <div className="flex items-center gap-3 w-full sm:w-auto justify-end z-10">
              <button
                onClick={() => onSelectMedia(alert.media_id)}
                className="flex items-center gap-1.5 px-5 py-2.5 rounded-xl font-bold text-sm bg-[#FF6B00] hover:bg-[#FF8533] text-white shadow-lg shadow-[#FF6B00]/30 hover:shadow-[#FF6B00]/50 transition-all duration-200 cursor-pointer active:scale-95"
              >
                <span>View Season {alert.season_num}</span>
                <ChevronRight className="w-4 h-4" />
              </button>
              <button
                onClick={() => onDismiss(alert.media_id)}
                className="p-2.5 rounded-xl text-white/60 hover:text-white hover:bg-white/10 transition-colors cursor-pointer"
                title="Dismiss alert"
                aria-label="Dismiss alert"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
};
