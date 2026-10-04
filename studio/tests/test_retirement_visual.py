"""Real-member layouts and native clocks, independent of Blender."""
import unittest
from types import SimpleNamespace
import animate

class RetirementVisualTests(unittest.TestCase):
    def setUp(self):
        import retirement_visual
        self.v=retirement_visual
    def test_dense_birth_cohort_keeps_every_slot_in_bounds(self):
        members={i:{'id':i,'age':20 if i<3000 else 65,'kind':'imitator','retired':False}for i in range(8100)}
        positions,counts=self.v.cohort_layout(members)
        self.assertEqual(counts,{20:3000,65:5100})
        self.assertEqual(set(positions),set(range(8100)))
        self.assertEqual(len(set(positions.values())),8100)
        self.assertTrue(all(0<=x<=1 and 0<=y<=1 for x,y in positions.values()))
    def test_sampled_clock_uses_real_periods_and_never_invents_midstep_state(self):
        d=SimpleNamespace(ticks=2,frames=[SimpleNamespace(period=p)for p in(0,5,9)])
        timing=animate.Timing(1,end_tick=2)
        self.assertEqual([self.v.filmed_frame(d,timing,f).period for f in(1,16,31,46,61)],[0,5,5,9,9])
    def test_minority_example_chooses_first_actual_sample_with_simultaneous_bound(self):
        trace=[{'tick':i,'mode':65,'share':.14,'rolling_events':20}for i in range(11)]
        selection={'frame_periods':[0,5,10],'retained_trace':trace}
        trace[0]['mode']=None
        self.assertEqual(self.v.minority_period(selection),5)
    def test_exact_decision_and_birth_identity_labels_keep_slot_zero(self):
        d={'threshold_units':500000,'threshold_scale':1000000,'counted':2,'retired_counted':1,'retired_after':True}
        self.assertEqual(self.v.decision_summary(d),'1 / 2 = 0.50  ≥  0.50 → retire')
        self.assertEqual(self.v.identity({'id':0,'born':1}),'slot 0 · born 1')
    def test_histogram_rebin_keeps_every_retirement(self):
        self.assertEqual(self.v.rebin([2,0,3,0,5],2),[5,5])

    def test_timing_histogram_keeps_attained_and_censored_outcomes(self):
        rows=[{'first95':1},{'first95':100},{'first95':None}]
        self.assertEqual(self.v.timing_histogram(rows,'first95',100,2),([1,1],1))

    def test_comparison_method_labels_identify_actual_policy_and_group_configs(self):
        original={'threshold':.5,'spread':0}
        revised={'threshold':.75,'spread':.25/(3**.5)}
        self.assertEqual(self.v.policy_legend(original,revised),'Blue: original threshold.5\nCoral: revised U[.5,1]')
        config={'rational':.1,'random':.05,'groups':{'coupling':.05}}
        self.assertEqual(self.v.group_method(config),'A rational0% · B rational10%\nBoth random5% · coupling.05')
        self.assertEqual([self.v.policy_color(k)for k in('policy_original','policy_revised','policy_censored')],['blue','coral','coral'])

    def test_denominator_and_group_reference_legends_name_each_actual_trace(self):
        self.assertEqual(self.v.trace_legend('denominator'),'Blue: eligible · coral: all')
        self.assertEqual(self.v.trace_legend('contact',.2,.05),'Blue:A.20 · coral:B.20\nTeal:A.05 · lilac:B.05')
