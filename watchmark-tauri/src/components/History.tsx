import { logger } from "../utils/logger";
import { formatImagePath } from "../utils/imageFormat";
import { useState, useEffect, useRef, useCallback } from "react";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { SafeImage } from "./ui/SafeImage";
import { useAppStore } from "../store/useAppStore";
import { motion } from "framer-motion";

export default function History() {
  const fastHistory = useAppStore(state => state.fastHistory);
  const [history, setHistory] = useState<any[]>(fastHistory || []);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [loading, setLoading] = useState(false);
  const asyncInvoke = useAsyncInvoke();
  const observerTarget = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Instant initial load with cache
    if (history.length === 0 && fastHistory.length > 0) {
      setHistory(fastHistory);
    }
    loadHistory(0, true);
  }, [asyncInvoke]);

  const [topPadding, setTopPadding] = useState(0);

  const loadHistory = async (pageNum: number, isInitial = false) => {
    if (loading) return;
    setLoading(true);
    try {
      // Chunk loading with limit 50 per requirement
      const res: any = await asyncInvoke("fetch_history", { page: pageNum, pageSize: 50 });
      if (res) {
        if (isInitial) {
          setHistory(res);
          setTopPadding(0);
        } else {
          setHistory(prev => {
            const next = [...prev, ...res];
            // Memory-safe array management: Truncate to strictly 500 items max
            if (next.length > 500) {
                const chopCount = next.length - 500;
                // Roughly estimate row height (e.g. 150px) to preserve scrollbar height.
                setTopPadding(prevPad => prevPad + (chopCount * 150));
                return next.slice(chopCount);
            }
            return next;
          });
        }
        setHasMore(res.length === 50);
        setPage(pageNum);
      }
    } catch (e) {
      logger.error("Failed to load history", e);
    } finally {
      setLoading(false);
    }
  };

  const handleLoadMore = useCallback(() => {
    if (!loading && hasMore) {
      logger.click(`'Load More' History (Page ${page + 1})`);
      loadHistory(page + 1);
    }
  }, [loading, hasMore, page]);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !loading) {
          handleLoadMore();
        }
      },
      { threshold: 0.1 }
    );

    if (observerTarget.current) {
      observer.observe(observerTarget.current);
    }

    return () => observer.disconnect();
  }, [handleLoadMore, hasMore, loading]);

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

  return (
    <div id="turbo-scroll-history" className="h-full overflow-y-auto p-12 pt-24">
      <h1 className="text-4xl font-extrabold tracking-tight mb-8">Watch History</h1>

      {history.length === 0 && !loading ? (
        <div className="flex flex-col items-center justify-center py-32 text-center h-[60vh]">
          <div className="text-white/5 mb-6">
            <svg xmlns="http://www.w3.org/2000/svg" width="120" height="120" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-history">
              <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
              <path d="M3 3v5h5"/>
              <path d="M12 7v5l4 2"/>
            </svg>
          </div>
          <h2 className="text-3xl font-bold text-white mb-2">Your watch diary is empty.</h2>
          <p className="text-[#A0AEC0] max-w-md">Movies and episodes you watch will appear here as a chronological timeline.</p>
        </div>
      ) : (
        <div className="relative pb-24 max-w-5xl mx-auto" style={{ paddingTop: `${topPadding}px` }}>
          {/* Central absolute axis line */}
          <div className="absolute top-0 bottom-0 left-8 lg:left-1/2 w-0.5 bg-white/10 -translate-x-1/2 z-0" />

          {history.map((group, i) => {
            const entry = group.main_entry;
            const isLegacy = entry.is_legacy === 1;
            const isEven = i % 2 === 0;

            let bingeSubtitle = null;
            if (group.type === "binge_block" && group.entries && group.entries.length > 0) {
              const startTs = group.entries[group.entries.length - 1].timestamp;
              const endTs = group.entries[0].timestamp;
              const startDate = new Date(startTs * 1000);
              const endDate = new Date(endTs * 1000);

              if (startDate.toDateString() !== endDate.toDateString()) {
                bingeSubtitle = `${getDayName(startDate)} ${getVibeLabel(startDate)} – ${getDayName(endDate)} ${getVibeLabel(endDate)}`;
              }
            }

            // Date block generation (placeholder formatting - assume daily grouping will be implemented in subsequent feature 13.4 logic)
            // But we inject the structural classes for mb-8 spacing here.
            // 13.3.4 Inter-block temporal spacing.
            const needsDateHeader = i === 0 || new Date(history[i-1].main_entry.timestamp * 1000).toDateString() !== new Date(entry.timestamp * 1000).toDateString();

            return (
              <div key={i} className={`relative mb-8 z-10 w-full`}>

                {needsDateHeader && (
                  <div className="flex justify-start lg:justify-center w-full mb-8 relative z-20 pointer-events-none">
                     <div className="bg-[#0D0F14] px-4 py-1 ml-12 lg:ml-0 rounded-full font-bold text-sm text-white/80 border border-white/10 shadow-xl">
                       {new Date(entry.timestamp * 1000).toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric'})}
                     </div>
                  </div>
                )}

                <div className={`flex items-center w-full ${isEven ? 'lg:flex-row-reverse' : 'lg:flex-row'}`}>

                  {/* Left or Right spacing block for Desktop */}
                  <div className="hidden lg:block lg:w-[calc(50%-2rem)]" />

                  {/* Central Connector Dot */}
                  <div className="absolute left-8 lg:left-1/2 w-4 h-4 bg-[#FF6B00] rounded-full border-4 border-[#0D0F14] -translate-x-1/2 z-20 shadow-[0_0_10px_#FF6B00]" />

                  {/* Card Container */}
                  <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "100px" }}
                    transition={{ duration: 0.3 }}
                    className={`ml-16 lg:ml-0 lg:w-[calc(50%-2rem)] ${isEven ? 'lg:mr-auto' : 'lg:ml-auto'}`}
                  >
                    <div className={`relative flex p-4 rounded-xl border border-white/5 items-center gap-6 overflow-hidden shadow-2xl ${
                      isLegacy ? "bg-[#1F222A]/60 opacity-60" : "bg-[#1F222A]/60 backdrop-blur-md"
                    }`}>
                      {isLegacy && (
                        <div className="absolute top-2 right-2 bg-white/10 text-muted font-bold text-[10px] px-2 py-1 rounded-full uppercase tracking-wider">
                          Archived
                        </div>
                      )}
                      <SafeImage
                        srcPath={entry.poster_path ? formatImagePath(entry.poster_path, "w500") : ""}
                        type="poster"
                        title={entry.show_title}
                        altText={entry.show_title}
                        className="w-16 h-24 object-cover rounded-md shadow-md flex-shrink-0"
                      />
                      <div className="flex-1 min-w-0">
                        <h3 className={`text-lg font-bold truncate ${isLegacy ? "text-white opacity-100" : "text-white"}`}>
                          {entry.show_title}
                        </h3>
                        {group.type === "binge_block" ? (
                          <div>
                            <p className="text-muted">Watched {group.episode_count} Episodes</p>
                            {bingeSubtitle && (
                              <p className="text-sm text-gray-500 font-medium italic mt-1 truncate">
                                {bingeSubtitle}
                              </p>
                            )}
                          </div>
                        ) : (
                          <p className="text-muted truncate">
                            {entry.media_type === "TV"
                              ? `S${entry.season_num} E${entry.ep_num} - ${entry.ep_title}`
                              : entry.ep_title}
                          </p>
                        )}
                        <div className="mt-2 flex gap-4 text-sm font-medium flex-wrap">
                          <span className="text-[#FF6B00]">
                            {new Date(entry.timestamp * 1000).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })}
                          </span>
                          {entry.time_capsule && (
                            <span
                              className={`px-2 rounded-md font-bold whitespace-nowrap ${
                                entry.time_capsule.is_early
                                  ? "bg-blue-500/20 text-blue-400"
                                  : "bg-white/10 text-gray-300"
                              }`}
                            >
                              {entry.time_capsule.is_early
                                ? "Early Watch"
                                : entry.time_capsule.total_days === 0
                                ? "Watched on Release Day"
                                : `Watched ${
                                    entry.time_capsule.years > 0 ? `${entry.time_capsule.years}y ` : ""
                                  }${
                                    entry.time_capsule.months > 0 ? `${entry.time_capsule.months}m ` : ""
                                  }${
                                    entry.time_capsule.days > 0 ? `${entry.time_capsule.days}d ` : ""
                                  }later`.trim()}
                            </span>
                          )}
                        </div>
                      </div>
                      {group.type === "binge_block" ? (
                        <div className="text-right text-sm text-muted mt-6 mr-2 hidden sm:block whitespace-nowrap">
                          {Math.floor(group.total_runtime / 60)}h{" "}
                          {group.total_runtime % 60}m
                        </div>
                      ) : !isLegacy && entry.completion_ratio < 0.9 ? (
                        <div className="text-right text-sm text-muted mt-6 mr-2 hidden sm:block whitespace-nowrap">
                          Paused ({Math.round(entry.completion_ratio * 100)}%)
                        </div>
                      ) : null}
                    </div>
                  </motion.div>
                </div>
              </div>
            );
          })}

          {hasMore && (
            <div ref={observerTarget} className="flex justify-center items-center mt-8 h-20 relative">
              {loading ? (
                <div className="absolute left-1/2 -translate-x-1/2 lg:left-1/2">
                   <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-loader-2 animate-spin text-[#FF6B00]">
                     <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
                   </svg>
                </div>
              ) : (
                <button
                  onClick={handleLoadMore}
                  disabled={loading}
                  className="px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-lg transition-colors disabled:opacity-50"
                >
                  Load More
                </button>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
