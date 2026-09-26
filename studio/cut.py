"""The final cut: beats' PNG sequences, each with its caption laid over it
and faded in and out, joined with cross-dissolves by ffmpeg."""

import pathlib

CAPTION_IN, CAPTION_RAMP, CAPTION_OUT = 0.3, 0.4, 0.5


def check_folders(folders, frames, captions):
    """What would make the cut wrong: a beat folder without exactly its
    frames 0001…NNNN.png, or a captioned beat without its caption.png.
    (ffmpeg would absorb a short or long beat into the dissolves unnoticed.)"""
    problems = []
    for folder, count, caption in zip(folders, frames, captions):
        f = pathlib.Path(folder)
        found = sorted(p.name for p in f.glob("[0-9][0-9][0-9][0-9].png"))
        want = [f"{i:04d}.png" for i in range(1, count + 1)]
        if found != want:
            problems.append(f"{f.name}: {len(found)} frames, expected {count} (0001–{count:04d}.png)")
        if caption is not None and not pathlib.Path(caption).exists():
            problems.append(f"{f.name}: missing caption {caption}")
    return problems


def total_frames(frames, dissolve):
    return sum(frames) - dissolve * (len(frames) - 1)


def _seconds(frames, fps):
    return f"{frames / fps:g}"


def command(folders, frames, out, captions=None, dissolve=12, fps=30):
    """The ffmpeg argv: `folders[i]` holds beat i's frames (0001.png …),
    `frames[i]` their count, `captions[i]` a transparent PNG or None."""
    captions = captions or [None] * len(folders)
    argv = ["ffmpeg", "-y", "-loglevel", "error"]
    for folder in folders:
        argv += ["-framerate", str(fps), "-start_number", "1", "-i", f"{folder}/%04d.png"]
    parts, labels = [], [f"[{i}:v]" for i in range(len(folders))]
    extra = len(folders)
    for i, png in enumerate(captions):
        if png is None:
            continue
        length = frames[i] / fps
        argv += ["-loop", "1", "-framerate", str(fps), "-t", _seconds(frames[i], fps), "-i", png]
        fade_out = length - CAPTION_OUT
        parts.append(
            f"[{extra}:v]format=rgba,"
            f"fade=t=in:st={CAPTION_IN:g}:d={CAPTION_RAMP:g}:alpha=1,"
            f"fade=t=out:st={fade_out:g}:d={CAPTION_RAMP:g}:alpha=1[c{i}]"
        )
        parts.append(f"{labels[i]}[c{i}]overlay=shortest=1[b{i}]")
        labels[i] = f"[b{i}]"
        extra += 1
    label, elapsed = labels[0], 0
    for i in range(1, len(folders)):
        elapsed += frames[i - 1] - dissolve
        nxt = f"[v{i}]"
        parts.append(
            f"{label}{labels[i]}xfade=transition=fade:duration={dissolve / fps:g}:offset={round(elapsed / fps, 4):g}{nxt}"
        )
        label = nxt
    encode = ["-c:v", "libx264", "-preset", "slow", "-crf", "16", "-pix_fmt", "yuv420p", "-r", str(fps), "-movflags", "+faststart"]
    if not parts:
        return argv + encode + [out]
    return argv + ["-filter_complex", ";".join(parts), "-map", label] + encode + [out]
