import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { Play, Star } from "lucide-react";

// Types matching the Rust backend structure
interface Episode {
  id: number;
  media_id: number;
  show_title: string | null;
  title: string | null;
  season_num: number;
  ep_num: number;
  backdrop_path: string | null;
  still_path: string | null;
  file_path: string | null;
  status: string;
  last_position: number;
  runtime: number; // in minutes
  media_type: string;
}

interface Media {
  id: number;
  title: string | null;
  poster_path: string | null;
  backdrop_path: string | null;
  user_rating: number;
  total_episodes: number;
  completed_eps: number;
  media_type: string;
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

export default function Dashboard({ onMediaSelect, refreshTrigger, searchQuery = "" }: { onMediaSelect: (id: number) => void, refreshTrigger: number, searchQuery?: string }) {
  const [data, setData] = useState<DashboardData | null>(null);

  useEffect(() => {
    invoke<DashboardData>("get_dashboard_data")
      .then(setData)
      .catch((err) => {
        console.error("Failed to load dashboard data:", err);
        // Fallback or empty state if needed
      });
  }, [refreshTrigger]);

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
    return Math.min(100, (lastPos / (runtimeMins * 60)) * 100);
  };

  const filteredCW = data.cw_eps?.filter(ep =>
    !searchQuery ||
    ep.show_title?.toLowerCase().includes(searchQuery.toLowerCase()) ||
    ep.title?.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  const filteredRecent = data.recent_media?.filter(m =>
    !searchQuery ||
    m.title?.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  return (
    <div className="flex-1 overflow-y-auto px-10 py-6 pb-24 pt-24 scrollbar-hide">
      {/* A. Hero Banner (Up Next) */}
      {data.hero_ep ? (
        <div className="relative aspect-video w-full max-h-[450px] rounded-2xl overflow-hidden group">
          <img
            src={data.hero_ep.backdrop_path ? `https://image.tmdb.org/t/p/original${data.hero_ep.backdrop_path}` : PLACEHOLDER_BACKDROP}
            alt="Hero Backdrop"
            className="w-full h-full object-cover"
          />
          {/* Layered directional gradient: Bottom-left pure black fading up to top-right transparent */}
          <div className="absolute inset-0 bg-gradient-to-tr from-[#0D0F14] via-[#0D0F14]/80 to-transparent" />

          <div className="absolute bottom-8 left-8 w-full max-w-2xl z-10">
            <h2 className="text-[#FF6B00] font-bold tracking-widest text-xs mb-2 uppercase drop-shadow-md">
              {data.hero_ep.status === "Watching" && data.hero_ep.last_position > 0 ? "Resume Session" : "Up Next"}
            </h2>
            <h1 className="text-5xl font-black text-white mb-2 tracking-tight truncate drop-shadow-lg">
              {data.hero_ep.show_title || "Unknown Show"}
            </h1>
            <p className="text-lg text-gray-400 mb-6 truncate drop-shadow-md font-medium">
              {data.hero_ep.media_type === "TV"
                ? `S${String(data.hero_ep.season_num).padStart(2, '0')}E${String(data.hero_ep.ep_num).padStart(2, '0')} - ${data.hero_ep.title || 'Unknown Episode'}`
                : data.hero_ep.title || "No Title"}
            </p>

            <div className="flex items-center gap-4">
              <button
                onClick={() => {
                  invoke("play_episode_cmd", {
                    episodeId: data.hero_ep!.id,
                    filePath: data.hero_ep!.file_path,
                    lastPosition: data.hero_ep!.last_position,
                  }).catch(alert);
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
                    <Play fill="currentColor" className="w-5 h-5" /> Play Next
                  </>
                ) : (
                  "❌ Missing File"
                )}
              </button>
              <button
                onClick={() => onMediaSelect(data.hero_ep!.media_id)}
                className="px-8 py-3 rounded-lg font-bold bg-white/10 hover:bg-white/20 text-white backdrop-blur-md transition-all duration-300 hover:scale-105"
              >
                More Info
              </button>
            </div>

            {data.hero_ep.status === "Watching" && data.hero_ep.runtime > 0 && (
              <div className="w-64 h-1.5 bg-white/20 rounded-full mt-6 overflow-hidden">
                <div
                  className="h-full bg-[#FF6B00] rounded-full"
                  style={{ width: `${calculateProgress(data.hero_ep.last_position, data.hero_ep.runtime)}%` }}
                />
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="w-full h-[450px] bg-[#1F222A]/80 backdrop-blur-xl rounded-2xl flex flex-col items-center justify-center text-center">
          <h1 className="text-4xl font-bold mb-4 text-white">Welcome to WatchMark</h1>
          <p className="text-gray-400 max-w-md">Scan your local folder or search TMDB to get started and build your library.</p>
        </div>
      )}

      {/* B. Continue Watching (Horizontal Row) */}
      {filteredCW.length > 0 && (
        <>
          <h2 className="text-2xl font-bold mt-12 mb-6 text-white">Continue Watching</h2>
          <div className="flex gap-6 overflow-x-auto pb-4 scrollbar-hide snap-x">
            {filteredCW.map((ep) => {
              const imagePath = ep.still_path || ep.backdrop_path;
              const imgUrl = imagePath ? `https://image.tmdb.org/t/p/w500${imagePath}` : PLACEHOLDER_BACKDROP;
              const progress = calculateProgress(ep.last_position, ep.runtime);

              return (
                <motion.div
                  key={ep.id}
                  onClick={() => onMediaSelect(ep.media_id)}
                  whileHover={{ scale: 1.02 }}
                  className="flex-none min-w-[320px] bg-[#1F222A] rounded-xl overflow-hidden cursor-pointer group transition-all duration-300 hover:ring-2 hover:ring-[#FF6B00]/50 snap-start shadow-lg relative"
                >
                  <div className="w-full h-[180px] relative overflow-hidden">
                    <img
                      src={imgUrl}
                      alt={ep.show_title || "Show Thumbnail"}
                      className="w-full h-full object-cover opacity-80 group-hover:opacity-100 group-hover:scale-105 transition-all duration-500"
                    />
                    <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300" />

                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        invoke("play_episode_cmd", {
                          episodeId: ep.id,
                          filePath: ep.file_path,
                          lastPosition: ep.last_position,
                        }).catch(alert);
                      }}
                      className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-12 h-12 bg-[#FF6B00] text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all scale-75 group-hover:scale-100 z-20 hover:bg-[#E66000]"
                    >
                      <Play fill="currentColor" className="w-5 h-5 ml-1" />
                    </button>
                  </div>

                  {ep.status === "Watching" && ep.runtime > 0 && (
                    <div className="h-1 w-full bg-white/10">
                      <div
                        className="h-full bg-[#FF6B00]"
                        style={{ width: `${progress}%` }}
                      />
                    </div>
                  )}

                  <div className="p-4 flex justify-between items-center bg-[#1F222A]">
                    <h3 className="font-bold text-white truncate pr-2">{ep.show_title || "Unknown Show"}</h3>
                    <span className="text-xs font-medium text-gray-400 whitespace-nowrap">
                      {ep.media_type === "TV" ? `S${ep.season_num}E${ep.ep_num}` : (ep.title || "No Title")}
                    </span>
                  </div>
                </motion.div>
              );
            })}
          </div>
        </>
      )}

      {/* C. Recently Added (Poster Grid Row) */}
      {filteredRecent.length > 0 && (
        <>
          <h2 className="text-2xl font-bold mt-12 mb-6 text-white">Recently Added</h2>
          <div className="flex gap-6 overflow-x-auto pb-8 scrollbar-hide snap-x pt-2">
            {filteredRecent.map((media) => {
              const imgUrl = media.poster_path ? `https://image.tmdb.org/t/p/w500${media.poster_path}` : PLACEHOLDER_POSTER;

              return (
                <div
                  key={media.id}
                  onClick={() => onMediaSelect(media.id)}
                  className="relative flex-none min-w-[180px] aspect-[2/3] rounded-xl overflow-hidden cursor-pointer group snap-start shadow-xl transition-all duration-300 hover:scale-105 hover:z-10 hover:shadow-2xl hover:shadow-[#FF6B00]/10 bg-[#1F222A]"
                >
                  <img
                    src={imgUrl}
                    alt={media.title || "Unknown Title"}
                    className="w-full h-full object-cover"
                  />

                  <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center backdrop-blur-sm">
                    <div className="w-14 h-14 bg-[#FF6B00] text-white rounded-full flex items-center justify-center shadow-lg shadow-black/50 scale-75 group-hover:scale-100 transition-transform duration-300">
                      <Play fill="currentColor" className="w-6 h-6 ml-1" />
                    </div>
                  </div>

                  <div className="absolute top-2 right-2 bg-black/60 backdrop-blur-md px-2 py-1 rounded-lg text-xs font-bold text-[#FF6B00] shadow-md flex items-center gap-1">
                    ★ {media.user_rating > 0 ? media.user_rating.toFixed(1) : 'Unrated'}
                  </div>

                  {media.total_episodes > 0 && (
                    <div className="absolute bottom-0 left-0 w-full h-1 bg-white/20">
                      <div
                        className="h-full bg-green-500"
                        style={{ width: `${Math.min(100, (media.completed_eps / media.total_episodes) * 100)}%` }}
                      />
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </>
      )}

      {/* D. Quick Stats */}
      <h2 className="text-2xl font-bold mt-12 mb-6 text-white">Your Stats</h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
        <div className="bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-6 border border-white/5 transition-all duration-300 hover:bg-[#1F222A]/80 shadow-xl flex flex-col items-center justify-center text-center">
          <h3 className="text-sm font-medium text-gray-400 uppercase tracking-widest mb-2">Shows Tracked</h3>
          <p className="text-5xl font-black text-[#FF6B00] drop-shadow-md">{data.stats.shows_completed}</p>
        </div>
        <div className="bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-6 border border-white/5 transition-all duration-300 hover:bg-[#1F222A]/80 shadow-xl flex flex-col items-center justify-center text-center">
          <h3 className="text-sm font-medium text-gray-400 uppercase tracking-widest mb-2">Hours Watched</h3>
          <p className="text-5xl font-black text-[#FF6B00] drop-shadow-md">{data.stats.hrs_watched}</p>
        </div>
        <div className="bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-6 border border-white/5 transition-all duration-300 hover:bg-[#1F222A]/80 shadow-xl flex flex-col items-center justify-center text-center">
          <h3 className="text-sm font-medium text-gray-400 uppercase tracking-widest mb-2">Average Rating</h3>
          <p className="text-5xl font-black text-[#FF6B00] drop-shadow-md flex items-center justify-center gap-2">
            <Star className="w-8 h-8 fill-current" /> {data.stats.avg_rating.toFixed(1)}
          </p>
        </div>
      </div>
    </div>
  );
}
