"""Check the film's actual text bounds throughout each camera move.

After generating the thresholds dumps, run:
    blender -b --factory-startup --python-exit-code 1 -P studio/tests/blender_thresholds_layout.py
This uses the real font, camera, cards, recorded frames and dynamic readouts.
"""
import itertools
import json
import pathlib
import sys
import unittest

STUDIO = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(STUDIO))

import bpy
from bpy_extras.object_utils import world_to_camera_view
from mathutils import Vector

import dump
import episode
from blender import overlays, scene


def projected_bounds(obj, current_scene):
    points = [world_to_camera_view(current_scene, current_scene.camera,
                                  obj.matrix_world @ Vector(corner))
              for corner in obj.bound_box]
    return (min(p.x for p in points), min(p.y for p in points),
            max(p.x for p in points), max(p.y for p in points))


def intersects(a, b):
    # Less than one output pixel is harmless rounding at the boundary.
    return (min(a[2], b[2]) - max(a[0], b[0]) > 1 / 1920 and
            min(a[3], b[3]) - max(a[1], b[1]) > 1 / 1080)


class ThresholdsLayoutTests(unittest.TestCase):
    def test_captions_including_end_credit_stay_in_frame(self):
        for beat in episode.load_episode('thresholds'):
            bpy.ops.wm.read_factory_settings(use_empty=False)
            overlays.caption_scene(beat.caption, beat.caption_y, True, title=beat.title)
            bpy.context.view_layer.update()
            # The shadow intentionally overlaps the foreground words.
            obj = next(obj for obj in bpy.context.scene.objects
                       if obj.type == 'FONT' and obj.name != 'caption-shadow')
            b = projected_bounds(obj, bpy.context.scene)
            with self.subTest(beat=beat.name):
                self.assertGreaterEqual(min(b[:2]), 0)
                self.assertLessEqual(max(b[2:]), 1)

    def test_text_stays_visible_through_every_filmed_frame(self):
        measured = json.loads((STUDIO / 'episodes/thresholds/measurements.json').read_text())
        dumps = {}
        def load(shot):
            if shot not in dumps:
                dumps[shot] = dump.load(STUDIO / 'out/thresholds/dumps' / f'{shot}.frames.json')
            return dumps[shot]

        failures = []
        for beat in episode.load_episode('thresholds'):
            if not beat.shot:
                continue
            bpy.ops.wm.read_factory_settings(use_empty=False)
            scene.install(scene.build_beat(beat, load(beat.shot), True,
                                          load(beat.compare) if beat.compare else None, measured))
            current = bpy.context.scene
            texts = [obj for obj in current.objects if obj.type == 'FONT']
            cards = [obj for obj in current.objects
                     if obj.type == 'MESH' and 'card' in obj.name and obj.parent]
            problems = set()
            for frame in range(1, beat.frames + 1):
                current.frame_set(frame)
                bpy.context.view_layer.update()
                self.assertFalse(scene.ERRORS, scene.ERRORS)
                bounds = {obj.name: projected_bounds(obj, current) for obj in texts + cards}
                for obj in texts:
                    if obj.hide_render:
                        continue
                    b = bounds[obj.name]
                    if min(b[:2]) < 0 or max(b[2:]) > 1:
                        problems.add(f'{obj.name} clipped')
                    for panel in cards:
                        # Words belong inside their own card. Other cards must not cover them.
                        if obj.parent != panel.parent and intersects(b, bounds[panel.name]):
                            problems.add(f'{obj.name} covered by {panel.name}')
                for a, b in itertools.combinations(texts, 2):
                    if not (a.hide_render or b.hide_render) and intersects(bounds[a.name], bounds[b.name]):
                        problems.add(f'{a.name} overlaps {b.name}')
            print(f'layout: {beat.name}: {beat.frames} frames, {len(problems)} problems', flush=True)
            failures.extend(f'{beat.name}: {problem}' for problem in sorted(problems))
        self.assertEqual(failures, [], '\n'.join(failures))


if __name__ == '__main__':
    unittest.main(argv=[sys.argv[0]])
