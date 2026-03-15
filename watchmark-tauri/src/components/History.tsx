import { logger } from "../utils/logger";
import { formatImagePath } from "../utils/imageFormat";
import { useState, useEffect } from "react";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { SafeImage } from "./ui/SafeImage";

export default function History() {
  const [history, setHistory] = useState<any[]>([]);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [loading, setLoading] = useState(false);
  const asyncInvoke = useAsyncInvoke();

  useEffect(() => {
    loadHistory(0, true);
  }, [asyncInvoke]);

  const loadHistory = async (pageNum: number, isInitial = false) => {
    if (loading) return;
    setLoading(true);
    try {
      // Chunk loading with limit 50 per requirement
      const res: any = await asyncInvoke("fetch_history", { page: pageNum, pageSize: 50 });
      if (res) {
        if (isInitial) {
          setHistory(res);
        } else {
          setHistory(prev => [...prev, ...res]);
        }
        setHasMore(res.length === 50);
        setPage(pageNum);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const handleLoadMore = () => {
    if (!loading && hasMore) {
      logger.click(`'Load More' History (Page ${page + 1})`);
      loadHistory(page + 1);
    }
  };

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
    <div className="p-12 pt-24">
      <h1 className="text-4xl font-extrabold tracking-tight mb-8">Watch History</h1>

      {history.length === 0 ? (
        <p className="text-gray-500">No history recorded yet.</p>
      ) : (
        <div className="space-y-6 max-w-4xl pb-24">
          {history.map((group, i) => {
            const entry = group.main_entry;
            const isLegacy = entry.is_legacy === 1;

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

            return (
              <div
                key={i}
                className={`relative flex p-4 rounded-xl border border-white/5 items-center gap-6 overflow-hidden ${
                  isLegacy ? "bg-[#1F222A]/60 opacity-60" : "bg-[#1F222A]/60 backdrop-blur-md"
                }`}
              >
                {isLegacy && (
                  <div className="absolute top-2 right-2 bg-white/10 text-gray-400 font-bold text-[10px] px-2 py-1 rounded-full uppercase tracking-wider">
                    Archived
                  </div>
                )}
                <SafeImage
                  srcPath={entry.poster_path ? formatImagePath(entry.poster_path, "w500") : ""}
                  type="poster"
                  title={entry.show_title}
                  altText={entry.show_title}
                  className="w-16 h-24 object-cover rounded-md shadow-md"
                />
                <div className="flex-1">
                  <h3 className={`text-lg font-bold ${isLegacy ? "text-white opacity-100" : "text-white"}`}>
                    {entry.show_title}
                  </h3>
                  {group.type === "binge_block" ? (
                    <div>
                      <p className="text-gray-400">Watched {group.episode_count} Episodes</p>
                      {bingeSubtitle && (
                        <p className="text-sm text-gray-500 font-medium italic mt-1">
                          {bingeSubtitle}
                        </p>
                      )}
                    </div>
                  ) : (
                    <p className="text-gray-400">
                      {entry.media_type === "TV"
                        ? `Season ${entry.season_num} Episode ${entry.ep_num} - ${entry.ep_title}`
                        : entry.ep_title}
                    </p>
                  )}
                  <div className="mt-2 flex gap-4 text-sm font-medium">
                    <span className="text-[#FF6B00]">
                      {new Date(entry.timestamp * 1000).toLocaleString()}
                    </span>
                    {entry.time_capsule && (
                      <span
                        className={`px-2 rounded-md font-bold ${
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
                  <div className="text-right text-sm text-gray-400 mt-6 mr-2">
                    Binge Duration: {Math.floor(group.total_runtime / 60)}h{" "}
                    {group.total_runtime % 60}m
                  </div>
                ) : !isLegacy && entry.completion_ratio < 0.9 ? (
                  <div className="text-right text-sm text-gray-400 mt-6 mr-2">
                    Paused ({Math.round(entry.completion_ratio * 100)}%)
                  </div>
                ) : null}
              </div>
            );
          })}

          {hasMore && (
            <div className="flex justify-center mt-8">
              <button
                onClick={handleLoadMore}
                disabled={loading}
                className="px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-lg transition-colors disabled:opacity-50"
              >
                {loading ? "Loading..." : "Load More"}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
