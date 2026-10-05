"""Real-font/stage bounds, exact recorded source identities and quiet closing."""
import json
import pathlib
import sys
import unittest
STUDIO=pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0,str(STUDIO))
import bpy
from bpy_extras.object_utils import world_to_camera_view
from mathutils import Vector
import dump
import episode
from retirement_visual import filmed_frame, teaching_members, music_seconds, CastTimeline, nearby_ids
from retirement_dance import dance_pose, join_pose, join_amount
import math
from blender import overlays,scene


def bounds(obj,current):
    points=[world_to_camera_view(current,current.camera,obj.matrix_world@Vector(c))for c in obj.bound_box]
    return min(p.x for p in points),min(p.y for p in points),max(p.x for p in points),max(p.y for p in points)

class RetirementLayoutTests(unittest.TestCase):
    def test_decision_boundary_and_random_access_motion(self):
        measured=json.loads((STUDIO/'episodes/retirement/measurements.json').read_text())
        beat=next(b for b in episode.load_episode('retirement') if b.name=='decision')
        d=dump.load(STUDIO/'out/retirement/dumps/teaching.frames.json')
        scene.install(scene.build_beat(beat,d,True,measured=measured))
        current=bpy.context.scene
        root=next(o for o in current.objects if o.get('slot')==7595)
        feet=sorted((o for o in root.children_recursive if '.foot' in o.name),key=lambda o:o.name)
        upper=next(o for o in root.children_recursive if o.name.endswith('.upper'))
        boundary=math.floor(beat.frames*.55)+1
        def sample(frame):
            current.frame_set(frame);bpy.context.view_layer.update()
            return tuple(root.location), tuple(upper.rotation_euler), tuple(tuple(f.matrix_world.translation) for f in feet)
        before=sample(boundary-1)
        self.assertFalse(root['retired'])
        after=sample(boundary)
        self.assertTrue(root['retired'])
        self.assertLess((Vector(before[0])-Vector(after[0])).length,1e-7)
        for a,b in zip(before[2],after[2]):self.assertLess((Vector(a)-Vector(b)).length,1e-7)
        expected=sample(boundary+30)
        sample(beat.frames);sample(1)
        self.assertEqual(sample(boundary+30),expected)
        self.assertFalse(scene.ERRORS,scene.ERRORS)

    def test_all_beats_actual_cast_background_and_caption_clearance(self):
        measured=json.loads((STUDIO/'episodes/retirement/measurements.json').read_text())
        cache={}
        def load(key):
            if key not in cache:cache[key]=dump.load(STUDIO/'out/retirement/dumps'/f'{key}.frames.json')
            return cache[key]
        for beat in episode.load_episode('retirement'):
            d=load(beat.shot)
            scene.install(scene.build_beat(beat,d,True,compare=load(beat.compare)if beat.compare else None,measured=measured))
            current=bpy.context.scene
            history={}
            timelines={}
            for frame in(1,beat.frames//2,beat.frames):
                current.frame_set(frame);bpy.context.view_layer.update()
                self.assertFalse(scene.ERRORS,scene.ERRORS)
                self.assertFalse(any('card' in o.name for o in current.objects))
                for obj in [o for o in current.objects if o.type=='FONT' and not o.hide_render]:
                    b=bounds(obj,current)
                    self.assertGreaterEqual(obj.data.size,.038-1e-6,(beat.name,obj.name))
                    self.assertGreaterEqual(min(b[:2]),0,(beat.name,obj.name,b))
                    self.assertLessEqual(max(b[2:]),1,(beat.name,obj.name,b))
                    self.assertGreaterEqual(b[1],.20,(beat.name,obj.name,b))
                if beat.name=='contact':
                    footer=bpy.data.objects['retirement-method-footer'].data.body
                    self.assertIn('All groups / contacts: 50/50 attained',footer)
                    self.assertIn('0 censored · horizon 600',footer)
                if beat.name=='renewal' and frame==1:
                    self.assertIn('Age 60 · born -40',bpy.data.objects['retirement-main-measure'].data.body)
                if beat.name=='end':continue
                meshes=[o for o in current.objects if o.name.startswith('retirement-population-mesh')]
                self.assertEqual(len(meshes),2 if beat.name in ('denominator','contact') else 1)
                for mesh in meshes:
                    source=load(mesh['run']);f=filmed_frame(source,beat.timing(source.ticks),frame)
                    self.assertEqual(mesh['agents'],8100)
                    self.assertEqual(len(mesh.data.polygons),8100)
                    self.assertEqual(list(mesh['source_slots']),sorted(f.members))
                    focus=list(mesh['focus_slots'])
                    roots=[o for o in current.objects if o.get('run')==mesh['run'] and 'slot' in o]
                    self.assertEqual(sorted(o['slot'] for o in roots),sorted(focus))
                    if beat.name not in ('ages','decision') and mesh['run'] not in timelines:
                        timing=beat.timing(source.ticks)
                        initial=nearby_ids(beat.name,source,measured,filmed_frame(source,timing,1))
                        timelines[mesh['run']]=CastTimeline(source,timing,beat.frames,initial,
                            lambda f:nearby_ids(beat.name,source,measured,f),
                            eligible=beat.name not in ('habits','renewal'),
                            groups=beat.name in ('groups','contact'),policy=beat.name in ('policy','thresholds'))
                    if mesh['run'] in timelines:
                        self.assertEqual(focus,timelines[mesh['run']].ids(frame))
                    expected=teaching_members(measured['selected']['teaching']['decision'],beat.name=='decision' and frame>beat.frames*.55) if beat.name in ('ages','decision') else f.members
                    for root in roots:
                        m=expected[root['slot']]
                        self.assertEqual((root['born'],root['age'],root['retired']),(m['born'],m['age'],m['retired']))
                        self.assertEqual(root['kind'],m.get('kind','unobserved'))
                        pose=dance_pose(music_seconds(beat.name,frame),m)
                        if mesh['run'] in timelines:
                            timeline=timelines[mesh['run']];seat=focus.index(root['slot'])
                            amount=timeline.amount(frame,seat)
                            self.assertAlmostEqual(root['dance_amount'],amount)
                            pose=join_pose(music_seconds(beat.name,frame),m,amount)
                            self.assertAlmostEqual(root['identity_visibility'],timeline.visibility(frame,seat))
                        if beat.name=='decision' and root['slot']==measured['selected']['teaching']['decision']['id']:
                            pose=join_pose(music_seconds(beat.name,frame),m,join_amount((frame-1)/30,math.floor(beat.frames*.55)/30))
                        feet=sorted((o for o in root.children_recursive if '.foot' in o.name),key=lambda o:o.name)
                        for foot,side,offset in zip(feet,(-1,1),(pose.left_foot,pose.right_foot)):
                            expected_foot=(side*.16+offset[0],-.06+offset[1],.06+offset[2])
                            for actual,want in zip(foot.location,expected_foot):self.assertAlmostEqual(actual,want,places=6)
                        elderly=m['age']>=65
                        for accessory in (o for o in root.children_recursive if '.silver-eyebrow' in o.name or '.gray-stitch' in o.name):
                            self.assertEqual(accessory.hide_render,not elderly or root.get('age_amount',1.)<=0)

                        identity=(root['run'],root['slot'],root['born'])
                        if history.get(identity):self.assertTrue(root['retired'])
                        history[identity]=root['retired']
                        polygon=mesh.data.polygons[sorted(f.members).index(root['slot'])]
                        self.assertTrue(all(mesh.data.vertices[i].co.length==0 for i in polygon.vertices))
                        body=next(o for o in root.children_recursive if o.name.endswith('.body'))
                        b=bounds(body,current)
                        self.assertGreaterEqual(b[1],.20,(beat.name,root.name,b))
                        self.assertGreaterEqual(b[0],0,(beat.name,root.name,b))
                        self.assertLessEqual(b[2],1,(beat.name,root.name,b))
                    for edge in [o for o in current.objects if o.get('run')==mesh['run'] and 'from_slot' in o and not o.hide_render]:
                        expected_network=measured['selected']['teaching']['decision']['neighbors'] if beat.name in ('ages','decision') else None
                        self.assertIn(edge['to_slot'],[m['id'] for m in expected_network] if expected_network else f.members[edge['from_slot']]['network'])
                self.assertFalse(any(o.instance_type=='COLLECTION'for o in current.objects))
            print('retirement source/layout:',beat.name,flush=True)
        beat=episode.load_episode('retirement')[-1]
        host=bpy.data.objects['host'];self.assertEqual((host['slot'],host['born'],host['age'],host['retired']),(7595,-75,98,True))
        original=host.matrix_world.copy();closed=[]
        for frame in range(1,beat.frames+1):
            current.frame_set(frame)
            self.assertEqual(host.matrix_world,original)
            if bpy.data.objects['host.eyes'].scale.z<1:closed.append(frame)
        self.assertEqual(closed,list(range(min(closed),max(closed)+1)))
        self.assertGreater(min(closed),.7*beat.frames)

    def test_every_recorded_cast_retirement_and_birth_boundary(self):
        measured=json.loads((STUDIO/'episodes/retirement/measurements.json').read_text())
        cache={}
        def load(key):
            if key not in cache: cache[key]=dump.load(STUDIO/'out/retirement/dumps'/f'{key}.frames.json')
            return cache[key]
        for beat in episode.load_episode('retirement'):
            if beat.name in ('ages','decision','end'): continue
            d=load(beat.shot)
            scene.install(scene.build_beat(beat,d,True,compare=load(beat.compare) if beat.compare else None,measured=measured))
            current=bpy.context.scene
            timelines={}
            boundaries=set()
            for key in [beat.shot]+([beat.compare] if beat.name in ('denominator','contact') else []):
                source=load(key);timing=beat.timing(source.ticks)
                timeline=CastTimeline(source,timing,beat.frames,
                    nearby_ids(beat.name,source,measured,filmed_frame(source,timing,1)),
                    lambda f:nearby_ids(beat.name,source,measured,f),
                    eligible=beat.name not in ('habits','renewal'),groups=beat.name in ('groups','contact'),
                    policy=beat.name in ('policy','thresholds'))
                timelines[key]=timeline
                for track in timeline.boundaries: boundaries.update(track)
                for row in timeline.onsets.values(): boundaries.update(x for x in row if x is not None)
                for row in timeline.age_onsets.values(): boundaries.update(x for x in row if x is not None)
            for boundary in sorted(boundaries):
                for frame in (boundary-.001,boundary,boundary+.001):
                    current.frame_set(math.floor(frame),subframe=frame%1)
                    frame=current.frame_current+current.frame_subframe
                    self.assertFalse(scene.ERRORS,scene.ERRORS)
                    for key,timeline in timelines.items():
                        source=load(key);f=filmed_frame(source,beat.timing(source.ticks),frame)
                        ids=timeline.ids(frame)
                        roots=[o for o in current.objects if o.get('run')==key and 'slot' in o]
                        self.assertEqual(sorted(o['slot'] for o in roots),sorted(ids))
                        self.assertEqual(len(roots),len(set(ids)))
                        private=[]
                        for root in roots:
                            m=f.members[root['slot']];seat=ids.index(root['slot'])
                            self.assertEqual((root['born'],root['age'],root['retired']),(m['born'],m['age'],m['retired']))
                            self.assertAlmostEqual(root['dance_amount'],timeline.amount(frame,seat),places=5)
                            self.assertAlmostEqual(root['identity_visibility'],timeline.visibility(frame,seat),places=5)
                            body=next(o for o in root.children_recursive if o.name.endswith('.body'))
                            if beat.name not in ('groups','contact'): private.append(body.data.materials[0].as_pointer())
                            for foot in (o for o in root.children_recursive if '.foot' in o.name):
                                if not m['retired']:
                                    self.assertAlmostEqual(foot.location.y,-.06,places=6)
                                    self.assertAlmostEqual(foot.location.z,.06,places=6)
                            for part in (o for o in root.children_recursive if '.silver-eyebrow' in o.name):
                                self.assertEqual(tuple(part.scale),(1.,1.,1.))
                                opacity=part.data.materials[0].node_tree.nodes['Age opacity'].inputs[0].default_value
                                self.assertAlmostEqual(opacity,timeline.age_amount(frame,seat),places=5)
                        self.assertEqual(len(private),len(set(private)))
                        for edge in (o for o in current.objects if o.get('run')==key and 'from_slot' in o and not o.hide_render):
                            self.assertIn(edge['to_slot'],f.members[edge['from_slot']]['network'])
            print('retirement source boundaries:',beat.name,len(boundaries),flush=True)

    def test_birth_change_and_wide_retired_toe_motion_are_visible(self):
        measured=json.loads((STUDIO/'episodes/retirement/measurements.json').read_text())
        beats=episode.load_episode('retirement')
        beat=next(b for b in beats if b.name=='renewal')
        d=dump.load(STUDIO/'out/retirement/dumps/teaching.frames.json')
        scene.install(scene.build_beat(beat,d,True,measured=measured))
        boundary=beat.timing(d.ticks).frame(.5)
        current=bpy.context.scene
        rig=next(o for o in current.objects if o.get('slot')==4047)
        for delta,born in ((-6,-40),(6,1)):
            current.frame_set(int(boundary+delta),subframe=(boundary+delta)%1)
            self.assertEqual(rig['born'],born)
            self.assertAlmostEqual(rig.scale.x,.75,places=5)
        beat=next(b for b in beats if b.name=='observable')
        d=dump.load(STUDIO/'out/retirement/dumps/all.frames.json')
        scene.install(scene.build_beat(beat,d,True,measured=measured))
        current=bpy.context.scene
        retired=next(o for o in current.objects if 'slot' in o and o['retired'])
        feet=[o for o in retired.children_recursive if '.foot' in o.name]
        lifts=[]
        for frame in range(1,60,3):
            current.frame_set(frame);bpy.context.view_layer.update()
            for foot in feet:
                actual=world_to_camera_view(current,current.camera,foot.matrix_world.translation)
                neutral=world_to_camera_view(current,current.camera,foot.parent.matrix_world@Vector((foot.location.x,foot.location.y,.06)))
                lifts.append(abs(actual.y-neutral.y)*540)
        self.assertGreaterEqual(max(lifts),3.0)

    def test_chart_fades_wait_until_original_text_has_cleared(self):
        measured=json.loads((STUDIO/'episodes/retirement/measurements.json').read_text())
        d=dump.load(STUDIO/'out/retirement/dumps/all.frames.json')
        for beat in (b for b in episode.load_episode('retirement') if b.name in ('observable','meaning')):
            scene.install(scene.build_beat(beat,d,True,measured=measured))
            current=bpy.context.scene
            entry,exit=.48*beat.frames,.94*beat.frames
            for frame in [entry+x for x in (-12,-9,-6,-3,0,6,12)]+[exit+x for x in (-12,-6,0,3,6,9,12)]:
                current.frame_set(math.floor(frame),subframe=frame%1)
                title=bpy.data.objects['retirement-method-title']
                detail=bpy.data.objects['retirement-method-detail']
                plot=bpy.data.objects['retirement-age-events-0']
                opacity=plot.data.materials[0].node_tree.nodes['Mix'].inputs[0].default_value
                main=bpy.data.objects['retirement-main-measure']
                if main.location.y>.390+1e-6:
                    self.assertAlmostEqual(title.data.materials[0].node_tree.nodes['Mix'].inputs[0].default_value,0.,places=6)
                if opacity>1e-6:
                    for original in (title,detail):
                        self.assertAlmostEqual(original.data.materials[0].node_tree.nodes['Mix'].inputs[0].default_value,0.,places=6)
                    self.assertAlmostEqual(bpy.data.objects['retirement-main-measure'].location.y,.465,places=6)
                self.assertFalse(scene.ERRORS,scene.ERRORS)

    def test_native_rolling_age_plots_match_period5(self):
        measured=json.loads((STUDIO/'episodes/retirement/measurements.json').read_text())
        d=dump.load(STUDIO/'out/retirement/dumps/all.frames.json')
        beat=next(b for b in episode.load_episode('retirement')if b.name=='observable')
        scene.install(scene.build_beat(beat,d,True,measured=measured))
        bpy.context.scene.frame_set(beat.frames//2)
        self.assertFalse(scene.ERRORS,scene.ERRORS)
        self.assertEqual(sum(measured['selected']['all']['retained_trace'][5]['events_by_age']),263)
        plotted=sum(bpy.data.objects['retirement-age-events-'+str(i)].scale.y*55/.12 for i in range(81)if bpy.data.objects['retirement-age-events-'+str(i)].scale.y>.00002)
        self.assertAlmostEqual(plotted,263,places=1)
        self.assertAlmostEqual(bpy.data.objects['retirement-age-rates-45'].scale.y/.12,55/439)
        self.assertIn('Mode 65',bpy.data.objects['retirement-main-measure'].data.body)
        self.assertIn('13.8%',bpy.data.objects['retirement-main-measure'].data.body)
        self.assertFalse(bpy.data.objects['retirement-age-events-45'].hide_render)
        bpy.context.scene.frame_set(1)
        self.assertTrue(bpy.data.objects['retirement-age-events-45'].hide_render)

    def test_exact_captions_fit_including_credit(self):
        for beat in episode.load_episode('retirement'):
            bpy.app.handlers.frame_change_pre.clear()
            bpy.ops.wm.read_factory_settings(use_empty=False)
            overlays.caption_scene(beat.caption,beat.caption_y,True,title=beat.title)
            bpy.context.view_layer.update()
            obj=next(o for o in bpy.context.scene.objects if o.type=='FONT'and o.name!='caption-shadow')
            b=bounds(obj,bpy.context.scene)
            self.assertGreaterEqual(min(b[:2]),0,(beat.name,b))
            self.assertLessEqual(max(b[2:]),1,(beat.name,b))

if __name__=='__main__':unittest.main(argv=[sys.argv[0]])
