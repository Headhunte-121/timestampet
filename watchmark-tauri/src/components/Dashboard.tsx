import { logger } from "../utils/logger";
import { Icon } from "./ui/Icon";
import { formatImagePath } from "../utils/imageFormat";
import { formatRuntime } from "../utils/dateFormatter";
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { Play, Star } from "lucide-react";
import { useAppStore } from "../store/useAppStore";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { SafeImage } from "./ui/SafeImage";
import { useHorizontalScroll } from "../hooks/useHorizontalScroll";
import { ChevronLeft, ChevronRight } from "lucide-react";

// Types matching the Rust backend structure
interface Episode {
  id: number;
  media_id: number;
  show_title: string;
  title: string;
  season_num: number;
  ep_num: number;
  backdrop_path: string;
  still_path: string;
  file_path: string;
  status: string;
  last_position: number;
  runtime: number; // in minutes
  media_type: string;
  is_date_known: boolean;
  progress_percentage: number;
}

interface Media {
  id: number;
  title: string;
  poster_path: string;
  backdrop_path: string;
  user_rating: number | null; // rating stays nullable because not everything is rated
  total_episodes: number;
  completed_eps: number;
  media_type: string;
  is_date_known: boolean;
}

interface Stats {
  hrs_watched: string;
  shows_completed: number;
  avg_rating: number;
}

interface DashboardData {
  hero_ep: Episode | null;
  cw_eps: Episode[];
  recent_media: Media[];
  stats: Stats;
}

// Unsplash placeholders
const PLACEHOLDER_BACKDROP = "https://images.unsplash.com/photo-1542293787-827fb705d15a?q=80&w=2560&auto=format&fit=crop";
const PLACEHOLDER_POSTER = "https://images.unsplash.com/photo-1534447677768-be436bb09401?q=80&w=600&auto=format&fit=crop";

interface EpisodeExtended extends Episode {
    is_fallback_image?: boolean;
    potential_spoiler?: boolean;
}

export default function Dashboard({ onMediaSelect, refreshTrigger, searchQuery = "" }: { onMediaSelect: (id: number) => void, refreshTrigger: number, searchQuery?: string }) {
  const { isCinemaMode } = useAppStore();
  const [data, setData] = useState<DashboardData | null>(null);
  const asyncInvoke = useAsyncInvoke();
  const cwScroll = useHorizontalScroll<HTMLDivElement>();
  const recentScroll = useHorizontalScroll<HTMLDivElement>();

  useEffect(() => {
    asyncInvoke<DashboardData>("get_dashboard_data")
      .then(res => {
        if (res) setData(res);
      })
      .catch((err) => {
        logger.error("Failed to load dashboard data", err);
        // Fallback or empty state if needed
      });
  }, [refreshTrigger, asyncInvoke]);

  if (!data) {
    // Skeleton Loading State
    return (
      <div className="flex-1 overflow-y-auto px-10 py-6 pb-24 scrollbar-hide animate-pulse">
        {/* Hero Skeleton */}
        <div className="relative w-full h-[450px] bg-[#1F222A] rounded-2xl overflow-hidden mb-12" />

        {/* Continue Watching Skeleton */}
        <div className="h-8 w-64 bg-[#1F222A] rounded mb-6 mt-12" />
        <div className="flex gap-6 overflow-x-hidden pb-4">
          {[1, 2, 3, 4].map(i => (
            <div key={i} className="flex-none min-w-[320px] h-[220px] bg-[#1F222A] rounded-xl" />
          ))}
        </div>

        {/* Recently Added Skeleton */}
        <div className="h-8 w-64 bg-[#1F222A] rounded mb-6 mt-12" />
        <div className="flex gap-6 overflow-x-hidden pb-8">
          {[1, 2, 3, 4, 5, 6].map(i => (
            <div key={i} className="flex-none min-w-[180px] aspect-[2/3] bg-[#1F222A] rounded-xl" />
          ))}
        </div>

        {/* Stats Skeleton */}
        <div className="h-8 w-64 bg-[#1F222A] rounded mb-6 mt-12" />
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          {[1, 2, 3].map(i => (
            <div key={i} className="h-32 bg-[#1F222A] rounded-xl" />
          ))}
        </div>
      </div>
    );
  }

  // Derived progress logic
  const calculateProgress = (lastPos: number, runtimeMins: number) => {
    if (runtimeMins <= 0) return 0;
    const progress = (lastPos / (runtimeMins * 60)) * 100;
    return Math.min(100, progress);
  };

  const filteredCW: EpisodeExtended[] = data.cw_eps?.filter((ep: any) =>
    !searchQuery ||
    ep.show_title?.toLowerCase().includes(searchQuery.toLowerCase()) ||
    ep.title?.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  const filteredRecent = data.recent_media?.filter((m: any) =>
    !searchQuery ||
    m.title?.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  return (
    <div className="flex-1 overflow-y-auto px-10 py-6 pb-24 pt-24 scrollbar-hide">
      {/* A. Hero Banner (Up Next) */}
      {data.hero_ep ? (
        <div className="relative aspect-video w-full max-h-[450px] rounded-2xl overflow-hidden group">
          <SafeImage
            srcPath={
              (data.hero_ep as any).still_path
                ? formatImagePath((data.hero_ep as any).still_path, "w1280")
                : data.hero_ep.backdrop_path
                ? formatImagePath(data.hero_ep.backdrop_path, "w1280")
                : ""
            }
            fallbackSrcPath={(data.hero_ep as any).backdrop_fallback ? formatImagePath((data.hero_ep as any).backdrop_fallback, "w1280") : undefined}
            type="backdrop"
            altText="Hero Backdrop"
            isFallbackImage={(data.hero_ep as any).is_fallback_image}
            potentialSpoiler={(data.hero_ep as any).potential_spoiler}
            isCompleted={data.hero_ep.status === "Completed"}
            className="w-full h-full object-cover origin-center"
            // Note: Since SafeImage uses motion.img under the hood, standard style pass-through applies, but to properly pass framer props we cast or just rely on the fallback structure.
            // SafeImage now returns a wrapper div when type="backdrop" containing motion.img
          />
          {/* Layered directional gradient: Bottom-left pure black fading up to top-right transparent */}
          <div className="absolute inset-0 bg-gradient-to-t from-black/80 to-transparent" />
          <div className="absolute inset-0 bg-gradient-to-tr from-[#0D0F14] via-[#0D0F14]/80 to-transparent" />

          <div className="absolute bottom-8 left-8 w-full max-w-2xl z-10 text-double-guard">
            <h2 className="text-[#FF6B00] font-bold tracking-widest text-xs mb-2 uppercase text-double-guard">
              {data.hero_ep.status === "Watching" && data.hero_ep.last_position > 0 ? "Resume Session" : "Up Next"}
            </h2>
            <h1 className="text-5xl font-black text-white mb-2 tracking-tighter truncate text-double-guard">
              {data.hero_ep.show_title || "Unknown Show"}
            </h1>
            <p className="text-lg text-muted mb-6 truncate font-normal text-double-guard">
              {data.hero_ep.media_type === "TV"
                ? `S${String(data.hero_ep.season_num).padStart(2, '0')}E${String(data.hero_ep.ep_num).padStart(2, '0')} - ${data.hero_ep.title || 'Unknown Episode'}`
                : data.hero_ep.title || "No Title"}
            </p>

            <div className="flex items-center gap-4">
              <button
                onClick={() => {
                  logger.click(`'Resume' on Hero (${data.hero_ep!.show_title} S${data.hero_ep!.season_num}E${data.hero_ep!.ep_num})`);
                  logger.ipcSend("play_episode_cmd", `Episode ${data.hero_ep!.id}`);
                  invoke("play_episode_cmd", {
                    episodeId: data.hero_ep!.id,
                    filePath: data.hero_ep!.file_path,
                    lastPosition: data.hero_ep!.last_position,
                  }).then(() => {
                    logger.ipcSuccess("VLC successfully launched. Waiting for heartbeat...");
                  }).catch(e => {
                    logger.error("VLC Launch Failed", e);
                    if (e === "VLC_AUTH_ERROR") {
                      toast.error("VLC Authentication Error: Failed to inject dynamic password or bind to port.", { duration: 8000 });
                    } else {
                      toast.error(`VLC Launch Failed: ${e}`);
                    }
                  });
                }}
                disabled={!data.hero_ep.file_path}
                className={`flex items-center gap-2 px-8 py-3 rounded-lg font-bold transition-all duration-300 ${
                  data.hero_ep.file_path
                    ? "bg-[#FF6B00] hover:bg-[#E66000] text-white hover:scale-105"
                    : "bg-red-900/50 text-red-200 cursor-not-allowed"
                }`}
              >
                {data.hero_ep.file_path ? (
                  <>
                    <Icon icon={Play} fill="currentColor" className="w-5 h-5" /> Play Next
                  </>
                ) : (
                  "❌ Missing File"
                )}
              </button>
              <button
                onClick={() => {
                    logger.click(`'More Info' for Hero (${data.hero_ep!.show_title})`);
                    onMediaSelect(data.hero_ep!.media_id);
                }}
                className="px-8 py-3 rounded-lg font-bold bg-white/10 hover:bg-white/20 text-white backdrop-blur-md transition-all duration-300 hover:scale-105"
              >
                More Info
              </button>
            </div>

            {data.hero_ep.status === "Watching" && data.hero_ep.runtime > 0 && (
              <div className="flex flex-col mt-6">
                <div className="text-sm font-bold text-white mb-2">
                  {formatRuntime(Math.max(0, data.hero_ep.runtime - Math.floor((data.hero_ep.last_position || 0) / 60)))} remaining
                </div>
                <div className="w-64 h-1.5 bg-white/20 rounded-full overflow-hidden flex">
                  {(() => {
                    const progress = calculateProgress(data.hero_ep.last_position, data.hero_ep.runtime);
                    if (progress <= 0) return null;
                    return (
                      <div
                        className="h-full rounded-full"
                        style={{
                          width: `${progress}%`,
                          minWidth: "2px",
                          backgroundColor: progress >= 90 ? "#1b5e20" : "#FF6B00"
                        }}
                      />
                    );
                  })()}
                </div>
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="w-full h-[450px] bg-[#1F222A]/80 backdrop-blur-xl rounded-2xl flex flex-col items-center justify-center text-center">
          <h1 className="text-4xl font-bold mb-4 text-white">Welcome to WatchMark</h1>
          <p className="text-muted max-w-md">Scan your local folder or search TMDB to get started and build your library.</p>
        </div>
      )}

      {/* B. Continue Watching (Horizontal Row) */}
      {filteredCW.length > 0 && (
        <div className="relative group/cw">
          <h2 className="text-2xl font-bold mt-12 mb-6 text-white">Continue Watching</h2>

          <button
              onClick={cwScroll.scrollLeft}
              className="absolute left-0 top-1/2 -translate-y-1/2 -ml-4 z-40 p-3 bg-black/40 backdrop-blur-md hover:bg-black/60 rounded-full text-white opacity-0 group-hover/cw:opacity-100 transition-opacity"
              aria-label="Scroll left"
          >
              <ChevronLeft className="w-8 h-8" />
          </button>
          <button
              onClick={cwScroll.scrollRight}
              className="absolute right-0 top-1/2 -translate-y-1/2 -mr-4 z-40 p-3 bg-black/40 backdrop-blur-md hover:bg-black/60 rounded-full text-white opacity-0 group-hover/cw:opacity-100 transition-opacity"
              aria-label="Scroll right"
          >
              <ChevronRight className="w-8 h-8" />
          </button>

          <motion.div ref={cwScroll.elRef} layout className="flex gap-6 overflow-x-auto pb-4 scrollbar-hide snap-x snap-mandatory pr-[20%]">
            <AnimatePresence mode="popLayout">
            {filteredCW.map((ep: any) => {
              const progress = calculateProgress(ep.last_position, ep.runtime);

              const stillUrl = ep.still_path ? formatImagePath(ep.still_path, "w500") : "";
              const fallbackUrl = ep.backdrop_path ? formatImagePath(ep.backdrop_path, "w500") : PLACEHOLDER_BACKDROP;

              return (
                <motion.div
                  key={ep.id}
                  layout="position"
                  initial={isCinemaMode ? { opacity: 0, scale: 0.9 } : { opacity: 1, scale: 1 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={isCinemaMode ? { opacity: 0, scale: 0.9, width: 0, marginLeft: -24 } : { opacity: 0 }}
                  transition={isCinemaMode ? { type: "spring", stiffness: 300, damping: 30 } : { duration: 0 }}
                  onClick={() => {
                      logger.click(`Continue Watching: ${ep.show_title} (Media ID: ${ep.media_id})`);
                      onMediaSelect(ep.media_id);
                  }}
                  whileHover={isCinemaMode ? { scale: 1.02 } : {}}
                  className="flex-none min-w-[320px] bg-[#1F222A] rounded-2xl overflow-hidden cursor-pointer group transition-all duration-300 hover:ring-2 hover:ring-[#FF6B00]/50 snap-start shadow-lg relative transform-gpu focus:outline-none focus:ring-2 focus:ring-[#FF6B00] focus:ring-offset-2 focus:ring-offset-[#0D0F14]"
                >
                  <div className="w-full h-[180px] relative overflow-hidden bg-black/40">
                    <SafeImage
                      srcPath={stillUrl}
                      fallbackSrcPath={fallbackUrl}
                      type="still"
                      episodeNumber={ep.ep_num}
                      title={ep.show_title || "Unknown Show"}
                      altText={ep.show_title || "Show Thumbnail"}
                      isFallbackImage={ep.is_fallback_image}
                      potentialSpoiler={ep.potential_spoiler}
                      isCompleted={ep.status === "Completed"}
                      className="w-full h-full object-cover opacity-80 group-hover:opacity-100 transition-all duration-500 group-hover:scale-105"
                    />
                    <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none" />

                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        logger.click(`Play on CW item (${ep.show_title} S${ep.season_num}E${ep.ep_num})`);
                        logger.ipcSend("play_episode_cmd", `Episode ${ep.id}`);
                        invoke("play_episode_cmd", {
                          episodeId: ep.id,
                          filePath: ep.file_path,
                          lastPosition: ep.last_position,
                        }).then(() => {
                           logger.ipcSuccess("VLC successfully launched. Waiting for heartbeat...");
                        }).catch(err => {
                           logger.error("VLC Launch Failed", err);
                           if (err === "VLC_AUTH_ERROR") {
                             toast.error("VLC Authentication Error: Failed to inject dynamic password or bind to port.", { duration: 8000 });
                           } else {
                             toast.error(`VLC Launch Failed: ${err}`);
                           }
                        });
                      }}
                      className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-12 h-12 bg-[#FF6B00] text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all scale-75 group-hover:scale-100 z-20 hover:bg-[#E66000]"
                    >
                      <Icon icon={Play} fill="currentColor" className="w-5 h-5 ml-1" />
                    </button>
                  </div>

                  {ep.status === "Watching" && ep.runtime > 0 && progress > 0 && (
                    <div className="absolute bottom-0 left-0 w-full h-1 bg-white/20 flex">
                      <div
                        className="h-full"
                        style={{
                          width: `${progress}%`,
                          minWidth: "2px",
                          backgroundColor: progress >= 90 ? "#1b5e20" : "#FF6B00"
                        }}
                      />
                    </div>
                  )}

                  <div className="p-4 flex justify-between items-center bg-[#1F222A]">
                    <h3 className="font-bold text-white truncate pr-2">{ep.show_title || "Unknown Show"}</h3>
                    <span className="text-xs font-normal text-muted whitespace-nowrap tabular-nums">
                      {ep.media_type === "TV" ? `S${ep.season_num}E${ep.ep_num}` : (ep.title || "No Title")}
                    </span>
                  </div>
                </motion.div>
              );
            })}
            </AnimatePresence>
          </motion.div>
        </div>
      )}

      {/* C. Recently Added (Poster Grid Row) */}
      {filteredRecent.length > 0 && (
        <div className="relative group/recent">
          <h2 className="text-2xl font-bold mt-12 mb-6 text-white">Recently Added</h2>

          <button
              onClick={recentScroll.scrollLeft}
              className="absolute left-0 top-1/2 -translate-y-1/2 -ml-4 z-40 p-3 bg-black/40 backdrop-blur-md hover:bg-black/60 rounded-full text-white opacity-0 group-hover/recent:opacity-100 transition-opacity"
              aria-label="Scroll left"
          >
              <ChevronLeft className="w-8 h-8" />
          </button>
          <button
              onClick={recentScroll.scrollRight}
              className="absolute right-0 top-1/2 -translate-y-1/2 -mr-4 z-40 p-3 bg-black/40 backdrop-blur-md hover:bg-black/60 rounded-full text-white opacity-0 group-hover/recent:opacity-100 transition-opacity"
              aria-label="Scroll right"
          >
              <ChevronRight className="w-8 h-8" />
          </button>

          <motion.div ref={recentScroll.elRef} layout className="flex gap-4 overflow-x-auto pb-8 scrollbar-hide snap-x snap-mandatory pt-2 pr-[20%]">
            <AnimatePresence mode="popLayout">
            {filteredRecent.map((media: any) => {
              const posterUrl = media.poster_path ? formatImagePath(media.poster_path, "w500") : "";

              return (
                <motion.div
                  key={media.id}
                  layout="position"
                  initial={isCinemaMode ? { opacity: 0, scale: 0.9 } : { opacity: 1, scale: 1 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={isCinemaMode ? { opacity: 0, scale: 0.9, width: 0, marginLeft: -16 } : { opacity: 0 }}
                  transition={isCinemaMode ? { type: "spring", stiffness: 300, damping: 30 } : { duration: 0 }}
                  onClick={() => {
                      logger.click(`Recent Additions: ${media.title} (Media ID: ${media.id})`);
                      onMediaSelect(media.id);
                  }}
                  whileHover={isCinemaMode ? { scale: 1.05, zIndex: 10 } : {}}
                  className="relative flex-none w-[140px] md:w-[160px] lg:w-[180px] aspect-[2/3] rounded-2xl overflow-hidden cursor-pointer group snap-start shadow-xl transition-shadow duration-300 hover:shadow-2xl hover:shadow-[#FF6B00]/10 bg-[#1F222A] transform-gpu focus:outline-none focus:ring-2 focus:ring-[#FF6B00] focus:ring-offset-2 focus:ring-offset-[#0D0F14]"
                >
                  <SafeImage
                    srcPath={posterUrl}
                    fallbackSrcPath={PLACEHOLDER_POSTER}
                    type="poster"
                    altText={media.title || "Unknown Title"}
                    title={media.title}
                    className="w-full h-full object-cover"
                  />

                  <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center backdrop-blur-sm">
                    <div className="w-14 h-14 bg-[#FF6B00] text-white rounded-full flex items-center justify-center shadow-lg shadow-black/50 scale-75 group-hover:scale-100 transition-transform duration-300">
                      <Icon icon={Play} fill="currentColor" className="w-6 h-6 ml-1" />
                    </div>
                  </div>

                  {/* Always-on Top-Right Star Rating pill */}
                  {media.user_rating !== null && media.user_rating > 0 && (
                    <div className="absolute top-2 right-2 bg-[#0D0F14]/60 backdrop-blur-md px-2 py-1 rounded-xl text-xs font-bold text-white shadow-md flex items-center gap-1 tabular-nums">
                       <Star className="w-3 h-3 fill-[#FF6B00] text-[#FF6B00]" /> {media.user_rating}
                    </div>
                  )}

                  {media.total_episodes > 0 && (
                    <div className="absolute bottom-0 left-0 w-full h-1 bg-white/20">
                      <div
                        className="h-full bg-green-500"
                        style={{ width: `${Math.min(100, (media.completed_eps / media.total_episodes) * 100)}%` }}
                      />
                    </div>
                  )}
                </motion.div>
              );
            })}
            </AnimatePresence>
          </motion.div>
        </div>
      )}

      {/* D. Quick Stats */}
      <h2 className="text-2xl font-bold mt-12 mb-6 text-white">Your Stats</h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        <div className="bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-6 border border-white/5 transition-all duration-300 hover:bg-[#1F222A]/80 shadow-xl flex flex-col items-center justify-center text-center">
          <h3 className="text-sm font-normal text-muted uppercase tracking-widest mb-2">Shows Tracked</h3>
          <p className="text-5xl font-black text-[#FF6B00] drop-shadow-md tabular-nums">{data.stats.shows_completed}</p>
        </div>
        <div className="bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-6 border border-white/5 transition-all duration-300 hover:bg-[#1F222A]/80 shadow-xl flex flex-col items-center justify-center text-center">
          <h3 className="text-sm font-normal text-muted uppercase tracking-widest mb-2">Hours Watched</h3>
          <p className="text-5xl font-black text-[#FF6B00] drop-shadow-md tabular-nums">{data.stats.hrs_watched}</p>
        </div>
        <div className="bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-6 border border-white/5 transition-all duration-300 hover:bg-[#1F222A]/80 shadow-xl flex flex-col items-center justify-center text-center">
          <h3 className="text-sm font-normal text-muted uppercase tracking-widest mb-2">Average Rating</h3>
          <p className="text-5xl font-black text-[#FF6B00] drop-shadow-md flex items-center justify-center gap-2 tabular-nums">
            <Icon icon={Star} className="w-8 h-8 fill-current" /> {data.stats.avg_rating.toFixed(1)}
          </p>
        </div>
      </div>
    </div>
  );
}
