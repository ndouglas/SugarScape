import unittest
from types import SimpleNamespace
import ants_visual as v

class AntsVisualTest(unittest.TestCase):
    def test_event_uses_recorded_cause_pair_and_clock(self):
        e=dict(kind='recruit',agent=10,partner=67,from_source=2,to_source=1,tick=3,update=1)
        frames=[SimpleNamespace(ants_events=[]),SimpleNamespace(ants_events=[e])]
        self.assertEqual(v.recorded_event(frames,e),e)
        with self.assertRaises(ValueError): v.recorded_event(frames,dict(e,partner=66))
    def test_neighborhood_counts_actual_opposite_sources(self):
        m={1:dict(source=1),2:dict(source=2),3:dict(source=1),4:dict(source=2)}
        self.assertEqual(v.neighborhood(1,[(1,2),(3,1)],m),([2,3],1))
    def test_histogram_retains_all_mass_including_endpoint(self):
        self.assertEqual(v.bins([1,0,2,0,3],2),[1,5])
    def test_source_color_marks_actual_independent_members(self):
        self.assertEqual(v.color(dict(source=1,independent=True)),'butter')
        self.assertNotEqual(v.color(dict(source=1,independent=False)),v.color(dict(source=2,independent=False)))

    def test_filmed_frame_never_advances_its_clock_early(self):
        from animate import Timing
        d=SimpleNamespace(frames=['step170','step180'],ticks=1)
        self.assertEqual(v.filmed_frame(d,Timing(1,end_tick=1),20),'step170')
        self.assertEqual(v.filmed_frame(d,Timing(1,end_tick=1),31),'step180')

    def test_settled_source_positions_colors_and_counts_use_same_frame(self):
        import json, math
        import animate, dump
        raw=dict(format=1,model='ants',seed=9,ticks=1,every=3,agents=2,
                 config={'sources':2},stats={},links=[],frames=[])
        for tick,sources in ((0,(1,2)),(3,(1,1))):
            members=[dict(id=i+1,source=s,independent=False,degree=1,elsewhere=int(sources[0]!=sources[1])) for i,s in enumerate(sources)]
            raw['frames'].append(dict(tick=tick,agents=members,counts=[sources.count(1),sources.count(2)],ants_events=[]))
        d=dump.parse(json.dumps(raw));timing=animate.Timing(1,end_tick=d.ticks)
        corners=animate.corner_heights(d.capacity,d.width,d.height);tracks=dump.tracks(d)
        for index,frame in ((0,1),(1,31),(1,300)):
            f=v.filmed_frame(d,timing,frame)
            self.assertEqual(f.period,(0,3)[index])
            self.assertEqual(f.counts,[sum(m['source']==s for m in f.members.values()) for s in (1,2)])
            for actor,member in f.members.items():
                p=animate.pose(tracks[actor],timing,frame,corners,d.width,d.height)
                a=f.agents[actor]
                self.assertEqual((p.x,p.y),animate.cell_center(a.x,a.y,d.width,d.height))
                self.assertEqual(a.x//(math.ceil(math.sqrt(len(f.members)))+2)+1,member['source'])
                self.assertEqual(v.color(member),'blue' if member['source']==1 else 'coral')
