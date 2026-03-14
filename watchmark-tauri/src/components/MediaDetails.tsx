import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { Play, ArrowLeft, Star, Trash2, CloudOff, Clock, Lock } from "lucide-react";
import { useUiStore } from "../store/uiStore";
import { toast } from "sonner";
import { StarRating } from "./ui/StarRating";

export default function MediaDetails({ mediaId, onBack, refreshTrigger }: any) {
  const { showConfirm } = useUiStore();
  const [data, setData] = useState<any>(null);
  const [activeSeason, setActiveSeason] = useState<number>(1);
  const [showFullSynopsis, setShowFullSynopsis] = useState<boolean>(false);
  const isAnimatingRef = useRef(false);
  const [contextMenu, setContextMenu] = useState<{ x: number, y: number, epId: number } | null>(null);

  useEffect(() => {
    const handleClick = () => setContextMenu(null);
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, []);

  useEffect(() => {
    invoke("get_media_details_db", { mediaId })
      .then((res: any) => {
        setData(res);
        if (res.seasons && res.seasons.length > 0) {
           setActiveSeason(res.seasons[0]);
        }
      })
      .catch(console.error);
  }, [mediaId, refreshTrigger]);

  if (!data) {
    return (
      <div className="flex h-full items-center justify-center p-8">
        <div className="w-16 h-16 border-4 border-[#FF6B00] border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <motion.div
      initial={{ x: "10%", opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: "10%", opacity: 0 }}
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className="relative min-h-screen pb-32"
    >
      {/* Edge-to-edge Hero Banner */}
      <div className="relative w-full h-[50vh] min-h-[400px]">
        <div className="absolute inset-0">
          <img
            src={`https://image.tmdb.org/t/p/original${data.backdrop_path}`}
            alt="Backdrop"
            className="w-full h-full object-cover opacity-60"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-[#0D0F14] via-[#0D0F14]/40 to-transparent" />
        </div>

        {/* Back Button */}
        <button
          onClick={onBack}
          className="absolute top-8 left-8 z-50 p-3 bg-black/40 hover:bg-black/60 backdrop-blur-md rounded-full text-white transition-all shadow-lg"
        >
          <ArrowLeft className="w-6 h-6" />
        </button>
      </div>

      {/* Content Area */}
      <div className="relative z-10 px-12 -mt-48 flex gap-12 items-start">
        {/* Poster (overlapping banner) */}
        <motion.div
          initial={{ y: 50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.2 }}
          className="relative w-64 shrink-0 shadow-2xl rounded-2xl overflow-hidden border border-white/10"
        >
          <img
            src={`https://image.tmdb.org/t/p/w500${data.poster_path}`}
            alt="Poster"
            className={`w-full h-auto object-cover ${data.is_unaired ? 'grayscale-[0.5] opacity-70' : ''}`}
          />
          {data.is_unaired && (
            <div className="absolute top-2 left-2 z-20 px-2 py-1 bg-blue-500/80 backdrop-blur-md rounded-md text-[10px] font-bold text-white shadow-md uppercase tracking-wider">
              Planned
            </div>
          )}
        </motion.div>

        {/* Title and Info */}
        <motion.div
          initial={{ y: 30, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.3 }}
          className="pt-16 flex-1 max-w-4xl"
        >
          <h1 className="text-6xl font-extrabold tracking-tight text-white mb-4 drop-shadow-lg">
            {data.title}
          </h1>

          <div className="flex items-center gap-4 mb-6 text-sm font-bold tracking-wider">
            <span className={`px-3 py-1 rounded-md backdrop-blur-md text-white ${data.type === 'Unknown' ? 'bg-red-500/80' : 'bg-white/10'}`}>
              {data.type}
            </span>
            <span className="text-gray-300">Aired: {data.release_date ? (data.is_exact_date ? data.release_date : data.release_date.substring(0, 4)) : "Unknown"}</span>
            <span className="flex items-center gap-1 text-[#F5C518] bg-black/50 px-3 py-1 rounded-full min-w-[70px] justify-center text-center">
              {data.vote_average === null || data.vote_average === undefined ? (
                <span className="font-bold text-gray-400 tracking-widest text-[10px] px-1">NO DATA</span>
              ) : data.vote_average === 0 ? (
                <span className="font-bold text-gray-400 tracking-widest text-[10px] px-1">NR</span>
              ) : (
                <>
                  <Star className="w-4 h-4 fill-current" /> {Number(data.vote_average).toFixed(1)}
                </>
              )}
            </span>

            <div className="flex items-center gap-2 bg-black/50 px-3 py-1 rounded-md">
              {data.user_rating === 0 && (
                <span className="text-orange-500 font-bold bg-white/10 px-2 py-0.5 rounded text-xs mr-2">
                  0.0
                </span>
              )}
              <span className="text-gray-400 mr-2 text-xs">My Rating:</span>
              <StarRating
                rating={data.user_rating}
                onChange={(rating) => {
                  setData((prev: any) => ({ ...prev, user_rating: rating }));
                  invoke("update_media_rating", { mediaId: data.id, rating })
                    .catch((err) => {
                      console.error("Failed to update rating:", err);
                      toast.error("Failed to update rating");
                    });
                }}
              />
            </div>
          </div>

          <div className="mb-8">
            <p className="text-lg text-gray-300 leading-relaxed drop-shadow-md whitespace-pre-line">
              {!data.synopsis || data.synopsis.trim() === ""
                ? "No overview available."
                : (showFullSynopsis || data.synopsis.length <= 300)
                ? data.synopsis
                : `${data.synopsis.substring(0, 300)}...`}
            </p>
            {data.synopsis && data.synopsis.length > 300 && (
              <button
                onClick={() => setShowFullSynopsis(!showFullSynopsis)}
                className="mt-2 text-[#FF6B00] hover:text-[#FF8533] font-bold text-sm"
              >
                {showFullSynopsis ? "View Less" : "View More"}
              </button>
            )}
          </div>

          <div className="flex gap-4">
            {data.is_unaired ? (
              <div className="flex items-center gap-2 px-8 py-4 bg-gray-700/50 text-gray-300 font-bold rounded-full backdrop-blur-md cursor-not-allowed">
                <Clock className="w-5 h-5" /> 📅 Coming Soon
              </div>
            ) : (
              <button className="flex items-center gap-2 px-8 py-4 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-full transition-all shadow-lg shadow-orange-500/20 hover:scale-105">
                <Play fill="currentColor" /> Play Next
              </button>
            )}
            <button
              onClick={async () => {
                try {
                  await invoke("mark_season_watched", { mediaId, seasonNum: activeSeason, archiveMode: false });
                  invoke("get_media_details_db", { mediaId }).then((res: any) => setData(res));
                  toast.success(`Marked Season ${activeSeason} as Watched`);
                } catch (e) {
                  toast.error(`Failed: ${e}`);
                }
              }}
              className="px-8 py-4 bg-white/10 hover:bg-white/20 text-white font-bold rounded-full backdrop-blur-md transition-all hover:scale-105">
              Mark Season Watched
            </button>
            <button
              onClick={async () => {
                if (isAnimatingRef.current) return;
                try {
                  const historyCount = await invoke<number>("get_media_history_count", { mediaId });
                  let warningMessage = "Are you sure you want to remove this show? This action cannot be undone.";
                  if (historyCount > 0) {
                    warningMessage = `This will permanently delete ${historyCount} entries from your Watch Diary. ` + warningMessage;
                  }

                  const confirmDelete = await showConfirm("Remove Media", warningMessage);
                  if (confirmDelete) {
                    isAnimatingRef.current = true;
                    try {
                      // Block UI until backend completes safely
                      await invoke("delete_media_cmd", { mediaId });
                      // Once database cleanup is confirmed, transition safely back
                      toast.success("Media removed successfully.");
                      onBack();
                    } catch (e) {
                      console.error("Failed to delete media:", e);
                      toast.error("Failed to delete media.");
                      isAnimatingRef.current = false;
                    }
                  }
                } catch (e) {
                  console.error("Failed to get history count:", e);
                  toast.error("Failed to check media history.");
                }
              }}
              className="p-4 bg-white/5 hover:bg-red-500/20 text-gray-400 hover:text-red-500 rounded-full transition-colors ml-auto border border-transparent hover:border-red-500/50"
            >
              <Trash2 className="w-5 h-5" />
            </button>
          </div>
        </motion.div>
      </div>

      {/* Episodes List (Mocked structure) */}
      <div className="mt-24 px-12">
        {/* Season Tabs */}
        {data.type === "TV" && data.seasons && (
          <div className="flex flex-wrap gap-2 mb-8">
            {data.seasons.map((s: number) => (
              <button
                key={s}
                onClick={() => setActiveSeason(s)}
                className={`px-6 py-2 rounded-full font-bold transition-all whitespace-nowrap ${
                  s === activeSeason ? 'bg-[#FF6B00] text-white shadow-lg shadow-[#FF6B00]/20' : 'bg-white/5 hover:bg-white/10 text-gray-300'
                }`}
              >
                {s === 0 ? "Specials" : `Season ${s}`}
              </button>
            ))}
          </div>
        )}

        <div className="grid gap-4 max-w-5xl">
          {data.episodes?.filter((ep: any) => ep.season_num === activeSeason).map((ep: any) => (
             <div
               key={ep.id}
               className="flex items-center bg-[#1F222A]/60 backdrop-blur-md p-4 rounded-xl border border-white/5 hover:bg-white/5 transition-colors group relative"
               onContextMenu={(e) => {
                 e.preventDefault();
                 if (ep.file_path) {
                   setContextMenu({ x: e.pageX, y: e.pageY, epId: ep.id });
                 }
               }}
             >
               <div className="w-40 aspect-video bg-black/40 rounded-lg overflow-hidden shrink-0 relative mr-6">
                 {ep.still_path ? (
                    <img
                        src={`https://image.tmdb.org/t/p/w500${ep.still_path}`}
                        alt={ep.title}
                        className="w-full h-full object-cover"
                    />
                 ) : (
                    <div className="w-full h-full flex items-center justify-center text-gray-600 text-xs font-bold uppercase tracking-widest bg-[#15171e]">
                       EP {ep.ep_num}
                    </div>
                 )}
                 <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 bg-black/40 transition-opacity">
                    {ep.is_unaired ? (
                      <div className="w-12 h-12 rounded-full bg-gray-700/80 flex items-center justify-center text-white shadow-lg cursor-not-allowed group/tooltip relative">
                        <Lock className="w-5 h-5" />
                        <div className="absolute -top-10 scale-0 group-hover/tooltip:scale-100 transition-transform bg-black text-white text-xs px-3 py-1 rounded-md whitespace-nowrap">
                          {ep.air_date ? `Airing ${ep.air_date}` : 'Unaired'}
                        </div>
                      </div>
                    ) : (
                      <button
                        onClick={async () => {
                          if (ep.file_path) {
                            try {
                                const validation: any = await invoke("validate_and_hash_file", { episodeId: ep.id, filePath: ep.file_path });
                                if (validation.status === "missing" || validation.status === "corrupted") {
                                    toast.error(`File is ${validation.status}.`, {
                                        action: {
                                            label: "Locate",
                                            onClick: async () => {
                                                try {
                                                    const { open } = await import('@tauri-apps/plugin-dialog');
                                                    const selected = await open({
                                                        multiple: false,
                                                        title: "Locate File",
                                                    });
                                                    if (selected && typeof selected === 'string') {
                                                        await invoke("update_local_file", { episodeId: ep.id, newPath: selected });
                                                        toast.success("File linked successfully!");
                                                        // Refresh data
                                                        invoke("get_media_details_db", { mediaId }).then((res: any) => setData(res));
                                                        // Play new file
                                                        invoke("play_episode_cmd", {
                                                            episodeId: ep.id,
                                                            filePath: selected,
                                                            lastPosition: ep.last_position,
                                                        });
                                                    }
                                                } catch (err) {
                                                    toast.error("Failed to locate file.");
                                                }
                                            }
                                        },
                                        duration: 5000,
                                    });
                                    return;
                                }
                            } catch (e) {
                                toast.error(`Validation error: ${e}`);
                                return;
                            }

                            invoke("play_episode_cmd", {
                              episodeId: ep.id,
                              filePath: ep.file_path,
                              lastPosition: ep.last_position,
                            }).catch(e => toast.error(e));
                          } else {
                            toast.error("Missing File Path. Scan directory to match file.");
                          }
                        }}
                        className={`w-12 h-12 rounded-full flex items-center justify-center text-white scale-75 hover:scale-100 transition-transform shadow-lg ${ep.file_path ? 'bg-[#FF6B00] shadow-orange-500/30' : 'bg-gray-600 shadow-gray-500/30'}`}>
                        {ep.file_path ? <Play className="w-5 h-5 ml-1" fill="currentColor" /> : <CloudOff className="w-5 h-5" />}
                      </button>
                    )}
                 </div>
                 {ep.status === "Watching" && ep.runtime > 0 && (
                    <div className="absolute bottom-0 left-0 w-full h-1 bg-white/10 flex">
                        {(() => {
                          const progress = Math.min(100, (ep.last_position / (ep.runtime * 60)) * 100);
                          if (progress <= 0) return null;
                          return (
                            <div
                                className="h-full"
                                style={{
                                  width: `${progress}%`,
                                  minWidth: "2px",
                                  backgroundColor: progress >= 90 ? "#1b5e20" : "#FF6B00"
                                }}
                            />
                          );
                        })()}
                    </div>
                 )}
               </div>

               <div className="flex-1">
                 <div className="flex items-center gap-4 mb-1">
                    <button
                      onClick={async (e) => {
                        e.stopPropagation();
                        try {
                          await invoke("toggle_episode_status", { episodeId: ep.id });
                          invoke("get_media_details_db", { mediaId }).then((res: any) => setData(res));
                        } catch (err) {
                          toast.error(`Failed to update status: ${err}`);
                        }
                      }}
                      className={`w-8 h-8 rounded-full border-2 flex items-center justify-center transition-colors ${ep.status === 'Completed' ? 'border-green-500 bg-green-500/20 text-green-500' : 'border-gray-600 hover:border-green-500 hover:bg-green-500/20'}`}>
                      {ep.status === 'Completed' ? "✓" : <div className="w-3 h-3 rounded-full bg-transparent" />}
                    </button>
                    <h3 className={`text-xl font-bold transition-colors ${ep.status === 'Completed' ? 'text-gray-400 font-normal' : 'text-white group-hover:text-[#FF6B00]'}`}>
                      {ep.ep_num}. {ep.title}
                    </h3>
                    <span className="text-gray-500 text-sm ml-auto">{ep.runtime > 0 ? `${ep.runtime}m` : ''}</span>
                 </div>
                 <p className="text-gray-400 text-sm pl-12 line-clamp-2">
                   {ep.overview}
                 </p>
               </div>
             </div>
          ))}
          {(!data.episodes || data.episodes.length === 0) && (
              <div className="p-8 text-gray-500">No episodes found for this media.</div>
          )}
        </div>
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <div
          className="fixed bg-[#2A2D35] border border-white/10 shadow-2xl rounded-lg py-2 z-50 min-w-[160px]"
          style={{ top: contextMenu.y, left: contextMenu.x }}
        >
          <button
            className="w-full text-left px-4 py-2 hover:bg-white/10 text-red-400 text-sm flex items-center gap-2"
            onClick={async () => {
              try {
                await invoke("remove_local_link", { episodeId: contextMenu.epId });
                toast.success("Local link removed.");
                invoke("get_media_details_db", { mediaId }).then((res: any) => setData(res));
              } catch (e) {
                toast.error(`Failed to remove link: ${e}`);
              }
            }}
          >
            <CloudOff className="w-4 h-4" /> Unlink Local File
          </button>
        </div>
      )}
    </motion.div>
  );
}
