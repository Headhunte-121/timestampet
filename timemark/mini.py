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
        w = 600
        h = 60
        x = (screen_width // 2) - (w // 2)
        y = 20
        self.geometry(f"{w}x{h}+{x}+{y}")

        self.frame = ctk.CTkFrame(self, fg_color="#1e1e1e", corner_radius=10, border_width=2, border_color="#3a7ebf")
        self.frame.pack(fill="both", expand=True, padx=2, pady=2)

        # Left Side (Rec and Export)
        self.ctrl_frame = ctk.CTkFrame(self.frame, fg_color="transparent")
        self.ctrl_frame.pack(side="left", fill="y", padx=5)

        self.rec_lbl = ctk.CTkLabel(self.ctrl_frame, text="⚫", text_color="gray", font=ctk.CTkFont(size=18))
        self.rec_lbl.pack(side="left", padx=5)

        self.export_btn = ctk.CTkButton(self.ctrl_frame, text="🎬 Export", width=100, height=28, fg_color="#FF8800", hover_color="#E67A00", command=self.on_export_callback)
        self.export_btn.pack(side="left", padx=5)

        # Middle (Pills Scrollable Frame)
        self.pills_frame = ctk.CTkScrollableFrame(self.frame, orientation="horizontal", height=30, fg_color="transparent")
        self.pills_frame.pack(side="left", fill="both", expand=True, padx=5, pady=5)

        # Right Side (Close)
        self.close_btn = ctk.CTkButton(self.frame, text="✖", width=30, height=28, fg_color="transparent", hover_color="#C62828", command=self.on_close_callback)
        self.close_btn.pack(side="right", padx=(0, 5))

        # Enable dragging window
        self._offset_x = 0
        self._offset_y = 0
        self.frame.bind("<Button-1>", self._on_click)
        self.frame.bind("<B1-Motion>", self._on_drag)
        self.ctrl_frame.bind("<Button-1>", self._on_click)
        self.ctrl_frame.bind("<B1-Motion>", self._on_drag)
        self.rec_lbl.bind("<Button-1>", self._on_click)
        self.rec_lbl.bind("<B1-Motion>", self._on_drag)

    def _on_click(self, event):
        self._offset_x = event.x
        self._offset_y = event.y

    def _on_drag(self, event):
        x = self.winfo_pointerx() - self._offset_x
        y = self.winfo_pointery() - self._offset_y
        self.geometry(f"+{x}+{y}")

    def update_status(self, recording: bool, segments: list):
        if recording:
            self.rec_lbl.configure(text="🔴", text_color="red")
        else:
            self.rec_lbl.configure(text="⚫", text_color="gray")

        # Draw Pills
        for widget in self.pills_frame.winfo_children():
            widget.destroy()

        for i, seg in enumerate(segments):
            duration = max(0, seg[1] - seg[0])
            pill = ctk.CTkFrame(self.pills_frame, fg_color="#3a7ebf", corner_radius=10, height=25)
            pill.pack(side="left", padx=2, pady=0)

            btn_left = ctk.CTkButton(pill, text="<", width=15, height=20, fg_color="transparent",
                                     command=lambda idx=i: self.parent._nudge_segment(idx, -0.5))
            btn_left.pack(side="left")

            lbl = ctk.CTkLabel(pill, text=f"{duration:.1f}s", font=ctk.CTkFont(size=10, weight="bold"))
            lbl.pack(side="left", padx=2)

            btn_right = ctk.CTkButton(pill, text=">", width=15, height=20, fg_color="transparent",
                                      command=lambda idx=i: self.parent._nudge_segment(idx, 0.5))
            btn_right.pack(side="left")

            btn_del = ctk.CTkButton(pill, text="✖", width=15, height=20, fg_color="transparent", hover_color="#C62828",
                                    command=lambda idx=i: self.parent._delete_segment(idx))
            btn_del.pack(side="left")
