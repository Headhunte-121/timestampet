import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { LayoutDashboard, Tv, Film, Search, Inbox, Clock, Settings } from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { motion, AnimatePresence, MotionConfig } from "framer-motion";
import { useAppStore } from "./store/useAppStore";
import Dashboard from "./components/Dashboard";
import Library from "./components/Library";
import SearchTMDB from "./components/SearchTMDB";
import History from "./components/History";
import InboxView from "./components/InboxView";
import SettingsView from "./components/SettingsView";
import MediaDetails from "./components/MediaDetails";
import { Toaster } from "sonner";
import { Modal } from "./components/ui/Modal";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type View = "dashboard" | "tv" | "movies" | "search" | "inbox" | "history" | "settings";

function App() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const [selectedMediaId, setSelectedMediaId] = useState<number | null>(null);
  const [globalSearchQuery, setGlobalSearchQuery] = useState("");

  // Settings Navigation Guard
  const [isSettingsDirty, setIsSettingsDirty] = useState(false);
  const [pendingNavigation, setPendingNavigation] = useState<{ view: View | null, mediaId: number | null } | null>(null);
  const [saveSettingsCallback, setSaveSettingsCallback] = useState<(() => Promise<boolean>) | null>(null);

  // Refresh UI hook
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  const { isCinemaMode, initialized, initializeSettings } = useAppStore();

  useEffect(() => {
    initializeSettings();
  }, [initializeSettings]);

  useEffect(() => {
    const unlisten = listen("vlc-closed", () => {
      setRefreshTrigger(prev => prev + 1);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  if (!initialized) {
    return null; // or a simple spinner
  }

  const navItems = [
    { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
    { id: "tv", label: "TV Shows", icon: Tv },
    { id: "movies", label: "Movies", icon: Film },
    { id: "search", label: "Search", icon: Search },
    { id: "inbox", label: "Inbox", icon: Inbox },
    { id: "history", label: "History", icon: Clock },
  ] as const;

  const handleNav = (view: View, mediaId: number | null = null) => {
    if (isSettingsDirty && currentView === "settings" && view !== "settings") {
      setPendingNavigation({ view, mediaId });
    } else {
      // Allow re-clicking the same tab to close the current item
      if (currentView === view && !selectedMediaId && !mediaId) {
          return; // Do nothing if already on the root of the tab
      }
      setCurrentView(view);
      setSelectedMediaId(mediaId);
    }
  };

  const confirmNavigation = async (save: boolean) => {
    if (save && saveSettingsCallback) {
        const success = await saveSettingsCallback();
        if (!success) return; // Keep modal open or stay on page if save fails
    }

    // We must reset the unsaved changes state in the components as well
    // but React state will reset when the view is unmounted.
    setIsSettingsDirty(false);
    if (pendingNavigation) {
        if (pendingNavigation.view) {
            setCurrentView(pendingNavigation.view);
        }
        setSelectedMediaId(pendingNavigation.mediaId);
        setPendingNavigation(null);
    }
  };

  const cancelNavigation = () => {
      setPendingNavigation(null);
  };

  return (
    <MotionConfig transition={isCinemaMode ? { type: "spring", stiffness: 300, damping: 30 } : { duration: 0 }}>
    <div className="h-screen overflow-hidden flex bg-cinema-black text-white selection:bg-brand-orange/30">
      {/* Glassy Sidebar */}
      <aside className={cn("flex-none w-64 flex flex-col bg-surface-gray/80 border-r border-white/5 z-50 transform-gpu will-change-transform motion-reduce:bg-surface-gray motion-reduce:backdrop-blur-none", isCinemaMode && "backdrop-blur-xl")}>
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
      <main className="flex-1 flex flex-col relative overflow-hidden">
        <Toaster theme="dark" position="bottom-right" richColors />
        <Modal />
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

        <div className="flex-1 overflow-y-auto relative">
          <AnimatePresence mode="wait">
            {selectedMediaId ? (
              <motion.div
                key="details"
                initial={isCinemaMode ? { opacity: 0, scale: 0.98 } : { opacity: 1 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={isCinemaMode ? { opacity: 0, scale: 0.98 } : { opacity: 0 }}
                transition={isCinemaMode ? { duration: 0.2 } : { duration: 0 }}
                className="h-full w-full"
              >
                <MediaDetails
                  mediaId={selectedMediaId}
                  onBack={() => handleNav(currentView)}
                  refreshTrigger={refreshTrigger}
                />
              </motion.div>
            ) : (
              <motion.div
                key={currentView}
                initial={isCinemaMode ? { opacity: 0, y: 10 } : { opacity: 1 }}
                animate={{ opacity: 1, y: 0 }}
                exit={isCinemaMode ? { opacity: 0, y: -10 } : { opacity: 0 }}
                transition={isCinemaMode ? { duration: 0.2 } : { duration: 0 }}
                className="h-full w-full"
              >
                {currentView === "dashboard" && <Dashboard onMediaSelect={(id) => handleNav("dashboard", id)} refreshTrigger={refreshTrigger} searchQuery={globalSearchQuery} />}
                {currentView === "tv" && <Library type="TV" onMediaSelect={(id) => handleNav("tv", id)} refreshTrigger={refreshTrigger} searchQuery={globalSearchQuery} />}
                {currentView === "movies" && <Library type="Movie" onMediaSelect={(id) => handleNav("movies", id)} refreshTrigger={refreshTrigger} searchQuery={globalSearchQuery} />}
                {currentView === "search" && <SearchTMDB onMediaSelect={(id) => handleNav("search", id)} />}
                {currentView === "inbox" && <InboxView onMatch={() => setRefreshTrigger(prev => prev + 1)} />}
                {currentView === "history" && <History />}
                {currentView === "settings" && <SettingsView setIsDirty={setIsSettingsDirty} setSaveCallback={setSaveSettingsCallback} />}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </main>

      {/* Navigation Guard Modal */}
      <AnimatePresence>
          {pendingNavigation && (
              <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-sm"
              >
                  <motion.div
                      initial={{ scale: 0.95, opacity: 0 }}
                      animate={{ scale: 1, opacity: 1 }}
                      exit={{ scale: 0.95, opacity: 0 }}
                      className="bg-[#1F222A] p-8 rounded-2xl border border-white/10 max-w-md w-full shadow-2xl"
                  >
                      <h2 className="text-2xl font-bold text-white mb-2">Unsaved Changes</h2>
                      <p className="text-gray-400 mb-8">You have unsaved changes in your settings. Do you want to save them before leaving?</p>
                      <div className="flex flex-col gap-3">
                          <button
                              onClick={() => confirmNavigation(true)}
                              className="w-full py-3 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-xl transition-colors"
                          >
                              Save & Leave
                          </button>
                          <button
                              onClick={() => confirmNavigation(false)}
                              className="w-full py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-xl transition-colors"
                          >
                              Discard & Leave
                          </button>
                          <button
                              onClick={cancelNavigation}
                              className="w-full py-3 bg-transparent hover:bg-white/5 text-gray-400 hover:text-white font-bold rounded-xl transition-colors mt-2"
                          >
                              Cancel
                          </button>
                      </div>
                  </motion.div>
              </motion.div>
          )}
      </AnimatePresence>

    </div>
    </MotionConfig>
  );
}

export default App;
