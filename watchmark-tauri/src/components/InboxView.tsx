import { Icon } from "./ui/Icon";
import { formatImagePath } from "../utils/imageFormat";
import { useState, useEffect } from "react";
import { FolderSearch, Search, SearchX, X } from "lucide-react";
import { formatLocaleDate } from "../utils/dateFormatter";
import { toast } from "../utils/toast";
import { open } from "@tauri-apps/plugin-dialog";
import { formatWindowsPath } from "../utils/pathUtils";
import { invokeWithTimeout } from "../utils/ipc";
import { invoke } from "@tauri-apps/api/core";
import { useTaskStore } from "../store/useTaskStore";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { logger } from "../utils/logger";
import { SafeImage } from "./ui/SafeImage";
import { motion } from "framer-motion";

export default function InboxView({ onMatch }: any) {
  const [unmatched, setUnmatched] = useState<any[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { setScanning, activeSyncs, isScanning, isScanPaused, setScanPaused } = useTaskStore();
  const asyncInvoke = useAsyncInvoke();

  useEffect(() => {
    fetchUnmatched();
  }, []);

  useEffect(() => {
    if (selectedGroup) {
      setSearchQuery(selectedGroup);
      setSearchResults([]);
    }
  }, [selectedGroup]);

  const fetchUnmatched = async () => {
    try {
      const res: any = await invoke("fetch_unmatched_files");
      setUnmatched(res);
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'")) {
        console.warn("Tauri invoke missing (likely running in a browser).");
      } else {
        console.error(e);
      }
    }
  };

  const grouped = unmatched.reduce((acc: any, curr: any) => {
    if (!acc[curr.group_key]) acc[curr.group_key] = [];
    acc[curr.group_key].push(curr);
    return acc;
  }, {});

  const performSearchForGroup = async (groupKey: string) => {
    if (!groupKey.trim()) return;
    setIsSearching(true);
    try {
      const res: any = await asyncInvoke("perform_tmdb_search", { query: groupKey, page: 1 });
      if (res) setSearchResults(res);
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'")) {
        console.warn("Tauri invoke missing.");
      } else {
        toast.error("Search failed: " + e);
      }
    } finally {
      setIsSearching(false);
    }
  };

  const performSearch = async () => {
    await performSearchForGroup(searchQuery);
  };

  const handleGroupSelect = (key: string) => {
    setSelectedGroup(key);
    setSearchQuery(key);
    setSearchResults([]);
    performSearchForGroup(key);
  };

  const ignoreGroup = async (groupKey: string) => {
    // Optimistic UI update
    const previousUnmatched = [...unmatched];
    setUnmatched(unmatched.filter(item => item.group_key !== groupKey));
    if (selectedGroup === groupKey) {
        setSelectedGroup(null);
    }

    let isUndone = false;

    // Show toast with undo action
    toast.success(`Ignored group: ${groupKey}`, {
      duration: 5000,
      action: {
        label: 'Undo',
        onClick: () => {
          isUndone = true;
          setUnmatched(previousUnmatched);
          toast.info(`Restored group: ${groupKey}`);
        }
      }
    });

    // Wait 5 seconds before making the actual DB call
    setTimeout(async () => {
      if (!isUndone) {
        try {
          await asyncInvoke("ignore_unmatched_group", { groupKey });
          logger.inboxIgnore(`Ignored group: ${groupKey}`);
        } catch (e: any) {
           console.error("Failed to ignore group:", e);
           // Revert on failure
           setUnmatched(previousUnmatched);
           toast.error("Failed to ignore group.");
        }
      }
    }, 5000);
  };

  const assignShow = async (tmdbId: string, mediaType: string) => {
    if (!selectedGroup) return;
    logger.inboxMatch(tmdbId);
    try {
      await asyncInvoke("assign_unmatched_to_tracker", {
        tmdbId,
        mediaType,
        groupKey: selectedGroup
      });
      const matchedGroup = unmatched.find(g => g.group_key === selectedGroup);
      logger.inboxSuccess(matchedGroup?.files?.length || 0, `TMDB ID: ${tmdbId}`);
      toast.success(`Successfully assigned files to tracker!`);
      setSelectedGroup(null);
      setIsModalOpen(false);
      fetchUnmatched();
      if (onMatch) onMatch();
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'")) {
        console.warn("Tauri invoke missing.");
      } else {
        logger.error("Inbox Assignment Failed", e);
        toast.error("Error assigning show: " + e);
      }
    }
  };

  const openSearchModal = () => {
    if (selectedGroup) {
        logger.inboxGroup(selectedGroup);
    }
    setIsModalOpen(true);
    if (selectedGroup) {
      performSearch();
    }
  };

  const triggerScan = async () => {
    try {
      let defaultPath;
      try {
          const { invoke } = await import('@tauri-apps/api/core');
          const settings: any = await invoke('get_settings');
          if (settings?.last_scanned_path) {
              defaultPath = settings.last_scanned_path;
          } else {
              const { videoDir } = await import('@tauri-apps/api/path');
              defaultPath = await videoDir();
          }
      } catch (e) {
          console.warn("Could not load default path for scanner:", e);
      }

      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Directory to Scan",
        defaultPath
      });

      if (!selected) return;

      if (typeof selected === 'string') {
        setScanning(true);
        try {
            const res = await invokeWithTimeout<any>("run_scan_directory", { directory: selected }, 300000);
            toast.scanComplete(res);
            try {
                const { invoke } = await import('@tauri-apps/api/core');
                const settings: any = await invoke('get_settings');
                settings.last_scanned_path = selected;
                await invoke('save_settings', { settings });
            } catch (e) {
                console.warn("Failed to save last_scanned_path:", e);
            }
            fetchUnmatched();
        } catch (scanErr: any) {
            if (scanErr?.type === "AccessDenied" || scanErr?.code === "ACCESS_DENIED") {
              toast.error(`Access Denied: WatchMark lacks permissions for ${scanErr.path}`);
            } else {
              throw scanErr;
            }
        } finally {
            setScanning(false);
        }
      }
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'") || e?.toString().includes("window.__TAURI_INTERNALS__")) {
        console.warn("Tauri invoke missing. Cannot open system file dialog in browser.");
      } else {
        toast.error("Scan Error: " + e);
      }
    }
  };

  return (
    <div className="p-12 pb-24 pt-24 h-full flex flex-col">
      <div className="flex justify-between items-center mb-8">
        <h1 className="text-4xl font-extrabold tracking-tight">Inbox</h1>
        <div className="flex gap-4">
          <button
            onClick={async () => {
              if (confirm("Are you sure you want to clear the entire Inbox? This will not delete any files.")) {
                logger.inboxIgnore("Entire Inbox");
                try {
                  await invoke("clear_unmatched_files");
                  toast.success("Inbox cleared safely.");
                  fetchUnmatched();
                } catch (e: any) {
                  logger.error("Inbox Clear Failed", e);
                  toast.error("Error clearing inbox: " + e);
                }
              }
            }}
            className="flex items-center gap-2 px-6 py-3 bg-red-600/80 hover:bg-red-600 text-white font-bold rounded-lg transition-colors shadow-lg"
          >
            Clear Inbox
          </button>
          {isScanning ? (
            <div className="flex items-center gap-2">
              <button
                  onClick={() => {
                      import('@tauri-apps/api/core').then(({ invoke }) => {
                          if (isScanPaused) {
                              invoke('resume_active_scan').then(() => setScanPaused(false)).catch(console.error);
                          } else {
                              invoke('pause_active_scan').then(() => setScanPaused(true)).catch(console.error);
                          }
                      });
                  }}
                  className="flex items-center gap-2 px-6 py-3 bg-orange-600/80 hover:bg-orange-500 text-white font-bold rounded-lg transition-colors"
              >
                  {isScanPaused ? "Resume Scan" : "Pause Scan"}
              </button>
              <button
                  onClick={() => {
                      import('@tauri-apps/api/core').then(({ invoke }) => {
                          invoke('cancel_active_scan').catch(console.error);
                      });
                  }}
                  className="flex items-center gap-2 px-6 py-3 bg-red-600/80 hover:bg-red-500 text-white font-bold rounded-lg transition-colors"
              >
                  <FolderSearch className="w-5 h-5" />
                  Cancel Scan
              </button>
            </div>
          ) : (
              <button
                  onClick={triggerScan}
                  className="flex items-center gap-2 px-6 py-3 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-lg transition-colors"
              >
                  <FolderSearch className="w-5 h-5" />
                  Scan Directory
              </button>
          )}
        </div>
      </div>

      <p className="text-muted mb-8">You have {Object.keys(grouped).length} unrecognized series on your hard drive.</p>

      {Object.keys(grouped).length === 0 ? (
        <div className="flex items-center justify-center flex-1">
           <p className="text-xl text-gray-500 font-bold">Inbox is empty. All files are matched!</p>
        </div>
      ) : (
        <div className="flex flex-col lg:flex-row gap-8 flex-1 min-h-0">
          <div className="lg:w-1/3 max-h-[40vh] lg:max-h-none h-full bg-[#0D0F14] backdrop-blur-md rounded-2xl p-6 overflow-y-auto border border-white/5 scrollbar-hide">
            {Object.entries(grouped).map(([key, files]: [string, any]) => (
              <div
                key={key}
                onClick={() => handleGroupSelect(key)}
                className={`p-4 flex items-center justify-between gap-4 rounded-xl cursor-pointer transition-colors mb-2 border-l-2 group ${
                  selectedGroup === key ? "bg-[#FF6B00]/10 border-[#FF6B00]" : "border-transparent hover:bg-white/5"
                }`}
              >
                <div className="flex items-center gap-4 min-w-0 flex-1">
                  <SafeImage
                    srcPath=""
                    type="poster"
                    altText={key}
                    title={key}
                    className="w-12 h-16 shrink-0 shadow-md"
                  />
                  <div className="flex-1 min-w-0">
                    <h3 className="text-white font-bold truncate">{key}</h3>
                    <div className="flex items-center gap-2 mt-1">
                      <span className="bg-[#FF6B00] flex-shrink-0 text-white text-xs font-bold px-2 py-0.5 rounded-full text-center min-w-[20px]">
                        {files.length > 99 ? "99+" : files.length}
                      </span>
                      <p className="text-gray-500 text-xs font-semibold tracking-wider">
                        {files.length === 1 ? 'FILE' : 'FILES'}
                      </p>
                    </div>
                  </div>
                </div>

                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    ignoreGroup(key);
                  }}
                  className="p-2 bg-transparent hover:bg-red-600/20 text-red-500 hover:text-red-400 rounded-lg transition-colors opacity-0 group-hover:opacity-100 flex-shrink-0"
                  title="Ignore group"
                >
                  <Icon icon={X} className="w-5 h-5" />
                </button>
              </div>
            ))}
          </div>

          <div className="lg:w-2/3 h-full bg-[#1F222A] backdrop-blur-md rounded-2xl p-8 border border-[#2A2D35] flex flex-col overflow-y-auto">
            {!selectedGroup ? (
              <div className="flex flex-col items-center justify-center text-center h-full">
                <FolderSearch className="w-24 h-24 text-gray-700 mb-6" />
                <h2 className="text-2xl font-bold mb-4 text-white">Select a group on the left to begin matching files to TMDB.</h2>
                <p className="text-muted max-w-md">
                  Groups are generated automatically from filenames.
                </p>
              </div>
            ) : (
              <div className="flex flex-col h-full">
                <div className="flex justify-between items-center mb-6">
                  <div>
                    <h2 className="text-2xl font-bold text-white mb-1">Group: {selectedGroup}</h2>
                    <p className="text-muted text-sm">{grouped[selectedGroup]?.length || 0} files</p>
                  </div>
                  <button
                    onClick={openSearchModal}
                    className="px-6 py-3 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-lg transition-colors shadow-lg shadow-orange-500/20"
                  >
                    + Add Tracker
                  </button>
                </div>

                <div className="flex-1 overflow-y-auto bg-black/20 rounded-xl border border-white/5 p-4">
                  <h3 className="text-sm font-bold text-gray-500 uppercase tracking-wider mb-4">Files in this group</h3>
                  <div className="space-y-2">
                    {grouped[selectedGroup]?.map((file: any, index: number) => (
                      <div key={index} className="flex flex-col gap-1 p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-colors">
                        <div className="flex items-center gap-3">
                          <div className="w-2 h-2 rounded-full bg-[#FF6B00]"></div>
                          <p className="text-sm text-gray-300 font-mono break-all">{file.filename}</p>
                        </div>
                        <p className="text-xs text-gray-600 font-mono break-all pl-5">{formatWindowsPath(file.file_path)}</p>
                      </div>
                    ))}
                  </div>
                </div>

              </div>
            )}
          </div>
        </div>
      )}

      {isModalOpen && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 md:p-12">
          <div className="bg-[#1F222A] w-full max-w-5xl max-h-full rounded-2xl border border-white/10 shadow-2xl flex flex-col overflow-hidden">
            <div className="p-6 border-b border-white/5 flex justify-between items-center bg-white/5">
              <h2 className="text-xl font-bold text-white flex items-center gap-2">
                <Search className="w-5 h-5 text-[#FF6B00]" />
                Assign Tracker for "{selectedGroup}"
              </h2>
              <button
                onClick={() => setIsModalOpen(false)}
                className="p-2 text-muted hover:text-white hover:bg-white/10 rounded-full transition-colors"
              >
                <Icon icon={X} className="w-5 h-5" />
              </button>
            </div>

            <div className="p-6 border-b border-white/5 bg-black/20">
              <div className="flex gap-4">
                <input
                  type="text"
                  placeholder="Search TMDB..."
                  value={searchQuery}
                  onChange={(e) => {
                      if (searchQuery !== e.target.value) {
                          logger.inboxEdit(searchQuery, e.target.value);
                      }
                      setSearchQuery(e.target.value);
                  }}
                  onKeyDown={(e) => e.key === "Enter" && performSearch()}
                  className="flex-1 bg-[#15171e] text-white px-6 py-3 rounded-full border border-white/5 focus:outline-none focus:border-[#FF6B00] transition-colors"
                />
                <button
                  onClick={performSearch}
                  disabled={isSearching}
                  className="px-8 py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-full transition-colors disabled:opacity-50"
                >
                  {isSearching ? "Searching..." : "Search"}
                </button>
              </div>
            </div>

            <div className="p-6 overflow-y-auto flex-1 bg-[#0D0F14]">
              {isSearching ? (
                <div className="flex justify-center p-12">
                   <div className="w-8 h-8 rounded-full border-4 border-[#FF6B00] border-t-transparent animate-spin"></div>
                </div>
              ) : searchResults.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-center text-gray-500 p-12">
                  <SearchX className="w-16 h-16 text-gray-700 mb-4" />
                  <h3 className="text-xl font-bold text-white mb-2">No TMDB match found for "{searchQuery}".</h3>
                  <p className="max-w-md">The automatic extraction might have missed some details. Try refining your search by typing a different name.</p>
                </div>
              ) : (
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
                  {searchResults.map((item, i) => (
                    <div key={i} className="bg-[#1F222A] rounded-xl overflow-hidden group border border-white/5 shadow-xl">
                      <div className="aspect-[2/3] relative">
                        {activeSyncs[`tmdb_${item.tmdb_id}`] !== undefined && (
                          <div className="absolute top-0 left-0 w-full h-1 z-50 bg-black/50">
                            <motion.div
                              initial={{ width: 0 }}
                              animate={{ width: `${activeSyncs[`tmdb_${item.tmdb_id}`]}%` }}
                              className="h-full bg-[#FF6B00] shadow-[0_0_10px_#FF6B00]"
                            />
                          </div>
                        )}
                        <SafeImage
                          srcPath={item.poster_path ? formatImagePath(item.poster_path, "w500") : ""}
                          type="poster"
                          altText={item.title}
                          title={item.title}
                          className="w-full h-full object-cover"
                        />
                        <div className="absolute inset-0 bg-black/80 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col justify-end p-4">
                          <button
                            onClick={() => assignShow(item.tmdb_id, item.type)}
                            className="w-full py-3 bg-[#FF6B00] text-white font-bold rounded-lg hover:bg-[#E66000] transition-colors shadow-lg"
                          >
                            Assign Show
                          </button>
                        </div>
                      </div>
                      <div className="p-4">
                        <h3 className="text-white font-bold text-sm truncate">{item.title}</h3>
                        <p className="text-xs text-[#FF6B00] font-bold uppercase mt-1">
                          {item.type} • <span className="text-gray-500">{item.is_date_known ? (item.is_exact_date ? formatLocaleDate(item.release_date) : item.release_date.substring(0, 4)) : <span className="px-1.5 py-0.5 bg-gray-800 rounded text-xs font-semibold uppercase tracking-wider text-muted">TBD</span>}</span>
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
