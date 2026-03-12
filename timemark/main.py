import os
import time
import customtkinter as ctk
from tkinter import messagebox, filedialog
from typing import Optional, Any, List, Tuple
from pathlib import Path
import threading
from PIL import Image

from .data import DataManager, POSTER_CACHE_DIR
from .scanner import scan_directory
from .vlc_integration import get_vlc_path, play_in_vlc, get_vlc_status
from .tmdb_api import search_media, get_media_details, get_tv_season_episodes, download_poster
from .toast import ToastNotification

ctk.set_appearance_mode("dark")
ctk.set_default_color_theme("blue")

VLC_ORANGE = "#FF8800"
VLC_ORANGE_HOVER = "#E67A00"

class App(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.data_manager = DataManager()
        self.title("WatchMark Media Tracker")

        geo = self.data_manager.settings.get("window_geometry", "1200x800")
        pos = self.data_manager.settings.get("window_position", "+100+100")
        self.geometry(f"{geo}{pos}")

        if not self.data_manager.settings.get("vlc_path"):
            auto_path = get_vlc_path()
            if auto_path:
                self.data_manager.settings["vlc_path"] = auto_path
                self.data_manager.save_settings()

        self.protocol("WM_DELETE_WINDOW", self._on_closing)

        self.current_unmatched_files = []
        self._setup_layout()
        self._show_dashboard()

    def _on_closing(self):
        self.data_manager.settings["window_geometry"] = f"{self.winfo_width()}x{self.winfo_height()}"
        self.data_manager.settings["window_position"] = f"+{self.winfo_x()}+{self.winfo_y()}"
        self.data_manager.save_settings()
        self.destroy()

    def _setup_layout(self):
        self.grid_rowconfigure(0, weight=1)
        self.grid_columnconfigure(1, weight=1)

        # --- SIDEBAR ---
        self.sidebar_frame = ctk.CTkFrame(self, width=200, corner_radius=0)
        self.sidebar_frame.grid(row=0, column=0, sticky="nsew")
        self.sidebar_frame.grid_rowconfigure(6, weight=1) # Push settings to bottom

        logo_label = ctk.CTkLabel(self.sidebar_frame, text="WatchMark", font=ctk.CTkFont(size=20, weight="bold"))
        logo_label.grid(row=0, column=0, padx=20, pady=(20, 20))

        self.nav_btns = {}

        def create_nav_btn(row, text, command):
            btn = ctk.CTkButton(self.sidebar_frame, text=text, anchor="w", fg_color="transparent",
                                text_color=("gray10", "gray90"), hover_color=("gray70", "gray30"), command=command)
            btn.grid(row=row, column=0, padx=10, pady=5, sticky="ew")
            self.nav_btns[text] = btn
            return btn

        create_nav_btn(1, "🏠 Dashboard", self._show_dashboard)
        create_nav_btn(2, "📺 TV Shows", self._show_tv_shows)
        create_nav_btn(3, "🎬 Movies", self._show_movies)
        create_nav_btn(4, "🔍 Search", self._show_search)
        create_nav_btn(5, "❓ Unmatched Files", self._show_unmatched)

        create_nav_btn(7, "⚙️ Settings", self._show_settings)

        # --- MAIN CONTENT AREA ---
        self.main_frame = ctk.CTkFrame(self, corner_radius=0, fg_color="transparent")
        self.main_frame.grid(row=0, column=1, sticky="nsew")

    def _clear_main_frame(self):
        for widget in self.main_frame.winfo_children():
            widget.destroy()

    def _highlight_nav(self, active_text):
        for text, btn in self.nav_btns.items():
            if text == active_text:
                btn.configure(fg_color=("gray75", "gray25"))
            else:
                btn.configure(fg_color="transparent")

    # =========================================================================
    # DASHBOARD
    # =========================================================================
    def _show_dashboard(self):
        self._highlight_nav("🏠 Dashboard")
        self._clear_main_frame()

        ctk.CTkLabel(self.main_frame, text="Dashboard", font=ctk.CTkFont(size=24, weight="bold")).pack(anchor="w", padx=20, pady=20)

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        # Total Stats
        cursor.execute("SELECT COUNT(*) as count FROM Episodes WHERE status = 'Completed'")
        eps_watched = cursor.fetchone()['count']

        cursor.execute("SELECT COUNT(*) as count FROM History")
        total_watches = cursor.fetchone()['count']

        stats_frame = ctk.CTkFrame(self.main_frame)
        stats_frame.pack(fill="x", padx=20, pady=10)
        ctk.CTkLabel(stats_frame, text=f"Total Episodes Completed: {eps_watched}  |  Total Rewatches Logged: {total_watches}", font=ctk.CTkFont(size=16)).pack(pady=20)

        # Continue Watching (Episodes in 'Watching' status or next unwatched)
        ctk.CTkLabel(self.main_frame, text="Continue Watching", font=ctk.CTkFont(size=18, weight="bold")).pack(anchor="w", padx=20, pady=(20, 10))

        cw_frame = ctk.CTkScrollableFrame(self.main_frame, orientation="horizontal", height=200)
        cw_frame.pack(fill="x", padx=20)

        # 1. Get most recently watched shows based on History
        # 2. For those shows, get the lowest episode with status 'Watching' or 'Unwatched'
        cursor.execute("""
            SELECT e.media_id, MAX(h.timestamp) as last_watched
            FROM History h
            JOIN Episodes e ON h.episode_id = e.id
            GROUP BY e.media_id
            ORDER BY last_watched DESC
            LIMIT 10
        """)
        recent_media_ids = [r['media_id'] for r in cursor.fetchall()]

        cw_eps = []
        for m_id in recent_media_ids:
            cursor.execute("""
                SELECT e.*, m.title as show_title, m.poster_path
                FROM Episodes e
                JOIN Media m ON e.media_id = m.id
                WHERE e.media_id = ? AND e.status IN ('Watching', 'Unwatched')
                ORDER BY e.season_num ASC, e.ep_num ASC
                LIMIT 1
            """, (m_id,))
            ep = cursor.fetchone()
            if ep:
                cw_eps.append(ep)

        # If no history or no unwatched from history, just get some unwatched
        if not cw_eps:
             cursor.execute("""
                 SELECT e.*, m.title as show_title, m.poster_path
                 FROM Episodes e
                 JOIN Media m ON e.media_id = m.id
                 WHERE e.status IN ('Watching', 'Unwatched')
                 ORDER BY e.status DESC, e.season_num ASC, e.ep_num ASC
                 LIMIT 10
             """)
             cw_eps = cursor.fetchall()

        if not cw_eps:
            ctk.CTkLabel(cw_frame, text="Nothing to continue watching.").pack(padx=20, pady=20)
        else:
            for ep in cw_eps:
                self._create_episode_card(cw_frame, ep)

        conn.close()

    def _create_episode_card(self, parent, ep_row):
        card = ctk.CTkFrame(parent, width=150, height=200)
        card.pack(side="left", padx=10)
        card.pack_propagate(False)

        img_label = ctk.CTkLabel(card, text="No Image", width=130, height=100, fg_color="gray30")
        img_label.pack(pady=5)

        if ep_row['poster_path']:
            local_img = POSTER_CACHE_DIR / ep_row['poster_path'].lstrip('/')
            if local_img.exists():
                img = ctk.CTkImage(light_image=Image.open(local_img), dark_image=Image.open(local_img), size=(130, 100))
                img_label.configure(image=img, text="")

        title = f"{ep_row['show_title']}\nS{ep_row['season_num']}E{ep_row['ep_num']}"
        ctk.CTkLabel(card, text=title, font=ctk.CTkFont(size=12, weight="bold"), wraplength=130).pack(pady=5)

        if ep_row['status'] == 'Watching':
            if ep_row['runtime'] > 0:
                pct = int((ep_row['last_position'] / (ep_row['runtime'] * 60)) * 100)
                status_text = f"Watching ({pct}%)"
            else:
                status_text = "Watching (Resumable)"
        else:
            status_text = "Unwatched"
        ctk.CTkLabel(card, text=status_text, font=ctk.CTkFont(size=10), text_color="gray").pack()

        card.bind("<Button-1>", lambda e, eid=ep_row['media_id']: self._show_media_details(eid))
        for child in card.winfo_children():
            child.bind("<Button-1>", lambda e, eid=ep_row['media_id']: self._show_media_details(eid))

    # =========================================================================
    # TV SHOWS / MOVIES LIBRARY
    # =========================================================================
    def _show_tv_shows(self):
        self._highlight_nav("📺 TV Shows")
        self._show_library("TV")

    def _show_movies(self):
        self._highlight_nav("🎬 Movies")
        self._show_library("Movie")

    def _show_library(self, media_type):
        self._clear_main_frame()

        header_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        header_frame.pack(fill="x", padx=20, pady=20)

        title = "TV Shows" if media_type == "TV" else "Movies"
        ctk.CTkLabel(header_frame, text=title, font=ctk.CTkFont(size=24, weight="bold")).pack(side="left")

        scan_btn = ctk.CTkButton(header_frame, text="📂 Scan Local Folder", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, command=self._scan_folder)
        scan_btn.pack(side="right")

        grid_frame = ctk.CTkScrollableFrame(self.main_frame)
        grid_frame.pack(fill="both", expand=True, padx=20, pady=10)

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM Media WHERE type=?", (media_type,))
        media_items = cursor.fetchall()
        conn.close()

        if not media_items:
            ctk.CTkLabel(grid_frame, text=f"No {title} tracked yet. Use Search to add some!").pack(pady=50)
            return

        col = 0
        row = 0
        max_cols = 5

        for item in media_items:
            card = ctk.CTkFrame(grid_frame, width=160, height=280)
            card.grid(row=row, column=col, padx=10, pady=10)
            card.grid_propagate(False)

            img_label = ctk.CTkLabel(card, text="No Poster", width=140, height=210, fg_color="gray30")
            img_label.pack(pady=(10, 5))

            if item['poster_path']:
                local_img = POSTER_CACHE_DIR / item['poster_path'].lstrip('/')
                if local_img.exists():
                    img = ctk.CTkImage(light_image=Image.open(local_img), dark_image=Image.open(local_img), size=(140, 210))
                    img_label.configure(image=img, text="")

            ctk.CTkLabel(card, text=item['title'], font=ctk.CTkFont(weight="bold"), wraplength=140).pack()

            card.bind("<Button-1>", lambda e, mid=item['id']: self._show_media_details(mid))
            for child in card.winfo_children():
                child.bind("<Button-1>", lambda e, mid=item['id']: self._show_media_details(mid))

            col += 1
            if col >= max_cols:
                col = 0
                row += 1

    # =========================================================================
    # SEARCH & DISCOVER
    # =========================================================================
    def _show_search(self):
        self._pending_group_match = None # Clear pending matches when opening standard search
        self._highlight_nav("🔍 Search")
        self._clear_main_frame()

        top_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        top_frame.pack(fill="x", padx=20, pady=20)

        self.search_entry = ctk.CTkEntry(top_frame, placeholder_text="Search TMDB for Shows or Movies...", width=400)
        self.search_entry.pack(side="left", padx=(0, 10))
        self.search_entry.bind("<Return>", lambda e: self._perform_search())

        ctk.CTkButton(top_frame, text="Search", command=self._perform_search).pack(side="left")

        self.results_frame = ctk.CTkScrollableFrame(self.main_frame)
        self.results_frame.pack(fill="both", expand=True, padx=20, pady=10)

    def _perform_search(self):
        query = self.search_entry.get().strip()
        if not query: return

        api_key = self.data_manager.settings.get("tmdb_api_key")
        if not api_key:
            messagebox.showwarning("Missing API Key", "Please add your TMDB API Key in Settings first.")
            return

        for widget in self.results_frame.winfo_children():
            widget.destroy()

        ctk.CTkLabel(self.results_frame, text="Searching...").pack(pady=20)
        self.update()

        def run_search():
            import requests
            try:
                results = search_media(api_key, query)
                self.after(0, lambda: self._display_search_results(results))
            except requests.exceptions.HTTPError as e:
                if e.response.status_code == 401:
                    self.after(0, lambda: messagebox.showerror("API Error", "Invalid TMDB API Key. Please check your settings."))
                else:
                    self.after(0, lambda: ToastNotification(self, title="API Error", message=f"TMDB returned an error: {e.response.status_code}", duration=5000, color="#b71c1c"))
                self.after(0, self._clear_results_frame)
            except Exception as e:
                self.after(0, lambda: ToastNotification(self, title="Offline Mode", message="Cannot reach TMDB.", duration=5000, color="#b71c1c"))
                self.after(0, self._clear_results_frame)

        threading.Thread(target=run_search, daemon=True).start()

    def _clear_results_frame(self):
        for widget in self.results_frame.winfo_children():
            widget.destroy()

    def _display_search_results(self, results):
        self._clear_results_frame()

        if not results:
            ctk.CTkLabel(self.results_frame, text="No results found.").pack(pady=20)
            return

        for i, res in enumerate(results):
            row = ctk.CTkFrame(self.results_frame)
            row.pack(fill="x", pady=5)

            info = f"[{res['type']}] {res['title']} ({res['release_date'][:4] if res['release_date'] else 'N/A'})"
            ctk.CTkLabel(row, text=info, font=ctk.CTkFont(weight="bold")).pack(side="left", padx=10, pady=10)

            btn = ctk.CTkButton(row, text="+ Add to Tracker", command=lambda r=res: self._add_to_tracker(r))
            btn.pack(side="right", padx=10, pady=10)

    def _add_to_tracker(self, media_data):
        api_key = self.data_manager.settings.get("tmdb_api_key")

    def _show_media_details(self, media_id):
        self._clear_main_frame()

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        cursor.execute("SELECT * FROM Media WHERE id=?", (media_id,))
        media = cursor.fetchone()

        if not media:
            conn.close()
            return

        # Header
        header = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        header.pack(fill="x", padx=20, pady=20)

        img_label = ctk.CTkLabel(header, text="No Poster", width=150, height=225, fg_color="gray30")
        img_label.pack(side="left", padx=(0, 20))

        if media['poster_path']:
            local_img = POSTER_CACHE_DIR / media['poster_path'].lstrip('/')
            if local_img.exists():
                img = ctk.CTkImage(light_image=Image.open(local_img), dark_image=Image.open(local_img), size=(150, 225))
                img_label.configure(image=img, text="")

        info_frame = ctk.CTkFrame(header, fg_color="transparent")
        info_frame.pack(side="left", fill="both", expand=True)

        ctk.CTkLabel(info_frame, text=media['title'], font=ctk.CTkFont(size=28, weight="bold")).pack(anchor="w")
        ctk.CTkLabel(info_frame, text=media['synopsis'], wraplength=700, justify="left").pack(anchor="w", pady=10)

        # Stats Bar
        cursor.execute("SELECT COUNT(*) as c FROM Episodes WHERE media_id=? AND status='Completed'", (media_id,))
        watched_eps = cursor.fetchone()['c']

        cursor.execute("SELECT SUM(watch_count) as s FROM Episodes WHERE media_id=?", (media_id,))
        total_watches_row = cursor.fetchone()
        total_watches = total_watches_row['s'] if total_watches_row['s'] else 0

        stats_text = f"Episodes Watched: {watched_eps} / {media['total_episodes']}  |  Total Rewatches: {total_watches}"
        ctk.CTkLabel(info_frame, text=stats_text, font=ctk.CTkFont(weight="bold", text_color=VLC_ORANGE)).pack(anchor="w", pady=10)

        # Main Area (Tabs for Seasons if TV)
        content_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        content_frame.pack(fill="both", expand=True, padx=20, pady=10)

        if media['type'] == 'TV':
            cursor.execute("SELECT DISTINCT season_num FROM Episodes WHERE media_id=? ORDER BY season_num", (media_id,))
            seasons = [r['season_num'] for r in cursor.fetchall()]

            if seasons:
                # Top horizontal scroll for season buttons
                season_scroll = ctk.CTkScrollableFrame(content_frame, orientation="horizontal", height=50)
                season_scroll.pack(fill="x", pady=(0, 10))

                self.ep_list_frame = ctk.CTkScrollableFrame(content_frame)
                self.ep_list_frame.pack(fill="both", expand=True)

                for s in seasons:
                    btn = ctk.CTkButton(season_scroll, text=f"Season {s}", width=80,
                                        command=lambda s_num=s, m_id=media_id: self._load_episodes(m_id, s_num))
                    btn.pack(side="left", padx=5)

                # Load first season by default
                self._load_episodes(media_id, seasons[0])
        else:
            self.ep_list_frame = ctk.CTkScrollableFrame(content_frame)
            self.ep_list_frame.pack(fill="both", expand=True)
            self._load_episodes(media_id, 1)

        conn.close()

    def _load_episodes(self, media_id, season_num):
        for widget in self.ep_list_frame.winfo_children():
            widget.destroy()

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        cursor.execute("""
            SELECT e.*, l.file_path
            FROM Episodes e
            LEFT JOIN Local_Files l ON e.id = l.episode_id
            WHERE e.media_id=? AND e.season_num=?
            ORDER BY e.ep_num
        """, (media_id, season_num))

        episodes = cursor.fetchall()
        conn.close()

        for ep in episodes:
            row = ctk.CTkFrame(self.ep_list_frame)
            row.pack(fill="x", pady=2)

            # Status Icon
            icon = "⬛" # Unwatched
            if ep['status'] == 'Completed':
                icon = "✅"
            elif ep['status'] == 'Watching':
                icon = "⏳"

            ctk.CTkLabel(row, text=icon, width=30).pack(side="left", padx=5)

            # Title
            title_text = f"{ep['ep_num']}. {ep['title']}"
            color = "white" if ep['status'] == 'Completed' else "gray"
            ctk.CTkLabel(row, text=title_text, width=250, anchor="w", text_color=color).pack(side="left", padx=10)

            # Play Button
            has_file = bool(ep['file_path'])
            play_color = VLC_ORANGE if has_file else "gray30"
            play_hover = VLC_ORANGE_HOVER if has_file else "gray30"

            play_btn = ctk.CTkButton(row, text="▶", width=40, fg_color=play_color, hover_color=play_hover,
                                     command=lambda e=ep: self._play_episode(e) if e['file_path'] else None)
            play_btn.pack(side="left", padx=10)

            # Watch Count controls
            ctk.CTkButton(row, text="-", width=30, fg_color="gray", command=lambda e_id=ep['id'], m_id=media_id, s=season_num: self._adj_watch(e_id, m_id, s, -1)).pack(side="left", padx=2)
            ctk.CTkLabel(row, text=f"Count: {ep['watch_count']}", width=70).pack(side="left", padx=5)
            ctk.CTkButton(row, text="+", width=30, fg_color="gray", command=lambda e_id=ep['id'], m_id=media_id, s=season_num: self._adj_watch(e_id, m_id, s, 1)).pack(side="left", padx=2)

    def _adj_watch(self, episode_id, media_id, season_num, delta):
        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        cursor.execute("SELECT watch_count FROM Episodes WHERE id=?", (episode_id,))
        count = cursor.fetchone()['watch_count']

        new_count = max(0, count + delta)
        status = 'Completed' if new_count > 0 else 'Unwatched'

        cursor.execute("UPDATE Episodes SET watch_count=?, status=? WHERE id=?", (new_count, status, episode_id))
        if delta > 0:
            cursor.execute("INSERT INTO History (episode_id) VALUES (?)", (episode_id,))

        conn.commit()
        conn.close()

        self._load_episodes(media_id, season_num)

    # =========================================================================
    # VLC PLAYBACK & TRACKING
    # =========================================================================
    def _play_episode(self, ep_data):
        vlc_path = self.data_manager.settings.get("vlc_path")
        if not vlc_path:
            messagebox.showwarning("VLC Path Not Set", "Please set VLC path in Settings.")
            return

        file_path = ep_data['file_path']
        if not os.path.exists(file_path):
            messagebox.showerror("File Missing", "The linked file no longer exists.")
            return

        # Start time
        start_sec = 0
        if ep_data['last_position'] > 0:
            start_sec = int(ep_data['last_position'])

        proc = play_in_vlc(vlc_path, file_path, start_time=start_sec)

        if proc:
            threading.Thread(target=self._vlc_heartbeat, args=(proc, ep_data['id'], ep_data['media_id'], ep_data['season_num']), daemon=True).start()

    def _vlc_heartbeat(self, proc, episode_id, media_id, season_num):
        high_water_mark = 0.0
        last_time_seconds = 0.0

        while proc.poll() is None:
            time.sleep(5)
            status = get_vlc_status()
            if status and status['length'] > 0:
                pos = status['time'] / status['length']
                if pos > high_water_mark:
                    high_water_mark = pos
                last_time_seconds = status['time']

        # Process closed
        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        if high_water_mark > 0.90:
            cursor.execute("""
                UPDATE Episodes
                SET watch_count = watch_count + 1, status = 'Completed', last_position = 0.0
                WHERE id = ?
            """, (episode_id,))
            cursor.execute("INSERT INTO History (episode_id) VALUES (?)", (episode_id,))
        else:
            # If we didn't hit 90%, but we watched something, mark as watching
            if high_water_mark > 0.05: # At least 5% to avoid accidental clicks
                cursor.execute("""
                    UPDATE Episodes
                    SET status = 'Watching', last_position = ?
                    WHERE id = ? AND status != 'Completed'
                """, (last_time_seconds, episode_id))

        conn.commit()
        conn.close()

        # Refresh UI if we are still on that page
        self.after(0, lambda: self._refresh_if_on_episode(media_id, season_num))

    def _refresh_if_on_episode(self, media_id, season_num):
        # We check if the ep_list_frame exists and is visible. Simplistic refresh.
        if hasattr(self, 'ep_list_frame') and self.ep_list_frame.winfo_exists():
            self._load_episodes(media_id, season_num)

    # =========================================================================
    # SCANNER & UNMATCHED
    # =========================================================================
    def _scan_folder(self):
        folder = filedialog.askdirectory(title="Select Media Folder")
        if folder:
            # Let the user know scanning started without freezing the UI completely
            ToastNotification(self, title="Scan Started", message=f"Scanning {folder} in background...", duration=3000)

            def run_scan():
                try:
                    unmatched = scan_directory(folder, self.data_manager)
                    self.after(0, lambda: self._finish_scan(unmatched))
                except Exception as e:
                    self.after(0, lambda: messagebox.showerror("Scan Error", str(e)))

            threading.Thread(target=run_scan, daemon=True).start()

    def _finish_scan(self, unmatched):
        self.current_unmatched_files.extend(unmatched)
        ToastNotification(self, title="Scan Complete", message=f"Finished. Found {len(unmatched)} unmatched files.", duration=4000, color="#1b5e20")
        # If user is currently looking at unmatched list, refresh it
        if hasattr(self, 'nav_btns') and self.nav_btns["❓ Unmatched Files"].cget("fg_color") == ("gray75", "gray25"):
            self._show_unmatched()

    def _show_unmatched(self):
        self._highlight_nav("❓ Unmatched Files")
        self._clear_main_frame()

        ctk.CTkLabel(self.main_frame, text="Unmatched Files", font=ctk.CTkFont(size=24, weight="bold")).pack(anchor="w", padx=20, pady=20)

        if not self.current_unmatched_files:
            ctk.CTkLabel(self.main_frame, text="No unmatched files.").pack(pady=20)
            return

        scroll = ctk.CTkScrollableFrame(self.main_frame)
        scroll.pack(fill="both", expand=True, padx=20, pady=10)

        # Group files
        groups = {}
        for idx, uf in enumerate(self.current_unmatched_files):
            g_key = uf.get('group_key', 'Unknown')
            if g_key not in groups:
                groups[g_key] = []
            groups[g_key].append((idx, uf))

        for group_name, files in groups.items():
            group_frame = ctk.CTkFrame(scroll, fg_color="transparent")
            group_frame.pack(fill="x", pady=5)

            header_frame = ctk.CTkFrame(group_frame, fg_color=("gray85", "gray15"))
            header_frame.pack(fill="x")

            # Label
            label_text = f"📁 {group_name} — {len(files)} files detected"
            header_label = ctk.CTkLabel(header_frame, text=label_text, font=ctk.CTkFont(weight="bold"))
            header_label.pack(side="left", padx=10, pady=5)

            # Content frame to be toggled
            content_frame = ctk.CTkFrame(group_frame, fg_color="transparent")

            # Toggle logic
            def toggle(e, cf=content_frame):
                if cf.winfo_ismapped():
                    cf.pack_forget()
                else:
                    cf.pack(fill="x", pady=(5, 0))

            header_frame.bind("<Button-1>", toggle)
            header_label.bind("<Button-1>", toggle)

            # Search & Match All
            match_btn = ctk.CTkButton(header_frame, text="🔍 Search & Match All",
                                      command=lambda gn=group_name, fs=files: self._match_group(gn, fs))
            match_btn.pack(side="right", padx=10, pady=5)

            # Files inside group
            for idx, uf in files:
                row = ctk.CTkFrame(content_frame)
                row.pack(fill="x", pady=2, padx=(20, 0))
                ctk.CTkLabel(row, text=uf['filename'], width=400, anchor="w").pack(side="left", padx=10)
                ctk.CTkButton(row, text="Assign...", command=lambda idx=idx: self._assign_unmatched(idx)).pack(side="right", padx=10)

    def _match_group(self, group_name, files):
        # Trigger TMDB search with the group name
        self._show_search()
        self.search_entry.delete(0, 'end')
        self.search_entry.insert(0, group_name)

        # Override the _add_to_tracker slightly for this workflow
        # Instead of just adding, we also assign the files in this group.
        # We can do this by setting a state or passing a callback.
        self._pending_group_match = files
        self._perform_search()

    def _add_to_tracker(self, media_data):
        api_key = self.data_manager.settings.get("tmdb_api_key")

        def fetch_and_save():
            try:
                # 1. Fetch details
                details = get_media_details(api_key, media_data['tmdb_id'], media_data['type'])
                if not details: return

                # Download Poster
                if details['poster_path']:
                    download_poster(details['poster_path'])

                conn = self.data_manager.get_db_connection()
                cursor = conn.cursor()

                # Check if exists
                cursor.execute("SELECT id FROM Media WHERE tmdb_id=?", (details['tmdb_id'],))
                existing = cursor.fetchone()

                media_id = None
                if existing:
                    media_id = existing['id']
                else:
                    # Insert Media
                    cursor.execute("""
                        INSERT INTO Media (tmdb_id, type, title, synopsis, poster_path, total_episodes, status)
                        VALUES (?, ?, ?, ?, ?, ?, ?)
                    """, (details['tmdb_id'], details['type'], details['title'], details['synopsis'],
                          details['poster_path'], details['total_episodes'], details['status']))

                    media_id = cursor.lastrowid

                    # Fetch Episodes if TV Show
                    if details['type'] == 'TV':
                        for season in details['seasons']:
                            s_num = season.get('season_number')
                            if s_num == 0: continue # Skip specials usually

                            eps = get_tv_season_episodes(api_key, details['tmdb_id'], s_num)
                            for ep in eps:
                                cursor.execute("""
                                    INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime)
                                    VALUES (?, ?, ?, ?, ?)
                                """, (media_id, s_num, ep['ep_num'], ep['title'], ep['runtime']))
                    else:
                        # Movie has 1 dummy episode
                        cursor.execute("""
                            INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime)
                            VALUES (?, 1, 1, ?, ?)
                        """, (media_id, 1, 1, details['title'], details.get('runtime', 0)))

                    conn.commit()

                # Check for pending group match
                pending_group = getattr(self, '_pending_group_match', None)
                if pending_group:
                    import sqlite3
                    assigned_count = 0
                    files_to_remove = []

                    for idx, uf in pending_group:
                        s_num = uf.get('parsed_season')
                        e_num = uf.get('parsed_episode')

                        if details['type'] == 'Movie':
                            s_num, e_num = 1, 1

                        if s_num is not None and e_num is not None:
                            cursor.execute("SELECT id FROM Episodes WHERE media_id=? AND season_num=? AND ep_num=?", (media_id, s_num, e_num))
                            ep = cursor.fetchone()
                            if ep:
                                try:
                                    cursor.execute("""
                                        INSERT INTO Local_Files (episode_id, file_path) VALUES (?, ?)
                                        ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path
                                    """, (ep['id'], uf['file_path']))
                                    assigned_count += 1
                                    files_to_remove.append(uf)
                                except sqlite3.IntegrityError:
                                    pass

                    conn.commit()

                    if files_to_remove:
                        self.current_unmatched_files = [f for f in self.current_unmatched_files if f not in files_to_remove]

                    self.after(0, lambda ac=assigned_count: messagebox.showinfo("Success", f"Added {details['title']} and assigned {ac} files!"))
                    self._pending_group_match = None
                else:
                    if not existing:
                        self.after(0, lambda: messagebox.showinfo("Success", f"Added {details['title']} to tracker!"))
                    else:
                        self.after(0, lambda: messagebox.showinfo("Exists", "This media is already tracked."))

                conn.close()
                self.after(0, self._refresh_if_on_unmatched)

            except Exception as e:
                self.after(0, lambda: messagebox.showerror("Error", f"Failed to add media: {e}"))

        threading.Thread(target=fetch_and_save, daemon=True).start()

    def _refresh_if_on_unmatched(self):
        if hasattr(self, 'nav_btns') and self.nav_btns["❓ Unmatched Files"].cget("fg_color") == ("gray75", "gray25"):
            self._show_unmatched()

    # =========================================================================
    # MEDIA DEEP DIVE
    # =========================================================================

    def _assign_unmatched(self, uf_idx):
        # Extremely simplified assignment logic for MVP
        uf = self.current_unmatched_files[uf_idx]

        dialog = ctk.CTkToplevel(self)
        dialog.title("Assign File")
        dialog.geometry("400x300")
        dialog.grab_set()

        ctk.CTkLabel(dialog, text=f"Assigning: {uf['filename']}").pack(pady=10)

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()
        cursor.execute("SELECT id, title, type FROM Media ORDER BY title")
        media_items = cursor.fetchall()
        conn.close()

        if not media_items:
            ctk.CTkLabel(dialog, text="No Media tracked in DB.").pack(pady=10)
            return

        media_names = [f"[{m['type']}] {m['title']}" for m in media_items]
        media_map = {f"[{m['type']}] {m['title']}": (m['id'], m['type']) for m in media_items}

        media_var = ctk.StringVar(value=media_names[0])
        opt = ctk.CTkOptionMenu(dialog, variable=media_var, values=media_names)
        opt.pack(pady=10)

        s_entry = ctk.CTkEntry(dialog, placeholder_text="Season (e.g. 1)")
        s_entry.pack(pady=5)

        e_entry = ctk.CTkEntry(dialog, placeholder_text="Episode (e.g. 1)")
        e_entry.pack(pady=5)

        # Pre-fill for TV shows
        if uf['parsed_season'] is not None:
            s_entry.insert(0, str(uf['parsed_season']))
        if uf['parsed_episode'] is not None:
            e_entry.insert(0, str(uf['parsed_episode']))

        def search_tmdb():
            dialog.destroy()
            self._show_search()
            # _show_search recreates self.search_entry, so we interact with it *after* calling it
            self.search_entry.delete(0, 'end')
            self.search_entry.insert(0, uf['parsed_series'] if uf['parsed_series'] else uf['filename'])
            self._perform_search()

        ctk.CTkButton(dialog, text="Search TMDB", command=search_tmdb, fg_color="#3a7ebf", hover_color="#2b5e8f").pack(pady=(10, 5))

        def save():
            try:
                m_id, m_type = media_map[media_var.get()]

                if m_type == "Movie":
                    s_num, e_num = 1, 1
                else:
                    s_num = int(s_entry.get())
                    e_num = int(e_entry.get())

                conn = self.data_manager.get_db_connection()
                cursor = conn.cursor()
                cursor.execute("SELECT id FROM Episodes WHERE media_id=? AND season_num=? AND ep_num=?", (m_id, s_num, e_num))
                ep = cursor.fetchone()

                if ep:
                    cursor.execute("""
                        INSERT INTO Local_Files (episode_id, file_path) VALUES (?, ?)
                        ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path
                    """, (ep['id'], uf['file_path']))
                    conn.commit()
                    conn.close()

                    self.current_unmatched_files.pop(uf_idx)
                    dialog.destroy()
                    self._show_unmatched()
                else:
                    messagebox.showerror("Error", "Episode does not exist in DB.")
                    conn.close()
            except Exception as e:
                messagebox.showerror("Error", str(e))

        ctk.CTkButton(dialog, text="Save", command=save, fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER).pack(pady=(5, 20))

    # =========================================================================
    # SETTINGS
    # =========================================================================
    def _show_settings(self):
        self._highlight_nav("⚙️ Settings")
        self._clear_main_frame()

        ctk.CTkLabel(self.main_frame, text="Settings", font=ctk.CTkFont(size=24, weight="bold")).pack(anchor="w", padx=20, pady=20)

        form_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        form_frame.pack(fill="x", padx=20)

        # TMDB Key
        ctk.CTkLabel(form_frame, text="TMDB API Key:", font=ctk.CTkFont(weight="bold")).pack(anchor="w", pady=(10, 5))
        tmdb_entry = ctk.CTkEntry(form_frame, width=400)
        tmdb_entry.pack(anchor="w")
        tmdb_entry.insert(0, self.data_manager.settings.get("tmdb_api_key", ""))

        # VLC Path
        ctk.CTkLabel(form_frame, text="VLC Executable Path:", font=ctk.CTkFont(weight="bold")).pack(anchor="w", pady=(20, 5))
        vlc_frame = ctk.CTkFrame(form_frame, fg_color="transparent")
        vlc_frame.pack(fill="x")

        vlc_entry = ctk.CTkEntry(vlc_frame, width=320)
        vlc_entry.pack(side="left")
        vlc_entry.insert(0, self.data_manager.settings.get("vlc_path", ""))

        def browse():
            p = filedialog.askopenfilename()
            if p:
                vlc_entry.delete(0, 'end')
                vlc_entry.insert(0, p)

        ctk.CTkButton(vlc_frame, text="Browse", width=70, command=browse).pack(side="left", padx=10)

        def save():
            self.data_manager.settings["tmdb_api_key"] = tmdb_entry.get().strip()
            self.data_manager.settings["vlc_path"] = vlc_entry.get().strip()
            self.data_manager.save_settings()
            messagebox.showinfo("Saved", "Settings saved successfully.")

        ctk.CTkButton(self.main_frame, text="Save Settings", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, command=save).pack(pady=40, padx=20, anchor="w")

if __name__ == "__main__":
    app = App()
    app.mainloop()
