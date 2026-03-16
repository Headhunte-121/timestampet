import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "../store/useAppStore";
import { useTaskStore } from "../store/useTaskStore";
import { toast } from "../utils/toast";
import { open, save } from "@tauri-apps/plugin-dialog";
import { formatWindowsPath } from "../utils/pathUtils";
import { invokeWithTimeout } from "../utils/ipc";
import { AnimatePresence, motion } from "framer-motion";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";
import { Loader2, UploadCloud, Sparkles, CheckCircle2, XCircle, AlertTriangle, Eye, EyeOff } from "lucide-react";
import { documentDir } from '@tauri-apps/api/path';
import { RestoreConfirmationModal } from "./ui/RestoreConfirmationModal";
import { open as openUrl } from "@tauri-apps/plugin-shell";
import { logger } from "../utils/logger";

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
  const [validationState, setValidationState] = useState<'idle' | 'loading' | 'success' | 'error' | 'ratelimit'>('idle');
  const [validationError, setValidationError] = useState<string>('');
  const [showApiKey, setShowApiKey] = useState(false);

  const [settings, setSettings] = useState<any>({
    vlc_path: "",
    tmdb_api_key: "",
    width: 1280,
    height: 800,
    x: 100,
    y: 100,
    cinema_mode: true,
    high_performance_mode: false,
    language: "en-US",
    auto_complete_threshold: 90,
    binge_grouping_hours: 6,
    auto_resume: true,
    auto_scan_on_boot: false,
    global_log_level: "info",
    module_logs: {},
    last_scanned_path: ""
  });

  const [availableModules, setAvailableModules] = useState<Record<string, string>>({});

  const { isCinemaMode, setCinemaMode } = useAppStore();
  const { isScanning, setScanning, isScanPaused, setScanPaused, isOptimizing, setOptimizing } = useTaskStore();
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
        if (validationState === 'success') {
             useAppStore.getState().setApiAuthorized(true);
        }
        return true;
    } catch (e) {
        toast.error("Error saving settings: " + e);
        return false;
    }
  }, [settings, setIsDirty, validationState]);

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

    invoke("get_available_modules")
      .then((res: any) => {
          setAvailableModules(res);
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
      if (key === 'tmdb_api_key') {
          setValidationState('idle');
          setValidationError('');
      }
  };

  useEffect(() => {
      const timer = setTimeout(() => {
          if (settings.tmdb_api_key !== '' && settings.tmdb_api_key !== initialSettings?.tmdb_api_key) {
              validateApiKey(settings.tmdb_api_key);
          } else if (settings.tmdb_api_key === '') {
              setValidationState('idle');
          }
      }, 500);

      return () => clearTimeout(timer);
  }, [settings.tmdb_api_key]);

  const validateApiKey = async (keyToValidate: string) => {
      setValidationState('loading');
      setValidationError('');
      try {
          const res: any = await invoke("validate_tmdb_key", { key: keyToValidate });

          if (res.sanitized_key !== keyToValidate) {
              setSettings((prev: any) => ({ ...prev, tmdb_api_key: res.sanitized_key }));
          }

          if (res.success) {
              setValidationState('success');
              toast.success("API Key Valid!");
          } else {
              if (res.error_msg && res.error_msg.startsWith("RATE_LIMIT:")) {
                  const retryAfter = parseInt(res.error_msg.split(":")[1]) || 1;
                  setValidationState('ratelimit');
                  setValidationError(`Key is valid, but TMDB is busy. Retrying in ${retryAfter} seconds.`);
                  toast.error(`TMDB is busy. Retrying in ${retryAfter} seconds.`);
                  setTimeout(() => {
                      if (settings.tmdb_api_key === res.sanitized_key) {
                          validateApiKey(res.sanitized_key);
                      }
                  }, retryAfter * 1000);
              } else {
                  setValidationState('error');
                  const errMsg = res.error_msg || "Invalid API Key";
                  setValidationError(errMsg);
                  toast.error(errMsg);
              }
          }
      } catch (e: any) {
          setValidationState('error');
          const errMsg = e.toString();
          setValidationError(errMsg);
          toast.error(errMsg);
      }
  };

  const runScan = async () => {
    try {
      let defaultPath;
      if (settings?.last_scanned_path) {
          defaultPath = settings.last_scanned_path;
      } else {
          try {
              const { videoDir } = await import('@tauri-apps/api/path');
              defaultPath = await videoDir();
          } catch (e) {
              console.warn("Could not get video dir:", e);
          }
      }

      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Directory to Scan",
        defaultPath
      });
      if (!selected) return;

      if (typeof selected === 'string') {
        logger.click("the 'Run Scan' button");
        setScanStatus("Scan started...");
        setScanning(true);
        // Ensure a generous timeout matching the backend (300 seconds)
        invokeWithTimeout<any>("run_scan_directory", { directory: selected }, 300000)
          .then((res: any) => {
            logger.ipcSuccess(`Scan complete. Found ${res.unmatched_count} unmatched files.`);
            toast.scanComplete(res);
            setSettings({ ...settings, last_scanned_path: selected });
            setScanStatus("");
            setScanning(false);
          })
          .catch((e: any) => {
            logger.error("Scan Failed", e);
            if (e?.type === "AccessDenied" || e?.code === "ACCESS_DENIED") {
              toast.error(`Access Denied: WatchMark lacks permissions for ${e.path}`);
            } else {
              toast.error("Error during scan: " + e);
            }
            setScanStatus("");
            setScanning(false);
          });
      }
    } catch (e: any) {
      if (e?.toString().includes("reading 'invoke'") || e?.toString().includes("window.__TAURI_INTERNALS__")) {
        console.warn("Tauri invoke missing. Cannot open system file dialog in browser.");
      } else {
        logger.error("Scan Dialog Failed", e);
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
    logger.click("'Repair Paths' button");
    try {
        const affected: number = await invoke("repair_paths", { oldRoot: oldRootPath, newRoot: newRootPath });
        logger.ipcSuccess(`Path repair complete. Updated ${affected} entries.`);
        toast.success(`Path repair complete. Updated ${affected} entries.`);
        setOldRootPath("");
        setNewRootPath("");
    } catch (e: any) {
        logger.error("Path Repair Failed", e);
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
      logger.click("'Confirm Restore' button");
      setIsRestoring(true);
      try {
          // Set a local storage flag so the frontend knows on boot that a restore just finished
          localStorage.setItem('restore_success', 'true');
          await invoke("prepare_restore", { backupPath: selectedRestoreFile });
          // If successful, app will restart. If we reach here, restart didn't happen immediately but was triggered.
      } catch (e: any) {
          localStorage.removeItem('restore_success');
          logger.error("Database Restore Failed", e);
          toast.error("Restore failed: " + e);
          setIsRestoring(false);
          setShowRestoreModal(false);
      }
  };

  const handleBackupDB = async () => {
      logger.click("'Backup DB' button");
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
              logger.ipcSuccess(`Database exported to ${selected}`);
              toast.success(`Database exported! Your media history is now safe in ${selected}`);
          }
      } catch (error: any) {
          if (error?.toString().includes("reading 'invoke'") || error?.toString().includes("window.__TAURI_INTERNALS__")) {
              console.warn("Tauri invoke missing. Cannot open system file dialog in browser.");
          } else {
              logger.error("Database Export Failed", error);
              toast.error(`Export failed: ${error}`);
          }
      } finally {
          setTimeout(() => {
              setIsBackingUp(false);
          }, 3000);
      }
  };

  const formatBytes = (bytes: number): string => {
      if (bytes === 0) return '0 Bytes';
      const k = 1024;
      const sizes = ['Bytes', 'KB', 'MB', 'GB'];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const handleOptimizeDB = async () => {
      logger.click("'Clean Database' button");
      setOptimizing(true);
      try {
          const [savedBytes, percentage]: [number, number] = await invoke("optimize_database");
          if (savedBytes > 0) {
             logger.ipcSuccess(`Database optimized. Reclaimed ${formatBytes(savedBytes)}.`);
             toast.success(`Optimization Complete! Database is now ${percentage}% smaller. ${formatBytes(savedBytes)} reclaimed.`);
          } else {
             logger.ipcSuccess(`Database is already fully optimized.`);
             toast.success("Database is already fully optimized.");
          }
      } catch (error: any) {
          logger.error("Database Optimization Failed", error);
          toast.error(`${error}`);
      } finally {
          setOptimizing(false);
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
                            : "text-muted hover:text-white hover:bg-white/5"
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
                            <p className="text-xs text-gray-500 mt-1 mb-2">Required to fetch poster art and synopsis from TMDB.</p>
                            {!settings.tmdb_api_key && (
                                <div className="text-xs bg-[#FF6B00]/10 border border-[#FF6B00]/20 rounded-lg p-3 mt-2">
                                    <p className="text-gray-300 mb-2" title="Why do I need this? TMDB is a free service used to provide the high-quality posters and details shown in the app.">
                                        TMDB is a free service used to provide the high-quality posters and details shown in the app.
                                    </p>
                                    <button
                                        onClick={() => openUrl("https://www.themoviedb.org/settings/api").catch(console.error)}
                                        className="text-[#FF6B00] font-bold hover:underline"
                                    >
                                        Get your free API key here &rarr;
                                    </button>
                                </div>
                            )}
                        </div>
                        <div>
                            <div className="relative">
                                <input
                                    type={showApiKey ? "text" : "password"}
                                    value={settings.tmdb_api_key}
                                    onChange={e => updateSetting('tmdb_api_key', e.target.value)}
                                    className="w-full bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none pr-24"
                                    placeholder="ey..."
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowApiKey(!showApiKey)}
                                    className="absolute right-12 top-1/2 -translate-y-1/2 text-gray-500 hover:text-white transition-colors"
                                >
                                    {showApiKey ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
                                </button>
                                <div className="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none">
                                    <AnimatePresence mode="wait">
                                        {validationState === 'loading' && (
                                            <motion.div key="loading" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
                                                <Loader2 className="w-5 h-5 animate-spin text-[#FF6B00]" />
                                            </motion.div>
                                        )}
                                        {validationState === 'success' && (
                                            <motion.div key="success" initial={{ scale: 0 }} animate={{ scale: 1 }} exit={{ scale: 0 }}>
                                                <CheckCircle2 className="w-5 h-5 text-green-500" />
                                            </motion.div>
                                        )}
                                        {validationState === 'error' && (
                                            <motion.div key="error" initial={{ scale: 0 }} animate={{ scale: 1 }} exit={{ scale: 0 }}>
                                                <XCircle className="w-5 h-5 text-red-500" />
                                            </motion.div>
                                        )}
                                        {validationState === 'ratelimit' && (
                                            <motion.div key="ratelimit" initial={{ scale: 0 }} animate={{ scale: 1 }} exit={{ scale: 0 }}>
                                                <AlertTriangle className="w-5 h-5 text-yellow-500" />
                                            </motion.div>
                                        )}
                                    </AnimatePresence>
                                </div>
                            </div>
                            <div className="mt-4 flex items-center justify-between">
                                {validationError ? (
                                    <p className={cn("text-xs", validationState === 'ratelimit' ? "text-yellow-500" : "text-red-500")}>
                                        {validationError}
                                    </p>
                                ) : (
                                    <span />
                                )}
                                <button
                                    type="button"
                                    onClick={() => validateApiKey(settings.tmdb_api_key)}
                                    disabled={validationState === 'loading' || !settings.tmdb_api_key}
                                    className="px-4 py-2 bg-white/10 hover:bg-white/20 font-bold rounded-lg transition-colors text-white text-sm disabled:opacity-50 flex items-center gap-2"
                                >
                                    {validationState === 'loading' ? (
                                        <>
                                            <Loader2 className="w-4 h-4 animate-spin text-[#FF6B00]" />
                                            Pinging TMDB...
                                        </>
                                    ) : (
                                        "Test Connection"
                                    )}
                                </button>
                            </div>
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
                            <h3 className="text-sm font-bold text-white">High Performance Mode</h3>
                            <p className="text-xs text-gray-500 mt-1">Reduces initial load times by fetching smaller image sizes (e.g., w342 instead of w500).</p>
                        </div>
                        <div className="flex items-center h-full">
                            <label className="relative inline-flex items-center cursor-pointer">
                                <input
                                    type="checkbox"
                                    checked={settings.high_performance_mode}
                                    onChange={(e) => updateSetting('high_performance_mode', e.target.checked)}
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
                                         settings.language === lang ? "bg-[#FF6B00] text-white" : "bg-white/5 text-muted hover:bg-white/10"
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
                             {isScanning ? (
                                <div className="flex items-center gap-2">
                                    <button
                                        onClick={() => {
                                            import('@tauri-apps/api/core').then(({ invoke }) => {
                                                if (isScanPaused) {
                                                    invoke('resume_active_scan').then(() => setScanPaused(false)).catch(console.error);
                                                } else {
                                                    invoke('pause_active_scan').then(() => setScanPaused(true)).catch(console.error);
                                                }
                                            });
                                        }}
                                        className="px-6 py-3 bg-orange-600/80 hover:bg-orange-500 text-white font-bold rounded-xl transition-colors flex items-center gap-2"
                                    >
                                        {isScanPaused ? "Resume Scan" : "Pause Scan"}
                                    </button>
                                    <button
                                        onClick={() => {
                                            import('@tauri-apps/api/core').then(({ invoke }) => {
                                                invoke('cancel_active_scan').catch(console.error);
                                            });
                                        }}
                                        className="px-6 py-3 bg-red-600/80 hover:bg-red-500 text-white font-bold rounded-xl transition-colors flex items-center gap-2"
                                    >
                                        Cancel Scan
                                    </button>
                                </div>
                             ) : (
                                <button
                                    onClick={runScan}
                                    className="px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-bold rounded-xl transition-colors relative"
                                >
                                    Run Scan
                                </button>
                             )}
                            {scanStatus && <div className="text-muted text-sm mt-2">{scanStatus}</div>}
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
                            <h3 className="text-sm font-bold text-white">Global Logging Level</h3>
                            <p className="text-xs text-gray-500 mt-1">Detail level for rust backend logs.</p>
                        </div>
                        <div className="flex flex-col gap-4">
                            <div className="flex gap-2">
                                 {['debug', 'info', 'warn', 'error'].map(level => (
                                     <button
                                         key={level}
                                         onClick={() => {
                                             updateSetting('global_log_level', level);

                                             // If we change global level, we also apply it to all modules that aren't specifically set differently,
                                             // or let the user choose. For now, we'll just set global level and update state.
                                             // Let's reset all module levels to the global level for ease of use.
                                             const newModules = { ...settings.module_logs };
                                             Object.keys(availableModules).forEach(mod => {
                                                 newModules[mod] = level;
                                             });
                                             updateSetting('module_logs', newModules);
                                         }}
                                         className={cn(
                                             "px-4 py-2 rounded-full text-sm font-bold transition-colors capitalize",
                                             settings.global_log_level === level ? "bg-[#FF6B00] text-white" : "bg-white/5 text-muted hover:bg-white/10"
                                         )}
                                     >
                                         {level}
                                     </button>
                                 ))}
                            </div>
                            <p className="text-xs text-gray-500">Changes apply immediately without restart. See Advanced tab for per-file logging.</p>
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
                                        whileTap={!isScanning && !isBackingUp && !isRestoring && !isOptimizing ? { scale: 0.98 } : undefined}
                                        onClick={handleImportBackup}
                                        disabled={isScanning || isBackingUp || isRestoring || isOptimizing}
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

                                <div className="relative group">
                                    <motion.button
                                        whileTap={!isBackingUp && !isRestoring && !isOptimizing ? { scale: 0.98 } : undefined}
                                        onClick={handleOptimizeDB}
                                        disabled={isBackingUp || isRestoring || isOptimizing}
                                        className={cn(
                                            "px-6 py-3 font-bold rounded-xl transition-all flex items-center gap-2",
                                            "border border-gray-600 text-gray-300 hover:border-[#FF6B00] hover:text-[#FF6B00]",
                                            (isBackingUp || isRestoring || isOptimizing) ? "opacity-50 cursor-not-allowed" : ""
                                        )}
                                    >
                                        {isOptimizing ? (
                                            <>
                                                <Loader2 className="w-5 h-5 animate-spin text-[#FF6B00]" />
                                                Cleaning...
                                            </>
                                        ) : (
                                            <>
                                                <Sparkles className="w-5 h-5" />
                                                Clean Database
                                            </>
                                        )}
                                    </motion.button>
                                </div>
                            </div>
                            <div className="mt-3 text-sm font-medium">
                                {settings.last_backup_timestamp === 0 && settings.last_backup_status !== "error" && (
                                    <span className="text-muted">Backup status: Pending first run</span>
                                )}
                                {settings.last_backup_timestamp > 0 && settings.last_backup_status !== "error" && (
                                    <span className="text-muted">Last automated backup: {new Date(settings.last_backup_timestamp * 1000).toLocaleString('en-US', { month: 'long', day: 'numeric', year: 'numeric', hour: 'numeric', minute: '2-digit', hour12: true })}</span>
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
                <div className="space-y-6">
                    <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                        <div>
                            <h3 className="text-sm font-bold text-white">Advanced Logging Control</h3>
                            <p className="text-xs text-gray-500 mt-1">
                                Adjust logging levels for individual system modules to surgically debug issues without flooding the console.
                            </p>
                        </div>
                        <div className="flex flex-col gap-6">
                            {Object.entries(
                                Object.keys(availableModules).reduce((acc: any, key) => {
                                    const parts = key.split('::');
                                    const group = parts.length > 1 ? parts[0] : 'core';
                                    const name = parts.length > 1 ? parts.slice(1).join('::') : key;

                                    if (!acc[group]) acc[group] = [];
                                    acc[group].push({ fullKey: key, displayName: name });
                                    return acc;
                                }, {})
                            ).map(([groupName, modules]: any) => (
                                <div key={groupName} className="bg-black/30 rounded-xl p-4 border border-white/5">
                                    <h4 className="text-xs font-bold text-muted mb-4 uppercase tracking-wider border-b border-white/10 pb-2">
                                        {groupName.replace(/_/g, ' ')}
                                    </h4>
                                    <div className="flex flex-col gap-3">
                                        {modules.map((mod: any) => {
                                            const currentLevel = settings.module_logs[mod.fullKey] || settings.global_log_level || 'info';
                                            return (
                                                <div key={mod.fullKey} className="flex justify-between items-center bg-white/5 p-3 rounded-lg">
                                                    <div className="text-sm font-medium text-white">{mod.displayName}.rs</div>
                                                    <div className="flex bg-black/50 rounded-lg p-1">
                                                        {['off', 'error', 'warn', 'info', 'debug'].map(level => {
                                                            const isActive = currentLevel === level;
                                                            return (
                                                                <button
                                                                    key={level}
                                                                    onClick={() => {
                                                                        const newModules = { ...settings.module_logs, [mod.fullKey]: level };
                                                                        updateSetting('module_logs', newModules);
                                                                    }}
                                                                    className={cn(
                                                                        "px-3 py-1 text-xs font-bold rounded-md transition-all capitalize",
                                                                        isActive
                                                                            ? (level === 'error' ? "bg-red-500 text-white" : level === 'warn' ? "bg-yellow-600 text-white" : level === 'off' ? "bg-gray-700 text-white" : "bg-[#FF6B00] text-white")
                                                                            : "text-muted hover:text-white hover:bg-white/10"
                                                                    )}
                                                                >
                                                                    {level === 'error' ? 'Err' : level === 'warn' ? 'Wrn' : level === 'info' ? 'Inf' : level === 'debug' ? 'Dbg' : 'Off'}
                                                                </button>
                                                            );
                                                        })}
                                                    </div>
                                                </div>
                                            );
                                        })}
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
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
                              disabled={validationState === 'loading' || validationState === 'error'}
                              className={cn(
                                  "px-8 py-3 font-bold rounded-xl transition-colors",
                                  validationState === 'loading' || validationState === 'error'
                                    ? "bg-gray-600 text-muted cursor-not-allowed"
                                    : "bg-[#FF6B00] hover:bg-[#E66000] text-white shadow-[0_0_15px_rgba(255,107,0,0.5)]"
                              )}
                          >
                              Save
                          </button>
                      </div>
                  </div>
              </motion.div>
          )}

            <AnimatePresence mode="popLayout">
                {activeTab === "Scanner" && (
                    <motion.div
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: -10 }}
                        className="bg-[#1F222A] rounded-2xl p-6 border border-white/5 shadow-xl"
                    >
                        <h2 className="text-xl font-bold text-white mb-6">Scanner Configuration</h2>
                        <div className="grid grid-cols-[250px_1fr] gap-6 items-start py-4 border-b border-white/5 last:border-0">
                            <div>
                                <h3 className="text-sm font-bold text-white">Supported Video Formats</h3>
                                <p className="text-xs text-gray-500 mt-1">Comma-separated list of extensions the scanner will index.</p>
                            </div>
                            <div>
                                <input
                                    type="text"
                                    value={settings.supported_extensions_raw ?? (settings.supported_extensions?.join(', ') || '')}
                                    onChange={(e) => {
                                        updateSetting('supported_extensions_raw', e.target.value);
                                    }}
                                    onBlur={() => {
                                        const raw = settings.supported_extensions_raw ?? (settings.supported_extensions?.join(', ') || '');
                                        const exts = raw.split(',').map((s: string) => s.trim().toLowerCase()).filter((s: string) => s.length > 0);
                                        const updatedSettings = { ...settings, supported_extensions: exts };
                                        delete updatedSettings.supported_extensions_raw;
                                        setSettings(updatedSettings);
                                        setIsDirty(true);
                                    }}
                                    className="w-full bg-[#0D0F14] border border-white/10 rounded-xl px-4 py-3 text-white focus:outline-none focus:border-[#FF6B00] transition-colors font-mono text-sm"
                                    placeholder="mp4, mkv, avi, mov..."
                                />
                            </div>
                        </div>
                    </motion.div>
                )}
            </AnimatePresence>
      </AnimatePresence>

    </div>
  );
}
