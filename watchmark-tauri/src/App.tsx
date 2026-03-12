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
                "w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200",
                currentView === item.id && !selectedMediaId
                  ? "bg-white/10 text-white shadow-sm"
                  : "text-gray-400 hover:text-white hover:bg-white/5"
              )}
            >
              <item.icon className={cn("w-5 h-5", currentView === item.id && !selectedMediaId && "text-[#FF6B00]")} />
              {item.label}
            </button>
          ))}
        </nav>

        <div className="p-4 mt-auto">
          <button
            onClick={() => handleNav("settings")}
            className={cn(
              "w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200",
              currentView === "settings" && !selectedMediaId
                ? "bg-white/10 text-white"
                : "text-gray-400 hover:text-white hover:bg-white/5"
            )}
          >
            <Settings className="w-5 h-5" />
            Settings
          </button>
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 relative overflow-y-auto overflow-x-hidden">
        {selectedMediaId ? (
          <MediaDetails
            mediaId={selectedMediaId}
            onBack={() => setSelectedMediaId(null)}
            refreshTrigger={refreshTrigger}
          />
        ) : (
          <div className="h-full w-full animate-in fade-in duration-300">
            {currentView === "dashboard" && <Dashboard onMediaSelect={setSelectedMediaId} refreshTrigger={refreshTrigger} />}
            {currentView === "tv" && <Library type="TV" onMediaSelect={setSelectedMediaId} refreshTrigger={refreshTrigger} />}
            {currentView === "movies" && <Library type="Movie" onMediaSelect={setSelectedMediaId} refreshTrigger={refreshTrigger} />}
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
