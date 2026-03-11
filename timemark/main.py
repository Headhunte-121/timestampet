import os
import customtkinter as ctk
from tkinter import ttk, messagebox, filedialog
from typing import Optional, Any, List, Tuple
import keyboard
import threading
from .data import DataManager, Timestamp, Episode, Season, Series
from .scanner import scan_directory
from .vlc_integration import get_vlc_path, parse_time_input, format_time_display, play_in_vlc, get_current_vlc_time, generate_highlight_playlist
from .ffmpeg import export_seamless_scene

# Configure basic appearance
ctk.set_appearance_mode("dark")
ctk.set_default_color_theme("blue") # We will manually set orange accents later

VLC_ORANGE = "#FF8800"
VLC_ORANGE_HOVER = "#E67A00"

class App(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.data_manager = DataManager()
        self.title("TimeMark - VLC Bookmarker")

        # Load geometry
        geo = self.data_manager.settings.get("window_geometry", "1000x700")
        pos = self.data_manager.settings.get("window_position", "+100+100")
        self.geometry(f"{geo}{pos}")

        # Auto-detect VLC path if empty
        if not self.data_manager.settings.get("vlc_path"):
            auto_path = get_vlc_path()
            if auto_path:
                self.data_manager.settings["vlc_path"] = auto_path
                self.data_manager.save_settings()

        # State tracking
        self.selected_node_id: Optional[str] = None
        self.selected_episode: Optional[Episode] = None
        self.editing_timestamp_idx: Optional[int] = None
        self.show_only_highlighted = ctk.BooleanVar(value=False)
        self.rapid_cut_mode = ctk.BooleanVar(value=False)
        self.active_tags: List[str] = []

        self.rapid_segments: List[Tuple[int, int]] = []
        self._current_rapid_start: int = -1

        self._setup_ui()
        self._populate_tree()

        self._setup_hotkeys()

        # Bind closing event to save geometry
        self.protocol("WM_DELETE_WINDOW", self._on_closing)

    def _setup_hotkeys(self):
        try:
            keyboard.add_hotkey('ctrl+shift+[', lambda: self.after(0, self._hotkey_fetch_start))
            keyboard.add_hotkey('ctrl+shift+]', lambda: self.after(0, self._hotkey_fetch_end))

            # Hook 'alt' key globally for dead man's switch in rapid cut mode
            keyboard.on_press_key('alt', lambda _: self.after(0, self._handle_rapid_down))
            keyboard.on_release_key('alt', lambda _: self.after(0, self._handle_rapid_up))
        except Exception as e:
            print(f"Failed to bind global hotkeys (you may need to run as administrator/root): {e}")

    def _handle_rapid_down(self):
        if not self.rapid_cut_mode.get() or not self.selected_episode: return
        # Prevent auto-repeat triggers if key is held down
        if self._current_rapid_start != -1: return

        time_sec = get_current_vlc_time()
        if time_sec >= 0:
            self._current_rapid_start = time_sec
            if hasattr(self, 'rapid_status_lbl'):
                self.rapid_status_lbl.configure(text="🔴 RECORDING...", text_color="red")

    def _handle_rapid_up(self):
        if not self.rapid_cut_mode.get() or not self.selected_episode: return
        if self._current_rapid_start == -1: return

        end_sec = get_current_vlc_time()
        if end_sec > self._current_rapid_start:
            self.rapid_segments.append([self._current_rapid_start, end_sec])

        self._current_rapid_start = -1
        self._update_rapid_ui()

    def _hotkey_fetch_start(self):
        # Only fetch if an episode is selected and we are in the episode view
        if self.selected_episode and hasattr(self, 'start_entry'):
            self._fetch_vlc_time("start")

    def _hotkey_fetch_end(self):
        if self.selected_episode and hasattr(self, 'end_entry'):
            self._fetch_vlc_time("end")

    def _on_closing(self):
        # Save geometry
        self.data_manager.settings["window_geometry"] = f"{self.winfo_width()}x{self.winfo_height()}"
        self.data_manager.settings["window_position"] = f"+{self.winfo_x()}+{self.winfo_y()}"
        self.data_manager.save_settings()
        self.destroy()

    def _setup_ui(self):
        self.grid_rowconfigure(0, weight=1)
        self.grid_columnconfigure(1, weight=1)

        # --- LEFT PANE ---
        self.left_frame = ctk.CTkFrame(self, width=300, corner_radius=0)
        self.left_frame.grid(row=0, column=0, sticky="nsew")
        self.left_frame.grid_rowconfigure(1, weight=1)

        # Left Pane Top: Scan Button & Filter Toggle
        top_left_frame = ctk.CTkFrame(self.left_frame, fg_color="transparent")
        top_left_frame.grid(row=0, column=0, padx=10, pady=(10, 5), sticky="ew")

        self.scan_btn = ctk.CTkButton(
            top_left_frame, text="📂 Scan Media Folder",
            fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER,
            command=self._scan_folder
        )
        self.scan_btn.pack(fill="x", pady=(0, 10))

        self.filter_switch = ctk.CTkSwitch(
            top_left_frame, text="👁️ Show Only Highlighted",
            variable=self.show_only_highlighted,
            command=self._populate_tree
        )
        self.filter_switch.pack(anchor="w")

        # Left Pane Middle: Treeview
        # Tkinter Treeview needs special styling for dark mode
        style = ttk.Style(self)
        style.theme_use("default")
        style.configure("Treeview",
                        background="#2b2b2b",
                        foreground="white",
                        fieldbackground="#2b2b2b",
                        borderwidth=0)
        style.map('Treeview', background=[('selected', '#3a7ebf')])

        self.tree = ttk.Treeview(self.left_frame, selectmode="browse", show="tree")
        self.tree.grid(row=1, column=0, padx=10, pady=10, sticky="nsew")
        self.tree.bind("<<TreeviewSelect>>", self._on_tree_select)

        # Add scrollbar for tree
        tree_scrollbar = ttk.Scrollbar(self.left_frame, orient="vertical", command=self.tree.yview)
        self.tree.configure(yscrollcommand=tree_scrollbar.set)
        tree_scrollbar.grid(row=1, column=1, sticky="ns")

        # Left Pane Bottom: Edit / Delete
        self.action_frame = ctk.CTkFrame(self.left_frame, fg_color="transparent")
        self.action_frame.grid(row=2, column=0, padx=10, pady=(0, 20), sticky="ew")
        self.action_frame.grid_columnconfigure((0, 1), weight=1)

        self.edit_btn = ctk.CTkButton(self.action_frame, text="✎ Edit Name", command=self._edit_tree_item)
        self.edit_btn.grid(row=0, column=0, padx=(0, 5), sticky="ew")

        self.delete_btn = ctk.CTkButton(self.action_frame, text="- Delete", fg_color="#C62828", hover_color="#B71C1C", command=self._delete_tree_item)
        self.delete_btn.grid(row=0, column=1, padx=(5, 0), sticky="ew")

        # Settings Button
        self.settings_btn = ctk.CTkButton(self.left_frame, text="⚙ Settings", fg_color="transparent", border_width=1, command=self._open_settings)
        self.settings_btn.grid(row=3, column=0, padx=10, pady=10, sticky="ew")

        # --- RIGHT PANE ---
        self.right_frame = ctk.CTkFrame(self, corner_radius=0)
        self.right_frame.grid(row=0, column=1, sticky="nsew", padx=10, pady=10)
        self.right_frame.grid_rowconfigure(1, weight=1) # Allow timestamp list to expand

        # Initially hide the right pane content until an episode is selected
        self._hide_right_pane()

    def _hide_right_pane(self):
        for widget in self.right_frame.winfo_children():
            widget.destroy()

        self.empty_label = ctk.CTkLabel(self.right_frame, text="Select an Episode from the left to view timestamps.", text_color="gray")
        self.empty_label.pack(expand=True)

    def _scan_folder(self):
        folder = filedialog.askdirectory(title="Select Media Folder to Scan")
        if folder:
            scan_directory(folder, self.data_manager)
            self._populate_tree()
            messagebox.showinfo("Scan Complete", "Finished scanning and updating library.")

    def _populate_tree(self):
        self.tree.delete(*self.tree.get_children())

        lib = self.data_manager.library
        filter_on = self.show_only_highlighted.get()

        # Populate Series
        for series_name, series in sorted(lib.series.items()):
            series_has_highlights = False
            s_node_children = []

            for season_num, season in sorted(series.seasons.items(), key=lambda x: int(x[0])):
                season_has_highlights = False
                se_node_children = []

                for ep_num, episode in sorted(season.episodes.items(), key=lambda x: int(x[0])):
                    has_ts = len(episode.timestamps) > 0
                    if not filter_on or has_ts:
                        se_node_children.append((episode.title, ("episode", series_name, season_num, ep_num)))
                        if has_ts:
                            season_has_highlights = True
                            series_has_highlights = True

                if not filter_on or season_has_highlights:
                    s_node_children.append((f"Season {season_num}", ("season", series_name, season_num), se_node_children))

            if not filter_on or series_has_highlights:
                s_node = self.tree.insert("", "end", text=series_name, values=("series", series_name))
                for s_title, s_vals, e_children in s_node_children:
                    se_node = self.tree.insert(s_node, "end", text=s_title, values=s_vals)
                    for e_title, e_vals in e_children:
                        self.tree.insert(se_node, "end", text=e_title, values=e_vals)

        # Populate Unmatched
        if lib.unmatched:
            u_children = []
            has_unmatched_ts = False
            for ep_key, episode in sorted(lib.unmatched.items()):
                has_ts = len(episode.timestamps) > 0
                if not filter_on or has_ts:
                    u_children.append((episode.title, ("unmatched_episode", ep_key)))
                    if has_ts:
                        has_unmatched_ts = True

            if not filter_on or has_unmatched_ts:
                u_node = self.tree.insert("", "end", text="[?] Unmatched Files", values=("unmatched_root",))
                for e_title, e_vals in u_children:
                    self.tree.insert(u_node, "end", text=e_title, values=e_vals)

    def _on_tree_select(self, event):
        selected = self.tree.selection()
        if not selected:
            return

        item = self.tree.item(selected[0])
        values = item.get("values", [])

        if not values:
            return

        node_type = values[0]

        if node_type == "episode":
            series_name, season_num, ep_num = str(values[1]), str(values[2]), str(values[3])
            episode = self.data_manager.library.series[series_name].seasons[season_num].episodes[ep_num]
            self._show_episode_view(episode)
        elif node_type == "unmatched_episode":
            ep_key = str(values[1])
            episode = self.data_manager.library.unmatched[ep_key]
            self._show_episode_view(episode)
        else:
            self._hide_right_pane()

    def _show_episode_view(self, episode: Episode):
        self.selected_episode = episode
        self.editing_timestamp_idx = None

        for widget in self.right_frame.winfo_children():
            widget.destroy()

        # TOP: Episode Info
        info_frame = ctk.CTkFrame(self.right_frame, fg_color="transparent")
        info_frame.pack(fill="x", padx=10, pady=10)

        title_text = f"{episode.series} - Season {episode.season} - {episode.title}" if episode.series != "?" else episode.title
        title_label = ctk.CTkLabel(info_frame, text=title_text, font=ctk.CTkFont(size=20, weight="bold"))
        title_label.pack(anchor="w")

        if not episode.file_path or not os.path.exists(episode.file_path):
            browse_btn = ctk.CTkButton(
                self.right_frame, text="Browse for Video File...",
                font=ctk.CTkFont(size=16), height=50,
                command=self._browse_video_file
            )
            browse_btn.pack(expand=True)
            return

        # Linked State
        path_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        path_frame.pack(fill="x", pady=5)

        path_label = ctk.CTkLabel(path_frame, text=episode.file_path, text_color="gray", wraplength=400)
        path_label.pack(side="left")

        change_btn = ctk.CTkButton(path_frame, text="Change File", width=100, command=self._browse_video_file)
        change_btn.pack(side="right")

        actions_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        actions_frame.pack(anchor="w", pady=10)

        play_btn = ctk.CTkButton(
            actions_frame, text="▶ Play Episode",
            fg_color="#3a7ebf", hover_color="#2b5e8f",
            command=lambda: self._play_timestamp(0)
        )
        play_btn.pack(side="left", padx=(0, 10))

        play_hl_btn = ctk.CTkButton(
            actions_frame, text="🎬 Play Highlight Reel",
            fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER,
            command=self._play_highlight_reel
        )
        play_hl_btn.pack(side="left")

        # MIDDLE: Timestamp List
        list_frame = ctk.CTkScrollableFrame(self.right_frame)
        list_frame.pack(fill="both", expand=True, padx=10, pady=10)

        # Headers
        h_frame = ctk.CTkFrame(list_frame, fg_color="transparent")
        h_frame.pack(fill="x", pady=5)
        h_frame.grid_columnconfigure(2, weight=1)

        ctk.CTkLabel(h_frame, text="Segments / Time", font=ctk.CTkFont(weight="bold"), width=150).grid(row=0, column=0, sticky="w")
        ctk.CTkLabel(h_frame, text="Tags", font=ctk.CTkFont(weight="bold"), width=100).grid(row=0, column=1, sticky="w", padx=10)
        ctk.CTkLabel(h_frame, text="Description", font=ctk.CTkFont(weight="bold")).grid(row=0, column=2, sticky="w", padx=10)
        ctk.CTkLabel(h_frame, text="Actions", font=ctk.CTkFont(weight="bold"), width=250).grid(row=0, column=3, sticky="e")

        for idx, ts in enumerate(episode.timestamps):
            row = ctk.CTkFrame(list_frame)
            row.pack(fill="x", pady=2)
            row.grid_columnconfigure(2, weight=1)

            num_segs = len(ts.segments)
            if num_segs == 1:
                st, et = ts.segments[0]
                time_range = f"{format_time_display(st)} - {format_time_display(et)}"
            else:
                time_range = f"{num_segs} Segments (Multi)"

            ctk.CTkLabel(row, text=time_range, width=150).grid(row=0, column=0, sticky="w", padx=5)

            tags_str = ", ".join(ts.tags)
            ctk.CTkLabel(row, text=tags_str, width=100).grid(row=0, column=1, sticky="w", padx=10)

            ctk.CTkLabel(row, text=ts.description).grid(row=0, column=2, sticky="w", padx=10)

            action_f = ctk.CTkFrame(row, fg_color="transparent")
            action_f.grid(row=0, column=3, sticky="e")

            first_start = ts.segments[0][0] if ts.segments else 0
            ctk.CTkButton(action_f, text="▶", width=30, fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, command=lambda t=first_start: self._play_timestamp(t)).pack(side="left", padx=2)
            ctk.CTkButton(action_f, text="🎬 Export Seamless", width=120, command=lambda t=ts: self._export_seamless(t)).pack(side="left", padx=2)
            ctk.CTkButton(action_f, text="Edit", width=40, command=lambda i=idx: self._start_edit_timestamp(i)).pack(side="left", padx=2)
            ctk.CTkButton(action_f, text="Del", width=40, fg_color="#C62828", hover_color="#B71C1C", command=lambda i=idx: self._delete_timestamp(i)).pack(side="left", padx=2)

        # BOTTOM: Add New Timestamp (Highlight Reel Optimizer)
        input_frame = ctk.CTkFrame(self.right_frame)
        input_frame.pack(fill="x", padx=10, pady=10)

        self.active_tags = []

        # Row 0: Rapid Cut Mode Toggle
        rapid_row = ctk.CTkFrame(input_frame, fg_color="transparent")
        rapid_row.pack(fill="x", pady=(5, 5), padx=5)

        self.rapid_switch = ctk.CTkSwitch(
            rapid_row, text="⚡ Enable Rapid-Cut Mode (Hold ALT to record segments)",
            variable=self.rapid_cut_mode,
            command=self._toggle_rapid_mode
        )
        self.rapid_switch.pack(side="left")

        self.rapid_status_lbl = ctk.CTkLabel(rapid_row, text="", width=150)
        self.rapid_status_lbl.pack(side="left", padx=20)

        self.rapid_clear_btn = ctk.CTkButton(rapid_row, text="Clear Segments", fg_color="gray", width=100, command=self._clear_rapid_segments)

        # Row 1: Time Fetchers (Single Segment Input)
        self.time_row = ctk.CTkFrame(input_frame, fg_color="transparent")
        self.time_row.pack(fill="x", pady=(5, 5), padx=5)

        ctk.CTkButton(self.time_row, text="[ Get Start Time ]", width=120, command=lambda: self._fetch_vlc_time("start")).pack(side="left", padx=(0, 5))
        self.start_entry = ctk.CTkEntry(self.time_row, placeholder_text="Start (MM:SS)", width=90)
        self.start_entry.pack(side="left", padx=(0, 15))

        ctk.CTkButton(self.time_row, text="[ Get End Time ]", width=120, command=lambda: self._fetch_vlc_time("end")).pack(side="left", padx=(0, 5))
        self.end_entry = ctk.CTkEntry(self.time_row, placeholder_text="End (MM:SS)", width=90)
        self.end_entry.pack(side="left")

        self.vlc_warning_label = ctk.CTkLabel(self.time_row, text="", text_color="#C62828", font=ctk.CTkFont(size=11))
        self.vlc_warning_label.pack(side="left", padx=10)

        # Row 2: Presets and Saving
        bot_row = ctk.CTkFrame(input_frame, fg_color="transparent")
        bot_row.pack(fill="x", pady=(5, 10), padx=5)

        tags_frame = ctk.CTkFrame(bot_row, fg_color="transparent")
        tags_frame.pack(side="left")

        ctk.CTkLabel(tags_frame, text="Tags:").pack(side="left", padx=(0, 5))

        presets = self.data_manager.settings.get("tags_presets", ["Action", "Funny", "Important"])
        self.tag_buttons = {}
        for preset in presets:
            btn = ctk.CTkButton(tags_frame, text=preset, width=60, fg_color="#555555", hover_color="#666666", command=lambda p=preset: self._toggle_tag(p))
            btn.pack(side="left", padx=2)
            self.tag_buttons[preset] = btn

        self.desc_entry = ctk.CTkEntry(bot_row, placeholder_text="Optional Note", width=150)
        self.desc_entry.pack(side="left", expand=True, fill="x", padx=10)

        self.save_ts_btn = ctk.CTkButton(bot_row, text="Save Highlight", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, command=self._save_timestamp)
        self.save_ts_btn.pack(side="left", padx=5)

        self.cancel_ts_btn = ctk.CTkButton(bot_row, text="Cancel", fg_color="gray", command=self._cancel_edit_timestamp)
        # Cancel btn is initially hidden

        # Initial UI refresh for rapid mode
        self._toggle_rapid_mode()

    def _toggle_rapid_mode(self):
        if self.rapid_cut_mode.get():
            self.time_row.pack_forget() # Hide single entry
            self.rapid_clear_btn.pack(side="left", padx=10)
            self._update_rapid_ui()
        else:
            self.time_row.pack(fill="x", pady=(5, 5), padx=5, before=self.save_ts_btn.master)
            self.rapid_clear_btn.pack_forget()
            self.rapid_status_lbl.configure(text="")

    def _update_rapid_ui(self):
        if self.rapid_cut_mode.get():
            n = len(self.rapid_segments)
            self.rapid_status_lbl.configure(text=f"{n} Segments Captured", text_color="white")

    def _clear_rapid_segments(self):
        self.rapid_segments = []
        self._update_rapid_ui()

    def _toggle_tag(self, tag: str):
        if tag in self.active_tags:
            self.active_tags.remove(tag)
            self.tag_buttons[tag].configure(fg_color="#555555") # Inactive state
        else:
            self.active_tags.append(tag)
            self.tag_buttons[tag].configure(fg_color="#3a7ebf") # Active state

    def _fetch_vlc_time(self, target: str):
        self.vlc_warning_label.configure(text="")

        time_seconds = get_current_vlc_time()

        if time_seconds >= 0:
            formatted_time = format_time_display(time_seconds)
            if target == "start":
                self.start_entry.delete(0, 'end')
                self.start_entry.insert(0, formatted_time)
            elif target == "end":
                self.end_entry.delete(0, 'end')
                self.end_entry.insert(0, formatted_time)
        else:
            self.vlc_warning_label.configure(text="⚠️ VLC connect fail")

    def _browse_video_file(self):
        if not self.selected_episode: return
        file_path = filedialog.askopenfilename(title="Select Video File", filetypes=[("Video Files", "*.mp4 *.mkv *.avi *.mov *.wmv *.flv *.webm"), ("All Files", "*.*")])
        if file_path:
            self.selected_episode.file_path = file_path
            self.data_manager.save_library()
            self._show_episode_view(self.selected_episode)

    def _play_timestamp(self, time_seconds: int):
        vlc_path = self.data_manager.settings.get("vlc_path", "")
        if not vlc_path:
            messagebox.showwarning("VLC Path Not Set", "Please set your VLC path in the Settings first.")
            return

        if self.selected_episode and self.selected_episode.file_path:
            play_in_vlc(vlc_path, self.selected_episode.file_path, time_seconds)

    def _play_highlight_reel(self):
        if not self.selected_episode or not self.selected_episode.file_path:
            return
        vlc_path = self.data_manager.settings.get("vlc_path", "")
        if not vlc_path:
            messagebox.showwarning("VLC Path Not Set", "Please set your VLC path in the Settings first.")
            return

        scenes = []
        for ts in self.selected_episode.timestamps:
            scenes.append({
                'file': self.selected_episode.file_path,
                'segments': ts.segments,
                'title': f"{ts.description} - {', '.join(ts.tags)}" if ts.description else "Highlight"
            })

        if not scenes:
            messagebox.showinfo("No Highlights", "There are no highlights saved for this episode.")
            return

        generate_highlight_playlist(vlc_path, scenes)

    def _export_seamless(self, ts: Timestamp):
        if not self.selected_episode or not self.selected_episode.file_path: return
        if not ts.segments: return

        # Open save dialog
        output_file = filedialog.asksaveasfilename(
            title="Export Seamless Video",
            defaultextension=".mp4",
            filetypes=[("MP4 Video", "*.mp4"), ("All Files", "*.*")]
        )

        if not output_file: return

        # Show a processing message (we run in thread to avoid freezing UI)
        msg_win = ctk.CTkToplevel(self)
        msg_win.title("Exporting...")
        msg_win.geometry("300x150")
        msg_win.attributes('-topmost', True)
        ctk.CTkLabel(msg_win, text="FFmpeg is extracting and stitching segments.\nPlease wait...").pack(expand=True)
        self.update() # Force UI refresh

        def run_export():
            success = export_seamless_scene(self.selected_episode.file_path, ts.segments, output_file)
            self.after(0, lambda: _export_done(success))

        def _export_done(success):
            msg_win.destroy()
            if success:
                messagebox.showinfo("Export Complete", f"Successfully exported seamless scene to:\n{output_file}")
            else:
                messagebox.showerror("Export Failed", "Failed to export seamless scene. See console for details.")

        threading.Thread(target=run_export, daemon=True).start()

    def _save_timestamp(self):
        if not self.selected_episode: return

        desc = self.desc_entry.get().strip()
        tags = list(self.active_tags)

        # Decide segments based on mode
        segments = []
        if self.rapid_cut_mode.get():
            if not self.rapid_segments:
                messagebox.showwarning("No Segments", "Rapid-Cut mode is on, but no segments were recorded (Hold ALT).")
                return
            segments = list(self.rapid_segments)
        else:
            start_str = self.start_entry.get().strip()
            end_str = self.end_entry.get().strip()

            if not start_str:
                messagebox.showwarning("Incomplete Input", "Start time is required.")
                return

            start_seconds = parse_time_input(start_str)
            end_seconds = parse_time_input(end_str) if end_str else start_seconds

            # Ensure start is before end
            if start_seconds > end_seconds:
                start_seconds, end_seconds = end_seconds, start_seconds

            segments = [[start_seconds, end_seconds]]

        new_ts = Timestamp(
            segments=segments,
            tags=tags,
            description=desc
        )

        if self.editing_timestamp_idx is not None:
            # Update existing
            self.selected_episode.timestamps[self.editing_timestamp_idx] = new_ts
        else:
            # Add new
            self.selected_episode.timestamps.append(new_ts)

        # Sort timestamps by earliest segment start time
        self.selected_episode.timestamps.sort(key=lambda x: x.segments[0][0] if x.segments else 0)

        self.data_manager.save_library()
        self._clear_rapid_segments() # Reset
        self._show_episode_view(self.selected_episode)

        # We need to refresh tree in case filter mode is active and we just added/deleted timestamps
        if self.show_only_highlighted.get():
            self._populate_tree()

    def _start_edit_timestamp(self, idx: int):
        ts = self.selected_episode.timestamps[idx]
        self.editing_timestamp_idx = idx

        # If it's a multi-segment scene, editing is restricted to tags/desc only for now
        # We switch to Rapid Cut mode visually but don't populate segments
        if len(ts.segments) > 1:
            self.rapid_cut_mode.set(True)
            self.rapid_segments = list(ts.segments)
            self._toggle_rapid_mode()
        else:
            self.rapid_cut_mode.set(False)
            self._toggle_rapid_mode()
            self.start_entry.delete(0, 'end')
            self.start_entry.insert(0, format_time_display(ts.segments[0][0]))
            self.end_entry.delete(0, 'end')
            self.end_entry.insert(0, format_time_display(ts.segments[0][1]))

        self.desc_entry.delete(0, 'end')
        self.desc_entry.insert(0, ts.description)

        # Reset tag buttons
        for p, btn in self.tag_buttons.items():
            btn.configure(fg_color="#555555")
        self.active_tags = []
        for tag in ts.tags:
            if tag in self.tag_buttons:
                self._toggle_tag(tag)

        self.save_ts_btn.configure(text="Update")
        self.cancel_ts_btn.pack(side="left", padx=5)

    def _cancel_edit_timestamp(self):
        self.editing_timestamp_idx = None
        self._clear_rapid_segments()
        self.start_entry.delete(0, 'end')
        self.end_entry.delete(0, 'end')
        self.desc_entry.delete(0, 'end')
        for p, btn in self.tag_buttons.items():
            btn.configure(fg_color="#555555")
        self.active_tags = []
        self.save_ts_btn.configure(text="Save Highlight")
        self.cancel_ts_btn.pack_forget()

    def _delete_timestamp(self, idx: int):
        if messagebox.askyesno("Confirm Delete", "Are you sure you want to delete this timestamp?"):
            del self.selected_episode.timestamps[idx]
            self.data_manager.save_library()
            self._show_episode_view(self.selected_episode)

    def _edit_tree_item(self):
        selected = self.tree.selection()
        if not selected: return

        item = self.tree.item(selected[0])
        values = item.get("values", [])
        if not values: return

        node_type = values[0]

        dialog = ctk.CTkInputDialog(text="Enter new name:", title="Edit Name")
        new_name = dialog.get_input()

        if not new_name: return

        # We need to update the dictionary keys, which means we might need to recreate entries
        lib = self.data_manager.library

        if node_type == "episode":
             series_name, season_num, ep_num = str(values[1]), str(values[2]), str(values[3])
             ep = lib.series[series_name].seasons[season_num].episodes[ep_num]
             ep.title = new_name
             self.data_manager.save_library()
             self._populate_tree()
        elif node_type == "series":
             old_name = str(values[1])
             if new_name in lib.series:
                 messagebox.showerror("Error", f"A series named '{new_name}' already exists.")
                 return

             series_obj = lib.series.pop(old_name)
             series_obj.name = new_name
             # Update series name in all child episodes
             for s_obj in series_obj.seasons.values():
                 for ep_obj in s_obj.episodes.values():
                     ep_obj.series = new_name

             lib.series[new_name] = series_obj
             self.data_manager.save_library()
             self._populate_tree()
             self._hide_right_pane()
        elif node_type == "unmatched_episode":
             ep_key = str(values[1])
             ep = lib.unmatched.pop(ep_key)
             ep.title = new_name

             # Re-evaluate the new name to see if it now matches a regex pattern
             from .scanner import parse_filename
             # Create a dummy filename with the new name and original extension
             dummy_ext = ".mp4"
             if ep.file_path and "." in ep.file_path:
                 dummy_ext = os.path.splitext(ep.file_path)[1]
             dummy_filename = new_name + dummy_ext

             series_name, season_num, episode_num = parse_filename(dummy_filename)

             if series_name and season_num and episode_num:
                 # It matched! Move it to the library hierarchy
                 if series_name not in lib.series:
                     lib.series[series_name] = Series(name=series_name)
                 series_obj = lib.series[series_name]

                 if season_num not in series_obj.seasons:
                     series_obj.seasons[season_num] = Season(number=season_num)
                 season_obj = series_obj.seasons[season_num]

                 ep.series = series_name
                 ep.season = season_num
                 ep.title = f"Episode {episode_num}"

                 # If an episode already exists there, merge timestamps and replace path
                 if episode_num in season_obj.episodes:
                     existing_ep = season_obj.episodes[episode_num]
                     existing_ep.file_path = ep.file_path
                     existing_ep.timestamps.extend(ep.timestamps)
                     # Sort timestamps
                     existing_ep.timestamps.sort(key=lambda x: x.start_time)
                 else:
                     season_obj.episodes[episode_num] = ep

                 messagebox.showinfo("Success", f"Successfully matched and moved to:\n{series_name} -> Season {season_num} -> Episode {episode_num}")
             else:
                 # Still unmatched, just update the key and title in unmatched dict
                 lib.unmatched[new_name] = ep

             self.data_manager.save_library()
             self._populate_tree()
             self._hide_right_pane()
        else:
             messagebox.showinfo("Not Supported", "Editing Season names directly is not supported yet.")

    def _delete_tree_item(self):
         selected = self.tree.selection()
         if not selected: return

         item = self.tree.item(selected[0])
         values = item.get("values", [])
         if not values: return

         node_type = values[0]

         if not messagebox.askyesno("Confirm Delete", "Are you sure you want to remove this item from your library? (The actual file will NOT be deleted)."):
             return

         lib = self.data_manager.library

         if node_type == "episode":
              series_name, season_num, ep_num = str(values[1]), str(values[2]), str(values[3])
              del lib.series[series_name].seasons[season_num].episodes[ep_num]
              # clean up empties
              if not lib.series[series_name].seasons[season_num].episodes:
                  del lib.series[series_name].seasons[season_num]
              if not lib.series[series_name].seasons:
                  del lib.series[series_name]
         elif node_type == "unmatched_episode":
              ep_key = str(values[1])
              del lib.unmatched[ep_key]
         elif node_type == "season":
              series_name, season_num = str(values[1]), str(values[2])
              del lib.series[series_name].seasons[season_num]
              if not lib.series[series_name].seasons:
                  del lib.series[series_name]
         elif node_type == "series":
              series_name = str(values[1])
              del lib.series[series_name]

         self.data_manager.save_library()
         self._hide_right_pane()
         self._populate_tree()

    def _open_settings(self):
        settings_win = ctk.CTkToplevel(self)
        settings_win.title("Settings")
        settings_win.geometry("500x300")
        settings_win.grab_set()

        ctk.CTkLabel(settings_win, text="VLC Executable Path:", font=ctk.CTkFont(weight="bold")).pack(pady=(20, 5), padx=20, anchor="w")

        vlc_frame = ctk.CTkFrame(settings_win, fg_color="transparent")
        vlc_frame.pack(fill="x", padx=20)

        vlc_entry = ctk.CTkEntry(vlc_frame)
        vlc_entry.pack(side="left", fill="x", expand=True, padx=(0, 10))
        vlc_entry.insert(0, self.data_manager.settings.get("vlc_path", ""))

        def browse_vlc():
             path = filedialog.askopenfilename(title="Select VLC Executable")
             if path:
                 vlc_entry.delete(0, 'end')
                 vlc_entry.insert(0, path)

        ctk.CTkButton(vlc_frame, text="Browse", width=80, command=browse_vlc).pack(side="left")

        def save_settings():
             self.data_manager.settings["vlc_path"] = vlc_entry.get()
             self.data_manager.save_settings()
             settings_win.destroy()

        ctk.CTkButton(settings_win, text="Save Settings", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, command=save_settings).pack(pady=20)

        ctk.CTkLabel(settings_win, text="Library Backup:", font=ctk.CTkFont(weight="bold")).pack(pady=(10, 5), padx=20, anchor="w")

        backup_frame = ctk.CTkFrame(settings_win, fg_color="transparent")
        backup_frame.pack(fill="x", padx=20)

        def export_lib():
             path = filedialog.asksaveasfilename(defaultextension=".json", filetypes=[("JSON files", "*.json")], title="Export Library")
             if path:
                 if self.data_manager.export_library(path):
                     messagebox.showinfo("Success", "Library exported successfully.")
                 else:
                     messagebox.showerror("Error", "Failed to export library.")

        def import_lib():
             path = filedialog.askopenfilename(filetypes=[("JSON files", "*.json")], title="Import Library")
             if path:
                 if self.data_manager.import_library(path):
                     messagebox.showinfo("Success", "Library imported successfully. Refreshing UI...")
                     self._populate_tree()
                     self._hide_right_pane()
                 else:
                     messagebox.showerror("Error", "Failed to import library.")

        ctk.CTkButton(backup_frame, text="Export Library", command=export_lib).pack(side="left", padx=(0, 10))
        ctk.CTkButton(backup_frame, text="Import Library", command=import_lib).pack(side="left")

if __name__ == "__main__":
    app = App()
    app.mainloop()
