import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "../store/useAppStore";
import { useUiStore } from "../store/uiStore";
import { toast } from "sonner";

export default function SettingsView() {
  const { showPrompt } = useUiStore();
  const [settings, setSettings] = useState<any>({
    vlc_path: "",
    tmdb_api_key: "",
    window_geometry: "1200x800",
    window_position: "+100+100",
    cinema_mode: true
  });

  const { isCinemaMode, setCinemaMode } = useAppStore();
  const [scanStatus, setScanStatus] = useState<string>("");

  useEffect(() => {
    invoke("get_settings")
      .then((res: any) => setSettings(res))
      .catch(console.error);

    const unlisten = listen("scan-match-batch", (event: any) => {
      const batch = event.payload.files;
      if (batch && batch.length > 0) {
        setScanStatus(`Scanned ${batch.length} files in latest batch...`);
      }
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  const saveSettings = () => {
    invoke("save_settings", { settings })
      .then(() => toast.success("Settings saved successfully."))
      .catch(e => toast.error("Error saving settings: " + e));
  };

  const runScan = async () => {
    const dir = await showPrompt("Run Scan", "Enter directory to scan:", "C:\\");
    if (dir) {
      setScanStatus("Scan started...");
      invoke("run_scan_directory", { directory: dir })
        .then((count) => {
          toast.success(`Scan complete. Found ${count} unmatched files.`);
          setScanStatus("");
        })
        .catch(e => {
          toast.error("Error during scan: " + e);
          setScanStatus("");
        });
    }
  };

  return (
    <div className="p-12 pb-24 max-w-3xl">
      <h1 className="text-4xl font-extrabold tracking-tight mb-8">Settings</h1>

      <div className="bg-[#1F222A] p-8 rounded-2xl border border-white/5 space-y-8 shadow-xl">
        <div>
          <label className="block text-sm font-bold text-gray-300 mb-2">TMDB API Key:</label>
          <input
            type="password"
            value={settings.tmdb_api_key}
            onChange={e => setSettings({ ...settings, tmdb_api_key: e.target.value })}
            className="w-full bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
            placeholder="ey..."
          />
        </div>

        <div>
          <label className="block text-sm font-bold text-gray-300 mb-2">VLC Executable Path:</label>
          <div className="flex gap-4">
            <input
              type="text"
              value={settings.vlc_path}
              onChange={e => setSettings({ ...settings, vlc_path: e.target.value })}
              className="flex-1 bg-black/40 text-white px-4 py-3 rounded-xl border border-white/10 focus:border-[#FF6B00] outline-none"
              placeholder="C:\Program Files\VideoLAN\VLC\vlc.exe"
            />
            <button className="px-6 py-3 bg-white/10 hover:bg-white/20 font-bold rounded-xl transition-colors text-white">
              Browse
            </button>
          </div>
        </div>

        <div>
          <label className="flex items-center gap-3 cursor-pointer text-sm font-bold text-gray-300">
            <input
              type="checkbox"
              checked={isCinemaMode}
              onChange={(e) => {
                const newMode = e.target.checked;
                setCinemaMode(newMode);
                setSettings({ ...settings, cinema_mode: newMode });
              }}
              className="accent-[#FF6B00] w-5 h-5 rounded focus:ring-[#FF6B00]"
            />
            Cinema Mode (High Quality Animations & Effects)
          </label>
        </div>

        <div className="flex flex-col gap-4 mt-8">
          <div className="flex gap-4">
            <button
              onClick={saveSettings}
              className="px-8 py-4 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-xl w-full sm:w-auto shadow-lg shadow-orange-500/20 hover:scale-105 transition-all duration-300"
            >
              Save Settings
            </button>

            <button
              onClick={runScan}
              className="px-8 py-4 bg-white/10 hover:bg-white/20 text-white font-bold rounded-xl w-full sm:w-auto shadow-lg hover:scale-105 transition-all duration-300"
            >
              Run Scan
            </button>
          </div>
          {scanStatus && <div className="text-gray-400 text-sm mt-2">{scanStatus}</div>}
        </div>
      </div>
    </div>
  );
}
