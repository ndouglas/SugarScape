import unittest

import measure


class MeasureTest(unittest.TestCase):
    def test_summary(self):
        self.assertEqual(measure.summary([3, 1, 2]), {"median": 2, "lo": 1, "hi": 3})

    def test_typical_seed_is_closest_to_the_medians(self):
        rows = {1: {"a": 0, "b": 10}, 2: {"a": 5, "b": 5}, 3: {"a": 10, "b": 0}}
        self.assertEqual(measure.typical_seed(rows, ["a", "b"]), 2)

    def test_a_constant_measure_does_not_divide_by_zero(self):
        rows = {1: {"a": 1, "b": 0}, 2: {"a": 1, "b": 4}, 3: {"a": 1, "b": 5}}
        self.assertEqual(measure.typical_seed(rows, ["a", "b"]), 2)


class VerdictTest(unittest.TestCase):
    ii2 = {s: {"pop25": 246, "on_hills": 0.92} for s in range(1, 4)}
    ii5 = {
        s: {"gini0": 0.23, "gini500": 0.48, "mean_over_median": 1.47, "top10_share": 0.30, "bottom50_share": 0.2}
        for s in range(1, 4)
    }

    def test_claims_that_hold(self):
        v = measure.verdicts(self.ii2, self.ii5, hill_share=0.31, rose=20)
        self.assertEqual([holds for _, holds, _ in v], [True] * 5)

    def test_an_even_split_fails_the_wealth_gap_claim(self):
        even = {s: dict(self.ii5[s], top10_share=0.12, bottom50_share=0.45) for s in self.ii5}
        v = dict((claim, holds) for claim, holds, _ in measure.verdicts(self.ii2, even, 0.31, 20))
        self.assertFalse(v["some have much more than others (question beat)"])

    def test_a_weak_selection_effect_fails_its_claim(self):
        v = dict((claim, holds) for claim, holds, _ in measure.verdicts(self.ii2, self.ii5, 0.31, rose=15))
        self.assertFalse(v["sight up, hunger down (beat 8 dials)"])


if __name__ == "__main__":
    unittest.main()
