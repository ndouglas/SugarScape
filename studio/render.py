"""Renders one beat inside Blender:

    blender -b --factory-startup -P studio/render.py -- EPISODE INDEX [--preview] [--still FRAME | --caption]

INDEX is 1-based. Frames go to studio/out/EPISODE/beats/NN/ (preview/NN/ with
--preview) beside the beat's .blend; --still renders one frame to still.png;
--caption renders the beat's caption alone, on transparency, to caption.png.
"""

import argparse
import pathlib
import sys

STUDIO = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(STUDIO))

import bpy  # noqa: E402

import dump  # noqa: E402
import episode  # noqa: E402
from blender import overlays, scene  # noqa: E402


def main(argv):
    p = argparse.ArgumentParser()
    p.add_argument("episode")
    p.add_argument("index", type=int)
    p.add_argument("--preview", action="store_true")
    p.add_argument("--still", type=int)
    p.add_argument("--caption", action="store_true", help="render only the caption, to caption.png")
    args = p.parse_args(argv)
    beat = episode.load_episode(args.episode)[args.index - 1]
    out = STUDIO / "out" / args.episode
    folder = out / ("preview" if args.preview else "beats") / f"{args.index:02d}"
    folder.mkdir(parents=True, exist_ok=True)
    s = bpy.context.scene
    if args.caption:
        overlays.caption_scene(beat.caption, beat.caption_y, args.preview)
        s.render.filepath = str(folder / "caption.png")
        bpy.ops.render.render(write_still=True)
        return
    d = dump.load(out / "dumps" / f"{beat.shot}.frames.json") if beat.shot else None
    scene.install(scene.build_beat(beat, d, args.preview))
    bpy.ops.wm.save_as_mainfile(filepath=str(folder / "beat.blend"))
    if args.still is not None:
        s.frame_set(args.still)
        s.render.filepath = str(folder / "still.png")
        bpy.ops.render.render(write_still=True)
    else:
        s.render.filepath = str(folder / "####")
        bpy.ops.render.render(animation=True)


main(sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else [])
