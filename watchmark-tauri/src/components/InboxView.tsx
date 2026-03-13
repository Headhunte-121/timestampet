import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FolderSearch, Search } from "lucide-react";

export default function InboxView({ onMatch }: any) {
  const [unmatched, setUnmatched] = useState<any[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [isSearching, setIsSearching] = useState(false);

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
      const res: any = await invoke("perform_tmdb_search", { query: searchQuery });
      setSearchResults(res);
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'")) {
        console.warn("Tauri invoke missing.");
      } else {
        alert("Search failed: " + e);
      }
    } finally {
      setIsSearching(false);
    }
  };

  const assignShow = async (tmdbId: string, mediaType: string) => {
    if (!selectedGroup) return;
    try {
      await invoke("assign_unmatched_to_tracker", {
        tmdbId,
        mediaType,
        groupKey: selectedGroup
      });
      alert(`Successfully assigned files to tracker!`);
      setSelectedGroup(null);
      fetchUnmatched();
      if (onMatch) onMatch();
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'")) {
        console.warn("Tauri invoke missing.");
      } else {
        alert("Error assigning show: " + e);
      }
    }
  };

  const triggerScan = async () => {
    // We would need the Rust File Dialog wrapper for this, using mocked path for demo or alert
    const dir = prompt("Enter full path to directory to scan:");
    if (dir) {
      try {
        const res = await invoke("run_scan_directory", { directory: dir });
        alert(`Found ${res} new unmatched files.`);
        fetchUnmatched();
      } catch (e: any) {
        if (e?.toString().includes("reading 'invoke'")) {
          console.warn("Tauri invoke missing.");
        } else {
          alert("Scan Error: " + e);
        }
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
                <div className="flex justify-between items-start mb-6">
                  <div>
                    <h2 className="text-2xl font-bold text-white mb-1">Assign: {selectedGroup}</h2>
                    <p className="text-gray-400 text-sm">{grouped[selectedGroup]?.length || 0} files selected</p>
                  </div>
                </div>

                <div className="flex gap-4 mb-8">
                  <div className="relative flex-1">
                    <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 w-5 h-5" />
                    <input
                      type="text"
                      placeholder="Search TMDB..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      onKeyDown={(e) => e.key === "Enter" && performSearch()}
                      className="w-full bg-[#15171e] text-white pl-12 pr-6 py-3 rounded-full border border-white/5 focus:outline-none focus:border-[#FF6B00] transition-colors"
                    />
                  </div>
                  <button
                    onClick={performSearch}
                    disabled={isSearching}
                    className="px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-full transition-colors disabled:opacity-50"
                  >
                    {isSearching ? "Searching..." : "Search"}
                  </button>
                </div>

                <div className="grid grid-cols-2 lg:grid-cols-3 gap-6">
                  {searchResults.map((item, i) => (
                    <div key={i} className="bg-[#15171e] rounded-xl overflow-hidden group border border-white/5">
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
                            className="w-full py-2 bg-[#FF6B00] text-white font-bold rounded-lg hover:bg-[#E66000] transition-colors"
                          >
                            Assign Show
                          </button>
                        </div>
                      </div>
                      <div className="p-4">
                        <h3 className="text-white font-bold truncate">{item.title}</h3>
                        <p className="text-xs text-gray-500 uppercase mt-1">
                          {item.type} • {item.release_date?.substring(0, 4) || "Unknown"}
                        </p>
                      </div>
                    </div>
                  ))}
                </div>

                {searchResults.length === 0 && !isSearching && (
                  <div className="mt-8 text-center text-gray-500">
                    Search for a show on TMDB to assign these files.
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
