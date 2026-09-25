import React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronLeft, ChevronRight, Flame, Zap, Calendar, Tv } from "lucide-react";
import { SafeImage } from "./ui/SafeImage";
import { useHorizontalScroll } from "../hooks/useHorizontalScroll";
import { useAppStore } from "../store/useAppStore";
import { formatImagePath } from "../utils/imageFormat";

export interface UpcomingAiring {
  episode_id: number;
  season_num: number;
  ep_num: number;
  ep_title: string;
  air_date: string;
  still_path: string;
  overview: string;
  media_id: number;
  show_title: string;
  poster_path: string;
  backdrop_path: string;
  network: string;
  days_until: number;
  is_today: boolean;
  is_tomorrow: boolean;
  relative_air_string: string;
}

interface UpcomingAiringRowProps {
  airings: UpcomingAiring[];
  onSelectMedia: (mediaId: number) => void;
}

const PLACEHOLDER_BACKDROP = "https://images.unsplash.com/photo-1542293787-827fb705d15a?q=80&w=2560&auto=format&fit=crop";

export const UpcomingAiringRow: React.FC<UpcomingAiringRowProps> = ({
  airings,
  onSelectMedia,
}) => {
  const { isCinemaMode } = useAppStore();
  const scroll = useHorizontalScroll<HTMLDivElement>();

  if (!airings || airings.length === 0) return null;

  return (
    <div className="relative group/airing mb-12">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-xl bg-[#FF6B00]/15 text-[#FF6B00] border border-[#FF6B00]/20">
            <Flame className="w-5 h-5 fill-[#FF6B00]" />
          </div>
          <div>
            <h2 className="text-2xl font-bold text-white tracking-tight flex items-center gap-2">
              Upcoming Airings
              <span className="text-xs px-2 py-0.5 rounded-full bg-[#FF6B00]/20 text-[#FF8533] border border-[#FF6B00]/30 font-semibold">
                {airings.length} {airings.length === 1 ? "release" : "releases"} this week
              </span>
            </h2>
            <p className="text-xs text-[#A0AEC0] mt-0.5">
              Episodes scheduled to air in the next 7 days from your tracked series
            </p>
          </div>
        </div>
      </div>

      {/* Navigation Chevrons */}
      <button
        onClick={scroll.scrollLeft}
        className="absolute left-0 top-1/2 -translate-y-1/2 -ml-4 z-40 p-3 bg-black/50 backdrop-blur-md hover:bg-black/75 rounded-full text-white opacity-0 group-hover/airing:opacity-100 transition-opacity duration-200 cursor-pointer shadow-xl border border-white/10"
        aria-label="Scroll left"
      >
        <ChevronLeft className="w-6 h-6" />
      </button>
      <button
        onClick={scroll.scrollRight}
        className="absolute right-0 top-1/2 -translate-y-1/2 -mr-4 z-40 p-3 bg-black/50 backdrop-blur-md hover:bg-black/75 rounded-full text-white opacity-0 group-hover/airing:opacity-100 transition-opacity duration-200 cursor-pointer shadow-xl border border-white/10"
        aria-label="Scroll right"
      >
        <ChevronRight className="w-6 h-6" />
      </button>

      {/* Carousel Container */}
      <motion.div
        ref={scroll.elRef}
        layout
        className={`flex gap-5 overflow-x-auto pb-4 pt-1 scrollbar-hide snap-x snap-mandatory pr-[20%] ${
          airings.length <= 2 ? "justify-start max-w-full" : ""
        }`}
      >
        <AnimatePresence mode="popLayout">
          {airings.map((item) => {
            const thumbnailPath = item.still_path
              ? formatImagePath(item.still_path, "w500")
              : item.backdrop_path
              ? formatImagePath(item.backdrop_path, "w500")
              : item.poster_path
              ? formatImagePath(item.poster_path, "w500")
              : "";

            return (
              <motion.div
                key={item.episode_id}
                layout="position"
                initial={isCinemaMode ? { opacity: 0, scale: 0.92 } : { opacity: 1, scale: 1 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={isCinemaMode ? { opacity: 0, scale: 0.9, width: 0, marginLeft: -20 } : { opacity: 0 }}
                transition={isCinemaMode ? { type: "spring", stiffness: 300, damping: 30 } : { duration: 0 }}
                onClick={() => onSelectMedia(item.media_id)}
                whileHover={isCinemaMode ? { scale: 1.03 } : {}}
                className={`flex-none w-[300px] md:w-[340px] bg-[#1F222A] rounded-2xl overflow-hidden cursor-pointer group/card transition-all duration-300 snap-start shadow-xl relative border ${
                  item.is_today
                    ? "border-[#FF6B00] shadow-[0_0_20px_rgba(255,107,0,0.25)] hover:shadow-[0_0_30px_rgba(255,107,0,0.4)] ring-1 ring-[#FF6B00]/40"
                    : "border-white/5 hover:border-white/20 hover:shadow-2xl hover:shadow-black/50"
                }`}
              >
                {/* Image Container with Badges */}
                <div className="w-full aspect-video relative overflow-hidden bg-[#0D0F14]">
                  {thumbnailPath ? (
                    <SafeImage
                      srcPath={thumbnailPath}
                      fallbackSrcPath={PLACEHOLDER_BACKDROP}
                      type="still"
                      altText={item.ep_title || item.show_title}
                      title={item.ep_title}
                      episodeNumber={item.ep_num}
                      className="w-full h-full object-cover transition-transform duration-500 group-hover/card:scale-105"
                    />
                  ) : (
                    <div className="w-full h-full flex flex-col items-center justify-center bg-gradient-to-br from-[#1F222A] to-[#0D0F14] text-muted">
                      <Tv className="w-10 h-10 text-[#FF6B00]/40 mb-2" />
                      <span className="text-xs uppercase font-mono tracking-widest text-white/50">
                        S{item.season_num} E{item.ep_num}
                      </span>
                    </div>
                  )}

                  {/* Gradient bottom overlay on image */}
                  <div className="absolute inset-0 bg-gradient-to-t from-[#1F222A] via-transparent to-black/30 pointer-events-none" />

                  {/* Top-Left Status Pill */}
                  <div className="absolute top-3 left-3 z-10">
                    {item.is_today ? (
                      <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-black uppercase tracking-wider bg-[#FF6B00] text-white shadow-lg shadow-[#FF6B00]/50 animate-pulse">
                        <Flame className="w-3.5 h-3.5 fill-white" />
                        Airing Today
                      </span>
                    ) : item.is_tomorrow ? (
                      <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-black uppercase tracking-wider bg-amber-500/90 text-black backdrop-blur-md shadow-md">
                        <Zap className="w-3.5 h-3.5 fill-black" />
                        Tomorrow
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold bg-black/60 text-white/90 backdrop-blur-md border border-white/10">
                        <Calendar className="w-3.5 h-3.5 text-[#FF8533]" />
                        {item.relative_air_string}
                      </span>
                    )}
                  </div>

                  {/* Network / Channel Pill (Top Right) */}
                  {item.network && (
                    <div className="absolute top-3 right-3 z-10">
                      <span className="px-2 py-0.5 rounded-lg text-[10px] font-bold uppercase tracking-wider bg-black/60 text-white/80 backdrop-blur-md border border-white/10 max-w-[110px] truncate block">
                        {item.network}
                      </span>
                    </div>
                  )}
                </div>

                {/* Card Meta Content */}
                <div className="p-4">
                  <h3 className="text-base font-bold text-white truncate tracking-tight group-hover/card:text-[#FF8533] transition-colors">
                    {item.show_title}
                  </h3>

                  <div className="flex items-center gap-2 mt-1 text-xs text-[#A0AEC0]">
                    <span className="font-semibold text-white/90">
                      S{item.season_num} • E{item.ep_num}
                    </span>
                    {item.ep_title && (
                      <>
                        <span className="opacity-40">•</span>
                        <span className="truncate">{item.ep_title}</span>
                      </>
                    )}
                  </div>

                  {item.overview && (
                    <p className="text-xs text-white/50 line-clamp-2 mt-2 leading-relaxed">
                      {item.overview}
                    </p>
                  )}
                </div>
              </motion.div>
            );
          })}
        </AnimatePresence>
      </motion.div>
    </div>
  );
};
