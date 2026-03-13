import { useState, useEffect } from "react";
import { FolderSearch, Search, X } from "lucide-react";
import { toast } from "sonner";
import { open } from "@tauri-apps/plugin-dialog";
import { formatWindowsPath } from "../utils/pathUtils";
import { invokeWithTimeout } from "../utils/ipc";
import { invoke } from "@tauri-apps/api/core";

export default function InboxView({ onMatch }: any) {
  const [unmatched, setUnmatched] = useState<any[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isModalOpen, setIsModalOpen] = useState(false);

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

  const performSearch = async () => {
    if (!searchQuery.trim()) return;
    setIsSearching(true);
    try {
      const res: any = await invokeWithTimeout("perform_tmdb_search", { query: searchQuery });
      setSearchResults(res);
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

  const assignShow = async (tmdbId: string, mediaType: string) => {
    if (!selectedGroup) return;
    try {
      await invokeWithTimeout("assign_unmatched_to_tracker", {
        tmdbId,
        mediaType,
        groupKey: selectedGroup
      });
      toast.success(`Successfully assigned files to tracker!`);
      setSelectedGroup(null);
      setIsModalOpen(false);
      fetchUnmatched();
      if (onMatch) onMatch();
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'")) {
        console.warn("Tauri invoke missing.");
      } else {
        toast.error("Error assigning show: " + e);
      }
    }
  };

  const openSearchModal = () => {
    setIsModalOpen(true);
    if (selectedGroup) {
      performSearch();
    }
  };

  const triggerScan = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Directory to Scan"
      });

      if (selected && typeof selected === 'string') {
        const res = await invoke("run_scan_directory", { directory: selected });
        toast.success(`Found ${res} new unmatched files.`);
        fetchUnmatched();
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
        <button
          onClick={triggerScan}
          className="flex items-center gap-2 px-6 py-3 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-lg transition-colors"
        >
          <FolderSearch className="w-5 h-5" />
          Scan Directory
        </button>
      </div>

      <p className="text-gray-400 mb-8">You have {Object.keys(grouped).length} unrecognized series on your hard drive.</p>

      {Object.keys(grouped).length === 0 ? (
        <div className="flex items-center justify-center flex-1">
           <p className="text-xl text-gray-500 font-bold">Inbox is empty. All files are matched!</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 flex-1 min-h-0">
          <div className="col-span-1 bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-6 overflow-y-auto border border-white/5">
            {Object.entries(grouped).map(([key, files]: [string, any]) => (
              <div
                key={key}
                onClick={() => setSelectedGroup(key)}
                className={`p-4 rounded-xl cursor-pointer transition-colors mb-2 ${
                  selectedGroup === key ? "bg-white/10 border border-[#FF6B00]/50" : "hover:bg-white/5"
                }`}
              >
                <h3 className="text-white font-bold truncate">{key}</h3>
                <p className="text-gray-500 text-sm mt-1">{files.length} files</p>
              </div>
            ))}
          </div>

          <div className="col-span-2 bg-[#1F222A]/60 backdrop-blur-md rounded-2xl p-8 border border-white/5 flex flex-col overflow-y-auto">
            {!selectedGroup ? (
              <div className="flex flex-col items-center justify-center text-center h-full">
                <h2 className="text-2xl font-bold mb-4 text-white">Select a group to triage</h2>
                <p className="text-gray-400 max-w-md">
                  Groups are generated automatically from filenames. Select one to assign it to a TMDB show.
                </p>
              </div>
            ) : (
              <div className="flex flex-col h-full">
                <div className="flex justify-between items-center mb-6">
                  <div>
                    <h2 className="text-2xl font-bold text-white mb-1">Group: {selectedGroup}</h2>
                    <p className="text-gray-400 text-sm">{grouped[selectedGroup]?.length || 0} files</p>
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
                className="p-2 text-gray-400 hover:text-white hover:bg-white/10 rounded-full transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="p-6 border-b border-white/5 bg-black/20">
              <div className="flex gap-4">
                <input
                  type="text"
                  placeholder="Search TMDB..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
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
                <div className="text-center text-gray-500 p-12">
                  No results found for "{searchQuery}". Try a different search term.
                </div>
              ) : (
                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
                  {searchResults.map((item, i) => (
                    <div key={i} className="bg-[#1F222A] rounded-xl overflow-hidden group border border-white/5 shadow-xl">
                      <div className="aspect-[2/3] relative">
                        {item.poster_path ? (
                          <img
                            src={`https://image.tmdb.org/t/p/w500${item.poster_path}`}
                            alt={item.title}
                            className="w-full h-full object-cover"
                          />
                        ) : (
                          <div className="w-full h-full flex items-center justify-center bg-black/40 text-gray-500 font-bold p-4 text-center">
                            {item.title}
                          </div>
                        )}
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
                          {item.type} • <span className="text-gray-500">{item.release_date?.substring(0, 4) || "Unknown"}</span>
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
