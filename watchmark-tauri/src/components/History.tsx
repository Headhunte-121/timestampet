import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function History() {
  const [history, setHistory] = useState<any[]>([]);

  useEffect(() => {
    invoke("fetch_history").then((res: any) => setHistory(res)).catch(console.error);
  }, []);

  return (
    <div className="p-12 pt-24">
      <h1 className="text-4xl font-extrabold tracking-tight mb-8">Watch History</h1>

      {history.length === 0 ? (
        <p className="text-gray-500">No history recorded yet.</p>
      ) : (
        <div className="space-y-6 max-w-4xl pb-24">
          {history.map((entry) => (
            <div key={entry.hist_id} className="flex bg-[#1F222A]/60 backdrop-blur-md p-4 rounded-xl border border-white/5 items-center gap-6">
               <img
                src={`https://image.tmdb.org/t/p/w200${entry.poster_path}`}
                alt={entry.show_title}
                className="w-16 h-24 object-cover rounded-md shadow-md"
              />
              <div className="flex-1">
                <h3 className="text-lg font-bold text-white">{entry.show_title}</h3>
                <p className="text-gray-400">
                  {entry.media_type === "TV" ? `Season ${entry.season_num} Episode ${entry.ep_num} - ${entry.ep_title}` : entry.ep_title}
                </p>
                <div className="mt-2 flex gap-4 text-sm font-medium">
                  <span className="text-[#FF6B00]">{new Date(entry.timestamp + "Z").toLocaleString()}</span>
                  {entry.is_legacy === 1 && <span className="bg-white/10 text-gray-300 px-2 rounded-md">Legacy Import</span>}
                </div>
              </div>
              {entry.completion_ratio < 0.90 && (
                <div className="text-right text-sm text-gray-400">
                  Paused ({Math.round(entry.completion_ratio * 100)}%)
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
