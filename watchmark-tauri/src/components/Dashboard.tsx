import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { Play } from "lucide-react";

export default function Dashboard({ onMediaSelect, refreshTrigger }: any) {
  const [data, setData] = useState<any>(null);

  useEffect(() => {
    invoke("get_dashboard_data").then(setData).catch(console.error);
  }, [refreshTrigger]);

  if (!data) return <div className="p-8 text-gray-400">Loading Dashboard...</div>;

  return (
    <div className="pb-24">
      {/* Hero Banner */}
      {data.hero_ep ? (
        <div className="relative w-full h-[60vh] min-h-[400px]">
          <div className="absolute inset-0">
            {data.hero_ep.backdrop_path ? (
              <img
                src={`https://image.tmdb.org/t/p/original${data.hero_ep.backdrop_path}`}
                alt="Hero"
                className="w-full h-full object-cover opacity-80"
              />
            ) : (
              <div className="w-full h-full bg-[#1F222A]" />
            )}
            <div className="absolute inset-0 bg-gradient-to-t from-[#0D0F14] via-[#0D0F14]/60 to-transparent" />
            <div className="absolute inset-0 bg-gradient-to-r from-[#0D0F14] via-[#0D0F14]/60 to-transparent" />
          </div>

          <div className="absolute bottom-0 left-0 p-12 w-full max-w-4xl z-10">
            <h2 className="text-[#FF6B00] font-bold tracking-widest text-sm mb-4 uppercase">
              {data.hero_ep.status === "Watching" && data.hero_ep.last_position > 0 ? "Resume Session" : "Up Next"}
            </h2>
            <h1 className="text-6xl font-extrabold text-white mb-4 tracking-tight">
              {data.hero_ep.show_title || "Unknown Show"}
            </h1>
            <p className="text-2xl text-gray-300 font-medium mb-8">
              {data.hero_ep.media_type === "TV"
                ? `S${String(data.hero_ep.season_num).padStart(2, '0')} E${String(data.hero_ep.ep_num).padStart(2, '0')} - ${data.hero_ep.title}`
                : data.hero_ep.title}
            </p>

            <div className="flex items-center gap-4">
              <button
                onClick={() => {
                  invoke("play_episode_cmd", {
                    episodeId: data.hero_ep.id,
                    filePath: data.hero_ep.file_path,
                    lastPosition: data.hero_ep.last_position,
                  }).catch(alert);
                }}
                disabled={!data.hero_ep.file_path}
                className={`flex items-center gap-2 px-8 py-4 rounded-full font-bold text-lg transition-all duration-300 ${
                  data.hero_ep.file_path
                    ? "bg-[#FF6B00] hover:bg-[#E66000] text-white hover:scale-105"
                    : "bg-red-900/50 text-red-200 cursor-not-allowed"
                }`}
              >
                {data.hero_ep.file_path ? (
                  <>
                    <Play fill="currentColor" /> Play Next
                  </>
                ) : (
                  "❌ Missing File"
                )}
              </button>
              <button
                onClick={() => onMediaSelect(data.hero_ep.media_id)}
                className="px-8 py-4 rounded-full font-bold text-lg bg-white/10 hover:bg-white/20 text-white backdrop-blur-md transition-all duration-300 hover:scale-105"
              >
                More Info
              </button>
            </div>
            {data.hero_ep.status === "Watching" && data.hero_ep.runtime > 0 && (
              <div className="mt-8 w-64 bg-white/10 h-1.5 rounded-full overflow-hidden">
                <div
                  className="bg-[#FF6B00] h-full"
                  style={{ width: `${Math.min(100, (data.hero_ep.last_position / (data.hero_ep.runtime * 60)) * 100)}%` }}
                />
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="h-[40vh] flex items-center justify-center bg-[#1F222A] rounded-2xl m-8">
          <div className="text-center">
            <h1 className="text-4xl font-bold mb-4">Welcome to WatchMark</h1>
            <p className="text-gray-400">Scan your local folder or search TMDB to get started.</p>
          </div>
        </div>
      )}

      {/* Up Next Horizontal Row */}
      {data.cw_eps?.length > 0 && (
        <div className="mt-12">
          <h2 className="text-2xl font-bold px-12 mb-6">Continue Watching</h2>
          <div className="flex overflow-x-auto px-12 pb-8 gap-6 snap-x hide-scrollbar">
            {data.cw_eps.map((ep: any) => (
              <motion.div
                key={ep.id}
                whileHover={{ scale: 1.05 }}
                className="relative flex-none w-[320px] aspect-video bg-[#1F222A] rounded-xl overflow-hidden cursor-pointer group snap-start"
                onClick={() => onMediaSelect(ep.media_id)}
              >
                <img
                  src={`https://image.tmdb.org/t/p/w500${ep.still_path || ep.backdrop_path}`}
                  alt={ep.show_title}
                  className="w-full h-full object-cover opacity-70 group-hover:opacity-100 transition-opacity"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent" />
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    invoke("play_episode_cmd", {
                      episodeId: ep.id,
                      filePath: ep.file_path,
                      lastPosition: ep.last_position,
                    }).catch(alert);
                  }}
                  className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-14 h-14 bg-[#FF6B00] text-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all scale-75 group-hover:scale-100 hover:bg-[#E66000] z-20"
                >
                  <Play fill="currentColor" />
                </button>
                <div className="absolute bottom-0 left-0 p-4 w-full z-10">
                  <h3 className="font-bold text-lg truncate">{ep.show_title}</h3>
                  <p className="text-sm text-gray-300 truncate">
                    {ep.media_type === "TV" ? `S${ep.season_num}E${ep.ep_num} - ${ep.title}` : ep.title}
                  </p>
                </div>
                {ep.status === "Watching" && ep.runtime > 0 && (
                  <div className="absolute bottom-0 left-0 w-full h-1 bg-white/10">
                    <div
                      className="h-full bg-[#FF6B00]"
                      style={{ width: `${Math.min(100, (ep.last_position / (ep.runtime * 60)) * 100)}%` }}
                    />
                  </div>
                )}
              </motion.div>
            ))}
          </div>
        </div>
      )}

      {/* Recently Added Grid */}
      {data.recent_media?.length > 0 && (
        <div className="mt-8 px-12">
          <h2 className="text-2xl font-bold mb-6">Recently Added</h2>
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 gap-6">
            {data.recent_media.map((media: any) => (
              <motion.div
                key={media.id}
                whileHover={{ scale: 1.05, y: -5 }}
                className="relative aspect-[2/3] bg-[#1F222A] rounded-xl overflow-hidden cursor-pointer group shadow-lg"
                onClick={() => onMediaSelect(media.id)}
              >
                <img
                  src={`https://image.tmdb.org/t/p/w500${media.poster_path}`}
                  alt={media.title}
                  className="w-full h-full object-cover"
                />
                <div className="absolute top-2 right-2 bg-black/60 backdrop-blur-md px-2 py-1 rounded-md text-xs font-bold text-[#FF6B00]">
                  ★ {media.user_rating > 0 ? `${media.user_rating}/5` : 'Unrated'}
                </div>
                {media.total_episodes > 0 && (
                  <div className="absolute bottom-0 left-0 w-full h-1 bg-white/20">
                    <div
                      className="h-full bg-green-500"
                      style={{ width: `${Math.min(100, (media.completed_eps / media.total_episodes) * 100)}%` }}
                    />
                  </div>
                )}
              </motion.div>
            ))}
          </div>
        </div>
      )}

      {/* Stats Row */}
      <div className="mt-16 px-12">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="bg-[#1F222A]/50 backdrop-blur-sm p-6 rounded-2xl border border-white/5">
            <h3 className="text-gray-400 font-medium mb-2">Total Hours Watched</h3>
            <p className="text-4xl font-black text-[#FF6B00]">{data.stats.hrs_watched}</p>
          </div>
          <div className="bg-[#1F222A]/50 backdrop-blur-sm p-6 rounded-2xl border border-white/5">
            <h3 className="text-gray-400 font-medium mb-2">Shows Completed</h3>
            <p className="text-4xl font-black text-[#FF6B00]">{data.stats.shows_completed}</p>
          </div>
          <div className="bg-[#1F222A]/50 backdrop-blur-sm p-6 rounded-2xl border border-white/5">
            <h3 className="text-gray-400 font-medium mb-2">Average Rating</h3>
            <p className="text-4xl font-black text-[#FF6B00]">★ {data.stats.avg_rating.toFixed(1)}</p>
          </div>
        </div>
      </div>

    </div>
  );
}
