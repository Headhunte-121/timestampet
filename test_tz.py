from datetime import datetime, timezone
import sys

dt_naive = datetime.strptime("2023-01-01 12:00:00", "%Y-%m-%d %H:%M:%S")
dt_aware = dt_naive.astimezone()
print(f"Naive: {dt_naive}, tz: {dt_naive.tzinfo}")
print(f"Aware: {dt_aware}, tz: {dt_aware.tzinfo}")

dt_utc = datetime.strptime("2023-01-01 12:00:00", "%Y-%m-%d %H:%M:%S").replace(tzinfo=timezone.utc)
dt_local = dt_utc.astimezone()

try:
    print(dt_naive < dt_local)
except TypeError as e:
    print(f"Error naive < local: {e}")

try:
    print(dt_aware < dt_local)
    print("Aware comparison works")
except TypeError as e:
    print(f"Error aware < local: {e}")
