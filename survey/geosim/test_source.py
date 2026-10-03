"""Catch wrong CCDF populations, marginal resampling and shrunk Holm families."""
import math,unittest
import numpy as np
from survey.geosim import source

class SourceTests(unittest.TestCase):
    def test_strict_unique_ccdf_recovers_exact_geometric_regression(self):
        sizes=[0]+[100]*16+[1000]*8+[10000]*4+[100000]*2+[1000000]+[10000000]
        f=source.source_fit(sizes)
        self.assertAlmostEqual(f['slope'],-math.log10(2),places=12)
        self.assertAlmostEqual(f['r2'],1.,places=12)
        self.assertEqual((f['war_count'],f['positive_count'],f['log_range'],len(f['fit_points'])),(33,32,5.,4))
    def test_boundary_ties_use_strict_survival_and_maximum_is_excluded(self):
        f=source.source_fit([0,100,10**2.5,10**2.5,1000,10000,100000])
        self.assertEqual(len(f['fit_points']),3)
        self.assertEqual(f['fit_points'][0],[2.5,math.log10(.5)])
    def test_unavailable_fit_does_not_invent_zero_slope_or_range(self):
        self.assertIsNone(source.source_fit([0,0])['log_range'])
        f=source.source_fit([1000]*7)
        self.assertEqual(f['log_range'],0.)
        self.assertIsNone(f['slope'])
    def test_nonfinite_and_negative_source_data_fail(self):
        for s in [float('nan'),float('inf'),-1]:
            with self.assertRaises(ValueError):source.source_fit([1,s])
    def test_rounding_interval_maximum_covers_central_gaps_ties_and_tail_floor(self):
        self.assertEqual(source.maximize_interval_p([0,1,2,3],[1.4,1.6])['p'],1.)
        self.assertEqual(source.maximize_interval_p([0,0,0,1,2,2,5],[0,0])['p'],1.)
        self.assertEqual(source.maximize_interval_p([0,1,2,3],[100,100])['p'],.4)
    def test_fixed88_and6_families_keep_unavailable_hypotheses(self):
        self.assertEqual(source.fixed_holm([.001]+[None]*87,88)[0],.088)
        self.assertEqual(source.fixed_holm([.001,.008,.01,.4,None,None],6),[.006,.04,.04,1.,None,None])
        with self.assertRaises(ValueError):source.fixed_holm([.001],88)
    def test_source_verdict_requires_completeness_but_rejection_has_precedence(self):
        self.assertEqual(source.arm_verdict([.001,None],False),'Incompatible')
        self.assertEqual(source.arm_verdict([.1]*8,False),'Unresolved')
        self.assertEqual(source.arm_verdict([.1]*8,True),'Compatible')
    def test_joint_prediction_preserves_affine_relation_across_metrics(self):
        i=np.arange(100,dtype=float);v=np.column_stack((i,2*i+1,3*i+2,4*i+3))
        d=source.predictive_targets(v,np.random.default_rng(7),draws=32)['replicates']
        np.testing.assert_array_equal(d[:,4],2*d[:,1]+1)
        np.testing.assert_array_equal(d[:,6],3*d[:,1]+2)
        with self.assertRaises(ValueError):source.predictive_targets(v[:99],np.random.default_rng(7),draws=32)
    def test_independent_contrast_does_not_pair_identical_history_labels(self):
        i=np.arange(100,dtype=float);v=np.column_stack((i,i,i))
        f=source.independent_contrast(v,v,np.random.default_rng(7),draws=32)
        self.assertTrue(np.any(f['replicates']!=0))
    def test_constant_group_difference_has_inclusive_plus_one_p(self):
        f=source.independent_contrast(np.full((100,3),5.),np.full((100,3),3.),np.random.default_rng(7),draws=16)
        self.assertEqual(f['estimate'],[2.,2.,2.])
        self.assertEqual(f['intervals'],[[2.,2.]]*3)
        self.assertEqual(f['p'],[2/17]*3)
    def test_contrast_sign_and_missingness_remain_distinct(self):
        self.assertEqual(source.contrast_verdict(2.,.01),'Holds')
        self.assertEqual(source.contrast_verdict(-2.,.01),'Fails')
        self.assertEqual(source.contrast_verdict(2.,.05),'Inconclusive')
        self.assertEqual(source.contrast_verdict(None,None),'Unresolved')

if __name__=='__main__':unittest.main()
