with open('timemark/main.py', 'r') as f:
    content = f.read()

# Add the status tag to the hover overlay next to or above the title
old_block = """        # Content for info_overlay
        title_text = item['title'] or 'Unknown Title'
        title_lbl = ctk.CTkLabel(info_overlay, text=title_text, font=("Inter", 16, "bold"), text_color=TEXT_PRIMARY, wraplength=120)
        title_lbl.pack(pady=(15, 5), padx=10, anchor="w")"""

new_block = """        # Content for info_overlay
        title_text = item['title'] or 'Unknown Title'
        title_lbl = ctk.CTkLabel(info_overlay, text=title_text, font=("Inter", 16, "bold"), text_color=TEXT_PRIMARY, wraplength=120)
        title_lbl.pack(pady=(10, 2), padx=10, anchor="w")

        # Status Tag
        if status:
            status_color = "#1F222A"
            if status == "Returning Series" or status == "Watching":
                status_color = "#1A2E1A" # Subtle green tint
            elif status == "Ended" or status == "Completed":
                status_color = "#2A1A1A" # Subtle red tint

            status_tag = ctk.CTkLabel(info_overlay, text=status, font=("Inter", 9, "bold"), text_color=TEXT_SECONDARY, fg_color=status_color, corner_radius=4, padx=6, pady=2)
            status_tag.pack(padx=10, pady=(0, 5), anchor="w")"""

content = content.replace(old_block, new_block)

with open('timemark/main.py', 'w') as f:
    f.write(content)
