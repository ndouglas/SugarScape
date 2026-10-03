import unittest
import episode


class ThresholdsClaimsTest(unittest.TestCase):
    def setUp(self):
        self.c = episode.load_module('thresholds', 'claims')

    def test_city_requires_exact_draw_count_and_fixed_tolerance(self):
        self.assertTrue(self.c.city_holds([0] * 2507 + [100] * 2493))
        self.assertFalse(self.c.city_holds([0] * 5000))
        self.assertFalse(self.c.city_holds([0, 100] * 10))

    def test_network_cutoffs_are_strict_small_and_inclusive_large(self):
        self.assertEqual(self.c.outcome_counts([1, 9, 10, 899, 900, 1000], 1000), (2, 2))

    def test_rescue_rule_requires_both_frequencies(self):
        self.assertTrue(self.c.rescue_holds([1] * 800 + [2] * 200))
        self.assertFalse(self.c.rescue_holds([1] * 801 + [2] * 199))
        self.assertFalse(self.c.rescue_holds([1] * 799 + [20] * 201))

    def test_fixed_seed_corpora_are_disjoint_from_audit_and_each_other(self):
        corpora = list(self.c.SEEDS.values())
        flat = [s for corpus in corpora for s in corpus]
        self.assertGreaterEqual(min(flat), 200001)
        self.assertEqual(len(flat), len(set(flat)))
        self.assertTrue(all(len(corpus) >= 20 for corpus in corpora))
