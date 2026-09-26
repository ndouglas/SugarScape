"""A Blender smoke test: renders one small still of each kind of beat in each
episode, from the episode's real dumps, and fails if any beat or caption
cannot be built or rendered.

    python3 studio/smoke.py [EPISODE ...]

A kind of beat is its combination of shot or none (the title card), close-up
or crowd, color mode and overlays, so every overlay, every scene path and
the title card get built from real data. Each still is the beat's middle
frame at a fifth of preview size with one sample; stills go to
studio/out/EPISODE/smoke/. Missing dumps are made first, as the build makes
them. Takes a few minutes; the unit tests don't run it.
"""

import argparse
import sys

import build
import episode


def kind(beat):
    return (beat.shot is None, beat.closeup, beat.params.get("colors"), tuple(sorted(beat.overlays)), beat.title)


def representatives(beats):
    """1-based indexes of the first beat of each kind."""
    seen, chosen = set(), []
    for i, b in enumerate(beats, start=1):
        if kind(b) not in seen:
            seen.add(kind(b))
            chosen.append(i)
    return chosen


def ensure_dumps(name, beats, chosen):
    out = build.STUDIO / "out" / name / "dumps"
    out.mkdir(parents=True, exist_ok=True)
    wanted = {s for i in chosen for s in (beats[i - 1].shot, beats[i - 1].compare) if s}
    missing = sorted(s for s in wanted if not (out / f"{s}.frames.json").exists())
    if missing:
        build.run(["cargo", "build", "--release", "-q", "-p", "sugarscape-cli"])
    for shot in missing:
        src = episode.episode_dir(name) / "shots" / f"{shot}.json"
        build.run([build.CLI, "shot", src, "--out", out / f"{shot}.frames.json"])


def main():
    p = argparse.ArgumentParser()
    p.add_argument("episodes", nargs="*")
    args = p.parse_args()
    names = args.episodes or sorted(
        d.name for d in (episode.ROOT / "episodes").iterdir() if (d / "beats.py").exists()
    )
    failed = []
    for name in names:
        beats = episode.load_episode(name)
        chosen = representatives(beats)
        print(f"{name}: beats {', '.join(map(str, chosen))} of {len(beats)}", flush=True)
        ensure_dumps(name, beats, chosen)
        try:
            build.run([build.BLENDER, "-b", "--factory-startup", "--python-exit-code", "1",
                       "-P", build.STUDIO / "smoke_render.py", "--", name, *chosen])
        except Exception as e:  # noqa: BLE001 — report every episode, then fail
            failed.append(f"{name}: {e}")
    if failed:
        sys.exit("smoke test failed:\n  " + "\n  ".join(failed))
    print("smoke test passed")


if __name__ == "__main__":
    main()
