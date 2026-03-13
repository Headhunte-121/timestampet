import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/useAppStore";

export default function SettingsView() {
  const [settings, setSettings] = useState<any>({
    vlc_path: "",
    tmdb_api_key: "",
    window_geometry: "1200x800",
    window_position: "+100+100",
    cinema_mode: true
  });

  const { isCinemaMode, setCinemaMode } = useAppStore();

  useEffect(() => {
    invoke("get_settings")
      .then((res: any) => setSettings(res))
      .catch(console.error);
  }, []);

  const saveSettings = () => {
    invoke("save_settings", { settings })
      .then(() => alert("Settings saved successfully."))
      .catch(e => alert("Error saving settings: " + e));
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

        <button
          onClick={saveSettings}
          className="mt-8 px-8 py-4 bg-[#FF6B00] hover:bg-[#E66000] text-white font-bold rounded-xl w-full sm:w-auto shadow-lg shadow-orange-500/20 hover:scale-105 transition-all duration-300"
        >
          Save Settings
        </button>
      </div>
    </div>
  );
}
