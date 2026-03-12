import os
import time

import customtkinter as ctk

# --- MONKEY PATCH TURBO SCROLL FOR CTKSCROLLABLEFRAME ---
original_mouse_wheel = ctk.windows.widgets.ctk_scrollable_frame.CTkScrollableFrame._mouse_wheel_all
def turbo_mouse_wheel(self, event):
    if self.winfo_exists():
        if event.state == 0 and event.delta:
            # Multiply scroll delta by 4 for native, fast feel
            event.delta = event.delta * 4
    return original_mouse_wheel(self, event)

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
            font=ctk.CTkFont(family="Inter", size=13)
        )
        self.quick_search_entry.pack(side="left")
        self.quick_search_entry.bind("<KeyRelease>", self._handle_quick_search)
        self.quick_search_entry.bind("<Return>", self._handle_quick_search)

        # --- SIDEBAR ---
        self.sidebar_frame = ctk.CTkFrame(self, width=220, corner_radius=0, fg_color=BG_COLOR)
        self.sidebar_frame.grid(row=1, column=0, sticky="nsew")
        self.sidebar_frame.grid_rowconfigure(6, weight=1) # Push settings to bottom

        logo_label = ctk.CTkLabel(self.sidebar_frame, text="▶ WatchMark", font=ctk.CTkFont(family="Inter", size=22, weight="bold"), text_color=TEXT_PRIMARY)
        logo_label.grid(row=0, column=0, padx=20, pady=(20, 30), sticky="w")

        self.nav_btns = {}
        self.nav_indicators = {}

        def create_nav_btn(row, text, command):
            container = ctk.CTkFrame(self.sidebar_frame, fg_color="transparent", height=40)
            container.grid(row=row, column=0, sticky="ew", pady=2)
            container.grid_propagate(False)

            indicator = ctk.CTkFrame(container, width=4, corner_radius=0, fg_color="transparent")
            indicator.pack(side="left", fill="y")

            btn = ctk.CTkButton(container, text=text, anchor="w", fg_color="transparent",
                                text_color=TEXT_SECONDARY, hover_color=SURFACE_COLOR, command=command,
                                font=ctk.CTkFont(family="Inter", size=14, weight="bold"))
            btn.pack(side="left", fill="both", expand=True, padx=(10, 15))

            self.nav_btns[text] = btn
            self.nav_indicators[text] = indicator
            return btn

        create_nav_btn(1, "Dashboard", self._show_dashboard)
        create_nav_btn(2, "TV Shows", self._show_tv_shows)
        create_nav_btn(3, "Movies", self._show_movies)
        create_nav_btn(4, "Search", self._show_search)
        create_nav_btn(5, "📥 Inbox", self._show_unmatched)

        # Internally keep track of Inbox without the icon to map easily
        self.nav_btns["Inbox"] = self.nav_btns["📥 Inbox"]
        self.nav_indicators["Inbox"] = self.nav_indicators["📥 Inbox"]
        del self.nav_btns["📥 Inbox"]
        del self.nav_indicators["📥 Inbox"]

        # Bottom section: Settings & Status
        bottom_frame = ctk.CTkFrame(self.sidebar_frame, fg_color="transparent")
        bottom_frame.grid(row=7, column=0, sticky="ew", pady=(0, 20), padx=20)

        settings_btn = ctk.CTkButton(bottom_frame, text="⚙️ Settings", anchor="w", fg_color="transparent",
                                     text_color=TEXT_SECONDARY, hover_color=SURFACE_COLOR, command=self._show_settings,
                                     font=ctk.CTkFont(family="Inter", size=14, weight="bold"))
        settings_btn.pack(fill="x", pady=(0, 10))
        self.nav_btns["Settings"] = settings_btn

        # Dummy status indicator
        status_lbl = ctk.CTkLabel(bottom_frame, text="🟢 DB Connected", font=ctk.CTkFont(family="Inter", size=11), text_color=SUCCESS_COLOR)
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

        # 1. Fetch the absolute most recently watched episode for the Hero Section
        cursor.execute("""
            SELECT e.*, m.title as show_title, m.backdrop_path, l.file_path, m.type as media_type
            FROM History h
            JOIN Episodes e ON h.episode_id = e.id
            JOIN Media m ON e.media_id = m.id
            LEFT JOIN Local_Files l ON e.id = l.episode_id
            ORDER BY h.timestamp DESC
            LIMIT 1
        """)
        hero_ep = cursor.fetchone()

        if hero_ep:
            # Render Hero Section
            hero_frame = ctk.CTkFrame(dash_scroll, height=350, fg_color=SURFACE_COLOR, corner_radius=12)
            hero_frame.pack(fill="x", padx=20, pady=(20, 10))
            hero_frame.pack_propagate(False)

            bg_label = ctk.CTkLabel(hero_frame, text="")
            bg_label.place(x=0, y=0, relwidth=1.0, relheight=1.0)

            def load_hero_bg():
                if hero_ep['backdrop_path']:
                    local_img = POSTER_CACHE_DIR / hero_ep['backdrop_path'].lstrip('/')
                    if local_img.exists():
                        try:
                            pil_img = Image.open(local_img)
                            w, h = pil_img.size
                            target_h = int(w * (350/1000))
                            if h > target_h:
                                top = (h - target_h) // 2
                                pil_img = pil_img.crop((0, top, w, top + target_h))
                            backdrop_img = ctk.CTkImage(light_image=pil_img, dark_image=pil_img, size=(1000, 350))
                            self.after(0, lambda: bg_label.configure(image=backdrop_img))
                        except Exception:
                            pass
            threading.Thread(target=load_hero_bg, daemon=True).start()

            # Text content
            content_frame = ctk.CTkFrame(hero_frame, fg_color="transparent")
            content_frame.place(relx=0.05, rely=0.5, anchor="w")

            ctk.CTkLabel(content_frame, text="UP NEXT", font=ctk.CTkFont(family="Inter", size=14, weight="bold"), text_color=VLC_ORANGE).pack(anchor="w")
            ctk.CTkLabel(content_frame, text=(hero_ep['show_title'] or 'Unknown Show'), font=ctk.CTkFont(family="Inter", size=48, weight="bold"), text_color=TEXT_PRIMARY).pack(anchor="w", pady=(5, 0))

            if hero_ep['media_type'] == 'TV':
                ep_sub = f"S{hero_ep['season_num']:02}E{hero_ep['ep_num']:02} - {(hero_ep['title'] or 'Unknown Title')}"
            else:
                ep_sub = (hero_ep['title'] or 'Unknown Title')

            ctk.CTkLabel(content_frame, text=ep_sub, font=ctk.CTkFont(family="Inter", size=18), text_color=TEXT_SECONDARY).pack(anchor="w", pady=(0, 20))

            if hero_ep['file_path']:
                play_btn = ctk.CTkButton(content_frame, text="▶ Resume", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, height=45, width=150,
                                         font=ctk.CTkFont(family="Inter", size=16, weight="bold"),
                                         command=lambda e=hero_ep: self._play_episode(e))
                play_btn.pack(anchor="w")
            else:
                play_btn = ctk.CTkButton(content_frame, text="❌ Missing File", fg_color=DANGER_COLOR, hover_color="#8e0000", height=45, width=150,
                                         font=ctk.CTkFont(family="Inter", size=16, weight="bold"), state="disabled")
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
            ctk.CTkLabel(hero_frame, text="Welcome to WatchMark", font=ctk.CTkFont(family="Inter", size=32, weight="bold")).pack(pady=(100, 10))
            ctk.CTkLabel(hero_frame, text="Scan your local folder or search TMDB to get started.", text_color=TEXT_SECONDARY).pack()

        # --- Horizontal Rows ---

        # Continue Watching (Other than hero)
        ctk.CTkLabel(dash_scroll, text="Continue Watching", font=ctk.CTkFont(family="Inter", size=20, weight="bold")).pack(anchor="w", padx=25, pady=(20, 10))
        cw_frame = ctk.CTkScrollableFrame(dash_scroll, orientation="horizontal", height=220, fg_color="transparent")
        cw_frame.pack(fill="x", padx=15)

        cursor.execute("""
            SELECT e.media_id, MAX(h.timestamp) as last_watched
            FROM History h
            JOIN Episodes e ON h.episode_id = e.id
            GROUP BY e.media_id
            ORDER BY last_watched DESC
            LIMIT 15
        """)
        recent_media_ids = [r['media_id'] for r in cursor.fetchall()]

        cw_eps = []
        for m_id in recent_media_ids:
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
        ctk.CTkLabel(dash_scroll, text="Recently Added", font=ctk.CTkFont(family="Inter", size=20, weight="bold")).pack(anchor="w", padx=25, pady=(20, 10))
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
        ctk.CTkLabel(dash_scroll, text="Your Stats", font=ctk.CTkFont(family="Inter", size=20, weight="bold")).pack(anchor="w", padx=25, pady=(20, 10))
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
            ctk.CTkLabel(f, text=title, font=ctk.CTkFont(family="Inter", size=14), text_color=TEXT_SECONDARY).pack(pady=(20, 5))
            ctk.CTkLabel(f, text=str(value), font=ctk.CTkFont(family="Inter", size=28, weight="bold"), text_color=VLC_ORANGE).pack()

        make_stat_card(stats_frame, "Episodes Watched", eps_watched)
        make_stat_card(stats_frame, "Hours Watched", hrs_watched)
        make_stat_card(stats_frame, "Shows Completed", shows_completed)

        conn.close()

    def _create_horizontal_episode_card(self, parent, ep_row):
        card = ctk.CTkFrame(parent, width=280, height=200, fg_color=SURFACE_COLOR, corner_radius=8)
        card.pack(side="left", padx=10, pady=5)
        card.pack_propagate(False)

        img_label = ctk.CTkLabel(card, text="No Image", width=280, height=158, fg_color="#15171e")
        img_label.pack(fill="x")

        # Prioritize still -> backdrop
        img_path = ep_row['still_path'] if 'still_path' in ep_row.keys() and ep_row['still_path'] else (ep_row['backdrop_path'] if 'backdrop_path' in ep_row.keys() else None)
        def load_img():
            if img_path:
                local_img = POSTER_CACHE_DIR / img_path.lstrip('/')
                if local_img.exists():
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_img), dark_image=Image.open(local_img), size=(280, 158))
                        self.after(0, lambda: img_label.configure(image=img, text=""))
                    except: pass
        threading.Thread(target=load_img, daemon=True).start()

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
        subtitle = f"S{ep_row['season_num']:02}E{ep_row['ep_num']:02}" if ep_row['media_type'] == 'TV' else (ep_row['title'] or 'Unknown Title')

        ctk.CTkLabel(info_frame, text=title, font=ctk.CTkFont(family="Inter", size=13, weight="bold"), anchor="w").pack(side="left")
        ctk.CTkLabel(info_frame, text=subtitle, font=ctk.CTkFont(family="Inter", size=12), text_color=TEXT_SECONDARY, anchor="e").pack(side="right")

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

    def _show_library(self, media_type, filter_query=None):
        self._clear_main_frame()

        header_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        header_frame.pack(fill="x", padx=20, pady=20)

        if media_type == "All":
            title = "Search Results"
        else:
            title = "TV Shows" if media_type == "TV" else "Movies"

        ctk.CTkLabel(header_frame, text=title, font=ctk.CTkFont(family="Inter", size=24, weight="bold")).pack(side="left")

        scan_btn = ctk.CTkButton(header_frame, text="📂 Scan Local Folder", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, command=self._scan_folder)
        scan_btn.pack(side="right")

        grid_frame = ctk.CTkScrollableFrame(self.main_frame, fg_color="transparent")
        grid_frame.pack(fill="both", expand=True, padx=20, pady=10)

        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()

        if media_type == "All":
            if filter_query:
                cursor.execute("SELECT * FROM Media WHERE title LIKE ?", (f"%{filter_query}%",))
            else:
                cursor.execute("SELECT * FROM Media")
        else:
            if filter_query:
                cursor.execute("SELECT * FROM Media WHERE type=? AND title LIKE ?", (media_type, f"%{filter_query}%"))
            else:
                cursor.execute("SELECT * FROM Media WHERE type=?", (media_type,))

        media_items = cursor.fetchall()
        conn.close()

        if not media_items:
            msg = f"No {title} tracked yet. Use Search to add some!"
            if filter_query:
                msg = f"'{filter_query}' not found in library. Press Enter in Search to query TMDB."

            ctk.CTkLabel(grid_frame, text=msg, font=ctk.CTkFont(family="Inter", size=16), text_color=TEXT_SECONDARY).pack(pady=50)
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
                                    width=50, height=50, corner_radius=25, font=ctk.CTkFont(size=20))
        overlay_btn.place(relx=0.5, rely=0.5, anchor="center")

        title_lbl = ctk.CTkLabel(card, text=((item['title'] or 'Unknown Title')), font=ctk.CTkFont(family="Inter", size=13, weight="bold"),
                                 wraplength=150, text_color=TEXT_PRIMARY)
        # title_lbl.pack(pady=(5, 0)) # Depending on layout needs, hide title to make it cleaner

        def load_poster():
            if item['poster_path']:
                local_img = POSTER_CACHE_DIR / item['poster_path'].lstrip('/')
                if local_img.exists():
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_img), dark_image=Image.open(local_img), size=(160, 240))
                        self.after(0, lambda: img_label.configure(image=img, text=""))
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
    # SEARCH & DISCOVER
    # =========================================================================
    def _show_search(self):
        self._pending_group_match = None # Clear pending matches when opening standard search
        self._highlight_nav("Search")
        self._clear_main_frame()

        top_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        top_frame.pack(fill="x", padx=20, pady=20)

        ctk.CTkLabel(top_frame, text="Discover Media", font=ctk.CTkFont(family="Inter", size=24, weight="bold"), text_color=TEXT_PRIMARY).pack(side="left")

        search_box = ctk.CTkFrame(top_frame, fg_color="transparent")
        search_box.pack(side="right")

        self.search_entry = ctk.CTkEntry(search_box, placeholder_text="Search TMDB for Shows or Movies...", width=350, height=36, corner_radius=18, fg_color=SURFACE_COLOR, border_color="#333", font=ctk.CTkFont(family="Inter", size=13))
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
        ctk.CTkLabel(self.results_frame, text=f"Searching TMDB for '{query}'...", font=ctk.CTkFont(family="Inter", size=16), text_color=TEXT_SECONDARY).pack(pady=50)
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
            ctk.CTkLabel(self.results_frame, text="No results found.", font=ctk.CTkFont(family="Inter", size=16), text_color=TEXT_SECONDARY).pack(pady=50)
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
        year = item['release_date'][:4] if item.get('release_date') else "N/A"
        title_lbl = ctk.CTkLabel(card, text=f"{((item['title'] or 'Unknown Title'))}\n({year})", font=ctk.CTkFont(family="Inter", size=12, weight="bold"),
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
                        self.after(0, lambda: img_label.configure(image=img, text=""))
                    except: pass
        threading.Thread(target=load_poster, daemon=True).start()

        # Hover overlay
        overlay_frame = ctk.CTkFrame(card, fg_color=BG_COLOR, corner_radius=8, width=160, height=240)

        btn_text = "✓ In Library" if is_tracked else "+ Add to Tracker"
        btn_color = SUCCESS_COLOR if is_tracked else VLC_ORANGE
        btn_hover = SUCCESS_COLOR if is_tracked else VLC_ORANGE_HOVER

        overlay_btn = ctk.CTkButton(overlay_frame, text=btn_text, fg_color=btn_color, hover_color=btn_hover,
                                    width=120, height=40, corner_radius=20, font=ctk.CTkFont(family="Inter", weight="bold"),
                                    state="disabled" if is_tracked else "normal")
        overlay_btn.place(relx=0.5, rely=0.5, anchor="center")

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
                self._add_to_tracker(item)
                overlay_btn.configure(text="Adding...", state="disabled")
            overlay_btn.configure(command=add_wrapper)

    def _add_to_tracker(self, media_data):
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
                local_img = POSTER_CACHE_DIR / media['backdrop_path'].lstrip('/')
                if local_img.exists():
                    try:
                        pil_img = Image.open(local_img)
                        w, h = pil_img.size
                        target_h = int(w * (200/1000))
                        if h > target_h:
                            top = (h - target_h) // 2
                            pil_img = pil_img.crop((0, top, w, top + target_h))
                        img = ctk.CTkImage(light_image=pil_img, dark_image=pil_img, size=(1000, 200))
                        self.after(0, lambda: banner_lbl.configure(image=img))
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
                local_img = POSTER_CACHE_DIR / media['poster_path'].lstrip('/')
                if local_img.exists():
                    try:
                        img = ctk.CTkImage(light_image=Image.open(local_img), dark_image=Image.open(local_img), size=(140, 210))
                        self.after(0, lambda: img_label.configure(image=img, text=""))
                    except: pass
        threading.Thread(target=load_poster, daemon=True).start()

        # Info Frame
        info_frame = ctk.CTkFrame(sub_header_frame, fg_color="transparent")
        info_frame.pack(side="left", anchor="nw", padx=(20, 0), fill="both", expand=True)

        title_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        title_frame.pack(anchor="w", fill="x")

        safe_title = media['title'] or 'Unknown Title'
        ctk.CTkLabel(title_frame, text=safe_title, font=ctk.CTkFont(family="Inter", size=32, weight="bold"), text_color=TEXT_PRIMARY).pack(side="left")

        cursor.execute("SELECT COUNT(*) as c FROM Episodes WHERE media_id=? AND status='Completed'", (media_id,))
        watched_eps = cursor.fetchone()['c']

        tags_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        tags_frame.pack(anchor="w", pady=(5, 10))

        safe_type = (media['type'] or 'Unknown Type') if (media['type'] or 'Unknown Type') else "Unknown Type"
        type_tag = ctk.CTkLabel(tags_frame, text=safe_type, fg_color=SURFACE_COLOR, corner_radius=10, font=ctk.CTkFont(size=12), padx=10)
        type_tag.pack(side="left", padx=(0, 5))

        total_episodes = media['total_episodes'] if media['total_episodes'] is not None else 0
        status_text = "Completed" if watched_eps == total_episodes and watched_eps > 0 else "Watching"
        status_color = SUCCESS_COLOR if status_text == "Completed" else VLC_ORANGE

        stat_tag = ctk.CTkLabel(tags_frame, text=f"{watched_eps} / {total_episodes} Eps", fg_color=status_color, corner_radius=10, font=ctk.CTkFont(size=12, weight="bold"), padx=10, text_color="white")
        stat_tag.pack(side="left")
        self._current_progress_badge = stat_tag
        self._current_watched_eps = watched_eps
        self._current_total_eps = total_episodes

        safe_synopsis = media['synopsis'] or 'No overview available.'
        ctk.CTkLabel(info_frame, text=safe_synopsis, font=ctk.CTkFont(family="Inter", size=13), text_color=TEXT_SECONDARY, wraplength=700, justify="left").pack(anchor="w", pady=5)

        # Ratings Row
        ratings_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
        ratings_frame.pack(anchor="w", pady=(5, 10))

        # TMDB Badge
        tmdb_score = round(media.get('vote_average', 0.0), 1)
        ctk.CTkLabel(ratings_frame, text=f"⭐ TMDB: {tmdb_score}/10", fg_color="#181A20", text_color="#F5C518",
                     font=ctk.CTkFont(family="Inter", size=12, weight="bold"), corner_radius=6, padx=8, pady=4).pack(side="left", padx=(0, 15))

        # User Rating Stars
        stars_frame = ctk.CTkFrame(ratings_frame, fg_color="transparent")
        stars_frame.pack(side="left")
        ctk.CTkLabel(stars_frame, text="My Score: ", font=ctk.CTkFont(family="Inter", size=12), text_color=TEXT_SECONDARY).pack(side="left", padx=(0, 5))

        user_rating = media.get('user_rating', 0)
        self.star_btns = []

        def set_rating(rating_val):
            conn = self.data_manager.get_db_connection()
            cursor = conn.cursor()
            cursor.execute("UPDATE Media SET user_rating=? WHERE id=?", (rating_val, media_id))
            conn.commit()
            conn.close()
            # Update star colors visually
            for i, btn in enumerate(self.star_btns):
                if i < rating_val:
                    btn.configure(text_color=VLC_ORANGE)
                else:
                    btn.configure(text_color="#444")

        for i in range(1, 6):
            star_color = VLC_ORANGE if i <= user_rating else "#444"
            btn = ctk.CTkButton(stars_frame, text="★", width=25, height=25, fg_color="transparent", hover_color="transparent",
                                text_color=star_color, font=ctk.CTkFont(size=22),
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
                                     font=ctk.CTkFont(family="Inter", weight="bold"), command=lambda e=next_ep: self._play_episode(e))
            play_btn.pack(side="left", padx=(0, 10))

        ctk.CTkButton(actions, text="✓ Mark All Watched", fg_color=SURFACE_COLOR, hover_color="#333", height=36,
                      command=lambda m=media_id: self._mark_all_watched(m)).pack(side="left", padx=(0, 10))

        sync_btn = ctk.CTkButton(actions, text="🔄 Refresh Data", fg_color="transparent", border_color="#555", border_width=1,
                                 hover_color=SURFACE_COLOR, text_color=TEXT_PRIMARY, height=36, font=ctk.CTkFont(family="Inter", weight="bold"),
                                 command=lambda m=media_id: self._sync_media(m))
        sync_btn.pack(side="left", padx=(0, 10))
        # Store a reference to the sync button to change its state during sync
        self._current_sync_btn = sync_btn

        # Main Area (Tabs for Seasons if TV)
        content_frame = ctk.CTkFrame(detail_scroll, fg_color="transparent")
        content_frame.pack(fill="both", expand=True, padx=20, pady=10)

        self.ep_list_frame = ctk.CTkFrame(content_frame, fg_color="transparent")

        if (media['type'] or 'Unknown Type') == 'TV':
            cursor.execute("SELECT DISTINCT season_num FROM Episodes WHERE media_id=? ORDER BY season_num", (media_id,))
            seasons = [r['season_num'] for r in cursor.fetchall()]

            if seasons:
                # Top horizontal scroll for season buttons
                season_scroll = ctk.CTkScrollableFrame(content_frame, orientation="horizontal", height=50, fg_color="transparent")
                season_scroll.pack(fill="x", pady=(0, 10))

                self.ep_list_frame.pack(fill="both", expand=True)

                self.season_btns = []

                for s in seasons:
                    btn = ctk.CTkButton(season_scroll, text=f"Season {s}", width=100, height=32, corner_radius=16,
                                        fg_color=SURFACE_COLOR, text_color=TEXT_SECONDARY, hover_color="#333",
                                        font=ctk.CTkFont(family="Inter", weight="bold"),
                                        command=lambda s_num=s, m_id=media_id: self._load_episodes(m_id, s_num))
                    btn.pack(side="left", padx=5)
                    self.season_btns.append((s, btn))

                # Load first season by default
                initial_season = target_season if target_season and target_season in seasons else seasons[0]
                self._load_episodes(media_id, initial_season)
            else:
                self.ep_list_frame.pack(fill="both", expand=True)
                ctk.CTkLabel(self.ep_list_frame, text="No episode data found. Try refreshing or re-adding this show.", font=ctk.CTkFont(family="Inter", size=14), text_color=TEXT_SECONDARY).pack(pady=50)
        else:
            self.ep_list_frame.pack(fill="both", expand=True)
            self._load_episodes(media_id, 1)

        conn.close()

    def _mark_all_watched(self, media_id):
        conn = self.data_manager.get_db_connection()
        cursor = conn.cursor()
        cursor.execute("UPDATE Episodes SET status='Completed', watch_count=MAX(1, watch_count) WHERE media_id=?", (media_id,))
        conn.commit()
        conn.close()
        self._show_media_details(media_id)

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

                details = get_media_details(api_key, tmdb_id, m_type)
                if not details:
                    conn.close()
                    return

                # Update Media table properties
                cursor.execute("""
                    UPDATE Media SET
                        title=?, synopsis=?, poster_path=?, backdrop_path=?, total_episodes=?, status=?, vote_average=?
                    WHERE id=?
                """, ((details['title'] or 'Unknown Title'), details['synopsis'], details['poster_path'], details.get('backdrop_path', ''), details['total_episodes'], details['status'], details.get('vote_average', 0.0), media_id))

                if m_type == 'TV':
                    for season in details['seasons']:
                        s_num = season.get('season_number')
                        if s_num == 0: continue

                        eps = get_tv_season_episodes(api_key, tmdb_id, s_num)
                        for ep in eps:
                            # Insert new episode, update if exists
                            # Check if episode exists first because sqlite3 might rollback the entire transaction on IntegrityError if not handled properly in python sqlite3 module
                            cursor.execute("SELECT id FROM Episodes WHERE media_id=? AND season_num=? AND ep_num=?", (media_id, s_num, ep['ep_num']))
                            existing_ep = cursor.fetchone()

                            if not existing_ep:
                                cursor.execute("""
                                    INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview)
                                    VALUES (?, ?, ?, ?, ?, ?, ?)
                                """, (media_id, s_num, ep['ep_num'], (ep['title'] or 'Unknown Title'), (ep['runtime'] or 0), ep.get('still_path', ''), ep.get('overview', '')))
                            else:
                                cursor.execute("""
                                    UPDATE Episodes SET title=?, runtime=?, still_path=?, overview=?
                                    WHERE id=?
                                """, ((ep['title'] or 'Unknown Title'), (ep['runtime'] or 0), ep.get('still_path', ''), ep.get('overview', ''), existing_ep['id']))

                # We don't need to do anything else for movies since there's only one dummy episode which rarely updates.

                conn.commit()
                conn.close()

                # Fetching new posters if they changed might be heavy, but download_image caches them based on filename
                if details['poster_path']: download_image(details['poster_path'])
                if details.get('backdrop_path'): download_image(details['backdrop_path'])

                self.after(0, lambda: self._finish_sync(media_id))
            except Exception as e:
                self.after(0, lambda: self._fail_sync(str(e)))

        threading.Thread(target=perform_sync, daemon=True).start()

    def _finish_sync(self, media_id):
        # Refresh UI, assuming we are still looking at the same show
        self._show_media_details(media_id)
        ToastNotification(self, title="Sync Complete", message="Data refreshed successfully.", duration=3000, color="#1b5e20")

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
            img_lbl = ctk.CTkLabel(img_frame, text="", fg_color="#1A1C23", font=ctk.CTkFont(size=10))
            img_lbl.pack(fill="both", expand=True)

            def load_still(ep_data, fallback_backdrop):
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
                            img_lbl.configure(image=img, text=f"EP {ep_data['ep_num']}" if is_fallback else "",
                                              font=ctk.CTkFont(family="Inter", size=16, weight="bold"), text_color="#B3B3B3")
                        self.after(0, update_ui)
                    except Exception as e:
                        pass
                else:
                    self.after(0, lambda: img_lbl.configure(text=f"EP {ep_data['ep_num']}", text_color=TEXT_SECONDARY))

            threading.Thread(target=load_still, args=(ep, m_backdrop), daemon=True).start()

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
                                       text_color=icon_color, font=ctk.CTkFont(family="Inter", size=18, weight="bold"))
            status_btn.pack(side="left", padx=(0, 5))

            color = TEXT_PRIMARY if ep['status'] != 'Completed' else TEXT_SECONDARY

            if m_type == 'TV':
                ep_id_text = f"{ep['ep_num']}. "
                ctk.CTkLabel(title_row, text=ep_id_text, font=ctk.CTkFont(family="Inter", size=15, weight="bold"), text_color=TEXT_SECONDARY).pack(side="left")

            title_font = ctk.CTkFont(family="Inter", size=15, weight="bold") if ep['status'] != 'Completed' else ctk.CTkFont(family="Inter", size=15)

            # Keep a reference to the label to mutate its color/font later
            ep_title_lbl = ctk.CTkLabel(title_row, text=(ep['title'] or 'Unknown Title'), font=title_font, text_color=color, anchor="w")
            ep_title_lbl.pack(side="left")

            # Bind the toggle command
            status_btn.configure(command=lambda btn=status_btn, lbl=ep_title_lbl, eid=ep['id'], mid=media_id, s=season_num, c=is_completed: self._toggle_watch_status_inplace(btn, lbl, eid, mid, s, not c))

            runtime_text = f"{(ep['runtime'] or 0)}m" if (ep['runtime'] or 0) else ""
            if runtime_text:
                ctk.CTkLabel(title_row, text=runtime_text, font=ctk.CTkFont(family="Inter", size=12), text_color=TEXT_SECONDARY).pack(side="left", padx=(10, 0))

            # Episode Synopsis
            if 'overview' in ep.keys() and (ep['overview'] or ''):
                synopsis = (ep['overview'] or '')
                if len(synopsis) > 120:
                    synopsis = synopsis[:117] + "..."
                ctk.CTkLabel(mid_frame, text=synopsis, font=ctk.CTkFont(family="Inter", size=11), text_color=TEXT_SECONDARY, anchor="w", justify="left").pack(anchor="w", padx=(38, 0), pady=(0, 0))

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
                                         font=ctk.CTkFont(size=18), command=lambda e=ep: self._play_episode(e))
                play_btn.pack(side="right")
            else:
                play_btn = ctk.CTkButton(right_frame, text="☁️", width=40, height=40, corner_radius=20,
                                         fg_color="transparent", text_color=TEXT_SECONDARY, state="disabled", font=ctk.CTkFont(size=18))
                play_btn.pack(side="right")

    def _toggle_watch_status_inplace(self, btn, lbl, episode_id, media_id, season_num, mark_as_completed):
        # 1. Update UI Instantly
        if mark_as_completed:
            btn.configure(text="✓", text_color=SUCCESS_COLOR, hover_color="#333")
            lbl.configure(text_color=TEXT_SECONDARY, font=ctk.CTkFont(family="Inter", size=15))
            self._current_watched_eps += 1
        else:
            btn.configure(text="○", text_color=TEXT_SECONDARY, hover_color=SUCCESS_COLOR)
            lbl.configure(text_color=TEXT_PRIMARY, font=ctk.CTkFont(family="Inter", size=15, weight="bold"))
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
        def db_update():
            conn = self.data_manager.get_db_connection()
            cursor = conn.cursor()
            if mark_as_completed:
                cursor.execute("UPDATE Episodes SET watch_count=MAX(1, watch_count), status='Completed' WHERE id=?", (episode_id,))
                cursor.execute("INSERT INTO History (episode_id) VALUES (?)", (episode_id,))
            else:
                cursor.execute("UPDATE Episodes SET watch_count=0, status='Unwatched' WHERE id=?", (episode_id,))
            conn.commit()
            conn.close()

        threading.Thread(target=db_update, daemon=True).start()

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
                SET watch_count = watch_count + 1, status = 'Completed', last_position = 0
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
                """, (int(last_time_seconds), episode_id))

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

        ctk.CTkLabel(self.main_frame, text="Unmatched Files", font=ctk.CTkFont(size=24, weight="bold")).pack(anchor="w", padx=20, pady=(20, 5))

        if not unmatched_files:
            ctk.CTkLabel(self.main_frame, text="No unmatched files on your hard drive.").pack(pady=20)
            return

        ctk.CTkLabel(self.main_frame, text=f"You have {group_count} unrecognized series on your hard drive.", font=ctk.CTkFont(size=14), text_color="gray").pack(anchor="w", padx=20, pady=(0, 20))

        # Split pane for Unmatched
        split_frame = ctk.CTkFrame(self.main_frame, fg_color="transparent")
        split_frame.pack(fill="both", expand=True, padx=20, pady=10)

        # Left Pane: Inbox List
        inbox_frame = ctk.CTkScrollableFrame(split_frame, width=300, fg_color=SURFACE_COLOR, corner_radius=12)
        inbox_frame.pack(side="left", fill="y", padx=(0, 10))

        # Right Pane: Action Area
        self.action_area = ctk.CTkFrame(split_frame, fg_color="transparent")
        self.action_area.pack(side="left", fill="both", expand=True)

        ctk.CTkLabel(self.action_area, text="Select a group to triage.", font=ctk.CTkFont(family="Inter", size=16), text_color=TEXT_SECONDARY).pack(pady=100)

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
                                     font=ctk.CTkFont(family="Inter", size=14, weight="bold"), height=40,
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
        search_entry = ctk.CTkEntry(top, textvariable=search_var, font=ctk.CTkFont(family="Inter", size=24, weight="bold"),
                                    height=50, fg_color=SURFACE_COLOR, border_color="#333")
        search_entry.pack(side="left", fill="x", expand=True, padx=(0, 10))

        ctk.CTkButton(top, text="Search TMDB", fg_color=VLC_ORANGE, hover_color=VLC_ORANGE_HOVER, height=50,
                      font=ctk.CTkFont(family="Inter", size=16, weight="bold"),
                      command=lambda: self._match_group(search_var.get(), files)).pack(side="left")

        ctk.CTkButton(top, text="🗑️ Ignore", fg_color=DANGER_COLOR, hover_color="#8e0000", height=50, width=50,
                      command=lambda: self._ignore_group(group_name)).pack(side="left", padx=(10, 0))

        # Middle: instructions / results / toggle
        mid_frame = ctk.CTkFrame(self.action_area, fg_color="transparent")
        mid_frame.pack(fill="x", pady=(0, 20))

        ctk.CTkLabel(mid_frame, text="Click 'Search TMDB' to find a match and assign all below files.",
                     font=ctk.CTkFont(family="Inter", size=14), text_color=TEXT_SECONDARY).pack(side="left")

        # Toggle for manual matching
        if not hasattr(self, "show_manual") or not restore_scroll:
            self.show_manual = ctk.BooleanVar(value=False)

        manual_switch = ctk.CTkSwitch(mid_frame, text="Advanced / Manual Match", variable=self.show_manual,
                                      command=lambda gn=group_name, fs=files: self._populate_triage(gn, fs, restore_scroll=True),
                                      font=ctk.CTkFont(family="Inter", size=12), text_color=TEXT_SECONDARY)
        manual_switch.pack(side="right")

        # Bottom: Clean table of files
        table_frame = ctk.CTkScrollableFrame(self.action_area, fg_color=SURFACE_COLOR, corner_radius=12)
        table_frame.pack(fill="both", expand=True)

        for i, uf in enumerate(files):
            row = ctk.CTkFrame(table_frame, fg_color="transparent" if i % 2 == 0 else "#252830", height=30)
            row.pack(fill="x")
            row.pack_propagate(False)

            ctk.CTkLabel(row, text=uf['filename'], font=ctk.CTkFont(family="Inter", size=13), anchor="w").pack(side="left", padx=10)

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

    def _add_to_tracker(self, media_data):
        api_key = self.data_manager.settings.get("tmdb_api_key")

        def fetch_and_save():
            try:
                # 1. Fetch details
                details = get_media_details(api_key, media_data['tmdb_id'], media_data['type'])
                if not details: return

                # Download Poster & Backdrop
                if details['poster_path']:
                    download_image(details['poster_path'])
                if details.get('backdrop_path'):
                    download_image(details['backdrop_path'])

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
                        INSERT INTO Media (tmdb_id, type, title, synopsis, poster_path, backdrop_path, total_episodes, status, vote_average)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """, (details['tmdb_id'], details['type'], (details['title'] or 'Unknown Title'), details['synopsis'],
                          details['poster_path'], details.get('backdrop_path', ''), details['total_episodes'], details['status'], details.get('vote_average', 0.0)))

                    media_id = cursor.lastrowid

                    # Fetch Episodes if TV Show
                    if details['type'] == 'TV':
                        for season in details['seasons']:
                            s_num = season.get('season_number')
                            if s_num == 0: continue # Skip specials usually

                            eps = get_tv_season_episodes(api_key, details['tmdb_id'], s_num)
                            for ep in eps:
                                still_path = ep.get('still_path', '')
                                # User requested NOT to download every episode image synchronously here.
                                # It will be downloaded lazy-loaded on the Media details screen.
                                cursor.execute("""
                                    INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview)
                                    VALUES (?, ?, ?, ?, ?, ?, ?)
                                """, (media_id, s_num, ep['ep_num'], (ep['title'] or 'Unknown Title'), (ep['runtime'] or 0), still_path, ep.get('overview', '')))
                    else:
                        # Movie has 1 dummy episode
                        cursor.execute("""
                            INSERT INTO Episodes (media_id, season_num, ep_num, title, runtime, still_path, overview)
                            VALUES (?, 1, 1, ?, ?, ?, ?)
                        """, (media_id, 1, 1, (details['title'] or 'Unknown Title'), details.get('runtime', 0), details.get('backdrop_path', ''), details['synopsis']))

                    conn.commit()

                # Check for pending group match
                pending_group = getattr(self, '_pending_group_match', None)
                if pending_group:
                    import sqlite3
                    assigned_count = 0

                    for uf in pending_group:
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

                                    # Delete from Unmatched_Files
                                    cursor.execute("DELETE FROM Unmatched_Files WHERE file_path=?", (uf['file_path'],))

                                    assigned_count += 1
                                except sqlite3.IntegrityError:
                                    pass

                    conn.commit()

                    self.after(0, lambda ac=assigned_count: messagebox.showinfo("Success", f"Added {(details['title'] or 'Unknown Title')} and assigned {ac} files!"))
                    self._pending_group_match = None
                else:
                    if not existing:
                        self.after(0, lambda: messagebox.showinfo("Success", f"Added {(details['title'] or 'Unknown Title')} to tracker!"))
                    else:
                        self.after(0, lambda: messagebox.showinfo("Exists", "This media is already tracked."))

                conn.close()
                self.after(0, self._refresh_if_on_unmatched)

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
