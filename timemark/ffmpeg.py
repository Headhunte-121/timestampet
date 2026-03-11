import os
import subprocess
import tempfile
from pathlib import Path
from typing import List, Tuple

def get_ffmpeg_path() -> str:
    """Attempts to auto-detect ffmpeg path."""
    try:
        res = subprocess.run(["ffmpeg", "-version"], capture_output=True, text=True)
        if res.returncode == 0:
            return "ffmpeg"
    except Exception:
        pass

    if os.name == 'nt':
        # Check common Windows paths or bundled path if we had one
        paths = [
            r"C:\ffmpeg\bin\ffmpeg.exe",
            r"C:\Program Files\ffmpeg\bin\ffmpeg.exe"
        ]
        for p in paths:
            if Path(p).exists():
                return p
    return ""

def export_seamless_scene(video_file: str, segments: List[Tuple[int, int]], output_file: str) -> bool:
    """
    Uses FFmpeg to extract multiple segments from a video and concatenate them
    losslessly using the stream copy codec (-c copy).
    """
    ffmpeg_path = get_ffmpeg_path()
    if not ffmpeg_path:
        print("Error: FFmpeg not found on system path.")
        return False

    if not os.path.exists(video_file):
        print(f"Error: Source video file not found: {video_file}")
        return False

    if not segments:
        return False

    temp_dir = tempfile.mkdtemp(prefix="timemark_ffmpeg_")
    segment_files = []

    try:
        # Step 1: Extract each segment losslessly
        for idx, (start, end) in enumerate(segments):
            duration = end - start
            if duration <= 0:
                continue

            seg_out = os.path.join(temp_dir, f"seg_{idx:03d}.mp4")

            # Fast seek (-ss before -i) for extraction, stream copy
            cmd = [
                ffmpeg_path,
                "-y", # Overwrite
                "-ss", str(start),
                "-i", video_file,
                "-t", str(duration),
                "-c", "copy",
                "-avoid_negative_ts", "make_zero",
                seg_out
            ]

            res = subprocess.run(cmd, capture_output=True)
            if res.returncode != 0:
                print(f"FFmpeg extract error for segment {idx}: {res.stderr.decode('utf-8', errors='ignore')}")
                continue

            segment_files.append(seg_out)

        if not segment_files:
            return False

        # Step 2: Create concat demuxer file
        concat_file = os.path.join(temp_dir, "concat.txt")
        with open(concat_file, 'w', encoding='utf-8') as f:
            for sf in segment_files:
                # FFmpeg concat demuxer requires forward slashes and single quotes
                safe_path = sf.replace('\\', '/')
                f.write(f"file '{safe_path}'\n")

        # Step 3: Concatenate segments losslessly
        concat_cmd = [
            ffmpeg_path,
            "-y",
            "-f", "concat",
            "-safe", "0",
            "-i", concat_file,
            "-c", "copy",
            output_file
        ]

        res = subprocess.run(concat_cmd, capture_output=True)
        if res.returncode != 0:
            print(f"FFmpeg concat error: {res.stderr.decode('utf-8', errors='ignore')}")
            return False

        return True

    finally:
        # Cleanup temporary files
        for f in segment_files:
            if os.path.exists(f):
                try: os.remove(f)
                except: pass

        concat_file = os.path.join(temp_dir, "concat.txt")
        if os.path.exists(concat_file):
            try: os.remove(concat_file)
            except: pass

        try: os.rmdir(temp_dir)
        except: pass
