import csv
import pathlib
import tempfile
import unittest
import episode


class FarolClaimsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        assert (pathlib.Path(episode.__file__).parent/'episodes/farol/claims.py').exists(), 'Farol measurement implementation is missing'
        cls.c = episode.load_module('farol', 'claims')

    def test_fresh_protocol_counts_horizons_and_explicit_rules(self):
        p = self.c.protocols()
        self.assertEqual(set(p), {'accuracy','advice','random','shared','m2','m6','m12'})
        for name, row in p.items():
            memory = name.startswith('m')
            self.assertEqual(row['seeds'], list(range(2001,2033) if memory else range(1001,1021)))
            self.assertEqual((row['ticks'],row['burn']), (10000,2000) if memory else (2000,400))
            self.assertEqual(row['config']['agents'], 101 if memory else 100)
            self.assertEqual(row['config']['strategies'], 2 if memory else 12)
            self.assertEqual(row['config']['at_capacity'], 'stay')
            self.assertEqual(row['config']['stop_at'], 0)
        self.assertEqual(p['m12']['config']['memory'],12)
        self.assertEqual(self.c.RULES['mean_margin'],2)
        self.assertEqual(self.c.RULES['chance_margin'],.05)

    def test_hand_series_variance_center_deviation_and_lag(self):
        r = self.c.moments([1,3,1,3], 3, 4)
        self.assertEqual((r['mean'],r['variance'],r['center_squared_deviation']), (2,1,2))
        self.assertEqual(r['lag1'],-.75)
        self.assertEqual(r['histogram'],[0,2,0,2,0])
        self.assertEqual(sum(r['histogram']),r['samples'])

    def test_zero_variance_lag_is_unavailable(self):
        self.assertIsNone(self.c.moments([2,2],2,4)['lag1'])

    def test_initial_and_burn_rows_are_excluded_and_clock_validated(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp)/'series.csv'
            path.write_text('tick,attendance\n0,0\n1,4\n2,1\n3,3\n')
            row, trace = self.c.reduce_series(path,dict(ticks=3,burn=1,config=dict(agents=4,game='el_farol',capacity=3)))
            self.assertEqual(trace,[1,3])
            self.assertEqual(row['samples'],2)
            self.assertEqual(row['first_round'],2)
            self.assertEqual(row['last_round'],3)
            path.write_text('tick,attendance\n0,0\n1,4\n3,3\n')
            with self.assertRaises(ValueError): self.c.reduce_series(path,dict(ticks=3,burn=1,config=dict(agents=4)))

    def test_median_variance_selection_breaks_ties_by_smallest_seed(self):
        self.assertEqual(self.c.select_seed({9:dict(variance=1),3:dict(variance=3),8:dict(variance=5),2:dict(variance=7)}),3)

    def test_mean_and_chance_margins_are_fixed(self):
        self.assertTrue(self.c.near_mean([58,62],60))
        self.assertFalse(self.c.near_mean([62.01],60))
        self.assertTrue(self.c.near_chance([.2,.3]))
        self.assertFalse(self.c.near_chance([.30001]))

    def test_paired_ordering_preserves_failures_and_effect(self):
        a={s:dict(variance=s+20) for s in range(1,21)}
        b={s:dict(variance=s) for s in range(1,21)}
        r=self.c.compare(a,b)
        self.assertTrue(r['holds'])
        self.assertEqual(r['positive_pairs'],20)
        self.assertEqual(r['mean_difference'],20)
        a[1]['variance']=0
        r=self.c.compare(a,b)
        self.assertEqual(r['differences']['1'],-1)
        self.assertEqual(r['positive_pairs'],19)

    def test_existing_survey_mann_whitney_exact_reference(self):
        self.assertAlmostEqual(self.c.mw_greater([4,5,6],[1,2,3]),.05)
        self.assertEqual(self.c.mw_greater([1,1],[1,1]),1)

    def test_shot_window_uses_true_clock_and_full_explicit_config(self):
        p=self.c.protocols()['m6']
        shot, selected=self.c.shot_spec('m6',2007,p)
        self.assertEqual(shot['ticks'],2240)
        self.assertEqual(shot['every'],5)
        self.assertEqual(selected['start_round'],2000)
        self.assertEqual(selected['end_round'],2240)
        self.assertEqual(selected['start_frame'],400)
        self.assertEqual(shot['config'],p['config'])


if __name__=='__main__': unittest.main()
