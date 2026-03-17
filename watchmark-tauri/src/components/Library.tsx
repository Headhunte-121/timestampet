import { logger } from "../utils/logger";
import { Icon } from "./ui/Icon";
import { formatImagePath } from "../utils/imageFormat";
import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Star, Library as LibraryIcon, Film } from "lucide-react";
import { formatLocaleDate } from "../utils/dateFormatter";
import { useAppStore } from "../store/useAppStore";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { SafeImage } from "./ui/SafeImage";
import { VirtualPoster } from "./ui/VirtualPoster";

export default function Library({ type, onMediaSelect, refreshTrigger, searchQuery = "", onSearchQueryChange }: any) {
  const { isCinemaMode } = useAppStore();
  const [data, setData] = useState<any[]>([]);
  const [sortBy, setSortBy] = useState("Recently Added");
  const [hideCompleted, setHideCompleted] = useState(false);
  const asyncInvoke = useAsyncInvoke();

  useEffect(() => {
    // Standard pagination starts at 0 for limit offsets
    asyncInvoke("get_library_data", { mediaType: type, sortBy, hideCompleted, page: 0 })
      .then((res: any) => {
        if (res) setData(res);
      })
      .catch(e => logger.error("Failed to fetch library data", e));
  }, [type, sortBy, hideCompleted, refreshTrigger, asyncInvoke]);

  const sortOptions = ["Recently Added", "Sort by Last Watched", "Alphabetical (A-Z)", "Release Year", "My Top Rated", "Sort by TMDB Rating"];

  const isFiltered = hideCompleted || searchQuery !== "";

  const filteredData = data.filter(item =>
    !searchQuery ||
    item.title?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const isDbEmpty = data.length === 0 && !isFiltered;

  return (
    <div className="px-6 py-24 pb-24 max-w-[1800px] mx-auto">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center mb-10 gap-6">
        <h1 className="text-4xl font-extrabold tracking-tight">
          {type === "TV" ? "TV Shows" : "Movies"}
        </h1>

        {!isDbEmpty && (
          <div className="flex items-center gap-4 bg-[#1F222A]/80 backdrop-blur-md p-2 rounded-xl border border-white/5">
            <select
              value={sortBy}
              onChange={e => setSortBy(e.target.value)}
              className="bg-transparent text-white outline-none font-normal px-2 py-1"
            >
              {sortOptions.map(opt => <option key={opt} value={opt} className="bg-[#1F222A]">{opt}</option>)}
            </select>

            <div className="w-px h-6 bg-white/10" />

            <label className="flex items-center gap-2 cursor-pointer px-2 text-sm font-normal text-gray-300 hover:text-white transition-colors">
              <input
                type="checkbox"
                checked={hideCompleted}
                onChange={e => setHideCompleted(e.target.checked)}
                className="accent-[#FF6B00] w-4 h-4 rounded focus:ring-[#FF6B00]"
              />
              Hide Completed
            </label>
          </div>
        )}
      </div>

      <motion.div layout className="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-6 md:gap-8">
        <AnimatePresence mode="popLayout">
          {filteredData.map((item, i) => {
            const isInitialStagger = i < 20;
            return (
              <VirtualPoster key={item.id} className="relative w-full aspect-[2/3] bg-[#1F222A] rounded-2xl overflow-hidden shadow-xl focus-within:ring-2 focus-within:ring-[#FF6B00] focus-within:ring-offset-2 focus-within:ring-offset-[#0D0F14]" heightClass="aspect-[2/3]">
                <motion.div
                  layout="position"
                  initial={isCinemaMode ? { opacity: 0, y: 20 } : { opacity: 1, y: 0 }}
                  animate={isCinemaMode && isInitialStagger ? { opacity: 1, y: 0 } : (!isCinemaMode ? { opacity: 1, y: 0 } : { opacity: 1, y: 0 })}
                  exit={isCinemaMode ? { opacity: 0, scale: 0.8 } : { opacity: 0, scale: 1 }}
                  transition={{
                    delay: isCinemaMode && isInitialStagger ? i * 0.05 : 0,
                    duration: 0.3
                  }}
                  whileHover={isCinemaMode ? { scale: 1.05, y: -5 } : {}}
                  className="w-full h-full cursor-pointer group transform-gpu focus:outline-none"
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      logger.click(`Library Item: ${item.title} (Media ID: ${item.id}) via Keyboard`);
                      onMediaSelect(item.id);
                    }
                  }}
                  onClick={() => {
                      logger.click(`Library Item: ${item.title} (Media ID: ${item.id})`);
                      onMediaSelect(item.id);
                  }}
                >
              <div className="absolute inset-0 w-full h-full">
                <SafeImage
                  srcPath={item.poster_path ? formatImagePath(item.poster_path, "w500") : ""}
                  type="poster"
                  altText={item.title}
                  title={item.title}
                  releaseDate={item.release_date}
                  isDateKnown={item.is_date_known}
                  isExactDate={item.is_exact_date}
                  className={`w-full h-full object-cover transition-transform duration-500 group-hover:scale-110 ${item.is_unaired ? 'grayscale-[0.5] opacity-70' : ''}`}
                />
              </div>

              {item.is_unaired && (
                <div className="absolute top-2 left-2 z-20">
                  <div className="px-2 py-1 bg-blue-500/80 backdrop-blur-md rounded-md text-[10px] font-bold text-white shadow-md uppercase tracking-wider">
                    Planned
                  </div>
                </div>
              )}

              {/* Dark overlay on hover */}
              <div className="absolute inset-0 bg-[#141519]/90 opacity-0 group-hover:opacity-100 group-focus:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4 z-30 pointer-events-none rounded-2xl inset-y-0 inset-x-0 border-transparent">
                <h3 className="text-white font-bold leading-tight mb-1 text-center line-clamp-2">{item.title}</h3>

                <p className="text-xs text-gray-300 mb-2 text-center flex items-center justify-center gap-1">
                  {item.is_date_known ? (item.is_exact_date ? formatLocaleDate(item.release_date) : item.release_date.substring(0, 4)) : <span className="px-1.5 py-0.5 bg-gray-800 rounded text-xs font-semibold uppercase tracking-wider text-muted">TBD</span>}
                  {item.status && <><span className="mx-1">•</span> <span className="uppercase text-[10px] tracking-wider">{item.status}</span></>}
                </p>

                <div className="flex items-center justify-center gap-1 text-[#FF6B00] text-sm font-bold mb-1">
                   {item.user_rating === 0 ? <span className="font-bold">0.0</span> : <Icon icon={Star} className="w-4 h-4 fill-current" />} {item.user_rating !== null ? `${(item.user_rating / 2).toFixed(1)}/5` : 'Unrated'}
                </div>

                <p className="text-xs text-[#8E929C] line-clamp-2 text-center mt-2 mx-2">
                  {item.synopsis}
                </p>
              </div>

              {/* Always-on Top-Right Star Rating pill */}
              {item.user_rating !== null && item.user_rating > 0 && (
                <div className="absolute top-2 right-2 bg-[#0D0F14]/60 backdrop-blur-md px-2 py-1 rounded-xl text-xs font-bold text-white shadow-md transition-opacity flex items-center gap-1 tabular-nums z-40">
                   <Star className="w-3 h-3 fill-[#FF6B00] text-[#FF6B00]" /> {item.user_rating}
                </div>
              )}

              {/* Always-on Bottom-Left Original Release Year pill */}
              <div className="absolute bottom-2 left-0 z-20 pointer-events-none group-hover:opacity-0 transition-opacity">
                 {item.is_date_known && item.release_date && item.release_date.split('-')[0] && item.release_date.split('-')[0] !== "0000" && (
                    <span className="px-2 py-1 bg-[#0D0F14] text-[10px] sm:text-xs font-bold tracking-wider text-white rounded-r-md tabular-nums">
                      {item.release_date.split('-')[0]}
                    </span>
                 )}
              </div>

              {/* Absolute bottom-edge library progress bar */}
              {(() => {
                const totalEps = item.total_available ?? item.total_episodes ?? 0;
                const completedEps = item.completed_eps ?? 0;
                let width = 0;
                if (totalEps > 0) {
                   width = (completedEps / totalEps) * 100;
                }
                if (isNaN(width)) width = 0;
                width = Math.min(100, Math.max(0, width));

                let bgColorClass = "bg-transparent"; // Fallback
                if (width > 0 && width < 100) bgColorClass = "bg-[#FF6B00]"; // In-Progress
                else if (width >= 100) bgColorClass = "bg-[#1b5e20]"; // Finished

                // For shows with few episodes: Min-width of 2px for partially watched
                let styleWidth = `${width}%`;

                return (
                  <div className="absolute bottom-0 left-0 right-0 h-1 z-20 bg-white/10">
                    <motion.div
                      className={`h-full ${bgColorClass}`}
                      style={{
                        width: styleWidth,
                        minWidth: (width > 0 && width < 100) ? '2px' : '0px'
                      }}
                      initial={{ width: 0 }}
                      animate={{ width: styleWidth }}
                      transition={{ duration: 0.3 }}
                    />
                  </div>
                );
              })()}
                </motion.div>
              </VirtualPoster>
            );
          })}
        </AnimatePresence>
        {filteredData.length === 0 && (
          <div className="col-span-full flex flex-col items-center justify-center min-h-[400px] w-full py-20 text-center">
            {isDbEmpty ? (
              <>
                <Icon icon={type === "TV" ? LibraryIcon : Film} size={80} className="text-white/5 mb-6" />
                <h2 className="text-2xl font-bold text-white mb-2">Your library is currently empty.</h2>
                <p className="text-[#A0AEC0] max-w-md mx-auto">
                  Start by scanning a local folder or searching for a show to add to your tracker.
                </p>
              </>
            ) : (
              <>
                <Icon icon={type === "TV" ? LibraryIcon : Film} size={80} className="text-white/5 mb-6" />
                <h2 className="text-2xl font-bold text-white mb-2">No Results</h2>
                <p className="text-[#A0AEC0] max-w-md mx-auto mb-6">
                  Try adjusting your filters or search terms to find what you're looking for.
                </p>
                <button
                  onClick={() => {
                    if (onSearchQueryChange) onSearchQueryChange("");
                    setHideCompleted(false);
                    setSortBy("Recently Added");
                  }}
                  className="px-6 py-2 bg-[#FF6B00] hover:bg-[#FF8533] text-white font-bold rounded-xl transition-colors shadow-lg"
                >
                  Clear All Filters
                </button>
              </>
            )}
          </div>
        )}
      </motion.div>
    </div>
  );
}
