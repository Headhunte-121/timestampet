import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FolderSearch } from "lucide-react";

export default function InboxView({ onMatch: _onMatch }: any) {
  const [unmatched, setUnmatched] = useState<any[]>([]);

  useEffect(() => {
    fetchUnmatched();
  }, []);

  const fetchUnmatched = async () => {
    try {
      const res: any = await invoke("fetch_unmatched_files");
      setUnmatched(res);
    } catch (e) {
      console.error(e);
    }
  };

  const grouped = unmatched.reduce((acc: any, curr: any) => {
    if (!acc[curr.group_key]) acc[curr.group_key] = [];
    acc[curr.group_key].push(curr);
    return acc;
  }, {});

  const triggerScan = async () => {
    // We would need the Rust File Dialog wrapper for this, using mocked path for demo or alert
    const dir = prompt("Enter full path to directory to scan:");
    if (dir) {
      try {
        const res = await invoke("run_scan_directory", { directory: dir });
        alert(`Found ${res} new unmatched files.`);
        fetchUnmatched();
      } catch (e) {
        alert("Scan Error: " + e);
      }
    }
  };

  return (
    <div className="p-12 pb-24 h-full flex flex-col">
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
          <div className="col-span-1 bg-[#1F222A] rounded-2xl p-6 overflow-y-auto border border-white/5">
            {Object.entries(grouped).map(([key, files]: [string, any]) => (
              <div key={key} className="p-4 hover:bg-white/5 rounded-xl cursor-pointer transition-colors mb-2">
                <h3 className="text-white font-bold truncate">{key}</h3>
                <p className="text-gray-500 text-sm mt-1">{files.length} files</p>
              </div>
            ))}
          </div>

          <div className="col-span-2 bg-[#1F222A]/50 rounded-2xl p-8 border border-white/5 flex flex-col items-center justify-center text-center">
            <h2 className="text-2xl font-bold mb-4 text-white">Select a group to triage</h2>
            <p className="text-gray-400 max-w-md">
              Groups are generated automatically from filenames. Select one to assign it to a TMDB show.
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
