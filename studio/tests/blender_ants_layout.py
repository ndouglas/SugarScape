"""Check the film's actual text bounds throughout each camera move.

After generating the ants dumps, run:
    blender -b --factory-startup --python-exit-code 1 -P studio/tests/blender_ants_layout.py
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


class AntsLayoutTests(unittest.TestCase):
    def test_captions_including_end_credit_stay_in_frame(self):
        for beat in episode.load_episode('ants'):
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

    def test_traces_theory_and_actual_neighbor_edges_have_geometry(self):
        measured=json.loads((STUDIO/'episodes/ants/measurements.json').read_text())
        for name,prefix in (('piles','selected-film'),('splits','exact-stationary'),('neighbors','actual-edge-1-')):
            beat=next(b for b in episode.load_episode('ants') if b.name==name)
            d=dump.load(STUDIO/'out/ants/dumps'/f'{beat.shot}.frames.json')
            bpy.ops.wm.read_factory_settings(use_empty=False)
            scene.install(scene.build_beat(beat,d,True,measured=measured))
            curves=[o for o in bpy.context.scene.objects if o.name.startswith(prefix)]
            self.assertEqual(len(curves),10 if name=='neighbors' else 1)
            for obj in curves:
                coords={tuple(point.co) for point in obj.data.splines[0].points}
                self.assertGreater(len(coords),1,obj.name)

    def test_closing_has_one_host_and_one_late_blink(self):
        beat=episode.load_episode('ants')[-1]
        bpy.ops.wm.read_factory_settings(use_empty=False)
        scene.install(scene.build_beat(beat,None,True))
        hosts=[obj for obj in bpy.context.scene.objects if obj.name=='host']
        self.assertEqual(len(hosts),1)
        import animate
        closed=[]
        for frame in range(1,beat.frames+1):
            bpy.context.scene.frame_set(frame)
            self.assertFalse(scene.ERRORS,scene.ERRORS)
            actual=bpy.data.objects['host.eyes'].scale.z
            self.assertAlmostEqual(actual,animate.closing_blink(frame,beat.frames))
            if actual<1: closed.append(frame)
        self.assertEqual(closed,list(range(min(closed),max(closed)+1)))
        self.assertGreater(min(closed),.7*beat.frames)

    def test_text_stays_visible_through_every_filmed_frame(self):
        measured = json.loads((STUDIO / 'episodes/ants/measurements.json').read_text())
        dumps = {}
        def load(shot):
            if shot not in dumps:
                dumps[shot] = dump.load(STUDIO / 'out/ants/dumps' / f'{shot}.frames.json')
            return dumps[shot]

        failures = []
        for beat in episode.load_episode('ants'):
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
                if beat.name in ('meet','self'):
                    event=measured['selected'][beat.shot]['event']
                    ids=[event['agent']]+([event['partner']] if event['partner'] else [])
                    for actor in ids:
                        marker=bpy.data.objects[f'actual-member-{actor}']
                        marker_bounds=projected_bounds(marker,current)
                        self.assertGreater(marker_bounds[2]-marker_bounds[0],4/960)
                        self.assertGreater(marker_bounds[3]-marker_bounds[1],2/540)
                        self.assertGreaterEqual(min(marker_bounds[:2]),0)
                        self.assertLessEqual(max(marker_bounds[2:]),1)
                    self.assertEqual('recorded-recruitment-pair' in bpy.data.objects,bool(event['partner']))
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
