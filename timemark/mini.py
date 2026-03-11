import customtkinter as ctk

class MiniBar(ctk.CTkToplevel):
    def __init__(self, parent, on_export_callback, on_close_callback):
        super().__init__(parent)
        self.parent = parent
        self.on_export_callback = on_export_callback
        self.on_close_callback = on_close_callback

        self.title("TimeMark Mini")
        self.overrideredirect(True)
        self.attributes('-topmost', True)
        self.attributes('-alpha', 0.9)

        # Center horizontally at top
        screen_width = self.winfo_screenwidth()
        w = 350
        h = 50
        x = (screen_width // 2) - (w // 2)
        y = 20
        self.geometry(f"{w}x{h}+{x}+{y}")

        self.frame = ctk.CTkFrame(self, fg_color="#1e1e1e", corner_radius=10, border_width=2, border_color="#3a7ebf")
        self.frame.pack(fill="both", expand=True, padx=2, pady=2)

        # UI Elements
        self.rec_lbl = ctk.CTkLabel(self.frame, text="⚫", text_color="gray", font=ctk.CTkFont(size=18))
        self.rec_lbl.pack(side="left", padx=(15, 5))

        self.status_lbl = ctk.CTkLabel(self.frame, text="0 Segments", width=80)
        self.status_lbl.pack(side="left", padx=5)

        self.export_btn = ctk.CTkButton(self.frame, text="🎬 Export Seamless", width=120, height=28, fg_color="#FF8800", hover_color="#E67A00", command=self.on_export_callback)
        self.export_btn.pack(side="left", padx=10)

        self.close_btn = ctk.CTkButton(self.frame, text="✖", width=30, height=28, fg_color="transparent", hover_color="#C62828", command=self.on_close_callback)
        self.close_btn.pack(side="right", padx=(0, 5))

        # Enable dragging window
        self._offset_x = 0
        self._offset_y = 0
        self.frame.bind("<Button-1>", self._on_click)
        self.frame.bind("<B1-Motion>", self._on_drag)
        self.status_lbl.bind("<Button-1>", self._on_click)
        self.status_lbl.bind("<B1-Motion>", self._on_drag)

    def _on_click(self, event):
        self._offset_x = event.x
        self._offset_y = event.y

    def _on_drag(self, event):
        x = self.winfo_pointerx() - self._offset_x
        y = self.winfo_pointery() - self._offset_y
        self.geometry(f"+{x}+{y}")

    def update_status(self, recording: bool, segment_count: int):
        if recording:
            self.rec_lbl.configure(text="🔴", text_color="red")
        else:
            self.rec_lbl.configure(text="⚫", text_color="gray")

        self.status_lbl.configure(text=f"{segment_count} Segments")
