import os
import time

import customtkinter as ctk

# --- MONKEY PATCH TURBO SCROLL FOR CTKSCROLLABLEFRAME ---
original_mouse_wheel = ctk.windows.widgets.ctk_scrollable_frame.CTkScrollableFrame._mouse_wheel_all
def turbo_mouse_wheel(self, event):
    if self.winfo_exists():
        # Execute the original scroll method multiple times per physical scroll event
        # to ensure a universally fast scroll across all OS (Windows, macOS, Linux)
        for _ in range(5):
            original_mouse_wheel(self, event)
    return

ctk.windows.widgets.ctk_scrollable_frame.CTkScrollableFrame._mouse_wheel_all = turbo_mouse_wheel
# --------------------------------------------------------
from tkinter import messagebox, filedialog
from typing import Optional, Any, List, Tuple
from pathlib import Path
import threading
from PIL import Image

from .data import DataManager, POSTER_CACHE_DIR
from .scanner import scan_directory
from .vlc_integration import get_vlc_path, play_in_vlc, get_vlc_status
from .tmdb_api import search_media, get_media_details, get_tv_season_episodes, download_image
from .toast import ToastNotification

ctk.set_appearance_mode("dark")
ctk.set_default_color_theme("blue")

VLC_ORANGE = "#FF6B00"
VLC_ORANGE_HOVER = "#E66000"
BG_COLOR = "#0D0F14"
SURFACE_COLOR = "#1F222A"
TEXT_PRIMARY = "#FFFFFF"
TEXT_SECONDARY = "#8E929C"
SUCCESS_COLOR = "#1b5e20"
DANGER_COLOR = "#b71c1c"

class App(ctk.CTk):
    def __init__(self):
        super().__init__()
        self.data_manager = DataManager()
        self.title("WatchMark Media Tracker")

        # Configure root app colors
        self.configure(fg_color=BG_COLOR)

        # Load custom fonts
        font_dir = Path(__file__).parent / "assets" / "fonts"
        if font_dir.exists():
            for f_name in ["Inter-Regular.ttf", "Inter-Medium.ttf", "Inter-Bold.ttf"]:
                f_path = font_dir / f_name
                if f_path.exists():
                    ctk.FontManager.load_font(str(f_path))

        geo = self.data_manager.settings.get("window_geometry", "1200x800")
        pos = self.data_manager.settings.get("window_position", "+100+100")
        self.geometry(f"{geo}{pos}")

        if not self.data_manager.settings.get("vlc_path"):
            auto_path = get_vlc_path()
            if auto_path:
                self.data_manager.settings["vlc_path"] = auto_path
                self.data_manager.save_settings()

        self.protocol("WM_DELETE_WINDOW", self._on_closing)

        self._setup_layout()
        self._show_dashboard()

    def _on_closing(self):
        self.data_manager.settings["window_geometry"] = f"{self.winfo_width()}x{self.winfo_height()}"
        self.data_manager.settings["window_position"] = f"+{self.winfo_x()}+{self.winfo_y()}"
        self.data_manager.save_settings()
        self.destroy()

    def _setup_layout(self):
        self.grid_rowconfigure(1, weight=1)
        self.grid_columnconfigure(1, weight=1)

        # --- TOP BAR (Custom Header below OS native) ---
        self.top_bar = ctk.CTkFrame(self, height=60, corner_radius=0, fg_color=BG_COLOR)
        self.top_bar.grid(row=0, column=0, columnspan=2, sticky="ew")
        self.top_bar.grid_columnconfigure(1, weight=1)
        self.top_bar.grid_propagate(False)

        # Search Bar in Top Bar
        search_container = ctk.CTkFrame(self.top_bar, fg_color="transparent")
        search_container.grid(row=0, column=1, pady=10)

        self.quick_search_var = ctk.StringVar()
        self.quick_search_entry = ctk.CTkEntry(
            search_container,
            textvariable=self.quick_search_var,
            placeholder_text="Quick Search Local Library...",
            width=350,
            height=36,
            corner_radius=18,
            fg_color=SURFACE_COLOR,
            border_color="#333",
            font=("Inter", 13, "normal")
        )
        self.quick_search_entry.pack(side="left")
        self.quick_search_entry.bind("<KeyRelease>", self._handle_quick_search)
        self.quick_search_entry.bind("<Return>", self._handle_quick_search)

        # --- SIDEBAR ---
        self.sidebar_frame = ctk.CTkFrame(self, width=220, corner_radius=0, fg_color=BG_COLOR)
        self.sidebar_frame.grid(row=1, column=0, sticky="nsew")
        self.sidebar_frame.grid_propagate(False) # Keep width consistent

        # Navigation container (top)
        nav_container = ctk.CTkFrame(self.sidebar_frame, fg_color="transparent")
        nav_container.pack(side="top", fill="both", expand=True)

        logo_label = ctk.CTkLabel(nav_container, text="▶ WatchMark", font=("Inter", 22, "bold"), text_color=TEXT_PRIMARY)
        logo_label.pack(padx=20, pady=(20, 30), anchor="w")

        self.nav_btns = {}
        self.nav_indicators = {}

        def create_nav_btn(text, command):
            container = ctk.CTkFrame(nav_container, fg_color="transparent", height=40)
            container.pack(fill="x", pady=2)
            container.pack_propagate(False)

            indicator = ctk.CTkFrame(container, width=4, corner_radius=0, fg_color="transparent")
            indicator.pack(side="left", fill="y")

            btn = ctk.CTkButton(container, text=text, anchor="w", fg_color="transparent",
                                text_color=TEXT_SECONDARY, hover_color=SURFACE_COLOR, command=command,
                                font=("Inter", 14, "bold"))
            btn.pack(side="left", fill="both", expand=True, padx=(10, 15))

            self.nav_btns[text] = btn
            self.nav_indicators[text] = indicator
            return btn

        create_nav_btn("Dashboard", self._show_dashboard)
        create_nav_btn("TV Shows", self._show_tv_shows)
        create_nav_btn("Movies", self._show_movies)
        create_nav_btn("Search", self._show_search)
        create_nav_btn("Inbox", self._show_unmatched)
        create_nav_btn("🕒 History", self._show_history)

        # Bottom section: Settings & Status
        bottom_frame = ctk.CTkFrame(self.sidebar_frame, fg_color="transparent")
        bottom_frame.pack(side="bottom", fill="x", pady=(0, 20), padx=20)

        settings_btn = ctk.CTkButton(bottom_frame, text="⚙️ Settings", anchor="w", fg_color="transparent",
                                     text_color=TEXT_SECONDARY, hover_color=SURFACE_COLOR, command=self._show_settings,
                                     font=("Inter", 14, "bold"))
        settings_btn.pack(fill="x", pady=(0, 10))
        self.nav_btns["Settings"] = settings_btn

        # Dummy status indicator
        status_lbl = ctk.CTkLabel(bottom_frame, text="🟢 DB Connected", font=("Inter", 11, "normal"), text_color=SUCCESS_COLOR)
        status_lbl.pack(anchor="w", padx=10)

        # --- MAIN CONTENT AREA ---
        self.main_frame = ctk.CTkFrame(self, corner_radius=0, fg_color=BG_COLOR)
        self.main_frame.grid(row=1, column=1, sticky="nsew")

    def _clear_main_frame(self):
        for widget in self.main_frame.winfo_children():
            widget.destroy()

    def _highlight_nav(self, active_text):
        for text, btn in self.nav_btns.items():
            if text == active_text:
                if text != "Settings":
                    btn.configure(fg_color=SURFACE_COLOR, text_color=TEXT_PRIMARY)
                    self.nav_indicators[text].configure(fg_color=VLC_ORANGE)
                else:
                    btn.configure(fg_color=SURFACE_COLOR, text_color=TEXT_PRIMARY)
            else:
                if text != "Settings":
                    btn.configure(fg_color="transparent", text_color=TEXT_SECONDARY)
                    self.nav_indicators[text].configure(fg_color="transparent")
                else:
                    btn.configure(fg_color="transparent", text_color=TEXT_SECONDARY)

    def _handle_quick_search(self, event):
        query = self.quick_search_var.get().strip().lower()
        if not query:
            return

        # Route to a generic library view that searches both TV and Movies
        self._show_library("All", filter_query=query)

    # =========================================================================
    # DASHBOARD
    # =========================================================================
    def _show_dashboard(self):
        self._highlight_nav("Dashboard")
        self._clear_main_frame()

        # Make main frame scrollable for dashboard
        dash_scroll = ctk.CTkScrollableFrame(self.main_frame, fg_color="transparent")
        dash_scroll.pack(fill="both", expand=True)

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        # 1. Fetch the most recently engaged SHOW that still has unwatched episodes
        cursor.execute("""
            SELECT e.media_id, MAX(h.timestamp) as last_watched
            FROM History h
            JOIN Episodes e ON h.episode_id = e.id
            WHERE EXISTS (
                SELECT 1 FROM Episodes e2
                WHERE e2.media_id = e.media_id AND e2.status IN ('Watching', 'Unwatched')
            )
            GROUP BY e.media_id
            ORDER BY last_watched DESC
            LIMIT 1
        """)
        hero_media = cursor.fetchone()

        hero_ep = None
        if hero_media:
            # Get the NEXT unwatched or currently watching episode for this media
            cursor.execute("""
                SELECT e.*, m.title as show_title, m.backdrop_path, l.file_path, m.type as media_type, e.still_path
                FROM Episodes e
                JOIN Media m ON e.media_id = m.id
                LEFT JOIN Local_Files l ON e.id = l.episode_id
                WHERE e.media_id = ? AND e.status IN ('Watching', 'Unwatched')
                ORDER BY e.season_num ASC, e.ep_num ASC
                LIMIT 1
            """, (hero_media['media_id'],))
            hero_ep = cursor.fetchone()

        if hero_ep:
            # Render Hero Section
            hero_frame = ctk.CTkFrame(dash_scroll, height=350, fg_color=SURFACE_COLOR, corner_radius=12)
            hero_frame.pack(fill="x", padx=20, pady=(20, 10))
            hero_frame.pack_propagate(False)

            bg_label = ctk.CTkLabel(hero_frame, text="")
            bg_label.place(x=0, y=0, relwidth=1.0, relheight=1.0)

            def load_hero_bg(ep_data, label):
                if ep_data['backdrop_path']:
                    from .tmdb_api import download_image
                    local_img_path = download_image(ep_data['backdrop_path'], size="w1280")
                    if local_img_path:
                        try:
                            pil_img = Image.open(local_img_path)
                            w, h = pil_img.size
                            target_h = int(w * (350/1000))
                            if h > target_h:
                                top = (h - target_h) // 2
                                pil_img = pil_img.crop((0, top, w, top + target_h))
                            backdrop_img = ctk.CTkImage(light_image=pil_img, dark_image=pil_img, size=(1000, 350))
                            self.after(0, lambda: label.configure(image=backdrop_img) if label.winfo_exists() else None)
                        except Exception:
                            pass
            threading.Thread(target=load_hero_bg, args=(hero_ep, bg_label), daemon=True).start()

            # Text content
            content_frame = ctk.CTkFrame(hero_frame, fg_color="transparent")
            content_frame.place(relx=0.05, rely=0.5, anchor="w")

            ctk.CTkLabel(content_frame, text="UP NEXT", font=("Inter", 14, "bold"), text_color=VLC_ORANGE).pack(anchor="w")
            ctk.CTkLabel(content_frame, text=(hero_ep['show_title'] or 'Unknown Show'), font=("Inter", 48, "bold"), text_color=TEXT_PRIMARY).pack(anchor="w", pady=(5, 0))

            if hero_ep['media_type'] == 'TV':
                ep_sub = f"S{hero_ep['season_num']:02}E{hero_ep['ep_num']:02} - {(hero_ep['title'] or 'Unknown Title')}"
            else:
                ep_sub = (hero_ep['title'] or 'Unknown Title')

            ctk.CTkLabel(content_frame, text=ep_sub, font=("Inter", 18, "normal"), text_color=TEXT_SECONDARY).pack(anchor="w", pady=(0, 20))

            if hero_ep['file_path']:
                play_btn = ctk.CTkButton(content_frame, text="▶ Resume", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, height=45, width=150,
                                         font=("Inter", 16, "bold"),
                                         command=lambda e=hero_ep: self._play_episode(e))
                play_btn.pack(anchor="w")
            else:
                play_btn = ctk.CTkButton(content_frame, text="❌ Missing File", fg_color=DANGER_COLOR, hover_color="#8e0000", height=45, width=150,
                                         font=("Inter", 16, "bold"), state="disabled")
                play_btn.pack(anchor="w")

            # Progress bar
            if hero_ep['status'] == 'Watching' and (hero_ep['runtime'] or 0) > 0:
                progress_val = min(1.0, hero_ep['last_position'] / ((hero_ep['runtime'] or 0) * 60))
                prog_bar = ctk.CTkProgressBar(content_frame, width=300, height=6, progress_color=VLC_ORANGE, fg_color=SURFACE_COLOR)
                prog_bar.pack(anchor="w", pady=(15, 0))
                prog_bar.set(progress_val)
        else:
            # Empty state hero
            hero_frame = ctk.CTkFrame(dash_scroll, height=300, fg_color=SURFACE_COLOR, corner_radius=12)
            hero_frame.pack(fill="x", padx=20, pady=(20, 10))
            hero_frame.pack_propagate(False)
            ctk.CTkLabel(hero_frame, text="Welcome to WatchMark", font=("Inter", 32, "bold")).pack(pady=(100, 10))
            ctk.CTkLabel(hero_frame, text="Scan your local folder or search TMDB to get started.", text_color=TEXT_SECONDARY).pack()

        # --- Horizontal Rows ---

        # Up Next Smart Queue (Continue Watching)
        ctk.CTkLabel(dash_scroll, text="Up Next", font=("Inter", 20, "bold")).pack(anchor="w", padx=25, pady=(20, 10))
        cw_frame = ctk.CTkScrollableFrame(dash_scroll, orientation="horizontal", height=220, fg_color="transparent")
        cw_frame.pack(fill="x", padx=15)

        # Logic: Find shows where at least one episode is watched, but not all episodes are watched.
        # Note: We use a subquery to count completed episodes to be accurate.
        cursor.execute("""
            SELECT m.id as media_id
            FROM Media m
            WHERE (SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') > 0
              AND (SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') < m.total_episodes
        """)
        active_media_ids = [r['media_id'] for r in cursor.fetchall()]

        cw_eps = []
        for m_id in active_media_ids:
            if hero_ep and m_id == hero_ep['media_id']:
                continue # Skip the one in the hero

            cursor.execute("""
                SELECT e.*, m.title as show_title, m.backdrop_path, m.poster_path, l.file_path, m.type as media_type, e.still_path
                FROM Episodes e
                JOIN Media m ON e.media_id = m.id
                LEFT JOIN Local_Files l ON e.id = l.episode_id
                WHERE e.media_id = ? AND e.status IN ('Watching', 'Unwatched')
                ORDER BY e.season_num ASC, e.ep_num ASC
                LIMIT 1
            """, (m_id,))
            ep = cursor.fetchone()
            if ep:
                cw_eps.append(ep)

        if not cw_eps:
            ctk.CTkLabel(cw_frame, text="No other shows in progress.", text_color=TEXT_SECONDARY).pack(padx=10, pady=50)
        else:
            for ep in cw_eps:
                self._create_horizontal_episode_card(cw_frame, ep)

        # Recently Added
        ctk.CTkLabel(dash_scroll, text="Recently Added", font=("Inter", 20, "bold")).pack(anchor="w", padx=25, pady=(20, 10))
        ra_frame = ctk.CTkScrollableFrame(dash_scroll, orientation="horizontal", height=280, fg_color="transparent")
        ra_frame.pack(fill="x", padx=15)

        cursor.execute("SELECT * FROM Media ORDER BY id DESC LIMIT 15")
        recent_media = cursor.fetchall()

        if not recent_media:
             ctk.CTkLabel(ra_frame, text="Library is empty.", text_color=TEXT_SECONDARY).pack(padx=10, pady=50)
        else:
            for item in recent_media:
                self._create_poster_card(ra_frame, item)

        # Stats Row
        ctk.CTkLabel(dash_scroll, text="Your Stats", font=("Inter", 20, "bold")).pack(anchor="w", padx=25, pady=(20, 10))
        stats_frame = ctk.CTkFrame(dash_scroll, fg_color="transparent")
        stats_frame.pack(fill="x", padx=20, pady=(0, 20))

        cursor.execute("SELECT COUNT(*) as count FROM Episodes WHERE status = 'Completed'")
        eps_watched = cursor.fetchone()['count']
        cursor.execute("SELECT SUM(runtime) as r FROM Episodes WHERE status = 'Completed'")
        r_val = cursor.fetchone()['r']
        hrs_watched = round((r_val or 0) / 60)
        cursor.execute("SELECT COUNT(*) as c FROM Media WHERE status = 'Completed'")
        shows_completed = cursor.fetchone()['c']

        def make_stat_card(parent, title, value):
            f = ctk.CTkFrame(parent, fg_color=SURFACE_COLOR, corner_radius=12, height=100)
            f.pack(side="left", fill="x", expand=True, padx=5)
            f.pack_propagate(False)
            ctk.CTkLabel(f, text=title, font=("Inter", 14, "normal"), text_color=TEXT_SECONDARY).pack(pady=(20, 5))
            ctk.CTkLabel(f, text=str(value), font=("Inter", 28, "bold"), text_color=VLC_ORANGE).pack()

        make_stat_card(stats_frame, "Episodes Watched", eps_watched)
        make_stat_card(stats_frame, "Hours Watched", hrs_watched)
        make_stat_card(stats_frame, "Shows Completed", shows_completed)

        conn.close()

    def _create_horizontal_episode_card(self, parent, ep_row):
        card = ctk.CTkFrame(parent, width=280, height=200, fg_color=SURFACE_COLOR, corner_radius=8)
        card.pack(side="left", padx=10, pady=5)
        card.pack_propagate(False)

        # Container for image and play overlay
        img_container = ctk.CTkFrame(card, width=280, height=158, fg_color="transparent")
        img_container.pack(fill="x")
        img_container.pack_propagate(False)

        img_label = ctk.CTkLabel(img_container, text="No Image", width=280, height=158, fg_color="#15171e")
        img_label.place(x=0, y=0, relwidth=1.0, relheight=1.0)

        # Prioritize still -> backdrop
        img_path = ep_row['still_path'] if 'still_path' in ep_row.keys() and ep_row['still_path'] else (ep_row['backdrop_path'] if 'backdrop_path' in ep_row.keys() else None)
        def load_img():
            if img_path:
                from .tmdb_api import download_image
                local_img_path = download_image(img_path, size="w500")
                if local_img_path:
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_img_path), dark_image=Image.open(local_img_path), size=(280, 158))
                        self.after(0, lambda: img_label.configure(image=img, text=""))
                    except: pass
        threading.Thread(target=load_img, daemon=True).start()

        # Play overlay button
        play_btn = ctk.CTkButton(img_container, text="▶", width=40, height=40, corner_radius=20,
                                 fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER,
                                 font=("Inter", 18, "normal"), command=lambda e=ep_row: self._play_episode(e))
        play_btn.place(relx=0.5, rely=0.5, anchor="center")

        # Progress bar at bottom of thumbnail
        progress_val = 0.0
        if ep_row['status'] == 'Watching' and (ep_row['runtime'] or 0) > 0:
            progress_val = min(1.0, ep_row['last_position'] / ((ep_row['runtime'] or 0) * 60))

        prog_bar = ctk.CTkProgressBar(card, height=4, progress_color=VLC_ORANGE, fg_color="#15171e", corner_radius=0)
        prog_bar.place(x=0, y=154, relwidth=1.0)
        prog_bar.set(progress_val)

        info_frame = ctk.CTkFrame(card, fg_color="transparent")
        info_frame.pack(fill="both", expand=True, padx=10)

        title = f"{(ep_row['show_title'] or 'Unknown Show')}"
        if ep_row['media_type'] == 'TV':
            subtitle = f"S{ep_row['season_num']:02}E{ep_row['ep_num']:02} - {(ep_row['title'] or 'Unknown')}"
        else:
            subtitle = (ep_row['title'] or 'Unknown Title')

        # Truncate subtitle if too long
        if len(subtitle) > 25:
            subtitle = subtitle[:22] + "..."

        ctk.CTkLabel(info_frame, text=title, font=("Inter", 13, "bold"), anchor="w").pack(side="left")
        ctk.CTkLabel(info_frame, text=subtitle, font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY, anchor="e").pack(side="right")

        card.bind("<Button-1>", lambda e, eid=ep_row['media_id']: self._show_media_details(eid))
        img_label.bind("<Button-1>", lambda e, eid=ep_row['media_id']: self._show_media_details(eid))

    # =========================================================================
    # TV SHOWS / MOVIES LIBRARY
    # =========================================================================
    def _show_tv_shows(self, filter_query=None):
        self._highlight_nav("TV Shows")
        self._show_library("TV", filter_query)

    def _show_movies(self):
        self._highlight_nav("Movies")
        self._show_library("Movie")

    def _show_library(self, media_type, filter_query=None, sort_by="Recently Added", hide_completed=False):
        self._clear_main_frame()

        header_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        header_frame.pack(fill="x", padx=20, pady=20)

        if media_type == "All":
            title = "Search Results"
        else:
            title = "TV Shows" if media_type == "TV" else "Movies"

        ctk.CTkLabel(header_frame, text=title, font=("Inter", 24, "bold")).pack(side="left")

        scan_btn = ctk.CTkButton(header_frame, text="📂 Scan Local Folder", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, command=self._scan_folder)
        scan_btn.pack(side="right")

        # Controls Bar (Sorting & Filtering)
        controls_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent", height=40)
        controls_frame.pack(fill="x", padx=20, pady=(0, 10))

        sort_var = ctk.StringVar(value=sort_by)
        hide_var = ctk.BooleanVar(value=hide_completed)

        sort_dropdown = ctk.CTkOptionMenu(controls_frame, values=["Recently Added", "Alphabetical (A-Z)", "Release Year", "My Top Rated"],
                                          variable=sort_var, command=lambda v: self._show_library(media_type, filter_query, v, hide_var.get()),
                                          fg_color=SURFACE_COLOR, button_color=SURFACE_COLOR, button_hover_color="#333", font=("Inter", 12, "normal"))
        sort_dropdown.pack(side="left")

        hide_switch = ctk.CTkSwitch(controls_frame, text="Hide Completed", variable=hide_var, font=("Inter", 12, "normal"),
                                    command=lambda: self._show_library(media_type, filter_query, sort_var.get(), hide_var.get()))
        hide_switch.pack(side="left", padx=20)

        grid_frame = ctk.CTkScrollableFrame(self.main_frame, fg_color="transparent")
        grid_frame.pack(fill="both", expand=True, padx=20, pady=10)

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        # Build query
        base_query = "SELECT m.* FROM Media m"
        where_clauses = []
        params = []

        if media_type != "All":
            where_clauses.append("m.type=?")
            params.append(media_type)

        if filter_query:
            where_clauses.append("m.title LIKE ?")
            params.append(f"%{filter_query}%")

        if hide_completed:
            where_clauses.append("(SELECT COUNT(*) FROM Episodes WHERE media_id = m.id AND status = 'Completed') < m.total_episodes")

        if where_clauses:
            base_query += " WHERE " + " AND ".join(where_clauses)

        # Handle Sorting
        if sort_by == "Alphabetical (A-Z)":
            base_query += " ORDER BY m.title ASC"
        elif sort_by == "Release Year":
            base_query += " ORDER BY CASE WHEN m.release_date IS NULL OR m.release_date = '' THEN 1 ELSE 0 END, m.release_date DESC"
        elif sort_by == "My Top Rated":
            base_query += " ORDER BY m.user_rating DESC, m.id DESC"
        else: # Default: Recently Added
            base_query += " ORDER BY m.id DESC"

        cursor.execute(base_query, tuple(params))
        media_items = cursor.fetchall()
        conn.close()

        if not media_items:
            msg = f"No {title} tracked yet. Use Search to add some!"
            if filter_query:
                msg = f"'{filter_query}' not found in library. Press Enter in Search to query TMDB."

            ctk.CTkLabel(grid_frame, text=msg, font=("Inter", 16, "normal"), text_color=TEXT_SECONDARY).pack(pady=50)
            return

        # Chunked rendering to prevent main thread freeze
        max_cols = 5
        self._library_render_state = {'index': 0, 'row': 0, 'col': 0, 'items': media_items, 'parent': grid_frame, 'max_cols': max_cols}
        self._render_library_chunk()

    def _render_library_chunk(self):
        state = getattr(self, '_library_render_state', None)
        if not state:
            return

        items = state['items']
        parent = state['parent']

        # Ensure parent still exists
        if not parent.winfo_exists():
            return

        chunk_size = 15 # Render 15 items per chunk
        end_idx = min(state['index'] + chunk_size, len(items))

        for i in range(state['index'], end_idx):
            item = items[i]
            self._create_poster_card(parent, item, grid_pos=(state['row'], state['col']))

            state['col'] += 1
            if state['col'] >= state['max_cols']:
                state['col'] = 0
                state['row'] += 1

        state['index'] = end_idx

        if state['index'] < len(items):
            self.after(50, self._render_library_chunk)
        else:
            self._library_render_state = None

    def _create_poster_card(self, parent, item, grid_pos=None):
        card = ctk.CTkFrame(parent, width=160, height=260, fg_color="transparent", corner_radius=8)

        if grid_pos:
            card.grid(row=grid_pos[0], column=grid_pos[1], padx=10, pady=10)
        else:
            card.pack(side="left", padx=10, pady=5)

        card.pack_propagate(False)
        if grid_pos:
            card.grid_propagate(False)

        img_label = ctk.CTkLabel(card, text="No Poster", width=160, height=240, fg_color=SURFACE_COLOR, corner_radius=8)
        img_label.pack()

        # Create overlay elements that we'll show on hover
        overlay_frame = ctk.CTkFrame(card, fg_color=BG_COLOR, corner_radius=8, width=160, height=240)
        # We don't pack it initially
        overlay_btn = ctk.CTkButton(overlay_frame, text="▶", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER,
                                    width=50, height=50, corner_radius=25, font=("Inter", 20, "normal"))
        overlay_btn.place(relx=0.5, rely=0.5, anchor="center")

        # Year-at-a-glance tag
        conn = self.data_manager.get_db_connection()
        c = conn.cursor()
        c.execute("SELECT COUNT(*) as c FROM Episodes WHERE media_id=? AND status='Completed'", (item['id'],))
        w_c = c.fetchone()['c']
        conn.close()

        if w_c > 0 and w_c == item['total_episodes']:
            if 'release_date' in item.keys() and item['release_date']:
                year_str = item['release_date'][:4]
                tag_text = f"Watched: {year_str}"
            else:
                tag_text = "Legacy"

            tag_lbl = ctk.CTkLabel(card, text=tag_text, fg_color="#181A20", text_color=TEXT_SECONDARY, font=("Inter", 11, "bold"), corner_radius=6, padx=8, pady=2)
            tag_lbl.place(relx=0.05, rely=0.05)

        title_lbl = ctk.CTkLabel(card, text=((item['title'] or 'Unknown Title')), font=("Inter", 13, "bold"),
                                 wraplength=150, text_color=TEXT_PRIMARY)
        # title_lbl.pack(pady=(5, 0)) # Depending on layout needs, hide title to make it cleaner

        def load_poster():
            if item['poster_path']:
                from .tmdb_api import download_image
                local_img_path = download_image(item['poster_path'], size="w500")
                if local_img_path:
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_img_path), dark_image=Image.open(local_img_path), size=(160, 240))
                        self.after(0, lambda: img_label.configure(image=img, text="") if img_label.winfo_exists() else None)
                    except: pass

        # Async load to prevent main thread blocking
        threading.Thread(target=load_poster, daemon=True).start()

        def on_enter(e):
            overlay_frame.place(x=0, y=0)
            # pseudo alpha, just dimming it by putting solid frame and relying on button

        def on_leave(e):
            # Check if mouse is still inside the card bounds
            x, y = e.widget.winfo_pointerxy()
            cx = card.winfo_rootx()
            cy = card.winfo_rooty()
            cw = card.winfo_width()
            ch = card.winfo_height()

            if not (cx <= x <= cx + cw and cy <= y <= cy + ch):
                 overlay_frame.place_forget()

        # Bindings
        img_label.bind("<Enter>", on_enter)
        overlay_frame.bind("<Leave>", on_leave)

        # Click actions
        overlay_btn.configure(command=lambda mid=item['id']: self._show_media_details(mid))
        overlay_frame.bind("<Button-1>", lambda e, mid=item['id']: self._show_media_details(mid))
        img_label.bind("<Button-1>", lambda e, mid=item['id']: self._show_media_details(mid))

    # =========================================================================
    # WATCH HISTORY LOG
    # =========================================================================
    def _show_history(self):
        self._highlight_nav("🕒 History")
        self._clear_main_frame()

        top_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        top_frame.pack(fill="x", padx=20, pady=20)
        ctk.CTkLabel(top_frame, text="Watch History", font=("Inter", 24, "bold"), text_color=TEXT_PRIMARY).pack(side="left")

        history_scroll = ctk.CTkScrollableFrame(self.main_frame, fg_color="transparent")
        history_scroll.pack(fill="both", expand=True, padx=20, pady=10)

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        # Query all history joined with episodes and media
        cursor.execute("""
            SELECT h.id as hist_id, h.timestamp, h.session_id, h.is_legacy, h.start_time, h.end_time, h.pause_count, h.completion_ratio,
                   e.id as episode_id, e.season_num, e.ep_num, e.title as ep_title, e.still_path, e.air_date,
                   m.id as media_id, m.title as show_title, m.poster_path, m.backdrop_path, m.type as media_type
            FROM History h
            JOIN Episodes e ON h.episode_id = e.id
            JOIN Media m ON e.media_id = m.id
            ORDER BY h.timestamp DESC
        """)
        history_entries = cursor.fetchall()
        conn.close()

        if not history_entries:
            ctk.CTkLabel(history_scroll, text="No history recorded yet.", text_color=TEXT_SECONDARY).pack(pady=50)
            return

        from datetime import datetime, timezone

        # Parse ALL entries to dt_local
        parsed_entries = []
        for row in history_entries:
            try:
                # If it's legacy, the timestamp might not be UTC, but let's parse consistently
                if row['is_legacy']:
                    dt_local = datetime.strptime(row['timestamp'], '%Y-%m-%d %H:%M:%S')
                else:
                    dt_utc = datetime.strptime(row['timestamp'], '%Y-%m-%d %H:%M:%S').replace(tzinfo=timezone.utc)
                    dt_local = dt_utc.astimezone()
                parsed_entries.append({'dt_local': dt_local, 'row': row})
            except Exception:
                pass

        # Grouping Pass 1: Binge Clusters by session_id
        session_groups = {}
        for item in parsed_entries:
            row = item['row']
            # Fallback grouping key if session_id is missing (old data)
            s_id = row['session_id']
            if not s_id:
                s_id = f"single_{row['hist_id']}"

            if s_id not in session_groups:
                session_groups[s_id] = {
                    'dt_local': item['dt_local'],
                    'is_legacy': row['is_legacy'],
                    'media_id': row['media_id'],
                    'show_title': row['show_title'],
                    'season_num': row['season_num'],
                    'poster_path': row['poster_path'],
                    'backdrop_path': row['backdrop_path'],
                    'media_type': row['media_type'],
                    'entries': []
                }
            session_groups[s_id]['entries'].append(item)

        # Sort session groups by newest dt_local
        sorted_sessions = sorted(list(session_groups.values()), key=lambda x: x['dt_local'], reverse=True)

        # Grouping Pass 2: Year/Month Headers
        grouped_data = {}
        for s_group in sorted_sessions:
            if s_group['is_legacy']:
                header_str = s_group['dt_local'].strftime("%B %Y").upper()
            else:
                # Active viewing gets specific day headers or standard month headers
                if s_group['dt_local'].year == datetime.now().year:
                    header_str = s_group['dt_local'].strftime("%A, %B %d")
                else:
                    header_str = s_group['dt_local'].strftime("%B %Y")

            if header_str not in grouped_data:
                grouped_data[header_str] = []

            grouped_data[header_str].append(s_group)

        # Chunked Rendering for History Timeline
        self._history_render_state = {
            'groups': list(grouped_data.items()),
            'group_idx': 0,
            'item_idx': 0,
            'parent': history_scroll
        }
        self._render_history_chunk()

    def _render_history_chunk(self):
        state = getattr(self, '_history_render_state', None)
        if not state:
            return

        groups = state['groups']
        parent = state['parent']

        if not parent.winfo_exists():
            return

        items_rendered = 0
        max_items_per_chunk = 15

        while state['group_idx'] < len(groups) and items_rendered < max_items_per_chunk:
            date_str, items = groups[state['group_idx']]

            # If it's the first item in the group, render the date header
            if state['item_idx'] == 0:
                header_frame = ctk.CTkFrame(parent, fg_color="transparent")
                header_frame.pack(fill="x", pady=(20, 10))
                ctk.CTkLabel(header_frame, text=date_str, font=("Inter", 16, "bold"), text_color=TEXT_PRIMARY).pack(side="left")

            while state['item_idx'] < len(items) and items_rendered < max_items_per_chunk:
                item = items[state['item_idx']]
                self._create_binge_block(parent, item)

                state['item_idx'] += 1
                items_rendered += 1

            # Move to next group if finished with current group
            if state['item_idx'] >= len(items):
                state['group_idx'] += 1
                state['item_idx'] = 0

        if state['group_idx'] < len(groups):
            self.after(50, self._render_history_chunk)
        else:
            self._history_render_state = None

    def _create_binge_block(self, parent, session_data):
        entries = session_data['entries']
        is_legacy = session_data['is_legacy']

        container = ctk.CTkFrame(parent, fg_color="transparent")
        container.pack(fill="x", pady=4)

        header_frame = ctk.CTkFrame(container, height=70, fg_color=SURFACE_COLOR, corner_radius=8)
        header_frame.pack(fill="x")
        header_frame.pack_propagate(False)

        # Poster thumbnail
        img_lbl = ctk.CTkLabel(header_frame, text="", width=45, height=60, fg_color="#1E1E1E", corner_radius=4)
        img_lbl.pack(side="left", padx=10, pady=5)

        def load_hist_poster(path, lbl):
            if path:
                from .tmdb_api import download_image
                local_path = download_image(path, size="w500")
                if local_path:
                    try:
                        from PIL import Image
                        img = ctk.CTkImage(light_image=Image.open(local_path), dark_image=Image.open(local_path), size=(45, 60))
                        self.after(0, lambda: lbl.configure(image=img) if lbl.winfo_exists() else None)
                    except: pass
        threading.Thread(target=load_hist_poster, args=(session_data['poster_path'], img_lbl), daemon=True).start()

        # Text Content
        text_frame = ctk.CTkFrame(header_frame, fg_color="transparent")
        text_frame.pack(side="left", fill="both", expand=True, padx=10, pady=10)

        show_title = session_data['show_title'] or 'Unknown Show'

        title_text = f"Watched: {show_title}"
        if len(entries) > 1:
            title_text = f"🔥 BINGE: {show_title} - Season {session_data['season_num']}"
            if session_data['media_type'] == 'Movie':
                title_text = f"🔥 BINGE: {show_title}"

        # Determine watch state (in progress vs completed)
        is_watching = False
        if not is_legacy:
            for item in entries:
                if item['row']['completion_ratio'] < 0.90:
                    is_watching = True
                    break

        if is_watching:
            title_text = f"Watching: {show_title} - Season {session_data['season_num']}"

        ctk.CTkLabel(text_frame, text=title_text, font=("Inter", 14, "bold"), text_color=TEXT_PRIMARY, anchor="w").pack(fill="x")

        # Subtext
        if is_legacy:
            subtext = f"Watched {len(entries)} Episodes"
        else:
            if len(entries) == 1:
                if session_data['media_type'] == 'TV':
                    subtext = f"S{entries[0]['row']['season_num']:02}E{entries[0]['row']['ep_num']:02} - {entries[0]['row']['ep_title'] or 'Unknown Title'}"
                else:
                    subtext = entries[0]['row']['ep_title'] or 'Movie'
            else:
                # E.g., "8 Episodes | Fri - Sat"
                start_dt = entries[-1]['dt_local'] # oldest
                end_dt = entries[0]['dt_local'] # newest
                day_span = ""
                if start_dt.date() == end_dt.date():
                    day_span = start_dt.strftime("%A")
                else:
                    day_span = f"{start_dt.strftime('%a')} - {end_dt.strftime('%a')}"
                subtext = f"{len(entries)} Episodes | {day_span}"

        ctk.CTkLabel(text_frame, text=subtext, font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY, anchor="w").pack(fill="x")

        # Time / Status
        if not is_legacy:
            if len(entries) == 1:
                time_str = entries[0]['dt_local'].strftime("%I:%M %p")
                ctk.CTkLabel(header_frame, text=time_str, font=("Inter", 12, "bold"), text_color=TEXT_SECONDARY).pack(side="right", padx=20)
            else:
                # Expand button or icon for non-legacy binges
                expand_lbl = ctk.CTkLabel(header_frame, text="▼", font=("Inter", 16, "normal"), text_color=TEXT_SECONDARY)
                expand_lbl.pack(side="right", padx=20)

        # Binge-Block Visual Progress Bar
        if len(entries) > 1 or is_watching:
            progress_val = sum([e['row']['completion_ratio'] for e in entries]) / len(entries) if entries else 0.0
            prog_bar = ctk.CTkProgressBar(header_frame, height=4, progress_color=VLC_ORANGE, fg_color="#15171e", corner_radius=0)
            prog_bar.place(x=0, rely=1.0, anchor="sw", relwidth=1.0)
            prog_bar.set(progress_val)

        # Expanded view container
        details_frame = ctk.CTkFrame(container, fg_color="transparent")

        self.expanded_states = getattr(self, "expanded_states", {})
        session_id = entries[0]['row']['session_id']
        is_expanded = self.expanded_states.get(session_id, False)

        def toggle_expand(e):
            if is_legacy or len(entries) == 1:
                self._show_media_details(session_data['media_id'])
                return

            is_expanded = self.expanded_states.get(session_id, False)
            if is_expanded:
                details_frame.pack_forget()
                self.expanded_states[session_id] = False
                expand_lbl.configure(text="▼")
            else:
                details_frame.pack(fill="x", padx=(40, 0), pady=(0, 10))
                self.expanded_states[session_id] = True
                expand_lbl.configure(text="▲")

        # Click bindings
        header_frame.bind("<Button-1>", toggle_expand)
        img_lbl.bind("<Button-1>", toggle_expand)
        for child in text_frame.winfo_children():
            child.bind("<Button-1>", toggle_expand)

        # Build expanded rows
        if not is_legacy and len(entries) > 1:
            # Sort chronologically for the expanded view
            sorted_entries = sorted(entries, key=lambda x: x['dt_local'])
            for ep_entry in sorted_entries:
                row = ep_entry['row']
                ep_frame = ctk.CTkFrame(details_frame, height=40, fg_color="#1A1C23", corner_radius=4)
                ep_frame.pack(fill="x", pady=2)
                ep_frame.pack_propagate(False)

                ep_text = f"S{row['season_num']:02}E{row['ep_num']:02} - {row['ep_title']}"
                if session_data['media_type'] == 'Movie':
                    ep_text = row['ep_title']

                ctk.CTkLabel(ep_frame, text=ep_text, font=("Inter", 12, "bold"), text_color=TEXT_PRIMARY, anchor="w").pack(side="left", padx=10)

                # Heartbeat Pause / Status Details
                status_text = "Completed"
                if row['completion_ratio'] < 0.90:
                    status_text = f"Paused ({int(row['completion_ratio']*100)}%)"

                time_str = ep_entry['dt_local'].strftime("%I:%M %p")
                detail_text = f"{status_text} | {time_str}"

                # Air Date Comparison
                if row['air_date']:
                    from datetime import datetime
                    try:
                        air_dt = datetime.strptime(row['air_date'], '%Y-%m-%d')
                        years_late = ep_entry['dt_local'].year - air_dt.year
                        if years_late > 0:
                            detail_text += f" | {years_late} yrs late"
                    except ValueError:
                        pass

                ctk.CTkLabel(ep_frame, text=detail_text, font=("Inter", 11, "normal"), text_color=TEXT_SECONDARY).pack(side="right", padx=10)

                # Micro Progress Bar
                if row['completion_ratio'] > 0:
                    m_prog = ctk.CTkProgressBar(ep_frame, height=2, progress_color=VLC_ORANGE, fg_color="transparent", corner_radius=0)
                    m_prog.place(x=0, rely=1.0, anchor="sw", relwidth=1.0)
                    m_prog.set(row['completion_ratio'])

            if is_expanded:
                details_frame.pack(fill="x", padx=(40, 0), pady=(0, 10))


    # =========================================================================
    # SEARCH & DISCOVER
    # =========================================================================
    def _show_search(self):
        self._pending_group_match = None # Clear pending matches when opening standard search
        self._highlight_nav("Search")
        self._clear_main_frame()

        top_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        top_frame.pack(fill="x", padx=20, pady=20)

        ctk.CTkLabel(top_frame, text="Discover Media", font=("Inter", 24, "bold"), text_color=TEXT_PRIMARY).pack(side="left")

        search_box = ctk.CTkFrame(top_frame, fg_color="transparent")
        search_box.pack(side="right")

        self.search_entry = ctk.CTkEntry(search_box, placeholder_text="Search TMDB for Shows or Movies...", width=350, height=36, corner_radius=18, fg_color=SURFACE_COLOR, border_color="#333", font=("Inter", 13, "normal"))
        self.search_entry.pack(side="left", padx=(0, 10))
        self.search_entry.bind("<Return>", lambda e: self._perform_search())

        ctk.CTkButton(search_box, text="Search TMDB", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, height=36, corner_radius=18, command=self._perform_search).pack(side="left")

        self.results_frame = ctk.CTkScrollableFrame(self.main_frame, fg_color="transparent")
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

        # Try to download skeleton posters immediately or just text
        ctk.CTkLabel(self.results_frame, text=f"Searching TMDB for '{query}'...", font=("Inter", 16, "normal"), text_color=TEXT_SECONDARY).pack(pady=50)
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
            ctk.CTkLabel(self.results_frame, text="No results found.", font=("Inter", 16, "normal"), text_color=TEXT_SECONDARY).pack(pady=50)
            return

        # Check existing media to mark "In Library"
        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()
        cursor.execute("SELECT tmdb_id FROM Media")
        existing_ids = set(r['tmdb_id'] for r in cursor.fetchall())
        conn.close()

        # Chunked rendering
        max_cols = 5
        self._search_render_state = {'index': 0, 'row': 0, 'col': 0, 'items': results, 'parent': self.results_frame, 'max_cols': max_cols, 'existing_ids': existing_ids}
        self._render_search_chunk()

    def _render_search_chunk(self):
        state = getattr(self, '_search_render_state', None)
        if not state:
            return

        items = state['items']
        parent = state['parent']
        existing_ids = state['existing_ids']

        if not parent.winfo_exists():
            return

        chunk_size = 15
        end_idx = min(state['index'] + chunk_size, len(items))

        for i in range(state['index'], end_idx):
            res = items[i]
            is_tracked = res['tmdb_id'] in existing_ids
            self._create_search_poster_card(parent, res, is_tracked, grid_pos=(state['row'], state['col']))

            state['col'] += 1
            if state['col'] >= state['max_cols']:
                state['col'] = 0
                state['row'] += 1

        state['index'] = end_idx

        if state['index'] < len(items):
            self.after(50, self._render_search_chunk)
        else:
            self._search_render_state = None

    def _create_search_poster_card(self, parent, item, is_tracked, grid_pos=None):
        card = ctk.CTkFrame(parent, width=160, height=280, fg_color="transparent", corner_radius=8)

        if grid_pos:
            card.grid(row=grid_pos[0], column=grid_pos[1], padx=10, pady=10)
        else:
            card.pack(side="left", padx=10, pady=5)

        card.pack_propagate(False)
        if grid_pos:
            card.grid_propagate(False)

        img_label = ctk.CTkLabel(card, text="No Poster", width=160, height=240, fg_color=SURFACE_COLOR, corner_radius=8)
        img_label.pack()

        # Year subtitle
        year = item['release_date'][:4] if ('release_date' in item.keys() and item['release_date'] is not None) else "N/A"
        title_lbl = ctk.CTkLabel(card, text=f"{((item['title'] or 'Unknown Title'))}\n({year})", font=("Inter", 12, "bold"),
                                 wraplength=150, text_color=TEXT_PRIMARY)
        title_lbl.pack(pady=(5, 0))

        # We download poster asynchronously if not cached
        def load_poster():
            if item['poster_path']:
                from .tmdb_api import download_image
                local_path = download_image(item['poster_path'])
                if local_path:
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_path), dark_image=Image.open(local_path), size=(160, 240))
                        self.after(0, lambda: img_label.configure(image=img, text="") if img_label.winfo_exists() else None)
                    except: pass
        threading.Thread(target=load_poster, daemon=True).start()

        # Hover overlay
        overlay_frame = ctk.CTkFrame(card, fg_color=BG_COLOR, corner_radius=8, width=160, height=240)

        btn_text = "✓ In Library" if is_tracked else "+ Add to Tracker"
        btn_color = SUCCESS_COLOR if is_tracked else VLC_ORANGE
        btn_hover = SUCCESS_COLOR if is_tracked else VLC_ORANGE_HOVER

        overlay_btn = ctk.CTkButton(overlay_frame, text=btn_text, fg_color=btn_color, hover_color=btn_hover,
                                    width=120, height=40, corner_radius=20, font=("Inter", 13, "bold"),
                                    state="disabled" if is_tracked else "normal")
        overlay_btn.place(relx=0.5, rely=0.45, anchor="center")

        archive_var = ctk.BooleanVar(value=False)
        if not is_tracked:
            archive_cb = ctk.CTkCheckBox(overlay_frame, text="Archive", variable=archive_var, font=("Inter", 11, "normal"), text_color=TEXT_SECONDARY, checkbox_height=14, checkbox_width=14)
            archive_cb.place(relx=0.5, rely=0.65, anchor="center")

        def on_enter(e):
            overlay_frame.place(x=0, y=0)

        def on_leave(e):
            x, y = e.widget.winfo_pointerxy()
            cx = card.winfo_rootx()
            cy = card.winfo_rooty()
            cw = card.winfo_width()
            ch = card.winfo_height()
            if not (cx <= x <= cx + cw and cy <= y <= cy + ch):
                 overlay_frame.place_forget()

        img_label.bind("<Enter>", on_enter)
        overlay_frame.bind("<Leave>", on_leave)

        if not is_tracked:
            # We pass a callback to disable button after adding
            def add_wrapper():
                self._add_to_tracker(item, archive=archive_var.get())
                overlay_btn.configure(text="Adding...", state="disabled")
            overlay_btn.configure(command=add_wrapper)

    def _add_to_tracker(self, media_data, archive=False):
        api_key = self.data_manager.settings.get("tmdb_api_key")

    def _show_media_details(self, media_id, target_season=None):
        self._clear_main_frame()
        self._highlight_nav(None) # Clear specific nav

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        cursor.execute("SELECT * FROM Media WHERE id=?", (media_id,))
        media = cursor.fetchone()

        if not media:
            conn.close()
            return

        detail_scroll = ctk.CTkScrollableFrame(self.main_frame, fg_color="transparent")
        detail_scroll.pack(fill="both", expand=True)
        self._current_detail_scroll = detail_scroll


        # Back Button
        ctk.CTkButton(detail_scroll, text="← Back", width=60, fg_color="transparent", hover_color=SURFACE_COLOR,
                      command=lambda: self._show_library((media['type'] or 'Unknown Type'))).pack(anchor="w", padx=20, pady=(10, 0))

        # Header with Backdrop Banner
        header = ctk.CTkFrame(detail_scroll, fg_color="transparent")
        header.pack(fill="x", padx=20, pady=(10, 20))

        # Banner Image
        banner_lbl = ctk.CTkLabel(header, text="", fg_color=SURFACE_COLOR, corner_radius=12, height=200)
        banner_lbl.pack(fill="x", side="top", anchor="n")

        def load_banner():
            if media['backdrop_path']:
                from .tmdb_api import download_image
                local_img_path = download_image(media['backdrop_path'], size="w1280")
                if local_img_path:
                    try:
                        pil_img = Image.open(local_img_path)
                        w, h = pil_img.size
                        target_h = int(w * (200/1000))
                        if h > target_h:
                            top = (h - target_h) // 2
                            pil_img = pil_img.crop((0, top, w, top + target_h))
                        img = ctk.CTkImage(light_image=pil_img, dark_image=pil_img, size=(1000, 200))
                        self.after(0, lambda: banner_lbl.configure(image=img) if banner_lbl.winfo_exists() else None)
                    except: pass
        threading.Thread(target=load_banner, daemon=True).start()

        # Sub-header Frame for Poster and Info
        sub_header_frame = ctk.CTkFrame(header, fg_color="transparent")
        # Give a negative top padding to make the poster overlap the banner slightly, if supported
        # CustomTkinter may not perfectly handle negative margins, so we just pad closely or use place just for the overlap effect.
        # Actually, standard grid/pack approach is safer as requested. We'll use pack.
        sub_header_frame.pack(fill="x", side="top", padx=20, pady=(10, 0))

        # Poster Image
        img_label = ctk.CTkLabel(sub_header_frame, text="No Poster", width=140, height=210, fg_color="#1E1E1E", corner_radius=8)
        img_label.pack(side="left", anchor="nw")

        def load_poster():
            if media['poster_path']:
                from .tmdb_api import download_image
                local_img_path = download_image(media['poster_path'], size="w500")
                if local_img_path:
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_img_path), dark_image=Image.open(local_img_path), size=(140, 210))
                        self.after(0, lambda: img_label.configure(image=img, text="") if img_label.winfo_exists() else None)
                    except: pass
        threading.Thread(target=load_poster, daemon=True).start()
        self._current_poster_lbl = img_label

        # Info Frame
        info_frame = ctk.CTkFrame(sub_header_frame, fg_color="transparent")
        info_frame.pack(side="left", anchor="nw", padx=(20, 0), fill="both", expand=True)

        title_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        title_frame.pack(anchor="w", fill="x")

        safe_title = media['title'] or 'Unknown Title'
        title_lbl = ctk.CTkLabel(title_frame, text=safe_title, font=("Inter", 32, "bold"), text_color=TEXT_PRIMARY)
        title_lbl.pack(side="left")
        self._current_title_lbl = title_lbl

        cursor.execute("SELECT COUNT(*) as c FROM Episodes WHERE media_id=? AND status='Completed'", (media_id,))
        watched_eps = cursor.fetchone()['c']

        tags_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        tags_frame.pack(anchor="w", pady=(5, 10))

        safe_type = (media['type'] or 'Unknown Type') if (media['type'] or 'Unknown Type') else "Unknown Type"
        type_tag = ctk.CTkLabel(tags_frame, text=safe_type, fg_color=SURFACE_COLOR, corner_radius=10, font=("Inter", 12, "normal"), padx=10)
        type_tag.pack(side="left", padx=(0, 5))

        total_episodes = media['total_episodes'] if media['total_episodes'] is not None else 0
        status_text = "Completed" if watched_eps == total_episodes and watched_eps > 0 else "Watching"
        status_color = SUCCESS_COLOR if status_text == "Completed" else VLC_ORANGE

        stat_tag = ctk.CTkLabel(tags_frame, text=f"{watched_eps} / {total_episodes} Eps", fg_color=status_color, corner_radius=10, font=("Inter", 12, "bold"), padx=10, text_color="white")
        stat_tag.pack(side="left")
        self._current_progress_badge = stat_tag
        self._current_watched_eps = watched_eps
        self._current_total_eps = total_episodes

        safe_synopsis = media['synopsis'] or 'No overview available.'
        synopsis_lbl = ctk.CTkLabel(info_frame, text=safe_synopsis, font=("Inter", 13, "normal"), text_color=TEXT_SECONDARY, wraplength=700, justify="left")
        synopsis_lbl.pack(anchor="w", pady=5)
        self._current_synopsis_lbl = synopsis_lbl

        # Ratings Row
        ratings_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        ratings_frame.pack(anchor="w", pady=(5, 10))

        # TMDB Badge
        tmdb_score = round(media['vote_average'] if 'vote_average' in media.keys() and media['vote_average'] is not None else 0.0, 1)
        tmdb_badge_lbl = ctk.CTkLabel(ratings_frame, text=f"⭐ TMDB: {tmdb_score}/10", fg_color="#181A20", text_color="#F5C518",
                     font=("Inter", 12, "bold"), corner_radius=6, padx=8, pady=4)
        tmdb_badge_lbl.pack(side="left", padx=(0, 15))
        self._current_tmdb_badge_lbl = tmdb_badge_lbl

        # My Watch vs Release Date
        release_yr = media['release_date'][:4] if ('release_date' in media.keys() and media['release_date']) else "Unknown"

        cursor.execute("SELECT MIN(timestamp) as first_watch, MAX(is_legacy) as legacy_flag FROM History h JOIN Episodes e ON h.episode_id = e.id WHERE e.media_id=?", (media_id,))
        w_data = cursor.fetchone()

        watch_yr = "Unwatched"
        if w_data and w_data['first_watch']:
            watch_yr = w_data['first_watch'][:4]
            if w_data['legacy_flag']:
                watch_yr += " (Legacy)"

        if watch_yr != "Unwatched":
            ctk.CTkLabel(ratings_frame, text=f"Released: {release_yr} | My Watch: {watch_yr}", font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY).pack(side="left", padx=(0, 15))

        # User Rating Stars
        stars_frame = ctk.CTkFrame(ratings_frame, fg_color="transparent")
        stars_frame.pack(side="left")
        ctk.CTkLabel(stars_frame, text="My Score: ", font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY).pack(side="left", padx=(0, 5))

        user_rating = media['user_rating'] if 'user_rating' in media.keys() and media['user_rating'] is not None else 0
        self.star_btns = []

        def set_rating(rating_val):
            def _write(conn):
                cursor = conn.cursor()
                cursor.execute("UPDATE Media SET user_rating=? WHERE id=?", (rating_val, media_id))
            self.data_manager.submit_write_task(_write)

            # Update star colors visually
            for i, btn in enumerate(self.star_btns):
                if i < rating_val:
                    btn.configure(text_color=VLC_ORANGE)
                else:
                    btn.configure(text_color="#444")

        for i in range(1, 6):
            star_color = VLC_ORANGE if i <= user_rating else "#444"
            btn = ctk.CTkButton(stars_frame, text="★", width=25, height=25, fg_color="transparent", hover_color=SURFACE_COLOR,
                                text_color=star_color, font=("Inter", 22, "normal"),
                                command=lambda r=i: set_rating(r))
            btn.pack(side="left", padx=1)
            self.star_btns.append(btn)

        # Action Row
        actions = ctk.CTkFrame(detail_scroll, fg_color="transparent")
        actions.pack(fill="x", padx=20, pady=(0, 20))

        # Get next episode to play
        cursor.execute("""
            SELECT e.*, l.file_path FROM Episodes e
            LEFT JOIN Local_Files l ON e.id = l.episode_id
            WHERE e.media_id = ? AND e.status IN ('Unwatched', 'Watching')
            ORDER BY e.season_num ASC, e.ep_num ASC LIMIT 1
        """, (media_id,))
        next_ep = cursor.fetchone()

        if next_ep and next_ep['file_path']:
            play_btn = ctk.CTkButton(actions, text="▶ Play Next", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, height=36,
                                     font=("Inter", 13, "bold"), command=lambda e=next_ep: self._play_episode(e))
            play_btn.pack(side="left", padx=(0, 10))

        archive_var = ctk.BooleanVar(value=False)
        ctk.CTkButton(actions, text="✓ Mark All Watched", fg_color=SURFACE_COLOR, hover_color="#333", height=36,
                      command=lambda m=media_id: self._mark_all_watched(m, archive_var.get())).pack(side="left", padx=(0, 10))

        ctk.CTkCheckBox(actions, text="Archive (No Hist.)", variable=archive_var, font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY).pack(side="left", padx=(0, 15))

        sync_btn = ctk.CTkButton(actions, text="🔄 Refresh Data", fg_color="transparent", border_color="#555", border_width=1,
                                 hover_color=SURFACE_COLOR, text_color=TEXT_PRIMARY, height=36, font=("Inter", 13, "bold"),
                                 command=lambda m=media_id: self._sync_media(m))
        sync_btn.pack(side="left", padx=(0, 10))
        # Store a reference to the sync button to change its state during sync
        self._current_sync_btn = sync_btn

        # Main Area (Tabs for Seasons if TV)
        content_frame = ctk.CTkFrame(detail_scroll, fg_color="transparent")
        content_frame.pack(fill="both", expand=True, padx=20, pady=10)

        # Destructive action: Remove Show button in left info column
        remove_btn = ctk.CTkButton(info_frame, text="🗑️ Remove Show from Library", fg_color="transparent", text_color=DANGER_COLOR,
                                   hover_color="#331111", border_width=1, border_color=DANGER_COLOR, height=36,
                                   command=lambda m=media_id: self._remove_show(m))
        remove_btn.pack(anchor="w", pady=(20, 0))

        self.ep_list_frame = ctk.CTkFrame(content_frame, fg_color="transparent")

        if (media['type'] or 'Unknown Type') == 'TV':
            cursor.execute("SELECT DISTINCT season_num FROM Episodes WHERE media_id=? ORDER BY season_num", (media_id,))
            seasons = [r['season_num'] for r in cursor.fetchall()]

            if seasons:
                # Top horizontal scroll for season buttons
                season_scroll = ctk.CTkScrollableFrame(content_frame, orientation="horizontal", height=50, fg_color="transparent")
                season_scroll.pack(fill="x", pady=(0, 10))

                self.ep_list_frame.pack(fill="both", expand=True)

                # Add Season Triage Action Frame
                self.season_triage_frame = ctk.CTkFrame(content_frame, fg_color="transparent")
                self.season_triage_frame.pack(fill="x", pady=(0, 10))

                # We will populate the triage frame based on the active season

                self.season_btns = []

                for s in seasons:
                    btn = ctk.CTkButton(season_scroll, text=f"Season {s}", width=100, height=32, corner_radius=16,
                                        fg_color=SURFACE_COLOR, text_color=TEXT_SECONDARY, hover_color="#333",
                                        font=("Inter", 13, "bold"),
                                        command=lambda s_num=s, m_id=media_id: self._load_episodes(m_id, s_num))
                    btn.pack(side="left", padx=5)
                    self.season_btns.append((s, btn))

                # Load first season by default
                initial_season = target_season if target_season and target_season in seasons else seasons[0]
                self._load_episodes(media_id, initial_season)
            else:
                self.ep_list_frame.pack(fill="both", expand=True)
                ctk.CTkLabel(self.ep_list_frame, text="No episode data found. Try refreshing or re-adding this show.", font=("Inter", 14, "normal"), text_color=TEXT_SECONDARY).pack(pady=50)
        else:
            self.ep_list_frame.pack(fill="both", expand=True)
            self._load_episodes(media_id, 1)

        conn.close()

    def _remove_show(self, media_id):
        # Spawn confirmation modal
        dialog = ctk.CTkToplevel(self)
        dialog.title("Confirm Removal")
        dialog.geometry("400x200")
        dialog.grab_set()

        ctk.CTkLabel(dialog, text="Are you sure?", font=("Inter", 16, "bold"), text_color=DANGER_COLOR).pack(pady=(20, 10))
        ctk.CTkLabel(dialog, text="This will delete all your watch history for this show.\nLocal files will NOT be deleted.",
                     font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY).pack(pady=(0, 20))

        btn_frame = ctk.CTkFrame(dialog, fg_color="transparent")
        btn_frame.pack()

        ctk.CTkButton(btn_frame, text="Cancel", fg_color=SURFACE_COLOR, hover_color="#333", command=dialog.destroy).pack(side="left", padx=10)

        def confirm_removal():
            dialog.destroy()

            # Fetch poster info for filesystem deletion before DB delete
            conn = self.data_manager.get_db_connection()
            cursor = conn.cursor()
            cursor.execute("SELECT poster_path, backdrop_path, type FROM Media WHERE id=?", (media_id,))
            m_info = cursor.fetchone()

            paths_to_remove = []
            if m_info:
                if m_info['poster_path']: paths_to_remove.append(m_info['poster_path'])
                if m_info['backdrop_path']: paths_to_remove.append(m_info['backdrop_path'])

            cursor.execute("SELECT still_path FROM Episodes WHERE media_id=?", (media_id,))
            eps = cursor.fetchall()
            for ep in eps:
                if ep['still_path']: paths_to_remove.append(ep['still_path'])
            conn.close()

            # Execute DB deletes synchronously via queue
            def _write(conn):
                cursor = conn.cursor()

                # First delete Local_Files
                cursor.execute("DELETE FROM Local_Files WHERE episode_id IN (SELECT id FROM Episodes WHERE media_id=?)", (media_id,))

                # Second delete History
                cursor.execute("DELETE FROM History WHERE episode_id IN (SELECT id FROM Episodes WHERE media_id=?)", (media_id,))

                # Third delete Episodes
                cursor.execute("DELETE FROM Episodes WHERE media_id=?", (media_id,))

                # Finally delete Media
                cursor.execute("DELETE FROM Media WHERE id=?", (media_id,))

            def on_complete():
                # Delete cached files in background
                def cleanup_files():
                    import os
                    from pathlib import Path
                    from .data import POSTER_CACHE_DIR

                    prefixes = ["w500", "w1280"]
                    for path in paths_to_remove:
                        filename_base = path.lstrip('/')
                        for prefix in prefixes:
                            cached_file = POSTER_CACHE_DIR / f"{prefix}_{filename_base}"
                            if cached_file.exists():
                                try:
                                    os.remove(cached_file)
                                except Exception:
                                    pass

                threading.Thread(target=cleanup_files, daemon=True).start()

                # Navigate back to library
                self.after(0, lambda m_type=m_info['type'] if m_info else 'TV': self._show_library(m_type))

            self.data_manager.submit_write_task(_write, callback=on_complete)

        ctk.CTkButton(btn_frame, text="Remove", fg_color=DANGER_COLOR, hover_color="#8e0000", command=confirm_removal).pack(side="left", padx=10)

    def _mark_all_watched(self, media_id, archive=False):
        def _write(conn):
            cursor = conn.cursor()
            cursor.execute("UPDATE Episodes SET status='Completed', watch_count=MAX(1, watch_count) WHERE media_id=?", (media_id,))
            if not archive:
                # Add all episodes to history
                cursor.execute("SELECT id FROM Episodes WHERE media_id=?", (media_id,))
                ep_ids = cursor.fetchall()
                for ep in ep_ids:
                    cursor.execute("INSERT INTO History (episode_id) VALUES (?)", (ep['id'],))

        # Perform an in-place seamless update
        def refresh_ui():
            # Update the progress badge
            if hasattr(self, '_current_progress_badge') and self._current_progress_badge.winfo_exists():
                self._current_watched_eps = self._current_total_eps
                self._current_progress_badge.configure(
                    text=f"{self._current_watched_eps} / {self._current_total_eps} Eps",
                    fg_color=SUCCESS_COLOR
                )

            # Rebuild the episode list
            active_season = 1
            if hasattr(self, 'season_btns'):
                for s, btn in self.season_btns:
                    if btn.cget('fg_color') == TEXT_PRIMARY:
                        active_season = s
                        break
            self._load_episodes(media_id, active_season)

        self.data_manager.submit_write_task(_write, callback=lambda: self.after(0, refresh_ui))

    def _sync_media(self, media_id):
        api_key = self.data_manager.settings.get("tmdb_api_key")
        if not api_key:
            messagebox.showwarning("Missing API Key", "Please add your TMDB API Key in Settings first.")
            return

        # Change button state to indicate loading
        if hasattr(self, '_current_sync_btn') and self._current_sync_btn.winfo_exists():
            self._current_sync_btn.configure(text="Fetching...", state="disabled", text_color=TEXT_SECONDARY)

        def perform_sync():
            try:
                conn = self.data_manager.get_db_connection()
                cursor = conn.cursor()
                cursor.execute("SELECT tmdb_id, type FROM Media WHERE id=?", (media_id,))
                media_info = cursor.fetchone()
                if not media_info:
                    conn.close()
                    return

                tmdb_id = media_info['tmdb_id']
                m_type = media_info['type']
                conn.close()

                details = get_media_details(api_key, tmdb_id, m_type)
                if not details:
                    return

                # Download images immediately here so they are ready
                if details['poster_path']: download_image(details['poster_path'])
                if details.get('backdrop_path'): download_image(details['backdrop_path'])

                def _write(w_conn):
                    w_cursor = w_conn.cursor()
                    w_cursor.execute("""
                        UPDATE Media SET
                            title=?, synopsis=?, poster_path=?, backdrop_path=?, total_episodes=?, status=?, vote_average=?
                        WHERE id=?
                    """, ((details['title'] or 'Unknown Title'), details['synopsis'], details['poster_path'], details.get('backdrop_path', ''), details['total_episodes'], details['status'], details.get('vote_average', 0.0), media_id))

                    # We will return whether any new episodes were added so we can avoid rebuilding ep_list_frame if unnecessary
                    new_episodes_added = False

                    if m_type == 'TV':
                        for season in details['seasons']:
                            s_num = season.get('season_number')
                            if s_num == 0: continue

                            eps = get_tv_season_episodes(api_key, tmdb_id, s_num)
                            for ep in eps:
                                w_cursor.execute("SELECT id FROM Episodes WHERE media_id=? AND season_num=? AND ep_num=?", (media_id, s_num, ep['ep_num']))
                                existing_ep = w_cursor.fetchone()

                                if not existing_ep:
                                    w_cursor.execute("""
                                        INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview, air_date)
                                        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                                    """, (media_id, s_num, ep['ep_num'], (ep['title'] or 'Unknown Title'), (ep['runtime'] or 0), ep.get('still_path', ''), ep.get('overview', ''), ep.get('air_date', '')))
                                    new_episodes_added = True
                                else:
                                    w_cursor.execute("""
                                        UPDATE Episodes SET title=?, runtime=?, still_path=?, overview=?, air_date=?
                                        WHERE id=?
                                    """, ((ep['title'] or 'Unknown Title'), (ep['runtime'] or 0), ep.get('still_path', ''), ep.get('overview', ''), ep.get('air_date', ''), existing_ep['id']))

                    return new_episodes_added

                # We submit write task but don't need its return value asynchronously,
                # however, to know about new_episodes_added cleanly, we can execute the DB write locally since we are in a background thread anyway.
                # Since the requirement is to use the task queue for writes, we'll wrap the logic to capture the result.

                result_container = {}
                def _write_wrapper(w_conn):
                    result_container['new_episodes'] = _write(w_conn)

                def on_complete():
                    self.after(0, lambda: self._finish_sync(media_id, details, result_container.get('new_episodes', False)))

                self.data_manager.submit_write_task(_write_wrapper, callback=on_complete)
            except Exception as e:
                self.after(0, lambda: self._fail_sync(str(e)))

        threading.Thread(target=perform_sync, daemon=True).start()

    def _finish_sync(self, media_id, details, new_episodes_added):
        # In-place UI Update
        if hasattr(self, '_current_title_lbl') and self._current_title_lbl.winfo_exists():
            self._current_title_lbl.configure(text=details['title'] or 'Unknown Title')

        if hasattr(self, '_current_synopsis_lbl') and self._current_synopsis_lbl.winfo_exists():
            self._current_synopsis_lbl.configure(text=details['synopsis'] or 'No overview available.')

        if hasattr(self, '_current_tmdb_badge_lbl') and self._current_tmdb_badge_lbl.winfo_exists():
            score = round(details.get('vote_average', 0.0), 1)
            self._current_tmdb_badge_lbl.configure(text=f"⭐ TMDB: {score}/10")

        # Update progress badge
        self._current_total_eps = details['total_episodes']
        if hasattr(self, '_current_progress_badge') and self._current_progress_badge.winfo_exists():
            status_text = "Completed" if self._current_watched_eps == self._current_total_eps and self._current_watched_eps > 0 else "Watching"
            status_color = SUCCESS_COLOR if status_text == "Completed" else VLC_ORANGE
            self._current_progress_badge.configure(
                text=f"{self._current_watched_eps} / {self._current_total_eps} Eps",
                fg_color=status_color
            )

        # Update Images if changed
        def update_imgs():
            if details['backdrop_path'] and hasattr(self, '_current_banner_lbl'):
                from .tmdb_api import download_image
                local_img_path = download_image(details['backdrop_path'], size="w1280")
                if local_img_path:
                    try:
                        pil_img = Image.open(local_img_path)
                        w, h = pil_img.size
                        target_h = int(w * (200/1000))
                        if h > target_h:
                            top = (h - target_h) // 2
                            pil_img = pil_img.crop((0, top, w, top + target_h))
                        img = ctk.CTkImage(light_image=pil_img, dark_image=pil_img, size=(1000, 200))
                        self.after(0, lambda: self._current_banner_lbl.configure(image=img) if self._current_banner_lbl.winfo_exists() else None)
                    except: pass

            if details['poster_path'] and hasattr(self, '_current_poster_lbl'):
                from .tmdb_api import download_image
                local_img_path = download_image(details['poster_path'], size="w500")
                if local_img_path:
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_img_path), dark_image=Image.open(local_img_path), size=(140, 210))
                        self.after(0, lambda: self._current_poster_lbl.configure(image=img) if self._current_poster_lbl.winfo_exists() else None)
                    except: pass

        threading.Thread(target=update_imgs, daemon=True).start()

        # Update Episode List only if necessary
        if new_episodes_added:
            # We determine the active season to reload it. If it's a TV show, we can just call show_media_details again,
            # but to be truly seamless, we should rebuild the ep_list_frame manually.
            # To keep it simple and safe based on instructions:
            # "only destroy and rebuild the ep_list_frame"
            active_season = 1
            if hasattr(self, 'season_btns'):
                for s, btn in self.season_btns:
                    if btn.cget('fg_color') == TEXT_PRIMARY:
                        active_season = s
                        break
            self._load_episodes(media_id, active_season)

        # Restore Sync Button State
        if hasattr(self, '_current_sync_btn') and self._current_sync_btn.winfo_exists():
            self._current_sync_btn.configure(text="Updated!", text_color=SUCCESS_COLOR)
            self.after(2000, lambda: self._current_sync_btn.configure(text="🔄 Refresh Data", state="normal", text_color=TEXT_PRIMARY) if self._current_sync_btn.winfo_exists() else None)


    def _fail_sync(self, error_msg):
        if hasattr(self, '_current_sync_btn') and self._current_sync_btn.winfo_exists():
            self._current_sync_btn.configure(text="🔄 Refresh Data", state="normal", text_color=TEXT_PRIMARY)
        messagebox.showerror("Sync Failed", f"Could not sync data from TMDB:\n{error_msg}")

    def _load_episodes(self, media_id, season_num):
        if hasattr(self, 'season_btns'):
            for s, btn in self.season_btns:
                if s == season_num:
                    btn.configure(fg_color=TEXT_PRIMARY, text_color=BG_COLOR)
                else:
                    btn.configure(fg_color=SURFACE_COLOR, text_color=TEXT_SECONDARY)

        if hasattr(self, 'season_triage_frame') and self.season_triage_frame.winfo_exists():
            for widget in self.season_triage_frame.winfo_children():
                widget.destroy()
            ctk.CTkButton(self.season_triage_frame, text="✓ Mark Season Watched", fg_color=SURFACE_COLOR, hover_color="#333", height=32,
                          command=lambda m=media_id, sn=season_num: self._mark_season_watched_popup(m, sn)).pack(side="left", padx=5)

            # Air-Date Context for the Season Header
            conn = self.data_manager.get_db_connection()
            cursor = conn.cursor()
            cursor.execute("SELECT MIN(air_date) as start_date, MAX(air_date) as end_date FROM Episodes WHERE media_id=? AND season_num=? AND air_date IS NOT NULL AND air_date != ''", (media_id, season_num))
            season_dates = cursor.fetchone()
            conn.close()

            if season_dates and season_dates['start_date']:
                from datetime import datetime
                try:
                    s_dt = datetime.strptime(season_dates['start_date'], '%Y-%m-%d')
                    start_str = s_dt.strftime('%b %Y')
                    if season_dates['end_date'] and season_dates['start_date'] != season_dates['end_date']:
                        e_dt = datetime.strptime(season_dates['end_date'], '%Y-%m-%d')
                        end_str = e_dt.strftime('%b %Y')
                        if start_str != end_str:
                            date_text = f"📅 Aired: {start_str} - {end_str}"
                        else:
                            date_text = f"📅 Aired: {start_str}"
                    else:
                        date_text = f"📅 Aired: {start_str}"

                    ctk.CTkLabel(self.season_triage_frame, text=date_text, font=("Inter", 12, "bold"), text_color=TEXT_SECONDARY).pack(side="left", padx=15)
                except ValueError:
                    pass

        # Reset scrollbar to the top when changing seasons
        if hasattr(self, '_current_detail_scroll') and self._current_detail_scroll.winfo_exists():
            self._current_detail_scroll._parent_canvas.yview_moveto(0)

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

        cursor.execute("SELECT type, backdrop_path FROM Media WHERE id=?", (media_id,))
        media_info = cursor.fetchone()
        m_type = media_info['type']
        m_backdrop = media_info['backdrop_path']
        conn.close()

        for ep in episodes:
            row = ctk.CTkFrame(self.ep_list_frame, fg_color=SURFACE_COLOR, corner_radius=8, height=80)
            row.pack(fill="x", pady=6)
            row.pack_propagate(False)

            # Still Image
            img_frame = ctk.CTkFrame(row, width=120, height=68, fg_color=BG_COLOR, corner_radius=6)
            img_frame.pack(side="left", padx=(6, 15), pady=6)
            img_frame.pack_propagate(False)

            # Loading State (Skeleton Box)
            img_lbl = ctk.CTkLabel(img_frame, text="", fg_color="#1A1C23", font=("Inter", 10, "normal"))
            img_lbl.pack(fill="both", expand=True)

            def load_still(ep_data, fallback_backdrop, target_lbl):
                from .tmdb_api import download_image
                from PIL import ImageFilter, ImageEnhance

                final_img_path = None
                is_fallback = False

                if ep_data['still_path']:
                    final_img_path = download_image(ep_data['still_path'])

                if not final_img_path and fallback_backdrop:
                    final_img_path = download_image(fallback_backdrop)
                    is_fallback = True

                if final_img_path:
                    try:
                        pil_img = Image.open(final_img_path)
                        if is_fallback:
                            # Blur the backdrop so it's clearly a fallback
                            pil_img = pil_img.filter(ImageFilter.GaussianBlur(radius=8))

                            # Add a heavy dark overlay
                            enhancer = ImageEnhance.Brightness(pil_img)
                            pil_img = enhancer.enhance(0.4)

                        # Crop to 16:9 aspect roughly (120x68)
                        w, h = pil_img.size
                        target_h = int(w * (68/120))
                        if h > target_h:
                            top = (h - target_h) // 2
                            pil_img = pil_img.crop((0, top, w, top + target_h))

                        img = ctk.CTkImage(light_image=pil_img, dark_image=pil_img, size=(120, 68))

                        def update_ui():
                            target_lbl.configure(image=img, text=f"EP {ep_data['ep_num']}" if is_fallback else "",
                                              font=("Inter", 16, "bold"), text_color="#B3B3B3")
                        self.after(0, update_ui)
                    except Exception as e:
                        pass
                else:
                    self.after(0, lambda: target_lbl.configure(text=f"EP {ep_data['ep_num']}", text_color=TEXT_SECONDARY))

            threading.Thread(target=load_still, args=(ep, m_backdrop, img_lbl), daemon=True).start()

            # Middle: Status + Title
            mid_frame = ctk.CTkFrame(row, fg_color="transparent")
            mid_frame.pack(side="left", fill="both", expand=True, pady=10)

            title_row = ctk.CTkFrame(mid_frame, fg_color="transparent")
            title_row.pack(anchor="w", fill="x")

            # Interactive Checkmark Button
            is_completed = ep['status'] == 'Completed'
            icon = "✓" if is_completed else "○"
            if ep['status'] == 'Watching': icon = "◐"

            icon_color = SUCCESS_COLOR if is_completed else (VLC_ORANGE if ep['status'] == 'Watching' else TEXT_SECONDARY)
            hover_color = SUCCESS_COLOR if not is_completed else "#333"

            status_btn = ctk.CTkButton(title_row, text=icon, width=30, height=30, corner_radius=15,
                                       fg_color="transparent", hover_color=hover_color,
                                       text_color=icon_color, font=("Inter", 18, "bold"))
            status_btn.pack(side="left", padx=(0, 5))

            color = TEXT_PRIMARY if ep['status'] != 'Completed' else TEXT_SECONDARY

            if m_type == 'TV':
                ep_id_text = f"{ep['ep_num']}. "
                ctk.CTkLabel(title_row, text=ep_id_text, font=("Inter", 15, "bold"), text_color=TEXT_SECONDARY).pack(side="left")

            title_font = ("Inter", 15, "bold") if ep['status'] != 'Completed' else ("Inter", 15, "normal")

            # Keep a reference to the label to mutate its color/font later
            ep_title_lbl = ctk.CTkLabel(title_row, text=(ep['title'] or 'Unknown Title'), font=title_font, text_color=color, anchor="w")
            ep_title_lbl.pack(side="left")

            # Bind the toggle command
            status_btn.configure(command=lambda btn=status_btn, lbl=ep_title_lbl, eid=ep['id'], mid=media_id, s=season_num, c=is_completed: self._toggle_watch_status_inplace(btn, lbl, eid, mid, s, not c))

            runtime_text = f"{(ep['runtime'] or 0)}m" if (ep['runtime'] or 0) else ""
            if runtime_text:
                ctk.CTkLabel(title_row, text=runtime_text, font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY).pack(side="left", padx=(10, 0))

            # Air date next to runtime
            if 'air_date' in ep.keys() and ep['air_date']:
                from datetime import datetime
                try:
                    dt = datetime.strptime(ep['air_date'], '%Y-%m-%d')
                    air_str = dt.strftime('%b %d, %Y')
                    ctk.CTkLabel(title_row, text=f"• Aired: {air_str}", font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY).pack(side="left", padx=(10, 0))
                except ValueError:
                    pass

            # Episode Synopsis
            if 'overview' in ep.keys() and (ep['overview'] or ''):
                synopsis = (ep['overview'] or '')
                if len(synopsis) > 120:
                    synopsis = synopsis[:117] + "..."
                ctk.CTkLabel(mid_frame, text=synopsis, font=("Inter", 11, "normal"), text_color=TEXT_SECONDARY, anchor="w", justify="left").pack(anchor="w", padx=(38, 0), pady=(0, 0))

            # Right side: Controls
            right_frame = ctk.CTkFrame(row, fg_color="transparent")
            right_frame.pack(side="right", padx=15, pady=10)

            # Play Button
            has_file = bool(ep['file_path'])
            if has_file:
                # Use a circular play button for sleekness
                play_btn = ctk.CTkButton(right_frame, text="▶", width=40, height=40, corner_radius=20,
                                         fg_color="transparent", border_color=VLC_ORANGE, border_width=2,
                                         hover_color=VLC_ORANGE_HOVER, text_color=VLC_ORANGE,
                                         font=("Inter", 18, "normal"), command=lambda e=ep: self._play_episode(e))
                play_btn.pack(side="right")
            else:
                play_btn = ctk.CTkButton(right_frame, text="☁️", width=40, height=40, corner_radius=20,
                                         fg_color="transparent", text_color=TEXT_SECONDARY, state="disabled", font=("Inter", 18, "normal"))
                play_btn.pack(side="right")

    def _toggle_watch_status_inplace(self, btn, lbl, episode_id, media_id, season_num, mark_as_completed):
        # 1. Update UI Instantly
        if mark_as_completed:
            btn.configure(text="✓", text_color=SUCCESS_COLOR, hover_color="#333")
            lbl.configure(text_color=TEXT_SECONDARY, font=("Inter", 15, "normal"))
            self._current_watched_eps += 1
        else:
            btn.configure(text="○", text_color=TEXT_SECONDARY, hover_color=SUCCESS_COLOR)
            lbl.configure(text_color=TEXT_PRIMARY, font=("Inter", 15, "bold"))
            self._current_watched_eps = max(0, self._current_watched_eps - 1)

        # Re-bind the opposite action
        btn.configure(command=lambda: self._toggle_watch_status_inplace(btn, lbl, episode_id, media_id, season_num, not mark_as_completed))

        # Update the main progress badge
        if hasattr(self, '_current_progress_badge') and self._current_progress_badge.winfo_exists():
            status_text = "Completed" if self._current_watched_eps == self._current_total_eps and self._current_watched_eps > 0 else "Watching"
            status_color = SUCCESS_COLOR if status_text == "Completed" else VLC_ORANGE
            self._current_progress_badge.configure(
                text=f"{self._current_watched_eps} / {self._current_total_eps} Eps",
                fg_color=status_color
            )

        # 2. Update Database Asynchronously
        def _write(conn):
            cursor = conn.cursor()
            if mark_as_completed:
                cursor.execute("UPDATE Episodes SET watch_count=MAX(1, watch_count), status='Completed' WHERE id=?", (episode_id,))
                cursor.execute("INSERT INTO History (episode_id) VALUES (?)", (episode_id,))
            else:
                cursor.execute("UPDATE Episodes SET watch_count=0, status='Unwatched' WHERE id=?", (episode_id,))

        self.data_manager.submit_write_task(_write)

    def _mark_season_watched_popup(self, media_id, season_num):
        dialog = ctk.CTkToplevel(self)
        dialog.title(f"Mark Season {season_num} Watched")
        dialog.geometry("450x300")
        dialog.grab_set()

        ctk.CTkLabel(dialog, text="How do you want to log this season?", font=("Inter", 16, "bold"), text_color=TEXT_PRIMARY).pack(pady=(20, 10))

        btn_frame = ctk.CTkFrame(dialog, fg_color="transparent")
        btn_frame.pack(fill="x", padx=20, pady=10)

        def log_today():
            self._execute_season_triage(media_id, season_num, mode="today")
            dialog.destroy()

        def log_archive():
            self._execute_season_triage(media_id, season_num, mode="archive")
            dialog.destroy()

        ctk.CTkButton(btn_frame, text="Log Today (Now)", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, height=40, command=log_today).pack(fill="x", pady=5)
        ctk.CTkButton(btn_frame, text="Archive (Skip History)", fg_color=SURFACE_COLOR, hover_color="#333", height=40, command=log_archive).pack(fill="x", pady=5)

        # Backdate UI
        bd_frame = ctk.CTkFrame(dialog, fg_color="transparent")
        bd_frame.pack(fill="x", padx=20, pady=(15, 5))

        ctk.CTkLabel(bd_frame, text="Or backdate to a specific time:", font=("Inter", 12, "bold"), text_color=TEXT_SECONDARY).pack(anchor="w")

        pickers = ctk.CTkFrame(bd_frame, fg_color="transparent")
        pickers.pack(fill="x", pady=5)

        from datetime import datetime
        current_year = datetime.now().year
        years = [str(y) for y in range(current_year, 1950, -1)]
        months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]

        year_var = ctk.StringVar(value=str(current_year))
        month_var = ctk.StringVar(value="")

        year_opt = ctk.CTkOptionMenu(pickers, variable=year_var, values=years, width=100)
        year_opt.pack(side="left", padx=(0, 10))

        month_opt = ctk.CTkOptionMenu(pickers, variable=month_var, values=["(Unknown Month)"] + months, width=140)
        month_opt.pack(side="left")

        def log_backdate():
            m = month_var.get()
            m_val = m if m != "(Unknown Month)" else None
            self._execute_season_triage(media_id, season_num, mode="backdate", year=year_var.get(), month=m_val)
            dialog.destroy()

        ctk.CTkButton(bd_frame, text="Backdate Season", fg_color="#3a7ebf", hover_color="#2b5e8f", height=36, command=log_backdate).pack(anchor="e", pady=10)


    def _execute_season_triage(self, media_id, season_num, mode, year=None, month=None):
        import uuid
        from datetime import datetime

        def _write(conn):
            cursor = conn.cursor()
            # Mark episodes as watched
            cursor.execute("UPDATE Episodes SET status='Completed', watch_count=MAX(1, watch_count) WHERE media_id=? AND season_num=?", (media_id, season_num))

            cursor.execute("SELECT id FROM Episodes WHERE media_id=? AND season_num=?", (media_id, season_num))
            ep_ids = [r['id'] for r in cursor.fetchall()]

            if mode == "today":
                session_id = str(uuid.uuid4())
                dt_str = datetime.now().strftime('%Y-%m-%d %H:%M:%S')
                for eid in ep_ids:
                    cursor.execute("""
                        INSERT INTO History (episode_id, timestamp, session_id, is_legacy, completion_ratio)
                        VALUES (?, ?, ?, 0, 1.0)
                    """, (eid, dt_str, session_id))
            elif mode == "backdate":
                # Static Binge Block logic
                session_id = f"legacy_binge_{uuid.uuid4()}"

                # Determine date string
                m_num = "01"
                if month:
                    months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]
                    try:
                        m_idx = months.index(month) + 1
                        m_num = f"{m_idx:02d}"
                    except ValueError:
                        pass

                # We use the 15th of the month at noon as a safe fallback date for grouping
                dt_str = f"{year}-{m_num}-15 12:00:00"

                for eid in ep_ids:
                    cursor.execute("""
                        INSERT INTO History (episode_id, timestamp, session_id, is_legacy, completion_ratio)
                        VALUES (?, ?, ?, 1, 1.0)
                    """, (eid, dt_str, session_id))
            elif mode == "archive":
                # Do nothing with History, just mark as completed in Episodes.
                pass

        def refresh_ui():
            # Update the progress badge
            conn = self.data_manager.get_db_connection()
            c = conn.cursor()
            c.execute("SELECT COUNT(*) as c FROM Episodes WHERE media_id=? AND status='Completed'", (media_id,))
            w_eps = c.fetchone()['c']
            conn.close()

            if hasattr(self, '_current_progress_badge') and self._current_progress_badge.winfo_exists():
                self._current_watched_eps = w_eps
                status_text = "Completed" if self._current_watched_eps == self._current_total_eps and self._current_watched_eps > 0 else "Watching"
                status_color = SUCCESS_COLOR if status_text == "Completed" else VLC_ORANGE
                self._current_progress_badge.configure(
                    text=f"{self._current_watched_eps} / {self._current_total_eps} Eps",
                    fg_color=status_color
                )
            self._load_episodes(media_id, season_num)

        self.data_manager.submit_write_task(_write, callback=lambda: self.after(0, refresh_ui))

    def _adj_watch(self, episode_id, media_id, season_num, delta):
        # Read synchronously
        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()
        cursor.execute("SELECT watch_count FROM Episodes WHERE id=?", (episode_id,))
        count = cursor.fetchone()['watch_count']
        conn.close()

        new_count = max(0, count + delta)
        status = 'Completed' if new_count > 0 else 'Unwatched'

        def _write(w_conn):
            w_cursor = w_conn.cursor()
            w_cursor.execute("UPDATE Episodes SET watch_count=?, status=? WHERE id=?", (new_count, status, episode_id))
            if delta > 0:
                w_cursor.execute("INSERT INTO History (episode_id) VALUES (?)", (episode_id,))

        self.data_manager.submit_write_task(_write, callback=lambda: self.after(0, lambda: self._load_episodes(media_id, season_num)))

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
            import uuid
            from datetime import datetime

            # Auto-binge detection logic
            conn = self.data_manager.get_db_connection()
            cursor = conn.cursor()

            # Find recent session for this show within last 6 hours
            cursor.execute("""
                SELECT session_id, timestamp FROM History
                WHERE episode_id IN (SELECT id FROM Episodes WHERE media_id=?) AND is_legacy=0
                ORDER BY timestamp DESC LIMIT 1
            """, (ep_data['media_id'],))
            last_hist = cursor.fetchone()

            session_id = str(uuid.uuid4())
            if last_hist:
                from datetime import timezone
                last_dt = datetime.strptime(last_hist['timestamp'], '%Y-%m-%d %H:%M:%S').replace(tzinfo=timezone.utc)
                now_dt = datetime.now(timezone.utc)

                # If gap is less than 6 hours (21600 seconds), join the binge
                if (now_dt - last_dt).total_seconds() < 21600:
                    session_id = last_hist['session_id']

            conn.close()

            start_dt_str = datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S')

            # Pre-insert History row
            def _insert_hist(conn):
                cursor = conn.cursor()
                cursor.execute("""
                    INSERT INTO History (episode_id, timestamp, session_id, is_legacy, start_time, pause_count, completion_ratio)
                    VALUES (?, ?, ?, 0, ?, 0, ?)
                """, (ep_data['id'], start_dt_str, session_id, start_dt_str, 0.0))
                # Also set episode to watching
                if ep_data['status'] == 'Unwatched':
                    cursor.execute("UPDATE Episodes SET status='Watching' WHERE id=?", (ep_data['id'],))

            self.data_manager.submit_write_task(_insert_hist)

            threading.Thread(target=self._vlc_heartbeat, args=(proc, ep_data['id'], ep_data['media_id'], ep_data['season_num'], session_id, start_dt_str), daemon=True).start()

    def _vlc_heartbeat(self, proc, episode_id, media_id, season_num, session_id, start_dt_str):
        high_water_mark = 0.0
        last_time_seconds = 0.0
        pause_count = 0
        was_paused = False

        while proc.poll() is None:
            time.sleep(5)
            status = get_vlc_status()
            if status and status['length'] > 0:
                pos = status['time'] / status['length']
                if pos > high_water_mark:
                    high_water_mark = pos

                # Check pause state
                is_paused = status.get('state') == 'paused'
                if is_paused and not was_paused:
                    pause_count += 1
                was_paused = is_paused

                last_time_seconds = status['time']

                # Continuous Heartbeat write for Dashboard resume safety
                def _heartbeat_write(conn):
                    cursor = conn.cursor()
                    cursor.execute("UPDATE Episodes SET last_position=? WHERE id=?", (int(last_time_seconds), episode_id))
                    cursor.execute("""
                        UPDATE History SET completion_ratio=?, pause_count=?
                        WHERE episode_id=? AND timestamp=? AND session_id=?
                    """, (high_water_mark, pause_count, episode_id, start_dt_str, session_id))
                self.data_manager.submit_write_task(_heartbeat_write)

        # Process closed
        from datetime import datetime, timezone
        end_dt_str = datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S')

        def _write(conn):
            cursor = conn.cursor()
            if high_water_mark > 0.90:
                cursor.execute("""
                    UPDATE Episodes
                    SET watch_count = watch_count + 1, status = 'Completed', last_position = 0
                    WHERE id = ?
                """, (episode_id,))
                # Ensure history row marks 100% completion
                cursor.execute("""
                    UPDATE History SET completion_ratio=1.0, end_time=?
                    WHERE episode_id=? AND timestamp=? AND session_id=?
                """, (end_dt_str, episode_id, start_dt_str, session_id))
            else:
                # If we didn't hit 90%, but we watched something, leave it in watching state
                if high_water_mark > 0.05: # At least 5% to avoid accidental clicks
                    cursor.execute("""
                        UPDATE Episodes
                        SET status = 'Watching', last_position = ?
                        WHERE id = ? AND status != 'Completed'
                    """, (int(last_time_seconds), episode_id))

                    cursor.execute("""
                        UPDATE History SET completion_ratio=?, end_time=?
                        WHERE episode_id=? AND timestamp=? AND session_id=?
                    """, (high_water_mark, end_dt_str, episode_id, start_dt_str, session_id))
                else:
                    # Too short, delete history row so we don't spam 2 second logs
                    cursor.execute("""
                        DELETE FROM History WHERE episode_id=? AND timestamp=? AND session_id=?
                    """, (episode_id, start_dt_str, session_id))

        self.data_manager.submit_write_task(_write)

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

    def _finish_scan(self, new_count):
        ToastNotification(self, title="Scan Complete", message=f"Finished. Found {new_count} new unmatched files.", duration=4000, color="#1b5e20")
        # If user is currently looking at unmatched list, refresh it
        if hasattr(self, 'nav_btns') and self.nav_btns["Inbox"].cget("fg_color") == SURFACE_COLOR:
            self._show_unmatched()

    def _show_unmatched(self):
        self._highlight_nav("Inbox")
        self._clear_main_frame()

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM Unmatched_Files")
        unmatched_files = cursor.fetchall()

        cursor.execute("SELECT COUNT(DISTINCT group_key) as group_count FROM Unmatched_Files")
        row = cursor.fetchone()
        group_count = row['group_count'] if row else 0
        conn.close()

        ctk.CTkLabel(self.main_frame, text="Unmatched Files", font=("Inter", 24, "bold")).pack(anchor="w", padx=20, pady=(20, 5))

        if not unmatched_files:
            ctk.CTkLabel(self.main_frame, text="No unmatched files on your hard drive.").pack(pady=20)
            return

        ctk.CTkLabel(self.main_frame, text=f"You have {group_count} unrecognized series on your hard drive.", font=("Inter", 14, "normal"), text_color="gray").pack(anchor="w", padx=20, pady=(0, 20))

        # Split pane for Unmatched
        split_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        split_frame.pack(fill="both", expand=True, padx=20, pady=10)

        # Left Pane: Inbox List
        inbox_frame = ctk.CTkScrollableFrame(split_frame, width=300, fg_color=SURFACE_COLOR, corner_radius=12)
        inbox_frame.pack(side="left", fill="y", padx=(0, 10))

        # Right Pane: Action Area
        self.action_area = ctk.CTkFrame(split_frame, fg_color="transparent")
        self.action_area.pack(side="left", fill="both", expand=True)

        ctk.CTkLabel(self.action_area, text="Select a group to triage.", font=("Inter", 16, "normal"), text_color=TEXT_SECONDARY).pack(pady=100)

        # Group files
        groups = {}
        for uf in unmatched_files:
            g_key = uf['group_key'] or 'Unknown'
            if g_key not in groups:
                groups[g_key] = []
            groups[g_key].append(uf)

        for group_name, files in groups.items():
            item_btn = ctk.CTkButton(inbox_frame, text=f"📁 {group_name} ({len(files)})", anchor="w",
                                     fg_color="transparent", hover_color="#333", text_color=TEXT_PRIMARY,
                                     font=("Inter", 14, "bold"), height=40,
                                     command=lambda gn=group_name, fs=files: self._populate_triage(gn, fs))
            item_btn.pack(fill="x", pady=2, padx=5)

    def _populate_triage(self, group_name, files, restore_scroll=False):
        # Save toggle state if it exists
        is_manual = getattr(self, "show_manual", ctk.BooleanVar(value=False)).get()

        for widget in self.action_area.winfo_children():
            widget.destroy()

        top = ctk.CTkFrame(self.action_area, fg_color="transparent")
        top.pack(fill="x", pady=20)

        search_var = ctk.StringVar(value=group_name)
        search_entry = ctk.CTkEntry(top, textvariable=search_var, font=("Inter", 24, "bold"),
                                    height=50, fg_color=SURFACE_COLOR, border_color="#333")
        search_entry.pack(side="left", fill="x", expand=True, padx=(0, 10))

        ctk.CTkButton(top, text="Search TMDB", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, height=50,
                      font=("Inter", 16, "bold"),
                      command=lambda: self._match_group(search_var.get(), files)).pack(side="left")

        ctk.CTkButton(top, text="🗑️ Ignore", fg_color=DANGER_COLOR, hover_color="#8e0000", height=50, width=50,
                      command=lambda: self._ignore_group(group_name)).pack(side="left", padx=(10, 0))

        # Middle: instructions / results / toggle
        mid_frame = ctk.CTkFrame(self.action_area, fg_color="transparent")
        mid_frame.pack(fill="x", pady=(0, 20))

        ctk.CTkLabel(mid_frame, text="Click 'Search TMDB' to find a match and assign all below files.",
                     font=("Inter", 14, "normal"), text_color=TEXT_SECONDARY).pack(side="left")

        # Toggle for manual matching
        if not hasattr(self, "show_manual") or not restore_scroll:
            self.show_manual = ctk.BooleanVar(value=False)

        manual_switch = ctk.CTkSwitch(mid_frame, text="Advanced / Manual Match", variable=self.show_manual,
                                      command=lambda gn=group_name, fs=files: self._populate_triage(gn, fs, restore_scroll=True),
                                      font=("Inter", 12, "normal"), text_color=TEXT_SECONDARY)
        manual_switch.pack(side="right")

        # Bottom: Clean table of files
        table_frame = ctk.CTkScrollableFrame(self.action_area, fg_color=SURFACE_COLOR, corner_radius=12)
        table_frame.pack(fill="both", expand=True)

        for i, uf in enumerate(files):
            row = ctk.CTkFrame(table_frame, fg_color="transparent" if i % 2 == 0 else "#252830", height=30)
            row.pack(fill="x")
            row.pack_propagate(False)

            ctk.CTkLabel(row, text=uf['filename'], font=("Inter", 13, "normal"), anchor="w").pack(side="left", padx=10)

            # Advanced Match
            if self.show_manual.get():
                ctk.CTkButton(row, text="Manual...", width=60, height=24, fg_color="transparent", hover_color="#333", text_color=TEXT_SECONDARY,
                              command=lambda f=uf: self._assign_unmatched(f)).pack(side="right", padx=10)

    def _ignore_group(self, group_name):
        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()
        cursor.execute("DELETE FROM Unmatched_Files WHERE group_key=?", (group_name,))
        conn.commit()
        conn.close()
        self._show_unmatched()

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

    def _add_to_tracker(self, media_data, archive=False):
        api_key = self.data_manager.settings.get("tmdb_api_key")

        def fetch_and_save():
            try:
                # 1. Fetch details (Network calls happen off the main thread, before the DB lock)
                details = get_media_details(api_key, media_data['tmdb_id'], media_data['type'])
                if not details: return

                # Download Poster & Backdrop
                if details['poster_path']:
                    download_image(details['poster_path'])
                if details.get('backdrop_path'):
                    download_image(details['backdrop_path'])

                # Pre-fetch TV seasons to avoid network calls inside the DB write lock
                all_eps = []
                if details['type'] == 'TV':
                    for season in details['seasons']:
                        s_num = season.get('season_number')
                        if s_num == 0: continue
                        eps = get_tv_season_episodes(api_key, details['tmdb_id'], s_num)
                        all_eps.extend(eps)

                # Capture pending group match early
                pending_group = getattr(self, '_pending_group_match', None)

                # Shared state to communicate result from queue thread back to main thread
                result_state = {"status": None, "assigned_count": 0}

                # 2. Execute DB writes safely via task queue
                def _write(conn):
                    cursor = conn.cursor()
                    cursor.execute("SELECT id FROM Media WHERE tmdb_id=?", (details['tmdb_id'],))
                    existing = cursor.fetchone()

                    if existing:
                        result_state["status"] = "exists"
                        media_id = existing['id']
                    else:
                        cursor.execute("""
                                INSERT INTO Media (tmdb_id, type, title, synopsis, poster_path, backdrop_path, total_episodes, status, vote_average, release_date)
                                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                        """, (details['tmdb_id'], details['type'], (details['title'] or 'Unknown Title'), details['synopsis'],
                                  details['poster_path'], details.get('backdrop_path', ''), details['total_episodes'], details['status'], details.get('vote_average', 0.0), details.get('release_date', '')))

                        media_id = cursor.lastrowid
                        result_state["status"] = "added"

                        if details['type'] == 'TV':
                            for ep in all_eps:
                                still_path = ep.get('still_path', '')
                                cursor.execute("""
                                    INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview, status, watch_count, air_date)
                                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                                """, (media_id, ep['season_num'], ep['ep_num'], (ep['title'] or 'Unknown Title'), (ep['runtime'] or 0), still_path, ep.get('overview', ''), 'Completed' if archive else 'Unwatched', 1 if archive else 0, ep.get('air_date', '')))
                        else:
                            cursor.execute("""
                                INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview, status, watch_count, air_date)
                                VALUES (?, 1, 1, ?, ?, ?, ?, ?, ?, ?)
                            """, (media_id, 1, 1, (details['title'] or 'Unknown Title'), details.get('runtime', 0), details.get('backdrop_path', ''), details['synopsis'], 'Completed' if archive else 'Unwatched', 1 if archive else 0, details.get('release_date', '')))

                    if pending_group:
                        import sqlite3
                        assigned_count = 0
                        for uf in pending_group:
                            s_num = uf['parsed_season'] if 'parsed_season' in uf.keys() and uf['parsed_season'] is not None else None
                            e_num = uf['parsed_episode'] if 'parsed_episode' in uf.keys() and uf['parsed_episode'] is not None else None
                            if details['type'] == 'Movie':
                                s_num, e_num = 1, 1

                            if s_num is not None and e_num is not None:
                                cursor.execute("SELECT id FROM Episodes WHERE media_id=? AND season_num=? AND ep_num=?", (media_id, s_num, e_num))
                                ep_row = cursor.fetchone()
                                if ep_row:
                                    try:
                                        cursor.execute("""
                                            INSERT INTO Local_Files (episode_id, file_path) VALUES (?, ?)
                                            ON CONFLICT(episode_id) DO UPDATE SET file_path=excluded.file_path
                                        """, (ep_row['id'], uf['file_path']))
                                        cursor.execute("DELETE FROM Unmatched_Files WHERE file_path=?", (uf['file_path'],))
                                        assigned_count += 1
                                    except sqlite3.IntegrityError:
                                        pass
                        result_state["assigned_count"] = assigned_count

                def on_complete():
                    if pending_group:
                        self.after(0, lambda: messagebox.showinfo("Success", f"Added {(details['title'] or 'Unknown Title')} and assigned {result_state['assigned_count']} files!"))
                        self._pending_group_match = None
                    else:
                        if result_state["status"] == "added":
                            self.after(0, lambda: messagebox.showinfo("Success", f"Added {(details['title'] or 'Unknown Title')} to tracker!"))
                        elif result_state["status"] == "exists":
                            self.after(0, lambda: messagebox.showinfo("Exists", "This media is already tracked."))

                    self.after(0, self._refresh_if_on_unmatched)

                self.data_manager.submit_write_task(_write, callback=on_complete)

            except Exception as e:
                self.after(0, lambda: messagebox.showerror("Error", f"Failed to add media: {e}"))

        threading.Thread(target=fetch_and_save, daemon=True).start()

    def _refresh_if_on_unmatched(self):
        if hasattr(self, 'nav_btns') and self.nav_btns["Inbox"].cget("fg_color") == SURFACE_COLOR:
            self._show_unmatched()

    # =========================================================================
    # MEDIA DEEP DIVE
    # =========================================================================

    def _assign_unmatched(self, uf):

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

        media_names = [f"[{m['type']}] {(m['title'] or 'Unknown Title')}" for m in media_items]
        media_map = {f"[{m['type']}] {(m['title'] or 'Unknown Title')}": (m['id'], m['type']) for m in media_items}

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
                    cursor.execute("DELETE FROM Unmatched_Files WHERE file_path=?", (uf['file_path'],))
                    conn.commit()
                    conn.close()

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
        self._highlight_nav("Settings")
        self._clear_main_frame()

        ctk.CTkLabel(self.main_frame, text="Settings", font=("Inter", 24, "bold")).pack(anchor="w", padx=20, pady=20)

        form_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        form_frame.pack(fill="x", padx=20)

        # TMDB Key
        ctk.CTkLabel(form_frame, text="TMDB API Key:", font=("Inter", 13, "bold")).pack(anchor="w", pady=(10, 5))
        tmdb_entry = ctk.CTkEntry(form_frame, width=400)
        tmdb_entry.pack(anchor="w")
        tmdb_entry.insert(0, self.data_manager.settings.get("tmdb_api_key", ""))

        # VLC Path
        ctk.CTkLabel(form_frame, text="VLC Executable Path:", font=("Inter", 13, "bold")).pack(anchor="w", pady=(20, 5))
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
