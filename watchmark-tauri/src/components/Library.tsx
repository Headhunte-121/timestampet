import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";
import { Star } from "lucide-react";

export default function Library({ type, onMediaSelect, refreshTrigger, searchQuery = "" }: any) {
  const [data, setData] = useState<any[]>([]);
  const [sortBy, setSortBy] = useState("Recently Added");
  const [hideCompleted, setHideCompleted] = useState(false);

  useEffect(() => {
    invoke("get_library_data", { mediaType: type, sortBy, hideCompleted })
      .then((res: any) => setData(res))
      .catch(console.error);
  }, [type, sortBy, hideCompleted, refreshTrigger]);

  const sortOptions = ["Recently Added", "Sort by Last Watched", "Alphabetical (A-Z)", "Release Year", "My Top Rated"];

  const filteredData = data.filter(item =>
    !searchQuery ||
    item.title?.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-12 pb-24 pt-24">
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center mb-10 gap-6">
        <h1 className="text-4xl font-extrabold tracking-tight">
          {type === "TV" ? "TV Shows" : "Movies"}
        </h1>

        <div className="flex items-center gap-4 bg-[#1F222A]/80 backdrop-blur-md p-2 rounded-xl border border-white/5">
          <select
            value={sortBy}
            onChange={e => setSortBy(e.target.value)}
            className="bg-transparent text-white outline-none font-medium px-2 py-1"
          >
            {sortOptions.map(opt => <option key={opt} value={opt} className="bg-[#1F222A]">{opt}</option>)}
          </select>

          <div className="w-px h-6 bg-white/10" />

          <label className="flex items-center gap-2 cursor-pointer px-2 text-sm font-medium text-gray-300 hover:text-white transition-colors">
            <input
              type="checkbox"
              checked={hideCompleted}
              onChange={e => setHideCompleted(e.target.checked)}
              className="accent-[#FF6B00] w-4 h-4 rounded focus:ring-[#FF6B00]"
            />
            Hide Completed
          </label>
        </div>
      </div>

      <div className="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] md:grid-cols-[repeat(auto-fill,minmax(160px,1fr))] lg:grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-4 md:gap-6">
        {filteredData.map((item, i) => (
          <motion.div
            key={item.id}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.05 }}
            whileHover={{ scale: 1.05, y: -5 }}
            className="relative w-full aspect-[2/3] bg-[#1F222A] rounded-xl overflow-hidden cursor-pointer group shadow-xl"
            onClick={() => onMediaSelect(item.id)}
          >
            {item.poster_path ? (
              <img
                src={`https://image.tmdb.org/t/p/w500${item.poster_path}`}
                alt={item.title}
                className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center bg-[#15171e] text-gray-500 font-bold p-4 text-center">
                {item.title}
              </div>
            )}

            {/* Dark overlay on hover */}
            <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4">
              <h3 className="text-white font-bold leading-tight mb-1">{item.title}</h3>
              <p className="text-xs text-gray-300 mb-2">
                {item.release_date?.substring(0, 4) || "Unknown"}
              </p>

              <div className="flex items-center gap-1 text-[#FF6B00] text-sm font-bold mb-1">
                 <Star className="w-4 h-4 fill-current" /> {item.user_rating > 0 ? `${item.user_rating}/5` : 'Unrated'}
              </div>

              <p className="text-xs text-gray-400 line-clamp-3">
                {item.synopsis}
              </p>
            </div>

            {/* Static badges */}
            <div className="absolute top-2 right-2 bg-black/60 backdrop-blur-md px-2 py-1 rounded-lg text-xs font-bold text-[#FF6B00] shadow-md group-hover:opacity-0 transition-opacity flex items-center gap-1">
              ★ {item.user_rating > 0 ? `${item.user_rating}/5` : 'Unrated'}
            </div>

            <div className="absolute top-2 left-2 bg-black/60 backdrop-blur-md px-2 py-1 rounded-lg text-[10px] font-bold text-gray-300 shadow-md group-hover:opacity-0 transition-opacity uppercase tracking-wider">
              {item.status}
            </div>

            {/* Progress Bar */}
            {item.total_episodes > 0 && (
              <div className="absolute bottom-0 left-0 w-full h-1.5 bg-black/80 z-20">
                <div
                  className={`h-full ${item.completed_eps === item.total_episodes ? 'bg-green-500' : 'bg-[#FF6B00]'}`}
                  style={{ width: `${Math.min(100, (item.completed_eps / item.total_episodes) * 100)}%` }}
                />
              </div>
            )}
          </motion.div>
        ))}
        {filteredData.length === 0 && (
          <div className="col-span-full py-32 text-center text-gray-500 text-lg">
            No media found in this view.
          </div>
        )}
      </div>
    </div>
  );
}
