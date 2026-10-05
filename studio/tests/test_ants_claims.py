import importlib.util
import pathlib
import unittest

PATH = pathlib.Path(__file__).resolve().parents[1] / 'episodes/ants/claims.py'

def load():
    spec = importlib.util.spec_from_file_location('ants_claims', PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

class AntsClaimsTests(unittest.TestCase):
    def test_protocol_counts_horizons_and_seeds(self):
        c = load()
        protocols = c.protocols()
        self.assertEqual(len(protocols['short']['seeds']), 1000)
        self.assertEqual(protocols['strong']['ticks'], 200000)
        self.assertEqual(protocols['micro']['ticks'], 1000)
        for name, p in protocols.items():
            self.assertEqual(len(p['seeds']), 1000 if name=='short' else 20)
            self.assertEqual(p['config']['stop_at'], 0)
            self.assertFalse(set(p['seeds']) & set(range(1, 1001)))
            if name.startswith(('ring_', 'random_', 'independent_')):
                self.assertEqual(p['ticks'], 300000)
                self.assertEqual(len(p['seeds']), 20)
        self.assertEqual(protocols['large']['ticks'] * protocols['large']['config']['meetings'], 10000000)

    def test_strict_boundary_judges(self):
        c = load()
        self.assertFalse(c.judge_strong(.65, 18, .03, [0,100]))
        self.assertTrue(c.judge_strong(.650001, 18, .03, [0,100]))
        self.assertFalse(c.judge_strong(.8, 17, .02, [0,100]))
        self.assertFalse(c.judge_short(.15, .20))
        self.assertTrue(c.judge_short(.35, .200001))
        self.assertFalse(c.judge_pull(.040001, [18,82]))
        self.assertTrue(c.judge_pull(.04, [18,82]))
        self.assertFalse(c.judge_growth(.15, 1, 6))
        self.assertTrue(c.judge_growth(.149, .7, 5.01))
        self.assertFalse(c.judge_growth(.1, 1.40001, 6))

    def test_event_selection_uses_smallest_seed_earliest_true_event(self):
        c = load()
        event = lambda t: {'kind':'recruit','tick':t,'update':1,'agent':1,'partner':2,'from_source':1,'to_source':2}
        rows = {200002:[event(1)], 200001:[event(9), event(3)]}
        self.assertEqual(c.select_event(rows,'recruit'), (200001,event(3)))

    def test_stream_retains_every_post_initial_sample(self):
        import tempfile
        c = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp)/'series.csv'
            path.write_text('tick,share\n0,.5\n1,.2\n2,.8\n3,.5\n')
            row = c.reduce_series(path, 10, 3)
        self.assertEqual(row['samples'],3)
        self.assertEqual(sum(row['histogram']),3)
        self.assertEqual(row['flips'],1)
        self.assertEqual(row['extreme'],2/3)
        self.assertEqual(row['actual_ticks'],3)

    def test_missing_or_held_ticks_are_rejected(self):
        import tempfile
        c = load()
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp)/'series.csv'
            for content in ('tick,share\n0,.5\n1,.5\n1,.5\n', 'tick,share\n0,.5\n1,.5\n'):
                path.write_text(content)
                with self.assertRaises(ValueError):
                    c.reduce_series(path, 10, 2)

    def test_aggregate_preserves_all_runs_and_histogram_samples(self):
        c = load()
        rows = {200001:dict(histogram=[2,0,0],variance=.1,extreme=1,flips=0),
                200002:dict(histogram=[0,0,2],variance=.2,extreme=1,flips=1)}
        aggregate = c.aggregate(rows)
        self.assertEqual(aggregate['runs'],2)
        self.assertEqual(aggregate['samples'],4)
        self.assertEqual(aggregate['histogram'],[2,0,2])
        self.assertEqual(aggregate['flip_runs'],1)

    def test_exact_capped_rule_modes_and_symmetry(self):
        c = load()
        for key,modes in [('strong',[0,100]),('pull',[18,82])]:
            distribution = c.exact(c.protocols()[key]['config'])
            self.assertAlmostEqual(sum(distribution),1)
            self.assertEqual([i for i,v in enumerate(distribution) if abs(v-max(distribution))<1e-12],modes)
            for left,right in zip(distribution,reversed(distribution)):
                self.assertAlmostEqual(left,right)

    def test_larger_and_independent_judge_boundaries(self):
        c = load()
        self.assertFalse(c.judge_large(.8,.4,.2,.1))
        self.assertFalse(c.judge_large(.8,.3,.2,.2))
        self.assertTrue(c.judge_large(.8,.399999,.2,.199999))
        self.assertFalse(c.judge_independent(.5))
        self.assertTrue(c.judge_independent(.499999))
