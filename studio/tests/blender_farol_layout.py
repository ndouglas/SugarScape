"""Check the film's actual text bounds throughout each camera move.

After generating the farol dumps, run:
    blender -b --factory-startup --python-exit-code 1 -P studio/tests/blender_farol_layout.py
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


class FarolLayoutTests(unittest.TestCase):
    def test_captions_including_end_credit_stay_in_frame(self):
        for beat in episode.load_episode('farol'):
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

    def test_closing_has_one_host_and_one_late_blink(self):
        beat=episode.load_episode('farol')[-1]
        bpy.ops.wm.read_factory_settings(use_empty=False)
        scene.install(scene.build_beat(beat,None,True))
        hosts=[obj for obj in bpy.context.scene.objects if obj.name=='host']
        self.assertEqual(len(hosts),1)
        self.assertEqual(tuple(hosts[0].rotation_euler),(0,0,0))
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
        measured = json.loads((STUDIO / 'episodes/farol/measurements.json').read_text())
        def load(shot):
            return dump.load(STUDIO / 'out/farol/dumps' / f'{shot}.frames.json')

        failures = []
        for beat in episode.load_episode('farol'):
            if not beat.shot:
                continue
            bpy.ops.wm.read_factory_settings(use_empty=False)
            d=load(beat.shot)
            scene.install(scene.build_beat(beat, d, True, measured=measured))
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
                from farol_visual import filmed_frame
                actual=filmed_frame(d,beat.timing(d.ticks),frame)
                agents=[obj for obj in current.objects if obj.name.startswith('flump') and obj.instance_type=='COLLECTION']
                self.assertEqual(len(agents),len(d.placed))
                for obj in agents:
                    i=int(obj.name[5:])
                    expected=actual.agents[i]
                    import animate
                    x,y=animate.cell_center(expected.x,expected.y,d.width,d.height)
                    self.assertAlmostEqual(obj.location.x,x,places=5)
                    self.assertAlmostEqual(obj.location.y,y,places=5)
                    # Conservative body box includes feet, eyes, and head.
                    for corner in ((-.4,-.4,0),(.4,.4,.85)):
                        point=world_to_camera_view(current,current.camera,obj.matrix_world @ Vector(corner))
                        self.assertTrue(0.02<point.x<.65 and .20<point.y<.89,(beat.name,frame,obj.name,tuple(point)))
                self.assertIn(f'Round {actual.period:,} ·',bpy.data.objects['actual-clock'].data.body)
                self.assertEqual(bpy.data.objects['actual-counts'].data.body,
                                 f"{actual.counts[0]} {'B' if d.config['game']=='minority' else 'home'}  /  {actual.counts[1]} {'A' if d.config['game']=='minority' else 'bar'}")
                for obj in texts:
                    if obj.hide_render:
                        continue
                    b = bounds[obj.name]
                    if b[1]<.20:
                        problems.add(f'{obj.name} intrudes on caption')
                    if min(b[:2]) < 0 or max(b[2:]) > 1:
                        problems.add(f'{obj.name} clipped')
                    for panel in cards:
                        # Words belong inside their own card. Other cards must not cover them.
                        if obj.parent != panel.parent and intersects(b, bounds[panel.name]):
                            problems.add(f'{obj.name} covered by {panel.name}')
                for a, b in itertools.combinations(texts, 2):
                    if not (a.hide_render or b.hide_render) and intersects(bounds[a.name], bounds[b.name]):
                        problems.add(f'{a.name} overlaps {b.name}')
            if beat.name not in ('bar','forecasts','decide','memory'):
                curves=[o for o in current.objects if o.name.startswith('recorded-trace-')]
                self.assertTrue(curves)
                for obj in curves:
                    self.assertGreater(len({tuple(p.co) for p in obj.data.splines[0].points}),1,obj.name)
                bars=[o for o in current.objects if o.name.startswith('ensemble-') and o.type=='MESH']
                self.assertGreaterEqual(len(bars),26)
            print(f'layout: {beat.name}: {beat.frames} frames, {len(problems)} problems', flush=True)
            failures.extend(f'{beat.name}: {problem}' for problem in sorted(problems))
            del d
            import gc
            gc.collect()
        self.assertEqual(failures, [], '\n'.join(failures))


if __name__ == '__main__':
    unittest.main(argv=[sys.argv[0]])
