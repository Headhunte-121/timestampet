import customtkinter as ctk

class ToastNotification(ctk.CTkToplevel):
    def __init__(self, parent, title="Notification", message="", duration=3000, color="#2b5e8f"):
        super().__init__(parent)
        self.overrideredirect(True)
        self.attributes('-topmost', True)

        # We need a small border or clean frame
        self.frame = ctk.CTkFrame(self, fg_color=color, corner_radius=5, border_width=2, border_color="#1e1e1e")
        self.frame.pack(fill="both", expand=True, padx=2, pady=2)

        title_lbl = ctk.CTkLabel(self.frame, text=title, font=ctk.CTkFont(weight="bold", size=14))
        title_lbl.pack(anchor="w", padx=10, pady=(10, 2))

        msg_lbl = ctk.CTkLabel(self.frame, text=message, font=ctk.CTkFont(size=12), justify="left", wraplength=250)
        msg_lbl.pack(anchor="w", padx=10, pady=(0, 10))

        # Position at bottom right of screen
        self.update_idletasks()
        width = self.winfo_width()
        height = self.winfo_height()
        screen_width = self.winfo_screenwidth()
        screen_height = self.winfo_screenheight()

        # Offsets
        x = screen_width - width - 20
        y = screen_height - height - 60 # Above taskbar

        self.geometry(f"+{x}+{y}")

        # Fade in or just show
        self.attributes("-alpha", 0.95)

        # Auto-destroy
        if duration > 0:
            self.after(duration, self.destroy)

        # Allow click to dismiss
        self.frame.bind("<Button-1>", lambda e: self.destroy())
        title_lbl.bind("<Button-1>", lambda e: self.destroy())
        msg_lbl.bind("<Button-1>", lambda e: self.destroy())
