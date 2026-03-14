import { useState } from "react";
import { Search } from "lucide-react";
import { useAsyncInvoke } from "../hooks/useAsyncInvoke";
import { motion } from "framer-motion";
import { toast } from "sonner";

export default function SearchTMDB({ onMediaSelect: _onMediaSelect }: any) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const asyncInvoke = useAsyncInvoke();

  const performSearch = async () => {
    if (!query.trim()) return;
    setLoading(true);
    try {
      const res: any = await asyncInvoke("perform_tmdb_search", { query });
      if (res) setResults(res);
    } catch (e) {
      toast.error("Search failed: " + e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-12 pt-24">
      <h1 className="text-4xl font-extrabold tracking-tight mb-8">Discover Media</h1>

      <div className="flex gap-4 mb-12 max-w-2xl">
        <div className="relative flex-1">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 w-5 h-5" />
          <input
            type="text"
            placeholder="Search TMDB for Shows or Movies..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && performSearch()}
            className="w-full bg-[#1F222A] text-white pl-12 pr-6 py-4 rounded-full border border-white/5 focus:outline-none focus:border-[#FF6B00] transition-colors shadow-inner text-lg font-medium"
          />
        </div>
        <button
          onClick={performSearch}
          disabled={loading}
          className="px-8 py-4 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-full transition-all hover:scale-105 shadow-lg shadow-orange-500/20 disabled:opacity-50 disabled:hover:scale-100"
        >
          {loading ? "Searching..." : "Search TMDB"}
        </button>
      </div>

      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 gap-6 pb-24">
        {results.map((item, i) => (
          <motion.div
            key={item.tmdb_id}
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: i * 0.05 }}
            className="relative aspect-[2/3] bg-[#1F222A] rounded-xl overflow-hidden group shadow-xl"
          >
            {item.poster_path ? (
              <img
                src={`https://image.tmdb.org/t/p/w500${item.poster_path}`}
                alt={item.title}
                className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center p-4 text-center text-gray-500 font-bold bg-[#15171e]">
                {item.title}
              </div>
            )}

            <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4">
              <h3 className="text-white font-bold leading-tight mb-1">{item.title}</h3>
              <p className="text-xs text-gray-300 mb-4 uppercase tracking-wider font-bold">
                {item.type} • {item.release_date ? (item.is_exact_date ? item.release_date : item.release_date.substring(0, 4)) : "Unknown"}
              </p>

              <button
                onClick={() => {
                  setLoading(true);
                  asyncInvoke("add_to_tracker", {
                    tmdbId: item.tmdb_id,
                    mediaType: item.type,
                    archive: false
                  }).then(() => {
                    toast.success("Added to Tracker!");
                    setLoading(false);
                  }).catch((e: any) => {
                    toast.error("Error: " + e);
                    setLoading(false);
                  });
                }}
                className="w-full py-2 bg-[#FF6B00] text-white font-bold rounded-lg hover:bg-[#E66000] transition-colors"
              >
                + Add to Tracker
              </button>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
