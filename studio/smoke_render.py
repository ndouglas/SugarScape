"""Renders smoke-test stills inside Blender (see smoke.py):

    blender -b --factory-startup -P studio/smoke_render.py -- EPISODE INDEX ...

For each beat (1-based), its middle frame and its caption, small and with one
sample, to studio/out/EPISODE/smoke/NN.png and NN-caption.png. Blender starts
afresh for each, as render.py does in its own process. Exits 1 if any fails,
after trying them all.
"""

import json
import pathlib
import sys
import traceback

STUDIO = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(STUDIO))

import bpy  # noqa: E402

import dump  # noqa: E402
import episode  # noqa: E402
from blender import overlays, scene  # noqa: E402

SMALL = 20  # percent of preview size


def fresh():
    bpy.ops.wm.read_factory_settings(use_empty=False)
    bpy.app.handlers.frame_change_pre.clear()


def shrink(s):
    s.render.resolution_percentage = SMALL
    s.eevee.taa_render_samples = 1
    s.render.use_motion_blur = False


def still(name, index, beat, dumps, measured, folder):
    fresh()
    load = lambda shot: dumps.setdefault(shot, dump.load(STUDIO / "out" / name / "dumps" / f"{shot}.frames.json"))  # noqa: E731
    d = load(beat.shot) if beat.shot else None
    compare = load(beat.compare) if beat.compare else None
    s = bpy.context.scene
    scene.install(scene.build_beat(beat, d, True, compare, measured))
    shrink(s)
    s.frame_set(max(beat.frames // 2, 1))
    if scene.ERRORS:
        raise RuntimeError(scene.ERRORS[0])
    s.render.filepath = str(folder / f"{index:02d}.png")
    bpy.ops.render.render(write_still=True)
    if scene.ERRORS:
        raise RuntimeError(scene.ERRORS[0])
    if beat.caption:
        fresh()
        overlays.caption_scene(beat.caption, beat.caption_y, True, title=beat.title)
        s = bpy.context.scene
        s.render.resolution_percentage = SMALL
        s.render.filepath = str(folder / f"{index:02d}-caption.png")
        bpy.ops.render.render(write_still=True)


def main(argv):
    name, indexes = argv[0], [int(a) for a in argv[1:]]
    beats = episode.load_episode(name)
    folder = STUDIO / "out" / name / "smoke"
    folder.mkdir(parents=True, exist_ok=True)
    measured_path = episode.episode_dir(name) / "measurements.json"
    measured = json.loads(measured_path.read_text()) if measured_path.exists() else {}
    dumps, failed = {}, []
    for i in indexes:
        beat = beats[i - 1]
        try:
            still(name, i, beat, dumps, measured, folder)
            print(f"smoke: {name} {i:02d} {beat.name} ok", flush=True)
        except Exception:
            failed.append(f"{name} {i:02d} {beat.name}")
            print(f"smoke: {name} {i:02d} {beat.name} FAILED\n{traceback.format_exc()}", flush=True)
    if failed:
        raise SystemExit(f"smoke: failed: {', '.join(failed)}")


main(sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else [])
