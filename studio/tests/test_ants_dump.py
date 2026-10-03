import json
import unittest
import dump


class AntsDumpTest(unittest.TestCase):
    def raw(self):
        return dict(format=1,model="ants",seed=9,ticks=1,every=3,agents=2,config={"sources":2},stats={},links=[[1,2]],frames=[
            dict(tick=0,counts=[1,1],agents=[dict(id=1,source=1,independent=True,degree=1,elsewhere=1),dict(id=2,source=2,independent=False,degree=1,elsewhere=1)],ants_events=[]),
            dict(tick=3,counts=[2,0],agents=[dict(id=1,source=1,independent=True,degree=1,elsewhere=0),dict(id=2,source=1,independent=False,degree=1,elsewhere=0)],ants_events=[dict(kind="recruit",agent=2,partner=1,from_source=2,to_source=1,tick=3,update=1)])])

    def test_real_identity_state_clock_and_events_survive_layout(self):
        d=dump.parse(json.dumps(self.raw()))
        self.assertEqual(d.model,"ants")
        self.assertEqual(d.frames[-1].period,3)
        self.assertEqual(d.frames[-1].counts,[2,0])
        self.assertEqual(d.frames[0].members[1]["independent"],True)
        self.assertEqual(d.frames[0].members[2]["elsewhere"],1)
        self.assertEqual(d.frames[-1].ants_events[0]["partner"],1)
        self.assertEqual(d.frames[-1].ants_events[0]["update"],1)
        self.assertEqual(d.frames[-1].members[2]["source"],1)
        self.assertEqual(d.links,[(1,2)])
        self.assertEqual(set(d.frames[0].agents),set(d.frames[1].agents))
        self.assertNotEqual(d.frames[0].agents[2].x,d.frames[1].agents[2].x)

    def test_missing_identity_is_rejected(self):
        raw=self.raw()
        raw["frames"][0]["agents"].pop()
        with self.assertRaises(ValueError):
            dump.parse(json.dumps(raw))
