import { formatImagePath } from "../utils/imageFormat";
import { useState, useEffect, useRef, useCallback } from "react";
import { Search, Tv, Clapperboard, XCircle } from "lucide-react";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { logger } from "../utils/logger";
import { motion } from "framer-motion";
import { toast } from "../utils/toast";
import { useAppStore } from "../store/useAppStore";
import { cn } from "../App";
import { SafeImage } from "./ui/SafeImage";

function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

export default function SearchTMDB({ initialQuery, onMediaSelect: _onMediaSelect }: any) {
  const [query, setQuery] = useState(initialQuery || "");
  const debouncedQuery = useDebounce(query, 500);
  const [results, setResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [_page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(true);
  const asyncInvoke = useAsyncInvoke();
  const { isApiAuthorized } = useAppStore();
  const observer = useRef<IntersectionObserver | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Initial page-load auto-focus
    const timer = setTimeout(() => {
      if (inputRef.current) {
        inputRef.current.focus();
      }
    }, 100);
    return () => clearTimeout(timer);
  }, []);

  const performSearch = useCallback(async (searchQuery: string, pageNum: number, append: boolean = false) => {
    if (!searchQuery.trim()) {
        setResults([]);
        return;
    }

    if (pageNum === 1) {
        logger.input(`Search Bar for '${searchQuery}'`);
    } else {
        logger.action(`Scrolling for more results (Page ${pageNum})`);
    }

    setLoading(true);
    try {
      const res: any = await asyncInvoke("perform_tmdb_search", { query: searchQuery, page: pageNum });
      if (res) {
        if (append) {
            setResults(prev => {
                // Deduplicate strictly by tmdb_id just in case the backend overlaps due to rapid scrolls
                const newIds = new Set(res.map((r: any) => `${r.type}-${r.tmdb_id}`));
                const filteredPrev = prev.filter(r => !newIds.has(`${r.type}-${r.tmdb_id}`));
                return [...filteredPrev, ...res];
            });
        } else {
            setResults(res);
        }
        setHasMore(res.length > 0);
      }
    } catch (e: any) {
      const errStr = typeof e === 'object' && e.message ? e.message : String(e);
      logger.error("TMDB Search Failed", errStr);
      if (errStr.includes("API Key is invalid") || errStr.includes("401") || errStr.includes("Unauthorized") || errStr.includes("API Key was revoked")) {
        toast.error(
          <div className="flex flex-col gap-1">
            <span className="font-bold">Invalid API Key</span>
            <span className="text-sm">Please check your TMDB API Key.</span>
            <button
              onClick={() => {
                // Navigate to settings (This works globally if event is emitted, but here we can just show the toast and they have to navigate)
                // A better approach would be to pass handleNav as prop or use custom event, but for now just showing text
              }}
              className="mt-2 w-max px-3 py-1 bg-white/20 hover:bg-white/30 rounded text-xs transition-colors"
            >
              Check Settings
            </button>
          </div>,
          { duration: 8000, style: { background: "#b71c1c", color: "#ffffff", border: "none" } }
        );
      } else if (errStr.includes("Task Timed Out") || errStr.includes("timeout") || errStr.toLowerCase().includes("connection")) {
        toast.warning("TMDB is taking too long to respond. Please check your internet connection.", { duration: 6000 });
      } else {
        // Fallback for other errors (they might be handled silently or show a generic toast)
        toast.error("Search failed: " + errStr);
      }
    } finally {
      setLoading(false);
    }
  }, [asyncInvoke]);

  // Sync if initialQuery changes from props, e.g. typing in global search bar then navigating
  useEffect(() => {
    if (initialQuery !== undefined && initialQuery !== query) {
      setQuery(initialQuery);
    }
  }, [initialQuery]);

  useEffect(() => {
    setPage(1);
    setHasMore(true);
    performSearch(debouncedQuery, 1, false);
  }, [debouncedQuery, performSearch]);

  const lastElementRef = useCallback((node: HTMLDivElement) => {
    if (loading) return;
    if (observer.current) observer.current.disconnect();

    observer.current = new IntersectionObserver(entries => {
        if (entries[0].isIntersecting && hasMore) {
            setPage(prevPage => {
                const nextPage = prevPage + 1;
                performSearch(debouncedQuery, nextPage, true);
                return nextPage;
            });
        }
    });

    if (node) observer.current.observe(node);
  }, [loading, hasMore, debouncedQuery, performSearch]);

  return (
    <div className="max-w-[1800px] mx-auto px-10 pt-24 relative">
      <div className="flex items-center gap-4 mb-8">
        <h1 className="text-4xl font-extrabold tracking-tight">Discover Media</h1>
      </div>

      {!isApiAuthorized && (
          <div className="mb-12 max-w-2xl w-full bg-[#FF6B00]/10 border border-[#FF6B00]/20 rounded-xl p-8 text-center">
              <h2 className="text-xl font-bold text-white mb-4">TMDB API Key Required</h2>
              <p className="text-gray-300 mb-6 max-w-lg mx-auto">
                  WatchMark uses the free TMDB service to provide high-quality posters, backdrops, and episode details. You need to connect your own free account.
              </p>
              <button
                  onClick={async () => {
                      try {
                          const { open } = await import('@tauri-apps/plugin-shell');
                          await open("https://www.themoviedb.org/settings/api");
                      } catch (e) {
                          console.error("Failed to open URL:", e);
                      }
                  }}
                  className="px-6 py-3 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-xl transition-colors"
              >
                  Get your free API key here
              </button>
          </div>
      )}

      <form
        onSubmit={(e) => {
          e.preventDefault();
          setPage(1);
          performSearch(query, 1, false);
        }}
        className="flex gap-4 mb-12 max-w-2xl"
      >
        <div className="relative flex-1">
          <Search className={cn("absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5", isApiAuthorized ? "text-muted" : "text-gray-600")} />
          <input
            ref={inputRef}
            type="text"
            placeholder={isApiAuthorized ? "Search TMDB for Shows or Movies..." : "API Key Required"}
            value={query}
            disabled={!isApiAuthorized}
            title={!isApiAuthorized ? "API Key is required to search. Add it in Settings." : ""}
            onChange={(e) => {
              setQuery(e.target.value);
              if (e.target.value === "") {
                setResults([]);
              }
            }}
            className="w-full bg-[#1F222A] text-white pl-12 pr-12 py-4 rounded-xl border border-white/5 focus:outline-none focus:border-[#FF6B00]/50 transition-colors shadow-inner font-medium text-lg disabled:opacity-50 disabled:cursor-not-allowed relative overflow-hidden"
          />
          {query.length > 0 && (
            <button
              type="button"
              onClick={() => {
                setQuery("");
                setResults([]);
                if (inputRef.current) inputRef.current.focus();
              }}
              className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white transition-colors"
            >
              <XCircle className="w-5 h-5" />
            </button>
          )}
          {loading && (
             <div className="absolute top-0 left-0 h-1 bg-[#FF6B00] animate-pulse rounded-t-xl" style={{ width: '100%' }} />
          )}
        </div>
        <button
          type="submit"
          disabled={loading || !isApiAuthorized}
          className="px-8 py-4 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-full transition-all hover:scale-105 shadow-lg shadow-orange-500/20 disabled:opacity-50 disabled:hover:scale-100"
        >
          {loading ? "Searching..." : "Search TMDB"}
        </button>
      </form>

      <div className="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-6 pb-24">
        {results.map((item, i) => (
          <motion.div
            layout
            key={`${item.type}-${item.tmdb_id}`}
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            whileHover={{ scale: 1.05 }}
            transition={{ delay: (i % 20) * 0.05, duration: 0.2 }}
            className="relative aspect-[2/3] bg-[#1F222A] rounded-xl overflow-hidden group shadow-md hover:shadow-2xl transition-shadow"
          >
            {/* Visual Badging */}
            <div className="absolute top-2 left-2 z-10">
              {item.type === "TV" ? (
                  <div className="flex items-center gap-1.5 px-2 py-1 bg-black/80 backdrop-blur-md border border-emerald-500/50 rounded-full shadow-lg">
                      <Tv className="w-3 h-3 text-emerald-400" />
                      <span className="text-[10px] font-bold text-emerald-400 uppercase tracking-widest">TV</span>
                  </div>
              ) : (
                  <div className="flex items-center gap-1.5 px-2 py-1 bg-black/80 backdrop-blur-md border border-blue-500/50 rounded-full shadow-lg">
                      <Clapperboard className="w-3 h-3 text-blue-400" />
                      <span className="text-[10px] font-bold text-blue-400 uppercase tracking-widest">MOVIE</span>
                  </div>
              )}
            </div>

            <SafeImage
              srcPath={item.poster_path ? formatImagePath(item.poster_path, "w500") : ""}
              type="poster"
              title={item.title}
              altText={item.title}
              className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
            />

            <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4 z-20">
              <h3 className="text-white font-bold leading-tight mb-1 line-clamp-2">{item.title}</h3>
              <p className="text-xs text-gray-300 mb-4 uppercase tracking-wider font-bold flex items-center gap-2">
                 <span>{item.is_date_known ? (item.type === "TV" ? `${item.release_date.substring(0, 4)}–` : item.release_date.substring(0, 4)) : <span className="px-1.5 py-0.5 bg-gray-800 rounded text-xs font-semibold uppercase tracking-wider text-muted">TBD</span>}</span>
              </p>

              <button
                onClick={() => {
                  logger.click(`'+ Add to Tracker' for '${item.title}'`);
                  setLoading(true);
                  asyncInvoke("add_to_tracker", {
                    tmdbId: item.tmdb_id,
                    mediaType: item.type,
                    archive: false
                  }).then(() => {
                    logger.ipcSuccess(`'${item.title}' added successfully.`);
                    toast.success("Added to Tracker!");
                    setLoading(false);
                  }).catch((e: any) => {
                    const errStr = typeof e === 'object' && e.message ? e.message : String(e);
                    logger.error("Add to Tracker Failed", errStr);
                    toast.error("Error: " + errStr);
                    setLoading(false);
                  });
                }}
                className="w-full py-2 bg-[#FF6B00] text-white font-bold rounded-lg hover:bg-[#E66000] transition-colors"
              >
                + Add to Tracker
              </button>
            </div>
          </motion.div>
        ))}

        {/* Intersection Observer target for infinite scrolling */}
        {results.length > 0 && hasMore && (
           <div ref={lastElementRef} className="col-span-full h-10 flex items-center justify-center">
              {loading && <div className="w-6 h-6 border-2 border-[#FF6B00] border-t-transparent rounded-full animate-spin" />}
           </div>
        )}
      </div>
    </div>
  );
}
