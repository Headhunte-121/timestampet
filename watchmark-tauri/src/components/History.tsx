import { logger } from "../utils/logger";
import { formatImagePath } from "../utils/imageFormat";
import { useState, useEffect, useRef, useCallback } from "react";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { SafeImage } from "./ui/SafeImage";
import { useAppStore } from "../store/useAppStore";
import { motion } from "framer-motion";
import { BingeBlock } from "./BingeBlock";
import { Play, Clock, CheckCircle2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

export default function History({ onNavigateToMedia }: { onNavigateToMedia: (mediaId: number, seasonNum?: number) => void }) {
  const { fastHistory, historyState, setHistoryState } = useAppStore();
  const [history, setHistory] = useState<any[]>(historyState?.history || fastHistory || []);
  const [page, setPage] = useState(historyState?.page || 0);
  const [hasMore, setHasMore] = useState(historyState?.hasMore ?? true);
  const [topPadding, setTopPadding] = useState(historyState?.topPadding || 0);
  const [loading, setLoading] = useState(false);
  const asyncInvoke = useAsyncInvoke();
  const observerTarget = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Use a ref to keep track of the latest state for the cleanup function
  const latestState = useRef({ history, page, hasMore, topPadding });
  useEffect(() => {
    latestState.current = { history, page, hasMore, topPadding };
  }, [history, page, hasMore, topPadding]);

  useEffect(() => {
    if (historyState) {
      // Restore scroll position
      if (containerRef.current) {
         requestAnimationFrame(() => {
             if (containerRef.current) containerRef.current.scrollTop = historyState.scrollPos;
         });
      }
    } else {
      // Instant initial load with cache
      if (history.length === 0 && fastHistory.length > 0) {
        setHistory(fastHistory);
      }
      loadHistory(0, true);
    }

    return () => {
      // Save state on unmount
      if (containerRef.current) {
        setHistoryState({
          scrollPos: containerRef.current.scrollTop,
          ...latestState.current
        });
      }
    };
  }, []);

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

    const getOrdinalSuffix = (d: number) => {
    if (d > 3 && d < 21) return 'th';
    switch (d % 10) {
      case 1:  return "st";
      case 2:  return "nd";
      case 3:  return "rd";
      default: return "th";
    }
  };

  const formatPauseTime = (seconds: number) => {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hrs > 0) {
      return `${hrs}:${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
    }
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const formatRelativeDate = (dateString: string) => {
    if (!dateString || dateString === "Unknown" || dateString === "0000-00-00") return "Unknown Date";

    // Parse strictly as local date based on YYYY-MM-DD
    const parts = dateString.split("-");
    const d = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, parseInt(parts[2]));

    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (d.getTime() === today.getTime()) {
      return "Today";
    } else if (d.getTime() === yesterday.getTime()) {
      return "Yesterday";
    }

    const weekday = d.toLocaleDateString("en-US", { weekday: "long" });
    const month = d.toLocaleDateString("en-US", { month: "long" });
    const day = d.getDate();
    return (
      <span>
        <span className="text-[#A0AEC0]">{weekday}, </span>
        <span className="text-white">{month} {day}{getOrdinalSuffix(day)}</span>
      </span>
    );
  };

  return (
    <div id="turbo-scroll-history" ref={containerRef} className="h-full overflow-y-auto p-12 pt-24">
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
            if (group.ui_type === "BINGE" && (!group.entries || group.entries.length === 0)) {
               return null;
            }

            const entry = group.main_entry;
            const isLegacy = entry.is_legacy === 1;
            const isEven = i % 2 === 0;

            const needsDateHeader = i === 0 || history[i-1]?.main_entry?.formatted_date !== entry.formatted_date;

            return (
              <div key={i} className={`relative mb-8 z-10 w-full`}>

                {needsDateHeader && (
                  <div className="flex justify-start lg:justify-center w-full mb-8 sticky top-[64px] z-30 pointer-events-none">
                     <div className="bg-[#0D0F14] px-4 py-1 ml-12 lg:ml-0 rounded-full font-bold text-sm text-white/80 border border-white/10 shadow-xl pointer-events-auto">
                       {formatRelativeDate(entry.formatted_date)}
                     </div>
                  </div>
                )}

                <div className={`flex items-center w-full ${isEven ? 'lg:flex-row-reverse' : 'lg:flex-row'}`}>

                  {/* Left or Right spacing block for Desktop */}
                  <div className="hidden lg:block lg:w-[calc(50%-2rem)]" />

                  {/* Central Connector Dot */}
                  <div className="absolute left-8 lg:left-1/2 w-4 h-4 bg-[#FF6B00] rounded-full border-4 border-[#0D0F14] -translate-x-1/2 z-20 shadow-[0_0_10px_#FF6B00]" />

                  {group.ui_type === "BINGE" ? (
                    <BingeBlock
                      group={group}
                      isEven={isEven}
                      onNavigateToMedia={onNavigateToMedia}
                      formatPauseTime={formatPauseTime}
                    />
                  ) : (
                  <>
                  {/* Card Container for Single Item */}
                  <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true, margin: "100px" }}
                    transition={{ duration: 0.3 }}
                    className={`ml-16 lg:ml-0 lg:w-[calc(50%-2rem)] ${isEven ? 'lg:mr-auto' : 'lg:ml-auto'}`}
                  >
                    <div
                      className={`relative flex p-4 rounded-2xl border border-white/5 items-center gap-5 overflow-hidden shadow-2xl cursor-pointer hover:bg-white/5 transition-colors group ${
                        isLegacy ? "bg-[#1F222A]/60 opacity-60" : "bg-[#1F222A]/60 backdrop-blur-md"
                      }`}
                      onClick={(e) => {
                        if (window.getSelection()?.toString().length) return;
                        const target = e.target as HTMLElement;
                        if (target.closest('button') || target.closest('a')) return;
                        onNavigateToMedia(entry.media_id, entry.season_num);
                      }}
                    >
                      {isLegacy && (
                        <div className="absolute top-2 right-2 bg-white/10 text-muted font-bold text-[10px] px-2 py-1 rounded-full uppercase tracking-wider">
                          Archived
                        </div>
                      )}

                      {/* Poster Thumbnail Container with Rigid Sizing */}
                      <div className="w-16 h-24 sm:w-20 sm:h-28 flex-shrink-0 rounded-lg overflow-hidden shadow-lg bg-[#0D0F14] border border-white/10 relative">
                        <SafeImage
                          srcPath={entry.poster_path ? formatImagePath(entry.poster_path, "w500") : ""}
                          type="poster"
                          title={entry.show_title}
                          altText={entry.show_title}
                          className="w-full h-full object-cover"
                        />
                      </div>

                      {/* Content Metadata Column */}
                      <div className="flex-1 min-w-0 pr-2 flex flex-col justify-center">
                        <h3 className={`text-base sm:text-lg font-bold text-white truncate group-hover:text-[#FF6B00] transition-colors`}>
                          {entry.show_title}
                        </h3>

                        <div className="flex items-center gap-2 mt-1 flex-wrap">
                          {entry.media_type === "TV" ? (
                            <>
                              <span className="bg-[#FF6B00]/15 text-[#FF6B00] text-xs font-semibold px-2 py-0.5 rounded border border-[#FF6B00]/20 flex-shrink-0">
                                S{entry.season_num} E{entry.ep_num}
                              </span>
                              <span className="text-sm text-gray-300 truncate font-medium">
                                {entry.ep_title || `Episode ${entry.ep_num}`}
                              </span>
                            </>
                          ) : (
                            <span className="bg-white/10 text-gray-300 text-xs font-semibold px-2 py-0.5 rounded">
                              Movie
                            </span>
                          )}
                        </div>

                        <div className="flex items-center gap-2 text-xs text-[#A0AEC0] flex-wrap mt-1.5">
                          <span className="flex items-center gap-1 font-medium">
                            <Clock size={12} className="opacity-70" />
                            {entry.formatted_time}
                          </span>
                          {entry.time_capsule && (
                            <span
                              className={`font-medium ${
                                entry.time_capsule.is_early
                                  ? "text-blue-400"
                                  : entry.time_capsule.total_days === 0
                                  ? "text-[#FF6B00] font-bold"
                                  : "text-[#A0AEC0]"
                              }`}
                            >
                              • {entry.time_capsule.is_early
                                ? "Early Watch"
                                : entry.time_capsule.total_days === 0
                                ? "Watched on premiere day"
                                : `Watched ${
                                    entry.time_capsule.years > 1 ? `${entry.time_capsule.years}y` :
                                    entry.time_capsule.years === 1 ? `1y` :
                                    entry.time_capsule.months > 1 ? `${entry.time_capsule.months}m` :
                                    entry.time_capsule.months === 1 ? `1m` :
                                    `${entry.time_capsule.days}d`
                                  } after airing`}
                            </span>
                          )}
                          {entry.is_utc_fallback && (
                            <span className="text-[#A0AEC0] text-[10px] flex items-center gap-0.5" title="Universal Time">
                              • UTC
                            </span>
                          )}
                        </div>

                        {!isLegacy && entry.completion_ratio >= 0.9 ? (
                          <div className="flex items-center gap-1 text-emerald-400 text-xs font-medium mt-2">
                            <CheckCircle2 size={13} />
                            <span>Watched</span>
                          </div>
                        ) : !isLegacy && entry.last_position > 0 ? (
                          <div className="mt-2 w-full max-w-[260px]">
                            <div className="flex justify-between text-[11px] text-[#A0AEC0] mb-1">
                              <span className="text-[#FF6B00] font-medium">Paused at {formatPauseTime(entry.last_position)}</span>
                              <span>{Math.round(entry.completion_ratio * 100)}%</span>
                            </div>
                            <div className="w-full bg-white/10 h-1.5 rounded-full overflow-hidden">
                              <div
                                className="bg-[#FF6B00] h-full rounded-full transition-all"
                                style={{ width: `${Math.min(100, Math.max(5, Math.round(entry.completion_ratio * 100)))}%` }}
                              />
                            </div>
                          </div>
                        ) : null}
                      </div>

                      {/* Instant Play / Resume Button */}
                      <div className="flex-shrink-0 ml-auto flex items-center pl-2">
                        <button
                          className="w-10 h-10 rounded-full bg-[#FF6B00]/20 hover:bg-[#FF6B00] text-[#FF6B00] hover:text-white flex items-center justify-center transition-all shadow-md hover:scale-110 flex-shrink-0 cursor-pointer"
                          title={entry.file_path ? (entry.last_position > 0 ? `Resume at ${formatPauseTime(entry.last_position)}` : "Play in VLC") : "Go to episode details"}
                          onClick={async (e) => {
                            e.stopPropagation();
                            if (entry.file_path) {
                              logger.click(`Play episode from History: S${entry.season_num}E${entry.ep_num}`);
                              try {
                                await invoke("play_episode_cmd", {
                                  episodeId: entry.episode_id,
                                  filePath: entry.file_path,
                                  lastPosition: entry.last_position || 0,
                                });
                                logger.ipcSuccess("VLC playback launched from History");
                              } catch (err: any) {
                                logger.error("VLC Launch Failed from History", err);
                                toast.error(`VLC Launch Failed: ${err}`);
                              }
                            } else {
                              onNavigateToMedia(entry.media_id, entry.season_num);
                            }
                          }}
                        >
                          <Play size={16} className="ml-0.5 fill-current" />
                        </button>
                      </div>
                    </div>
                  </motion.div>
                  </>
                  )}
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
