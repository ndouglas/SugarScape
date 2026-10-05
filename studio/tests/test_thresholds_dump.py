import json
import unittest
import dump


class ThresholdsDumpTest(unittest.TestCase):
    def test_exact_state_links_and_true_clock_survive_layout(self):
        raw=dict(format=1,model='thresholds',seed=200001,ticks=1,every=3,agents=2,config={},stats={},frames=[])
        for tick,active in [(0,False),(3,True)]:
            agents=[dict(id=i,threshold=(i-1)/2,threshold_num=i-1,threshold_den=2,
                         ceiling=None,degree=1,sees=int(active),of=1,acting=active,seed=i==1,crowd=1,neighbors=[3-i]) for i in (1,2)]
            raw['frames'].append(dict(tick=tick,step=tick,episodes=0,agents=agents,sizes=[0]*101))
        d=dump.parse(json.dumps(raw))
        self.assertEqual(d.model,'thresholds')
        self.assertEqual(d.frames[-1].period,3)
        self.assertEqual(d.frames[0].members[2]['neighbors'],[1])
        self.assertEqual(d.frames[0].members[2]['threshold_num'],1)
        self.assertEqual(d.frames[0].groups,{1:0,2:0})
        self.assertEqual(d.frames[1].groups,{1:1,2:1})
        self.assertEqual(set(d.frames[0].agents),{1,2})
        self.assertEqual(d.frames[0].agents,d.frames[1].agents)

    def test_missing_agents_are_rejected(self):
        raw=dict(format=1,model='thresholds',seed=1,ticks=0,every=1,agents=2,config={},stats={},frames=[dict(tick=0,step=0,episodes=0,agents=[],sizes=[0]*101)])
        with self.assertRaises(ValueError): dump.parse(json.dumps(raw))
