"""Real Blender fixture and large elderly/young face proofs."""
import pathlib
import sys
import unittest
STUDIO = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(STUDIO))
import bpy
from mathutils import Vector
import dump
from blender import materials
from blender import retirement_cast as cast

OUT = STUDIO.parent / '.superpowers/sdd/2026-10-03-retirement-dance/scratch/rig'

class CastTests(unittest.TestCase):
    def test_working_feet_stay_fixed_in_world_space(self):
        member = dict(id=7595, born=-75, age=98, retired=False, kind='imitator')
        rig = cast.build_character('stationary-worker', member, hero=True)
        rig.origin = (2, -1, .2)
        rig.root.rotation_euler.z = .6
        rig.root.scale = (1.3, 1.3, 1.3)
        samples = []
        for seconds in (0, .3, 1, 3, 8):
            cast.pose_character(rig, member, seconds)
            bpy.context.view_layer.update()
            samples.append(tuple(tuple(foot.matrix_world.translation) for foot in rig.feet))
        for sample in samples[1:]:
            for start, current in zip(samples[0], sample):
                self.assertLess((Vector(start)-Vector(current)).length, 1e-7)

    def test_join_color_eases_without_changing_source_or_shared_yarn(self):
        member = dict(id=7595, born=-75, age=98, retired=True, kind='imitator')
        rig = cast.build_character('join-color', member)
        friend = cast.build_character('friend-color', member)
        shared = friend.body.data.materials[0]
        initial = tuple(shared.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value)
        def sample(amount):
            cast.pose_character(rig, member, 25.4, dance_amount=amount)
            return tuple(rig.body.data.materials[0].node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value)
        worker = sample(0)
        self.assertTrue(rig.root['retired'])
        self.assertEqual(worker, tuple(materials.knit('teal').node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value))
        self.assertEqual(sample(1), initial)
        middle = sample(.5)
        sample(1);sample(0)
        self.assertEqual(sample(.5), middle)
        self.assertEqual(tuple(shared.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value), initial)
        self.assertIs(friend.body.data.materials[0], shared)

    def test_invitation_pose_resets_gaze_and_is_random_access(self):
        member = dict(id=7595, born=-75, age=98, retired=True, kind='imitator')
        rig = cast.build_character('invitation-worker', member, hero=True)
        cast.pose_character(rig, member, 25.4, dance_amount=0, gaze=.2)
        self.assertAlmostEqual(rig.upper.rotation_euler.z, .2)
        cast.pose_character(rig, member, 26.4, dance_amount=.5)
        bpy.context.view_layer.update()
        expected = tuple(tuple(foot.matrix_world.translation) for foot in rig.feet)
        self.assertEqual(rig.upper.rotation_euler.z, 0)
        cast.pose_character(rig, member, 29, dance_amount=1, gaze=.1)
        cast.pose_character(rig, member, 26.4, dance_amount=.5)
        bpy.context.view_layer.update()
        self.assertEqual(expected, tuple(tuple(foot.matrix_world.translation) for foot in rig.feet))

    def test_actual_identity_accessories_blink_and_renewal(self):
        for obj in list(bpy.data.objects):
            bpy.data.objects.remove(obj, do_unlink=True)
        d = dump.load(STUDIO / 'out/retirement/dumps/teaching.frames.json')
        old = d.frames[3].members[7595]
        rig = cast.build_character('proof', old, hero=True)
        self.assertEqual((rig.root['slot'], rig.root['born']), (7595, -75))
        self.assertTrue(rig.age_parts)
        self.assertTrue(rig.spectacles)
        ring = rig.spectacles[0]
        points = [p.co for p in ring.data.splines[0].points]
        self.assertAlmostEqual(max(p.x for p in points)-min(p.x for p in points),
                               max(p.z for p in points)-min(p.z for p in points), places=5)
        self.assertTrue(all(p.parent is rig.upper for p in rig.spectacles))
        self.assertTrue(all(p.parent is not rig.eyes for p in rig.age_parts))
        rig.eyes.scale.z = .08
        cast.pose_character(rig, old, .4, dancing=False)
        self.assertAlmostEqual(rig.eyes.scale.z, .08)
        self.assertEqual(tuple(rig.feet[0].location), rig.foot_origins[0])
        cast.pose_character(rig, old, .4, dancing=True)
        self.assertGreater(rig.feet[0].location.z, rig.foot_origins[0][2])
        current = bpy.context.scene
        bpy.ops.mesh.primitive_plane_add(size=200)
        bpy.context.object.data.materials.append(materials.felt())
        materials.lights_and_world(current, 4)
        camera_data = bpy.data.cameras.new('proof-camera')
        camera = bpy.data.objects.new('proof-camera', camera_data)
        current.collection.objects.link(camera)
        current.camera = camera
        camera.location = (0, -2.7, 1.05)
        camera.rotation_euler = (Vector((0, 0, .37))-camera.location).to_track_quat('-Z', 'Y').to_euler()
        camera_data.lens = 70
        current.render.engine = 'BLENDER_EEVEE'
        current.eevee.taa_render_samples = 32
        current.render.resolution_x = current.render.resolution_y = 900
        current.render.resolution_percentage = 100
        current.render.image_settings.file_format = 'PNG'
        for label, member in [('elderly', old), ('young', d.frames[1].members[4047])]:
            cast.pose_character(rig, member, 0, dancing=False)
            rig.eyes.scale.z = 1
            current.render.filepath = str(OUT / (label + '.png'))
            bpy.ops.render.render(write_still=True)
        self.assertFalse(any(p.hide_render is False for p in rig.age_parts))
        self.assertEqual(rig.root['born'], d.frames[1].members[4047]['born'])

if __name__ == '__main__':
    unittest.main(argv=[sys.argv[0]])
