import React, { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence, useReducedMotion } from "framer-motion";
import { ChevronDown, Moon, Play } from "lucide-react";
import { SafeImage } from "./ui/SafeImage";
import { formatImagePath } from "../utils/imageFormat";

interface BingeBlockProps {
  group: any;
  isEven: boolean;
  onNavigateToMedia: (mediaId: number, seasonNum?: number) => void;
  formatPauseTime: (seconds: number) => string;
}

export function BingeBlock({ group, isEven, onNavigateToMedia, formatPauseTime }: BingeBlockProps) {
  const [isOpen, setIsOpen] = useState(false);
  const shouldReduceMotion = useReducedMotion();
  const headerRef = useRef<HTMLDivElement>(null);

  // Guard clause: The "Orphaned Episode" Bug
  if (!group.entries || group.entries.length === 0) {
    return null;
  }

  const mainEntry = group.main_entry;
  const isLegacy = mainEntry.is_legacy === 1;

  // Calculate Subtitles and Vibe
  let bingeSubtitle = null;
  const startTs = group.entries[group.entries.length - 1].timestamp;
  const endTs = group.entries[0].timestamp;
  const startDate = new Date(startTs * 1000);
  const endDate = new Date(endTs * 1000);

  const getVibeLabel = (date: Date) => {
    const hour = date.getHours();
    if (hour >= 5 && hour < 12) return "Morning";
    if (hour >= 12 && hour < 17) return "Afternoon";
    if (hour >= 17 && hour < 21) return "Evening";
    return "Late Night";
  };

  const getDayName = (date: Date) => {
    return date.toLocaleDateString("en-US", { weekday: "short" });
  };

  if (startDate.toDateString() !== endDate.toDateString()) {
    bingeSubtitle = `${getDayName(startDate)} ${getVibeLabel(startDate)} – ${getDayName(endDate)} ${getVibeLabel(endDate)}`;
  } else {
    bingeSubtitle = `${getDayName(startDate)} ${getVibeLabel(startDate)}`;
  }

  // The "Month-Spanning" Binge check for midnight crossover
  // We use the start_time of the session. If it crosses midnight, show the moon icon.
  const hasMidnightCrossover = startDate.getDate() !== endDate.getDate();

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation();

    // Smart Click filter: Ignore if text is selected
    if (window.getSelection()?.toString().length) return;

    setIsOpen(!isOpen);
  };

  // Auto-Scroll Adjuster
  useEffect(() => {
    if (isOpen && headerRef.current) {
      // Small timeout to allow the accordion animation to start
      setTimeout(() => {
        if (headerRef.current) {
           const rect = headerRef.current.getBoundingClientRect();
           const viewportHeight = window.innerHeight || document.documentElement.clientHeight;

           // If expanding pushes the header off screen (or if it's very close to the bottom)
           if (rect.bottom > viewportHeight - 100 || rect.top < 64) {
               headerRef.current.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
           }
        }
      }, 50);
    }
  }, [isOpen]);

  const durationHours = Math.floor(group.total_runtime / 60);
  const durationMins = group.total_runtime % 60;

  const animationDuration = shouldReduceMotion ? 0 : 0.3;

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "100px" }}
      transition={{ duration: 0.3 }}
      className={`ml-16 lg:ml-0 lg:w-[calc(50%-2rem)] ${isEven ? 'lg:mr-auto' : 'lg:ml-auto'} z-10 relative`}
    >
      <div
        className={`flex flex-col rounded-2xl border border-[#2A2D35] bg-[#1F222A]/40 shadow-2xl transition-colors ${
          isLegacy ? "opacity-60" : "backdrop-blur-md"
        }`}
      >
        {/* Header Section (The Toggle Button) */}
        <div
          ref={headerRef}
          className="relative flex p-4 items-center gap-6 cursor-pointer hover:bg-white/5 transition-colors rounded-t-2xl z-20"
          onClick={handleToggle}
        >
          {isLegacy && (
            <div className="absolute top-2 right-2 bg-white/10 text-muted font-bold text-[10px] px-2 py-1 rounded-full uppercase tracking-wider">
              Archived
            </div>
          )}

          <SafeImage
            srcPath={mainEntry.poster_path ? formatImagePath(mainEntry.poster_path, "w500") : ""}
            type="poster"
            title={mainEntry.show_title}
            altText={mainEntry.show_title}
            className="w-16 h-24 object-cover rounded-md shadow-md flex-shrink-0"
          />

          <div className="flex-1 min-w-0 pr-4">
            <h3 className="text-lg truncate mb-1">
              <span className="text-[#A0AEC0]">Watched </span>
              <span className="font-bold text-[#FF6B00]">{group.episode_count} </span>
              <span className="text-[#A0AEC0]">
                {group.episode_count === 1 ? "Episode of " : "Episodes of "}
              </span>
              <span className="font-bold text-[#FF6B00]">{mainEntry.show_title}</span>
            </h3>

            <div className="flex items-center gap-2 text-sm text-gray-500 font-medium italic mt-1 truncate">
              {bingeSubtitle && <span>{bingeSubtitle}</span>}
              <span className="mx-1 opacity-40">•</span>
              <span>{durationHours > 0 ? `${durationHours}h ` : ''}{durationMins}m</span>
              {hasMidnightCrossover && (
                <Moon size={14} className="text-[#A0AEC0] ml-1" />
              )}
            </div>
          </div>

          <div className="flex-shrink-0 pr-4">
             <motion.div
                animate={{ rotate: isOpen ? 180 : 0 }}
                transition={{ duration: animationDuration, ease: "easeInOut" }}
             >
                <ChevronDown className="text-white/50" size={24} />
             </motion.div>
          </div>
        </div>

        {/* Expanded Accordion Content */}
        <AnimatePresence initial={false}>
          {isOpen && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ duration: animationDuration, ease: "easeInOut" }}
              className="overflow-hidden"
              onAnimationComplete={() => {
                 // The "Drop-Shadow" Clip test case resolution:
                 // Allow overflow to visible if needed after animation?
                 // Given the requirements we need strictly to contain to border, so overflow-hidden is fine to remain,
                 // but we can remove it if tooltips are needed. Usually Framer handles this, but since we use
                 // a wrapper with overflow-hidden, tooltips inside might be clipped.
                 // For now, adhering strictly to 'overflow-hidden utility class'.
              }}
            >
              <div className="px-4 pb-4 pt-0 border-t border-white/5 flex flex-col gap-3">
                {group.entries.map((entry: any, index: number) => {
                  const isCompleted = entry.completion_ratio >= 0.9 || entry.status === "Completed";
                  const isPaused = entry.last_position > 0 && entry.completion_ratio < 0.9;

                  return (
                    <div
                      key={index}
                      className="flex items-center gap-4 bg-black/20 rounded-lg p-2 hover:bg-white/5 transition-colors cursor-pointer group"
                      onClick={(e) => {
                        e.stopPropagation();
                        onNavigateToMedia(entry.media_id, entry.season_num);
                      }}
                    >
                      <div className="relative w-24 h-14 flex-shrink-0 rounded-md overflow-hidden bg-[#0D0F14]">
                        <SafeImage
                          srcPath={entry.still_path || entry.backdrop_path ? formatImagePath(entry.still_path || entry.backdrop_path, "w500") : ""}
                          type="backdrop"
                          title={entry.ep_title}
                          altText={entry.ep_title}
                          className="w-full h-full object-cover"
                        />
                        <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                           <Play size={20} className="text-white fill-white" />
                        </div>
                      </div>

                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-bold text-white truncate">
                          S{entry.season_num} E{entry.ep_num} - {entry.ep_title}
                        </p>
                        <p className="text-xs text-[#A0AEC0] mt-1">
                          {isLegacy ? (
                            entry.formatted_time
                          ) : isPaused ? (
                            <span className="text-[#FF6B00]">Paused at {formatPauseTime(entry.last_position)}</span>
                          ) : isCompleted ? (
                            <span className="text-green-500">Completed</span>
                          ) : (
                            entry.formatted_time
                          )}
                        </p>
                      </div>

                      {/* Circular Orange Play Button for instant resume */}
                      <button
                        className="w-8 h-8 rounded-full bg-[#FF6B00]/20 flex items-center justify-center text-[#FF6B00] hover:bg-[#FF6B00] hover:text-white transition-colors mr-2 flex-shrink-0"
                        onClick={(e) => {
                          e.stopPropagation();
                          // In a real implementation this would trigger VLC play,
                          // but for now we follow the same pattern as the rest of the row
                          onNavigateToMedia(entry.media_id, entry.season_num);
                        }}
                      >
                         <Play size={14} className="ml-0.5 fill-current" />
                      </button>
                    </div>
                  );
                })}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </motion.div>
  );
}
