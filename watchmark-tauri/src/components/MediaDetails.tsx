import { logger } from "../utils/logger";
import { Icon } from "./ui/Icon";
import { formatImagePath } from "../utils/imageFormat";
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { motion, AnimatePresence } from "framer-motion";
import { Play, ArrowLeft, Star, Trash2, CloudOff, Clock, Calendar } from "lucide-react";
import { formatLocaleDate, formatRuntime } from "../utils/dateFormatter";
import { useUiStore } from "../store/uiStore";
import { useTaskStore } from "../store/useTaskStore";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { toast } from "../utils/toast";
import { StarRating } from "./ui/StarRating";
import { useAppStore } from "../store/useAppStore";
import { cn } from "../App";
import { RefreshCw } from "lucide-react";
import { SafeImage } from "./ui/SafeImage";

export default function MediaDetails({ mediaId, onBack, refreshTrigger }: any) {
  const { showConfirm, setProcessing } = useUiStore();
  const { isApiAuthorized, isOffline } = useAppStore();
  const [data, setData] = useState<any>(null);
  const activeSyncs = useTaskStore((state) => state.activeSyncs);
  const syncProgress = data ? activeSyncs[data.id.toString()] : undefined;
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [activeSeason, setActiveSeason] = useState<number>(1);
  const [showFullSynopsis, setShowFullSynopsis] = useState<boolean>(false);
  const isAnimatingRef = useRef(false);
  const [contextMenu, setContextMenu] = useState<{ x: number, y: number, epId: number } | null>(null);
  const asyncInvoke = useAsyncInvoke();
  const backButtonRef = useRef<HTMLButtonElement>(null);
  const [isBackDisabled, setIsBackDisabled] = useState(false);

  useEffect(() => {
    const handleClick = () => setContextMenu(null);
    document.addEventListener("click", handleClick);
    return () => document.removeEventListener("click", handleClick);
  }, []);

  useEffect(() => {
    const unlistenDelete = listen("media-deleted", (event: any) => {
      if (event.payload.media_id === mediaId) {
        setProcessing(false);
        toast.success("Media removed successfully.");
        onBack();
      }
    });

    const unlistenDeleteFailed = listen("media-delete-failed", (event: any) => {
      if (event.payload.media_id === mediaId) {
        setProcessing(false);
        toast.error(`Failed to delete media: ${event.payload.error}`);
        isAnimatingRef.current = false;
      }
    });

    return () => {
      unlistenDelete.then((f) => f());
      unlistenDeleteFailed.then((f) => f());
    };
  }, [mediaId, onBack, setProcessing]);

  useEffect(() => {
    if (backButtonRef.current) {
        backButtonRef.current.focus();
    }
  }, []);

  useEffect(() => {
    asyncInvoke("get_media_details_db", { mediaId })
      .then((res: any) => {
        if (res) {
          setData(res);
          if (res.seasons && res.seasons.length > 0) {
            setActiveSeason(res.seasons[0]);
          }
        }
      })
      .catch(console.error);
  }, [mediaId, refreshTrigger, asyncInvoke]);

  if (!data) {
    return (
      <div className="flex h-full items-center justify-center p-8">
        <div className="w-16 h-16 border-4 border-[#FF6B00] border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  const handleBackClick = () => {
      if (isBackDisabled) return;
      setIsBackDisabled(true);
      setTimeout(() => setIsBackDisabled(false), 500);

      if (window.history.length <= 2) {
         // Fallback if history is missing or user deep-linked
         window.location.hash = "#/";
      } else {
         onBack();
      }
  };

  return (
    <motion.div
      initial={{ x: "10%", opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: "10%", opacity: 0 }}
      transition={{ type: "spring", stiffness: 300, damping: 30 }}
      className="relative min-h-screen pb-32"
    >
      {/* Edge-to-edge Hero Banner */}
      <div className="relative w-full h-[400px]">
        {syncProgress !== undefined && (
          <div className="absolute top-0 left-0 w-full h-1 z-50 bg-black/50">
            <motion.div
              initial={{ width: 0 }}
              animate={{ width: `${syncProgress}%` }}
              className="h-full bg-[#FF6B00] shadow-[0_0_10px_#FF6B00]"
            />
          </div>
        )}
        <AnimatePresence mode="wait">
          <motion.div
             key={data.backdrop_path || 'fallback'}
             initial={{ opacity: 0 }}
             animate={{ opacity: 1 }}
             exit={{ opacity: 0 }}
             transition={{ duration: 0.4 }}
             className="absolute inset-0"
          >
            <SafeImage
              srcPath={data.backdrop_path ? formatImagePath(data.backdrop_path, "w1280") : ""}
              fallbackSrcPath={data.backdrop_fallback ? formatImagePath(data.backdrop_fallback, "w1280") : undefined}
              type="backdrop"
              altText="Backdrop"
              className="w-full h-full object-cover"
            />
            {/* Body-Merge gradient overlay */}
            <div
               className="absolute inset-0"
               style={{
                   background: 'linear-gradient(to bottom, transparent 0%, rgba(13, 15, 20, 0.6) 70%, #0D0F14 100%)'
               }}
            />
          </motion.div>
        </AnimatePresence>

        {/* Back Button */}
        <motion.button
          ref={backButtonRef}
          onClick={handleBackClick}
          disabled={isBackDisabled}
          tabIndex={0}
          aria-label="Return to Library"
          whileHover={{
              scale: 1.05,
              backgroundColor: "rgba(0, 0, 0, 0.6)"
          }}
          className="absolute top-6 left-6 z-50 flex items-center gap-2 px-4 py-2 bg-black/40 backdrop-blur-md rounded-full text-white transition-all shadow-lg outline-none focus:ring-2 focus:ring-[#FF6B00] focus:ring-offset-2 focus:ring-offset-[#0D0F14] sticky-or-absolute"
        >
          <motion.div whileHover={{ scale: 1.2 }}>
            <ArrowLeft className="w-5 h-5 text-white" />
          </motion.div>
          <span className="font-bold text-sm text-white pr-2">Back</span>
        </motion.button>
      </div>

      {/* Content Area */}
      <div className="relative z-10 px-12 -mt-40 flex gap-12 items-center">
        {/* Poster (overlapping banner) */}
        <motion.div
          initial={{ y: 50, opacity: 0 }}
          animate={{ y: 0, opacity: 1 }}
          transition={{ delay: 0.2 }}
          style={{ filter: "drop-shadow(0 20px 30px rgba(0,0,0,0.9))" }}
          className="relative w-64 shrink-0 rounded-2xl overflow-hidden border border-white/10 aspect-[2/3]"
        >
          <SafeImage
            srcPath={data.poster_path ? formatImagePath(data.poster_path, "w500") : ""}
            type="poster"
            altText="Poster"
            title={data.title}
            className={`w-full h-full object-cover ${data.is_unaired ? 'grayscale-[0.5] opacity-70' : ''}`}
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
          className="flex-1 max-w-4xl"
        >
          <h1 className="text-6xl font-extrabold tracking-tight text-white mb-4 drop-shadow-lg">
            {data.title}
          </h1>

          <div className="flex items-center gap-4 mb-3 text-sm font-bold tracking-wider flex-wrap">
            <span className={`px-3 py-1 rounded-md backdrop-blur-md text-white ${data.type === 'Unknown' ? 'bg-red-500/80' : 'bg-white/10'}`}>
              {data.type}
            </span>
            <span className="text-gray-300">Aired: {data.is_date_known ? (data.is_exact_date ? formatLocaleDate(data.release_date) : data.release_date.substring(0, 4)) : <span className="px-1.5 py-0.5 bg-gray-800 rounded text-xs font-semibold uppercase tracking-wider text-muted">TBD</span>}</span>
            <span className="flex items-center gap-1 text-[#F5C518] bg-black/50 px-3 py-1 rounded-full min-w-[70px] justify-center text-center">
              {data.vote_average === null || data.vote_average === undefined ? (
                <span className="font-bold text-muted tracking-widest text-[10px] px-1">NO DATA</span>
              ) : data.vote_average === 0 ? (
                <span className="font-bold text-muted tracking-widest text-[10px] px-1">NR</span>
              ) : (
                <>
                  <Icon icon={Star} className="w-4 h-4 fill-current" /> {Number(data.vote_average).toFixed(1)}
                </>
              )}
            </span>

            <div className="flex items-center gap-2 bg-black/50 px-3 py-1 rounded-md">
              {data.user_rating === 0 && (
                <span className="text-orange-500 font-bold bg-white/10 px-2 py-0.5 rounded text-xs mr-2">
                  0.0
                </span>
              )}
              <span className="text-muted mr-2 text-xs">My Rating:</span>
              <StarRating
                rating={data.user_rating}
                onChange={(rating) => {
                  logger.click(`'Rating' changed to ${rating} stars`);
                  setData((prev: any) => ({ ...prev, user_rating: rating }));
                  logger.ipcSend("update_media_rating", `Rating ${rating}`);
                  invoke("update_media_rating", { mediaId: data.id, rating })
                    .catch((err: any) => {
                      logger.error("Rating Update Failed", err);
                      toast.error("Failed to update rating");
                    });
                }}
              />
            </div>

            {data.networks && data.networks.trim() !== "" && (
              <div className="flex items-center text-muted bg-white/5 px-3 py-1 rounded-md text-xs font-bold border border-white/10 uppercase">
                {data.networks}
              </div>
            )}
          </div>

          {data.genres && data.genres.trim() !== "" && (
            <div className="flex gap-2 mb-6 flex-wrap">
              {data.genres.split(",").map((g: string, i: number) => (
                <span key={i} className="px-3 py-1 rounded-full border border-white/20 bg-[#1F222A] text-gray-300 text-xs font-bold shadow-md tracking-wider">
                  {g.trim()}
                </span>
              ))}
            </div>
          )}

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
            <button
              onClick={async () => {
                if (!isApiAuthorized) {
                    toast.error("API Key required to refresh data.");
                    return;
                }
                if (isOffline) {
                    toast.error("Cannot refresh data while offline.");
                    return;
                }
                logger.click(`'Refresh Data' for '${data.title}'`);
                setIsRefreshing(true);
                try {
                  await asyncInvoke("add_to_tracker", {
                    tmdbId: data.tmdb_id,
                    mediaType: data.type,
                    archive: false
                  });
                  const res: any = await asyncInvoke("get_media_details_db", { mediaId });
                  if (res) {
                      setData(res);
                      logger.ipcSuccess(`Metadata Refreshed from TMDB for '${data.title}'.`);
                  }
                  toast.success("Data Refreshed Successfully");
                } catch (e: any) {
                  logger.error("Metadata Refresh Failed", e);
                  toast.error(`Refresh Failed: ${e}`);
                } finally {
                  setIsRefreshing(false);
                }
              }}
              disabled={isRefreshing || !isApiAuthorized || isOffline}
              title={isOffline ? "Requires internet connection." : !isApiAuthorized ? "API Key required" : "Refresh metadata from TMDB"}
              className={cn(
                "flex items-center gap-2 px-6 py-4 font-bold rounded-full backdrop-blur-md transition-all",
                isRefreshing || !isApiAuthorized || isOffline
                  ? "bg-white/5 text-gray-500 cursor-not-allowed opacity-50 grayscale pointer-events-none"
                  : "bg-white/10 hover:bg-white/20 text-white hover:scale-105"
              )}
            >
              <RefreshCw className={cn("w-5 h-5", isRefreshing && "animate-spin")} /> Refresh Data
            </button>
            {data.is_unaired ? (
              <div className="flex items-center gap-2 px-8 py-4 bg-gray-700/50 text-gray-300 font-bold rounded-full backdrop-blur-md cursor-not-allowed">
                <Clock className="w-5 h-5" /> 📅 Coming Soon
              </div>
            ) : (
              <button
                onClick={() => toast.success("Play Next algorithm is not implemented yet. Scroll down to play an episode.")}
                className="flex items-center gap-2 px-8 py-4 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-full transition-all shadow-lg shadow-orange-500/20 hover:scale-105"
              >
               
                <Icon icon={Play} fill="currentColor" /> Play Next
              </button>
            )}
            <button
              onClick={async () => {
                logger.click(`'Mark Season Watched' for Season ${activeSeason}`);
                logger.ipcSend("mark_season_watched", `Season ${activeSeason}`);
                try {
                  await invoke("mark_season_watched", { mediaId, seasonNum: activeSeason, archiveMode: false });
                  logger.ipcSuccess(`Marked Season ${activeSeason} as Watched.`);
                  asyncInvoke("get_media_details_db", { mediaId }).then((res: any) => { if (res) setData(res); });
                  toast.success(`Marked Season ${activeSeason} as Watched`);
                } catch (e: any) {
                  logger.error("Mark Season Watched Failed", e);
                  toast.error(`Failed: ${e}`);
                }
              }}
              className="px-8 py-4 bg-white/10 hover:bg-white/20 text-white font-bold rounded-full backdrop-blur-md transition-all hover:scale-105">
              Mark Season Watched
            </button>
            <button
              onClick={async () => {
                if (isAnimatingRef.current) return;
                logger.click(`'Remove from Library' for '${data.title}'`);
                try {
                  const historyCount = await invoke<number>("get_media_history_count", { mediaId });
                  let warningMessage = "Are you sure you want to remove this show? This action cannot be undone.";
                  if (historyCount > 0) {
                    warningMessage = `This will permanently delete ${historyCount} entries from your Watch Diary. ` + warningMessage;
                  }

                  const confirmDelete = await showConfirm("Remove Media", warningMessage);
                  if (confirmDelete) {
                    isAnimatingRef.current = true;
                    setProcessing(true, `Removing ${data?.title || 'Show'} from Library...`);
                    try {
                      // We push this to the background queue, and wait for the event listener to transition back
                      await invoke("delete_media_cmd", { mediaId });
                      logger.ipcSuccess(`'${data.title}' removal task enqueued.`);
                    } catch (e: any) {
                      setProcessing(false);
                      logger.error("Failed to enqueue delete media task", e);
                      toast.error("Failed to start deletion.");
                      isAnimatingRef.current = false;
                    }
                  } else {
                      logger.action(`User cancelled 'Remove Show'`);
                  }
                } catch (e: any) {
                  logger.error("Failed to get history count", e);
                  toast.error("Failed to check media history.");
                }
              }}
              className="p-4 bg-white/5 hover:bg-red-500/20 text-muted hover:text-red-500 rounded-full transition-colors ml-auto border border-transparent hover:border-red-500/50"
            >
              <Icon icon={Trash2} className="w-5 h-5" />
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
          {data.episodes?.filter((ep: any) => ep.season_num === activeSeason).map((ep: any) => {
             const stillUrl = ep.still_path ? formatImagePath(ep.still_path, "w500") : "";
             const fallbackUrl = data.backdrop_path ? formatImagePath(data.backdrop_path, "w1280") : "";
             return (
               <div
                 key={ep.id}
                 className="flex items-center bg-[#1F222A]/60 backdrop-blur-md p-4 rounded-xl border border-white/5 hover:bg-white/5 transition-colors group relative focus-within:ring-2 focus-within:ring-[#FF6B00] focus-within:ring-offset-2 focus-within:ring-offset-[#0D0F14]"
                 tabIndex={0}
                 onContextMenu={(e) => {
                   e.preventDefault();
                   if (ep.file_path) {
                     setContextMenu({ x: e.pageX, y: e.pageY, epId: ep.id });
                   }
                 }}
               >
                 <div className={`w-40 aspect-video bg-black/40 rounded-lg overflow-hidden shrink-0 relative mr-6 ${ep.is_unaired ? 'grayscale-[0.5] opacity-70' : ''}`}>
                   <SafeImage
                      srcPath={stillUrl}
                      fallbackSrcPath={fallbackUrl}
                      type="still"
                    episodeNumber={ep.ep_num}
                    altText={ep.title}
                    title={ep.title}
                    isFallbackImage={ep.is_fallback_image}
                    potentialSpoiler={ep.potential_spoiler}
                    isCompleted={ep.status === "Completed"}
                    className="w-full h-full object-cover"
                 />
                 <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 bg-black/40 transition-opacity">
                    {ep.is_unaired ? (
                      <div className="w-12 h-12 rounded-full bg-gray-700/80 flex items-center justify-center text-white shadow-lg cursor-not-allowed group/tooltip relative">
                        <Calendar className="w-5 h-5" />
                        <div className="absolute -top-10 scale-0 group-hover/tooltip:scale-100 transition-transform bg-black text-white text-xs px-3 py-1 rounded-md whitespace-nowrap">
                          {ep.is_date_known ? `Airing ${formatLocaleDate(ep.air_date)}` : 'Unaired / TBD'}
                        </div>
                      </div>
                    ) : (
                      <button
                        onClick={async () => {
                          logger.click(`'Play' on Episode S${ep.season_num}E${ep.ep_num}`);
                          if (ep.file_path) {
                            try {
                                const validation: any = await invoke("validate_and_hash_file", { episodeId: ep.id, filePath: ep.file_path });
                                if (validation.status === "missing" || validation.status === "corrupted") {
                                    logger.error("Playback Failed", `File is ${validation.status}`);
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
                                                        asyncInvoke("get_media_details_db", { mediaId }).then((res: any) => { if (res) setData(res); });
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
                        {ep.file_path ? <Icon icon={Play} className="w-5 h-5 ml-1" fill="currentColor" /> : <Icon icon={CloudOff} className="w-5 h-5" />}
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
                        logger.click(`'Toggle Status' on Episode S${ep.season_num}E${ep.ep_num}`);
                        logger.ipcSend("toggle_episode_status", `Episode ${ep.id}`);
                        try {
                          await invoke("toggle_episode_status", { episodeId: ep.id });
                          logger.ipcSuccess(`Episode status toggled.`);
                          asyncInvoke("get_media_details_db", { mediaId }).then((res: any) => { if (res) setData(res); });
                        } catch (err: any) {
                          logger.error("Episode Toggle Failed", err);
                          toast.error(`Failed to update status: ${err}`);
                        }
                      }}
                      className={`w-8 h-8 rounded-full border-2 flex items-center justify-center transition-colors ${ep.status === 'Completed' ? 'border-green-500 bg-green-500/20 text-green-500' : 'border-gray-600 hover:border-green-500 hover:bg-green-500/20'}`}>
                      {ep.status === 'Completed' ? "✓" : <div className="w-3 h-3 rounded-full bg-transparent" />}
                    </button>
                    <h3 className={`text-xl font-bold transition-colors ${ep.status === 'Completed' ? 'text-muted font-normal' : 'text-white group-hover:text-[#FF6B00]'}`}>
                      {ep.ep_num}. {ep.title}
                    </h3>
                    <span className="text-gray-500 text-sm ml-auto">{ep.runtime > 0 ? formatRuntime(ep.runtime) : ''}</span>
                 </div>
                 <p className="text-muted text-sm pl-12 line-clamp-2">
                   {ep.overview}
                 </p>
               </div>
             </div>
             );
          })}
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
              logger.click(`'Unlink Local File' context menu for Episode ${contextMenu.epId}`);
              try {
                await invoke("remove_local_link", { episodeId: contextMenu.epId });
                logger.ipcSuccess(`Local link removed from Episode ${contextMenu.epId}.`);
                toast.success("Local link removed.");
                asyncInvoke("get_media_details_db", { mediaId }).then((res: any) => { if (res) setData(res); });
              } catch (e: any) {
                logger.error("Remove Link Failed", e);
                toast.error(`Failed to remove link: ${e}`);
              }
            }}
          >
            <Icon icon={CloudOff} className="w-4 h-4" /> Unlink Local File
          </button>
        </div>
      )}
    </motion.div>
  );
}
