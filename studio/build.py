"""Builds an episode: shots → frame dumps → beat renders and captions → the cut.

    python3 studio/build.py EPISODE [--preview] [--beat N] [--skip-shots] [--skip-render] [--no-music]

--beat N renders one beat (1-based) and stops before the cut. The cut checks
that the video has exactly the frames the beats add up to.
"""

import argparse
import os
import pathlib
import subprocess
import sys

STUDIO = pathlib.Path(__file__).resolve().parent
REPO = STUDIO.parent
sys.path.insert(0, str(STUDIO))

import cut  # noqa: E402
import episode  # noqa: E402
import music  # noqa: E402

BLENDER = os.environ.get("BLENDER", "/Applications/Blender.app/Contents/MacOS/Blender")
SOUNDFONT = STUDIO / "out" / "soundfonts" / "FluidR3_GM.sf2"
CLI = REPO / "target" / "release" / "sugarscape"
DISSOLVE = 12


def run(argv, quiet=False):
    print("$", " ".join(str(a) for a in argv), flush=True)
    out = subprocess.DEVNULL if quiet else None
    subprocess.run([str(a) for a in argv], check=True, stdout=out)


def blender(*args):
    # Without --python-exit-code, Blender exits 0 when the script raises.
    run([BLENDER, "-b", "--factory-startup", "--python-exit-code", "1", "-P", STUDIO / "render.py", "--", *args], quiet=True)


def soundtrack(name, out, seconds):
    """The episode's own recording (music.wav/.aif/.aiff/.m4a in its folder,
    e.g. a GarageBand export) if there is one, else the generated draft of
    the tune fitted to `seconds`, else None (with a warning)."""
    for ext in ("wav", "aif", "aiff", "m4a"):
        own = episode.episode_dir(name) / f"music.{ext}"
        if own.exists():
            print(f"music: {own}")
            return str(own)
    if not SOUNDFONT.exists():
        print(f"music: none (no {SOUNDFONT}; see studio/README.md)")
        return None
    wav = music.render(seconds, out / "music", SOUNDFONT)
    print(f"music: generated {wav} (MIDI beside it)")
    return str(wav)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("episode")
    p.add_argument("--preview", action="store_true")
    p.add_argument("--beat", type=int)
    p.add_argument("--skip-shots", action="store_true")
    p.add_argument("--skip-render", action="store_true")
    p.add_argument("--no-music", action="store_true")
    args = p.parse_args()
    beats = episode.load_episode(args.episode)
    out = STUDIO / "out" / args.episode
    (out / "dumps").mkdir(parents=True, exist_ok=True)
    chosen = [args.beat] if args.beat else list(range(1, len(beats) + 1))
    if not args.skip_shots:
        run(["cargo", "build", "--release", "-q", "-p", "sugarscape-cli"])
        for shot in sorted({beats[i - 1].shot for i in chosen if beats[i - 1].shot}):
            src = episode.episode_dir(args.episode) / "shots" / f"{shot}.json"
            run([CLI, "shot", src, "--out", out / "dumps" / f"{shot}.frames.json"])
    kind = "preview" if args.preview else "beats"
    extra = ["--preview"] if args.preview else []
    if not args.skip_render:
        for i in chosen:
            # A beat's old frames would outlast a shorter re-render and join the cut.
            for old in (out / kind / f"{i:02d}").glob("[0-9][0-9][0-9][0-9].png"):
                old.unlink()
            blender(args.episode, i, *extra)
            if beats[i - 1].caption:
                blender(args.episode, i, "--caption", *extra)
    if args.beat:
        return
    folders = [out / kind / f"{i:02d}" for i in range(1, len(beats) + 1)]
    captions = [str(f / "caption.png") if b.caption else None for f, b in zip(folders, beats)]
    frames = [b.frames for b in beats]
    problems = cut.check_folders([str(f) for f in folders], frames, captions)
    if problems:
        sys.exit("cannot cut:\n  " + "\n  ".join(problems))
    movie = out / f"{args.episode}{'-preview' if args.preview else ''}.mp4"
    track = None if args.no_music else soundtrack(args.episode, out, cut.total_frames(frames, DISSOLVE) / 30)
    run(cut.command([str(f) for f in folders], frames, str(movie), captions, DISSOLVE, music=track))
    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-count_frames", "-select_streams", "v:0",
         "-show_entries", "stream=nb_read_frames", "-of", "csv=p=0", str(movie)],
        check=True, capture_output=True, text=True,
    )
    got, want = int(probe.stdout.strip()), cut.total_frames(frames, DISSOLVE)
    if got != want:
        sys.exit(f"{movie}: {got} frames, expected {want}")
    print(f"{movie}: {got} frames ({got / 30:.1f} s)")


if __name__ == "__main__":
    main()
