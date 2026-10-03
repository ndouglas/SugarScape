"""Hand-derived fixture checks; no learning or native study runs."""
import unittest
from analysis import category, holm, probabilities, contrast, compatible_counts, read_sessions


class AnalysisTests(unittest.TestCase):
    def test_power_politics_excludes_hegemony_and_eleven(self):
        self.assertEqual([category(x) for x in [1, 2, 3, 10, 11, 90, 91, 100]],
                         [0, 1, 2, 2, 3, 3, 4, 4])

    def test_holm_restores_input_order_and_caps_adjusted_values(self):
        self.assertEqual(holm([.04, .001, .9]), [.08, .003, .9])

    def test_jeffreys_uses_all_five_categories_even_when_empty(self):
        self.assertEqual(probabilities([2, 0, 0, 0, 0]), [5/9, 1/9, 1/9, 1/9, 1/9])

    def test_uncertain_boundaries_keep_all_possible_twenty_run_vectors(self):
        self.assertEqual(compatible_counts([[5, 5], [14, 14], [19, 19], [19, 19]]),
                         [[5, 9, 5, 0, 1]])

    def test_invalid_cumulative_boundaries_are_rejected(self):
        with self.assertRaises(ValueError):
            compatible_counts([[10, 10], [5, 5], [19, 19], [20, 20]])

    def test_positive_constant_contrast_has_positive_interval(self):
        out = contrast([[1., 1., 1.], [0., 0., 0.]], [1., -1.], seed=7, draws=99)
        self.assertEqual(out['estimate'], 1.)
        self.assertEqual(out['interval'], [1., 1.])
        self.assertEqual(out['p'], .02)

    def test_same_constant_contrast_is_not_positive_evidence(self):
        out = contrast([[1., 1.], [1., 1.]], [1., -1.], seed=7, draws=99)
        self.assertEqual(out['p'], 1.)

    def test_invalid_session_is_retained_and_resolved_config_is_checked(self):
        import tempfile, json
        from pathlib import Path
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)/'raw.jsonl'
            p.write_text(json.dumps({'arm':'a','seed':2,'config':{'horizon':1},
                                     'outcome':{'valid':False,'invalid_reason':'nonpositive'}})+'\n')
            rows = read_sessions(p)
            self.assertEqual(rows['a'][0]['outcome']['invalid_reason'], 'nonpositive')
            p.write_text(p.read_text()+p.read_text())
            with self.assertRaises(ValueError):
                read_sessions(p)

if __name__ == '__main__':
    unittest.main()
