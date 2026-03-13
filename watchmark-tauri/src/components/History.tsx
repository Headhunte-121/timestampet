import { useState, useEffect } from "react";
import { invokeWithTimeout } from "../utils/ipc";

export default function History() {
  const [history, setHistory] = useState<any[]>([]);
  const [page, setPage] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadHistory(0, true);
  }, []);

  const loadHistory = async (pageNum: number, isInitial = false) => {
    if (loading) return;
    setLoading(true);
    try {
      // Chunk loading with limit 100
      const res: any = await invokeWithTimeout("fetch_history", { page: pageNum, pageSize: 100 });
      if (isInitial) {
        setHistory(res);
      } else {
        setHistory(prev => [...prev, ...res]);
      }
      setHasMore(res.length === 100);
      setPage(pageNum);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  const handleLoadMore = () => {
    if (!loading && hasMore) {
      loadHistory(page + 1);
    }
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
            return (
            <div key={i} className="flex bg-[#1F222A]/60 backdrop-blur-md p-4 rounded-xl border border-white/5 items-center gap-6">
               <img
                src={`https://image.tmdb.org/t/p/w200${entry.poster_path}`}
                alt={entry.show_title}
                className="w-16 h-24 object-cover rounded-md shadow-md"
              />
              <div className="flex-1">
                <h3 className="text-lg font-bold text-white">{entry.show_title}</h3>
                {group.type === "binge_block" ? (
                  <p className="text-gray-400">
                    Watched {group.episode_count} Episodes
                  </p>
                ) : (
                  <p className="text-gray-400">
                    {entry.media_type === "TV" ? `Season ${entry.season_num} Episode ${entry.ep_num} - ${entry.ep_title}` : entry.ep_title}
                  </p>
                )}
                <div className="mt-2 flex gap-4 text-sm font-medium">
                  <span className="text-[#FF6B00]">{new Date(entry.timestamp * 1000).toLocaleString()}</span>
                  {entry.is_legacy === 1 && <span className="bg-white/10 text-gray-300 px-2 rounded-md">Legacy Import</span>}
                  {entry.time_capsule && (
                    <span className={`px-2 rounded-md font-bold ${entry.time_capsule.is_early ? 'bg-blue-500/20 text-blue-400' : 'bg-white/10 text-gray-300'}`}>
                      {entry.time_capsule.is_early ? (
                        "Early Watch"
                      ) : entry.time_capsule.total_days === 0 ? (
                        "Watched on Release Day"
                      ) : (
                        `Watched ${entry.time_capsule.years > 0 ? `${entry.time_capsule.years}y ` : ''}${entry.time_capsule.months > 0 ? `${entry.time_capsule.months}m ` : ''}${entry.time_capsule.days > 0 ? `${entry.time_capsule.days}d ` : ''}later`.trim()
                      )}
                    </span>
                  )}
                </div>
              </div>
              {group.type === "binge_block" ? (
                <div className="text-right text-sm text-gray-400">
                  Binge Duration: {Math.floor(group.total_runtime / 60)}h {group.total_runtime % 60}m
                </div>
              ) : entry.completion_ratio < 0.90 ? (
                <div className="text-right text-sm text-gray-400">
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
