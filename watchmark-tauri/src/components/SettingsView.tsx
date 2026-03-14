import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "../store/useAppStore";
import { useTaskStore } from "../store/useTaskStore";
import { toast } from "sonner";
import { open, save } from "@tauri-apps/plugin-dialog";
import { formatWindowsPath } from "../utils/pathUtils";
import { invokeWithTimeout } from "../utils/ipc";
import { AnimatePresence, motion } from "framer-motion";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { Loader2, UploadCloud } from "lucide-react";
import { documentDir } from '@tauri-apps/api/path';
import { RestoreConfirmationModal } from "./ui/RestoreConfirmationModal";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

type SettingsTab = "General" | "Playback" | "Scanner" | "System" | "Database" | "Advanced";

interface SettingsViewProps {
    setIsDirty: (isDirty: boolean) => void;
    setSaveCallback: (callback: (() => Promise<boolean>) | null) => void;
}

export default function SettingsView({ setIsDirty, setSaveCallback }: SettingsViewProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>("General");

  const [initialSettings, setInitialSettings] = useState<any>(null);
  const [settings, setSettings] = useState<any>({
    vlc_path: "",
    tmdb_api_key: "",
    width: 1280,
    height: 800,
    x: 100,
    y: 100,
    cinema_mode: true,
    language: "en-US",
    auto_complete_threshold: 90,
    binge_grouping_hours: 6,
    auto_resume: true,
    auto_scan_on_boot: false,
    logging_level: "Info"
  });

  const { isCinemaMode, setCinemaMode } = useAppStore();
  const { isScanning, setScanning } = useTaskStore();
  const [scanStatus, setScanStatus] = useState<string>("");
  const [_, setBackupStatus] = useState<{ status: string, error?: string, timestamp: number } | null>(null);
  const [isBackingUp, setIsBackingUp] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [showRestoreModal, setShowRestoreModal] = useState(false);
  const [selectedRestoreFile, setSelectedRestoreFile] = useState<string | null>(null);
  const isDirty = initialSettings && JSON.stringify(settings) !== JSON.stringify(initialSettings);

  useEffect(() => {
    setIsDirty(isDirty);
  }, [isDirty, setIsDirty]);

  const saveSettings = useCallback(async (): Promise<boolean> => {
    try {
        await invoke("save_settings", { settings });
        toast.success("Settings saved successfully.");
        setInitialSettings(JSON.parse(JSON.stringify(settings)));
        setIsDirty(false);
        return true;
    } catch (e) {
        toast.error("Error saving settings: " + e);
        return false;
    }
  }, [settings, setIsDirty]);

  useEffect(() => {
      // Because `setSaveCallback` is a React state setter taking a function,
      // we need to wrap our callback inside another function so React doesn't
      // evaluate it immediately.
      setSaveCallback(() => saveSettings as any);
      return () => {
        setSaveCallback(null);
      };
  }, [saveSettings, setSaveCallback]);

  useEffect(() => {
    invoke("get_settings")
      .then((res: any) => {
          setSettings(res);
          setInitialSettings(JSON.parse(JSON.stringify(res)));
      })
      .catch(console.error);

    const unlistenScan = listen("scan-match-batch", (event: any) => {
      const batch = event.payload.files;
      if (batch && batch.length > 0) {
        setScanStatus(`Scanned ${batch.length} files in latest batch...`);
      }
    });

    const unlistenBackup = listen("backup-finished", (event: any) => {
      const payload = event.payload;
      setBackupStatus(payload);
      if (payload.status === "success") {
          setSettings((prev: any) => ({ ...prev, last_backup_timestamp: payload.timestamp, last_backup_status: payload.status, last_backup_error: "" }));
          setInitialSettings((prev: any) => ({ ...prev, last_backup_timestamp: payload.timestamp, last_backup_status: payload.status, last_backup_error: "" }));
      } else {
          setSettings((prev: any) => ({ ...prev, last_backup_status: payload.status, last_backup_error: payload.error }));
          setInitialSettings((prev: any) => ({ ...prev, last_backup_status: payload.status, last_backup_error: payload.error }));
      }
    });

    return () => {
      unlistenScan.then(f => f());
      unlistenBackup.then(f => f());
    };
  }, []);

  const discardChanges = () => {
      if (initialSettings) {
          setSettings(JSON.parse(JSON.stringify(initialSettings)));
          setCinemaMode(initialSettings.cinema_mode); // Revert app state too
          setIsDirty(false); // Make sure to reset dirty state
      }
  };

  const updateSetting = (key: string, value: any) => {
      setSettings((prev: any) => ({ ...prev, [key]: value }));
  };

  const runScan = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Directory to Scan"
      });
      if (selected && typeof selected === 'string') {
        setScanStatus("Scan started...");
        setScanning(true);
        // Ensure a generous timeout matching the backend (300 seconds)
        invokeWithTimeout<number>("run_scan_directory", { directory: selected }, 300000)
          .then((count: number) => {
            toast.success(`Scan complete. Found ${count} unmatched files.`);
            setScanStatus("");
            setScanning(false);
          })
          .catch((e: any) => {
            toast.error("Error during scan: " + e);
            setScanStatus("");
            setScanning(false);
          });
      }
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'") || e?.toString().includes("window.__TAURI_INTERNALS__")) {
        console.warn("Tauri invoke missing. Cannot open system file dialog in browser.");
      } else {
        toast.error("Error opening dialog: " + e);
      }
    }
  };

  const browseVlcPath = async () => {
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: "Select VLC Executable",
        filters: [{ name: 'Executable', extensions: ['exe', 'app', 'bin'] }]
      });
      if (selected && typeof selected === 'string') {
        setSettings({ ...settings, vlc_path: formatWindowsPath(selected) });
      }
    } catch (e: any) {
       if (e?.toString().includes("reading 'invoke'") || e?.toString().includes("window.__TAURI_INTERNALS__")) {
        console.warn("Tauri invoke missing. Cannot open system file dialog in browser.");
      } else {
        toast.error("Error opening dialog: " + e);
      }
    }
  };

  const tabs: SettingsTab[] = ["General", "Playback", "Scanner", "System", "Database", "Advanced"];

  const [oldRootPath, setOldRootPath] = useState("");
  const [newRootPath, setNewRootPath] = useState("");

  const handleRepairPaths = async () => {
    if (!oldRootPath || !newRootPath) {
        toast.error("Both Old Root and New Root must be provided.");
        return;
    }
    try {
        const affected: number = await invoke("repair_paths", { oldRoot: oldRootPath, newRoot: newRootPath });
        toast.success(`Path repair complete. Updated ${affected} entries.`);
        setOldRootPath("");
        setNewRootPath("");
    } catch (e: any) {
        toast.error(`Error repairing paths: ${e}`);
    }
  };

  const handleImportBackup = async () => {
      try {
          const selected = await open({
              multiple: false,
              filters: [{
                  name: 'Database Backup',
                  extensions: ['db', 'sqlite', 'bak']
              }],
              title: "Select Database Backup to Restore"
          });

          if (selected && typeof selected === 'string') {
              setSelectedRestoreFile(selected);
              setShowRestoreModal(true);
          }
      } catch (e: any) {
          toast.error("Error opening dialog: " + e);
      }
  };

  const confirmRestore = async () => {
      if (!selectedRestoreFile) return;
      setIsRestoring(true);
      try {
          // Set a local storage flag so the frontend knows on boot that a restore just finished
          localStorage.setItem('restore_success', 'true');
          await invoke("prepare_restore", { backupPath: selectedRestoreFile });
          // If successful, app will restart. If we reach here, restart didn't happen immediately but was triggered.
      } catch (e: any) {
          localStorage.removeItem('restore_success');
          toast.error("Restore failed: " + e);
          setIsRestoring(false);
          setShowRestoreModal(false);
      }
  };

  const handleBackupDB = async () => {
      setIsBackingUp(true);
      try {
          const dateString = new Date().toISOString().split('T')[0];
          const suggestedName = `WatchMark-Backup-${dateString}.db`;
          let defaultPath = "";
          try {
              defaultPath = await documentDir();
          } catch (e) {
              console.warn("Failed to get document dir:", e);
          }

          const selected = await save({
              title: "Export Database",
              defaultPath: defaultPath ? `${defaultPath}/${suggestedName}` : suggestedName,
              filters: [{ name: "SQLite Database", extensions: ["db", "sqlite"] }]
          });

          if (selected && typeof selected === 'string') {
              await invoke("export_database", { targetPath: selected });
              toast.success(`Database exported! Your media history is now safe in ${selected}`);
          }
      } catch (error: any) {
          if (error?.toString().includes("reading 'invoke'") || error?.toString().includes("window.__TAURI_INTERNALS__")) {
              console.warn("Tauri invoke missing. Cannot open system file dialog in browser.");
          } else {
              toast.error(`Export failed: ${error}`);
          }
      } finally {
          setTimeout(() => {
              setIsBackingUp(false);
          }, 3000);
      }
  };

  return (
    <div className="p-12 pb-32 h-full relative">
      <h1 className="text-4xl font-extrabold tracking-tight mb-8">Settings</h1>

      <div className="flex h-full gap-8">
        {/* Settings Sidebar */}
        <div className="w-48 flex-shrink-0 flex flex-col gap-1">
            {tabs.map((tab) => (
                <button
                    key={tab}
                    onClick={() => setActiveTab(tab)}
                    className={cn(
                        "w-full text-left px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200 relative overflow-hidden",
                        activeTab === tab
                            ? "text-white bg-[#1F222A]"
                            : "text-gray-400 hover:text-white hover:bg-white/5"
                    )}
                >
                    {activeTab === tab && (
                        <div className="absolute left-0 top-0 bottom-0 w-1 bg-[#FF6B00] shadow-[0_0_10px_#FF6B00]" />
                    )}
                    {tab}
                </button>
            ))}
        </div>

        {/* Settings Content Area */}
        <div className="flex-1 bg-[#1F222A] p-8 rounded-2xl border border-white/5 shadow-xl overflow-y-auto">
            {activeTab === "General" && (
                <div className="space-y-6">
                    {/* Setting Row */}
                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">TMDB API Key</h3>
                            <p className="text-xs text-gray-500 mt-1">Required to fetch poster art and synopsis from TMDB.</p>
                        </div>
                        <div>
                            <input
                                type="password"
                                value={settings.tmdb_api_key}
                                onChange={e => updateSetting('tmdb_api_key', e.target.value)}
                                className="w-full bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
                                placeholder="ey..."
                            />
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">VLC Executable Path</h3>
                            <p className="text-xs text-gray-500 mt-1">Absolute path to your local VLC installation.</p>
                        </div>
                        <div className="flex gap-4">
                            <input
                                type="text"
                                value={formatWindowsPath(settings.vlc_path)}
                                onChange={e => updateSetting('vlc_path', e.target.value)}
                                className="flex-1 bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
                                placeholder="C:\Program Files\VideoLAN\VLC\vlc.exe"
                            />
                            <button onClick={browseVlcPath} className="px-6 py-3 bg-white/10 hover:bg-white/20 font-bold rounded-xl transition-colors text-white">
                                Browse
                            </button>
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Cinema Mode</h3>
                            <p className="text-xs text-gray-500 mt-1">Enable high quality animations and UI effects.</p>
                        </div>
                        <div className="flex items-center h-full">
                            <label className="relative inline-flex items-center cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={isCinemaMode}
                                    onChange={(e) => {
                                        setCinemaMode(e.target.checked);
                                        updateSetting('cinema_mode', e.target.checked);
                                    }}
                                    className="sr-only peer"
                                />
                                <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#FF6B00]"></div>
                            </label>
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Language</h3>
                            <p className="text-xs text-gray-500 mt-1">Preferred metadata language.</p>
                        </div>
                        <div className="flex gap-2">
                             {['en-US', 'es-ES', 'fr-FR'].map(lang => (
                                 <button
                                     key={lang}
                                     onClick={() => updateSetting('language', lang)}
                                     className={cn(
                                         "px-4 py-2 rounded-full text-sm font-bold transition-colors",
                                         settings.language === lang ? "bg-[#FF6B00] text-white" : "bg-white/5 text-gray-400 hover:bg-white/10"
                                     )}
                                 >
                                     {lang}
                                 </button>
                             ))}
                        </div>
                    </div>
                </div>
            )}

            {activeTab === "Playback" && (
                <div className="space-y-6">
                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Auto-complete Threshold (%)</h3>
                            <p className="text-xs text-gray-500 mt-1">Percentage required to mark an episode as watched.</p>
                        </div>
                        <div>
                            <input
                                type="number"
                                min="1"
                                max="100"
                                value={settings.auto_complete_threshold}
                                onChange={e => updateSetting('auto_complete_threshold', parseInt(e.target.value) || 90)}
                                className="w-full max-w-[150px] bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
                            />
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Binge Grouping (Hours)</h3>
                            <p className="text-xs text-gray-500 mt-1">Time gap allowed before breaking a binge session block.</p>
                        </div>
                        <div>
                            <input
                                type="number"
                                min="1"
                                max="48"
                                value={settings.binge_grouping_hours}
                                onChange={e => updateSetting('binge_grouping_hours', parseInt(e.target.value) || 6)}
                                className="w-full max-w-[150px] bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
                            />
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Auto-Resume</h3>
                            <p className="text-xs text-gray-500 mt-1">Automatically resume from last paused position.</p>
                        </div>
                        <div className="flex items-center h-full">
                            <label className="relative inline-flex items-center cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={settings.auto_resume}
                                    onChange={(e) => updateSetting('auto_resume', e.target.checked)}
                                    className="sr-only peer"
                                />
                                <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#FF6B00]"></div>
                            </label>
                        </div>
                    </div>
                </div>
            )}

            {activeTab === "Scanner" && (
                <div className="space-y-6">
                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Manual Scan</h3>
                            <p className="text-xs text-gray-500 mt-1">Scan a specific directory for new media.</p>
                        </div>
                        <div>
                             <button
                                onClick={runScan}
                                className="px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-xl transition-colors"
                            >
                                Run Scan
                            </button>
                            {scanStatus && <div className="text-gray-400 text-sm mt-2">{scanStatus}</div>}
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Auto-Scan on Boot</h3>
                            <p className="text-xs text-gray-500 mt-1">Automatically scan known directories when WatchMark starts.</p>
                        </div>
                        <div className="flex items-center h-full">
                            <label className="relative inline-flex items-center cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={settings.auto_scan_on_boot}
                                    onChange={(e) => updateSetting('auto_scan_on_boot', e.target.checked)}
                                    className="sr-only peer"
                                />
                                <div className="w-11 h-6 bg-gray-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[#FF6B00]"></div>
                            </label>
                        </div>
                    </div>
                </div>
            )}

            {activeTab === "System" && (
                <div className="space-y-6">
                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Logging Level</h3>
                            <p className="text-xs text-gray-500 mt-1">Detail level for rust backend logs.</p>
                        </div>
                        <div className="flex gap-2">
                             {['Debug', 'Info', 'Warn', 'Error'].map(level => (
                                 <button
                                     key={level}
                                     onClick={() => updateSetting('logging_level', level)}
                                     className={cn(
                                         "px-4 py-2 rounded-full text-sm font-bold transition-colors",
                                         settings.logging_level === level ? "bg-[#FF6B00] text-white" : "bg-white/5 text-gray-400 hover:bg-white/10"
                                     )}
                                 >
                                     {level}
                                 </button>
                             ))}
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Database Maintenance</h3>
                            <p className="text-xs text-gray-500 mt-1">Optimize and backup SQLite database.</p>
                        </div>
                        <div>
                            <div className="flex gap-4">
                                <motion.button
                                    whileTap={!isBackingUp && !isRestoring ? { scale: 0.98 } : undefined}
                                    onClick={handleBackupDB}
                                    disabled={isBackingUp || isRestoring}
                                    className={cn(
                                        "px-6 py-3 font-bold rounded-xl transition-all flex items-center gap-2 relative overflow-hidden",
                                        isBackingUp
                                            ? "border border-[#FF6B00] text-[#FF6B00] cursor-wait bg-transparent"
                                            : "border border-[#FF6B00] text-[#FF6B00] hover:bg-[#FF6B00] hover:text-white disabled:opacity-50 disabled:cursor-not-allowed"
                                    )}
                                >
                                    {isBackingUp && (
                                        <div className="absolute inset-0 w-full h-full bg-gradient-to-r from-transparent via-white/10 to-transparent animate-[shimmer_1.5s_infinite]" />
                                    )}
                                    {isBackingUp ? (
                                        <>
                                            <Loader2 className="w-4 h-4 animate-spin text-[#FF6B00]" />
                                            Backing Up...
                                        </>
                                    ) : (
                                        "Backup DB"
                                    )}
                                </motion.button>

                                <div className="relative group">
                                    <motion.button
                                        whileTap={!isScanning && !isBackingUp && !isRestoring ? { scale: 0.98 } : undefined}
                                        onClick={handleImportBackup}
                                        disabled={isScanning || isBackingUp || isRestoring}
                                        className={cn(
                                            "px-6 py-3 font-bold rounded-xl transition-all flex items-center gap-2 relative overflow-hidden",
                                            "border border-[#b71c1c] text-[#b71c1c]",
                                            (isScanning || isBackingUp || isRestoring)
                                                ? "opacity-50 cursor-not-allowed bg-transparent"
                                                : "hover:bg-[#b71c1c] hover:text-white"
                                        )}
                                    >
                                        <UploadCloud className="w-5 h-5" />
                                        Import Backup
                                    </motion.button>
                                    {isScanning && (
                                        <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-1.5 bg-black text-white text-xs rounded shadow-lg whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10">
                                            Cannot import while scanning
                                        </div>
                                    )}
                                </div>

                                <button
                                    onClick={() => toast.success("Database vacuum completed.")}
                                    className="px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-xl transition-colors disabled:opacity-50"
                                    disabled={isBackingUp || isRestoring}
                                >
                                    Vacuum DB
                                </button>
                            </div>
                            <div className="mt-3 text-sm font-medium">
                                {settings.last_backup_timestamp === 0 && settings.last_backup_status !== "error" && (
                                    <span className="text-gray-400">Backup status: Pending first run</span>
                                )}
                                {settings.last_backup_timestamp > 0 && settings.last_backup_status !== "error" && (
                                    <span className="text-gray-400">Last automated backup: {new Date(settings.last_backup_timestamp * 1000).toLocaleString('en-US', { month: 'long', day: 'numeric', year: 'numeric', hour: 'numeric', minute: '2-digit', hour12: true })}</span>
                                )}
                                {settings.last_backup_status === "error" && (
                                    <span className="text-[#EF4444] flex items-center gap-1">
                                        <span className="font-bold">(!)</span>
                                        {settings.last_backup_timestamp > 0 ?
                                            `Last successful: ${new Date(settings.last_backup_timestamp * 1000).toLocaleString('en-US', { month: 'long', day: 'numeric', year: 'numeric', hour: 'numeric', minute: '2-digit', hour12: true })} | `
                                            : ""}
                                        Backup failed. Retrying soon.
                                    </span>
                                )}
                            </div>
                        </div>
                    </div>

                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-[#EF4444]">Danger Zone</h3>
                            <p className="text-xs text-gray-500 mt-1">Irreversible destructive actions.</p>
                        </div>
                        <div>
                             <button
                                onClick={() => toast.error("Factory reset is not implemented yet.")}
                                className="px-6 py-3 border border-[#EF4444] text-[#EF4444] hover:bg-[#EF4444] hover:text-white font-bold rounded-xl transition-colors"
                            >
                                Factory Reset
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {activeTab === "Database" && (
                <div className="space-y-6">
                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Repair Paths</h3>
                            <p className="text-xs text-gray-500 mt-1">If a drive letter shifts (e.g., D:\ becomes E:\), you can quickly migrate paths here.</p>
                        </div>
                        <div className="flex flex-col gap-4">
                            <input
                                type="text"
                                value={oldRootPath}
                                onChange={e => setOldRootPath(e.target.value)}
                                className="w-full bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
                                placeholder="Old Root (e.g., D:\)"
                            />
                            <input
                                type="text"
                                value={newRootPath}
                                onChange={e => setNewRootPath(e.target.value)}
                                className="w-full bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
                                placeholder="New Root (e.g., E:\)"
                            />
                            <button
                                onClick={handleRepairPaths}
                                className="px-6 py-3 bg-[#FF6B00] hover:bg-[#E66000] font-bold rounded-xl transition-colors text-white mt-2 w-max"
                            >
                                Repair Paths
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {activeTab === "Advanced" && (
                <div className="flex items-center justify-center h-full text-gray-500 flex-col gap-4">
                    <span className="text-6xl">🚧</span>
                    <p className="font-bold">Advanced Settings Coming Soon</p>
                </div>
            )}
        </div>
      </div>

      <RestoreConfirmationModal
        isOpen={showRestoreModal}
        onConfirm={confirmRestore}
        onCancel={() => { setShowRestoreModal(false); setSelectedRestoreFile(null); }}
        isRestoring={isRestoring}
      />

      {/* Floating Action Bar */}
      <AnimatePresence>
          {isDirty && (
              <motion.div
                  initial={{ y: 100, opacity: 0 }}
                  animate={{ y: 0, opacity: 1 }}
                  exit={{ y: 100, opacity: 0 }}
                  transition={{ type: "spring", stiffness: 300, damping: 30 }}
                  className="fixed bottom-0 left-64 right-0 p-6 z-[100] pointer-events-none"
              >
                  <div className="bg-[#1F222A]/90 backdrop-blur-xl border border-white/10 shadow-2xl rounded-2xl p-4 flex justify-between items-center max-w-4xl mx-auto pointer-events-auto">
                      <div className="text-white font-bold px-4">Unsaved Changes</div>
                      <div className="flex gap-4">
                           <button
                              onClick={discardChanges}
                              className="px-8 py-3 bg-transparent border border-white/20 hover:bg-white/5 text-white font-bold rounded-xl transition-colors"
                          >
                              Discard
                          </button>
                          <button
                              onClick={saveSettings}
                              className="px-8 py-3 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-xl shadow-[0_0_15px_rgba(255,107,0,0.5)] transition-colors"
                          >
                              Save
                          </button>
                      </div>
                  </div>
              </motion.div>
          )}
      </AnimatePresence>

    </div>
  );
}
