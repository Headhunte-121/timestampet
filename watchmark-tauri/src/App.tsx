import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { LayoutDashboard, Tv, Film, Search, Inbox, Clock, Settings } from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import Dashboard from "./components/Dashboard";
import Library from "./components/Library";
import SearchTMDB from "./components/SearchTMDB";
import History from "./components/History";
import InboxView from "./components/InboxView";
import SettingsView from "./components/SettingsView";
import MediaDetails from "./components/MediaDetails";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type View = "dashboard" | "tv" | "movies" | "search" | "inbox" | "history" | "settings";

function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const [selectedMediaId, setSelectedMediaId] = useState<number | null>(null);
  const [globalSearchQuery, setGlobalSearchQuery] = useState("");

  // Refresh UI hook
  const [refreshTrigger, setRefreshTrigger] = useState(0);

  useEffect(() => {
    const unlisten = listen("vlc-closed", () => {
      setRefreshTrigger(prev => prev + 1);
    });
    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const navItems = [
    { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
    { id: "tv", label: "TV Shows", icon: Tv },
    { id: "movies", label: "Movies", icon: Film },
    { id: "search", label: "Search", icon: Search },
    { id: "inbox", label: "Inbox", icon: Inbox },
    { id: "history", label: "History", icon: Clock },
  ] as const;

  const handleNav = (view: View) => {
    setCurrentView(view);
    setSelectedMediaId(null);
  };

  return (
    <div className="flex h-screen w-full bg-[#0D0F14] text-white overflow-hidden selection:bg-orange-500/30">
      {/* Glassy Sidebar */}
      <aside className="w-64 flex flex-col bg-black/40 backdrop-blur-xl border-r border-white/5 z-50">
        <div className="p-6">
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <span className="text-[#FF6B00]">▶</span> WatchMark
          </h1>
        </div>

        <nav className="flex-1 px-4 space-y-1">
          {navItems.map((item) => (
            <button
              key={item.id}
              onClick={() => handleNav(item.id)}
              className={cn(
                "w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200 relative overflow-hidden",
                currentView === item.id && !selectedMediaId
                  ? "text-white bg-gradient-to-r from-white/5 to-transparent"
                  : "text-gray-400 hover:text-white hover:bg-white/5"
              )}
            >
              {currentView === item.id && !selectedMediaId && (
                <div className="absolute left-0 top-0 bottom-0 w-1 bg-[#FF6B00] shadow-[0_0_10px_#FF6B00]" />
              )}
              <item.icon className={cn("w-5 h-5", currentView === item.id && !selectedMediaId && "text-[#FF6B00]")} />
              {item.label}
            </button>
          ))}
        </nav>

        <div className="p-4 mt-auto">
          <button
            onClick={() => handleNav("settings")}
            className={cn(
              "w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200 relative overflow-hidden",
              currentView === "settings" && !selectedMediaId
                ? "text-white bg-gradient-to-r from-white/5 to-transparent"
                : "text-gray-400 hover:text-white hover:bg-white/5"
            )}
          >
            {currentView === "settings" && !selectedMediaId && (
              <div className="absolute left-0 top-0 bottom-0 w-1 bg-[#FF6B00] shadow-[0_0_10px_#FF6B00]" />
            )}
            <Settings className={cn("w-5 h-5", currentView === "settings" && !selectedMediaId && "text-[#FF6B00]")} />
            Settings
          </button>
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 relative overflow-y-auto overflow-x-hidden">
        {/* Global Search Bar */}
        <div className="absolute top-6 left-1/2 -translate-x-1/2 z-50 w-full max-w-xl px-4 pointer-events-none">
          <div className="relative pointer-events-auto">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 w-4 h-4" />
            <input
              type="text"
              placeholder="Quick Search..."
              value={globalSearchQuery}
              onChange={(e) => setGlobalSearchQuery(e.target.value)}
              className="w-full bg-[#1F222A]/80 backdrop-blur-xl text-white pl-12 pr-6 py-3 rounded-full border border-white/5 focus:outline-none focus:border-[#FF6B00]/50 transition-colors shadow-2xl text-sm font-medium"
            />
          </div>
        </div>

        {selectedMediaId ? (
          <MediaDetails
            mediaId={selectedMediaId}
            onBack={() => setSelectedMediaId(null)}
            refreshTrigger={refreshTrigger}
          />
        ) : (
          <div className="h-full w-full animate-in fade-in duration-300">
            {currentView === "dashboard" && <Dashboard onMediaSelect={setSelectedMediaId} refreshTrigger={refreshTrigger} searchQuery={globalSearchQuery} />}
            {currentView === "tv" && <Library type="TV" onMediaSelect={setSelectedMediaId} refreshTrigger={refreshTrigger} searchQuery={globalSearchQuery} />}
            {currentView === "movies" && <Library type="Movie" onMediaSelect={setSelectedMediaId} refreshTrigger={refreshTrigger} searchQuery={globalSearchQuery} />}
            {currentView === "search" && <SearchTMDB onMediaSelect={setSelectedMediaId} />}
            {currentView === "inbox" && <InboxView onMatch={() => setRefreshTrigger(prev => prev + 1)} />}
            {currentView === "history" && <History />}
            {currentView === "settings" && <SettingsView />}
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
