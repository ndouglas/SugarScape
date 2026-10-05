import unittest
import thresholds_visual as v


class ThresholdsVisualTest(unittest.TestCase):
    def test_hop_occurs_only_on_recorded_join(self):
        self.assertGreater(v.join_hop(False,True,.5),0)
        self.assertEqual(v.join_hop(True,True,.5),0)
        self.assertEqual(v.join_hop(True,False,.5),0)
        self.assertEqual(v.join_hop(False,True,1),0)

    def test_links_are_actual_and_undirected_duplicates_removed(self):
        members={1:{'neighbors':[2]},2:{'neighbors':[1,3]},3:{'neighbors':[2]}}
        self.assertEqual(v.links(members),[(1,2),(2,3)])

    def test_neighbor_panel_uses_actual_integer_rule(self):
        member=dict(threshold_num=18,threshold_den=100,sees=1,of=5,seed=False)
        self.assertTrue(v.reached(member))
        member['of']=6
        self.assertFalse(v.reached(member))
        member['seed']=True
        self.assertTrue(v.reached(member))

    def test_people_count_labels_only_apply_to_unweighted_whole_crowds(self):
        self.assertTrue(v.people_counts({'network':'everyone','friends':{'enabled':False}}))
        self.assertFalse(v.people_counts({'network':'everyone','friends':{'enabled':True}}))
        self.assertFalse(v.people_counts({'network':'random','friends':{'enabled':False}}))
