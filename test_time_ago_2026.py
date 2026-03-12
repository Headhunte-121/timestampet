from datetime import datetime, timezone

def _get_time_ago(timestamp_str: str) -> str:
    if not timestamp_str:
        return ""
    try:
        dt = datetime.strptime(timestamp_str, '%Y-%m-%d %H:%M:%S').replace(tzinfo=timezone.utc)
        now = datetime.now(timezone.utc)
        diff = now - dt

        if diff.days > 365:
            years = diff.days // 365
            return f"{years} year{'s' if years > 1 else ''} ago"
        if diff.days > 30:
            months = diff.days // 30
            return f"{months} month{'s' if months > 1 else ''} ago"
        if diff.days > 0:
            return f"{diff.days} day{'s' if diff.days > 1 else ''} ago"

        hours = diff.seconds // 3600
        if hours > 0:
            return f"{hours} hour{'s' if hours > 1 else ''} ago"

        minutes = diff.seconds // 60
        if minutes > 0:
            return f"{minutes} minute{'s' if minutes > 1 else ''} ago"

        return "Just now"
    except Exception:
        return ""

print("Current System UTC:", datetime.now(timezone.utc))
print("Tested with 2024-05-15 12:00:00 ->", _get_time_ago("2024-05-15 12:00:00"))
