import customtkinter as ctk

class ToastNotification(ctk.CTkToplevel):
    def __init__(self, parent, title="Notification", message="", duration=3000, color="#2b5e8f"):
        super().__init__(parent)
        self.overrideredirect(True)
        self.attributes('-topmost', True)

        # Sleeker dark theme look
        self.frame = ctk.CTkFrame(self, fg_color="#1F222A", corner_radius=8, border_width=2, border_color=color)
        self.frame.pack(fill="both", expand=True, padx=2, pady=2)

        title_lbl = ctk.CTkLabel(self.frame, text=title, font=ctk.CTkFont(family="Inter", weight="bold", size=14), text_color="#FFFFFF")
        title_lbl.pack(anchor="w", padx=15, pady=(15, 2))

        msg_lbl = ctk.CTkLabel(self.frame, text=message, font=ctk.CTkFont(family="Inter", size=12), text_color="#8E929C", justify="left", wraplength=270)
        msg_lbl.pack(anchor="w", padx=15, pady=(0, 15))

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

        # Semi transparent background
        self.attributes("-alpha", 0.95)

        # Auto-destroy
        if duration > 0:
            self.after(duration, self.destroy)

        # Allow click to dismiss
        self.frame.bind("<Button-1>", lambda e: self.destroy())
        title_lbl.bind("<Button-1>", lambda e: self.destroy())
        msg_lbl.bind("<Button-1>", lambda e: self.destroy())
