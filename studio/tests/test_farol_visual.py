"""Recorded-clock, advice and corpus contracts (hand-derived fixtures)."""
import importlib.util
import unittest
from types import SimpleNamespace
import animate

class FarolVisualTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(importlib.util.find_spec('farol_visual'), 'recorded Farol visual adapter missing')
        import farol_visual
        self.v=farol_visual

    def test_all_changes_and_readouts_use_one_atomic_snapshot(self):
        frames=[SimpleNamespace(period=k*5, agents={1:SimpleNamespace(x=k*12,y=0)}) for k in range(3)]
        d=SimpleNamespace(frames=frames,ticks=2,width=24,height=10)
        timing=animate.Timing(1,end_tick=2)
        self.assertEqual([self.v.filmed_frame(d,timing,f).period for f in (1,15,16,30,31,46)], [0,0,5,5,5,10])
        self.assertEqual(self.v.pose(d,1,timing,15).x,-11.5)
        self.assertEqual(self.v.pose(d,1,timing,16).x,.5)

    def test_forecast_uses_selected_held_strategy_before_score_update(self):
        member={'selected':1,'went':False,'strategies':[{'label':'low','forecast':12,'score':8}, {'label':'equal','forecast':60,'score':2}]}
        self.assertEqual(self.v.selected_strategy(member),member['strategies'][1])
        self.assertFalse(self.v.forecast_advice(60))
        self.assertTrue(self.v.forecast_advice(59))
        self.assertIsNone(self.v.selected_strategy(dict(member,selected=None)))

    def test_lookup_bits_show_oldest_to_newest_without_inventing_table(self):
        self.assertEqual(self.v.memory_bits(0b001011,6),'001011')
        self.assertEqual(self.v.memory_bits(None,6),'—')
        member={'memory':6,'selected':1,'went':True,'strategies':[{'attend':False,'score':7},{'attend':True,'score':9}]}
        self.assertEqual(self.v.lookup(member,11),('001011','A',9))

    def test_capacity_and_minority_equality_are_distinct(self):
        self.assertEqual(self.v.outcome(60,100,60,'el_farol'),'CROWDED')
        self.assertEqual(self.v.outcome(59,100,60,'el_farol'),'QUIET')
        self.assertEqual(self.v.outcome(50,101,50,'minority'),'A wins')
        self.assertEqual(self.v.outcome(51,101,50,'minority'),'B wins')
        self.assertEqual(self.v.outcome(50,100,50,'minority'),'Equal sides')

    def test_histogram_rebins_preserve_all_observations_and_endpoints(self):
        self.assertEqual(self.v.rebin([3,0,1,0,5],2),[4,5])
        self.assertAlmostEqual(sum(self.v.distribution([3,0,1,0,5],2)),1)
