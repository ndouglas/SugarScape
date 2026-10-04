import unittest
import numpy as np
from survey.democratic_peace import numerics


class NumericalTests(unittest.TestCase):
    def test_interval_maximization_covers_inclusive_jumps_and_gaps(self):
        samples = [0., 0., 1., 2., 2.]
        result = numerics.maximize_interval_p(samples, [.25, .75])
        self.assertEqual(result['p'], 1.)
        self.assertEqual(result['maximizer_kind'], 'open_interval')
        self.assertEqual(numerics.inclusive_p(samples, -1.), 2 / 6)

    def test_holm_preserves_absent_family_slots(self):
        values = [.0001] + [None] * 104
        self.assertEqual(numerics.fixed_holm(values, 105)[0], .0105)
        self.assertIsNone(numerics.fixed_holm(values, 105)[1])

    def test_extinction_draws_are_undefined_not_zero_or_replaced(self):
        values = [2., 4.] + [None] * 98
        result = numerics.predictive(values, np.random.default_rng(4), draws=50, minimum_defined=1)
        self.assertEqual(result['defined_draws'] + result['undefined_draws'], 50)
        self.assertGreater(result['undefined_draws'], 0)
        self.assertTrue(all(2 <= x <= 4 for x in result['replicates']))

    def test_all_extinction_retains_every_undefined_draw_without_replacement(self):
        result = numerics.predictive([None] * 100, np.random.default_rng(2), draws=50)
        self.assertEqual(result['reason'], 'insufficient_conditional_reference')
        self.assertEqual(result['attempted_draws'], 50)
        self.assertEqual(result['defined_draws'], 0)
        self.assertEqual(result['undefined_draws'], 50)
        self.assertEqual(result['undefined_fraction'], 1.)

    def test_one_survivor_retains_draw_mass_but_reference_remains_unavailable(self):
        result = numerics.predictive([1.] + [None] * 99, np.random.default_rng(2),
            draws=50, minimum_defined=1)
        self.assertEqual(result['reason'], 'insufficient_conditional_reference')
        self.assertEqual(result['attempted_draws'], 50)
        self.assertGreater(result['defined_draws'], 0)
        self.assertGreater(result['undefined_draws'], 0)
        self.assertEqual(result['defined_draws'] + result['undefined_draws'], 50)
        self.assertEqual(result['undefined_fraction'], result['undefined_draws'] / 50)
        self.assertTrue(all(value == 1. for value in result['replicates']))

    def test_contrast_uses_whole_independent_groups_and_exact_quantiles(self):
        result = numerics.contrast([1.] * 100, [0.] * 100,
            np.random.default_rng(1), np.random.default_rng(2), draws=20)
        self.assertEqual(result['estimate'], 1.)
        self.assertEqual(result['interval'], [1., 1.])
        self.assertEqual(result['p'], 1 / 21)


if __name__ == '__main__':
    unittest.main()
