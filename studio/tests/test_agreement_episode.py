"""Behavior checks for the agreement film's clocks, uncertainty and score."""
import pathlib
import sys
import unittest
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
import episode
import music
from agreement_visual import interval, overlap, marker_length, diagram_point, uncertainty_color, select_pair, outcome_clock

class AgreementVisualTests(unittest.TestCase):
    def test_intervals_keep_physical_endpoints(self):
        self.assertAlmostEqual(interval(.9, .3)[0], .6)
        self.assertAlmostEqual(interval(.9, .3)[1], 1.2)
    def test_overlap_uses_both_actual_ranges(self):
        self.assertAlmostEqual(overlap(-.8, .1, -.5, .5), .2)
    def test_example_pair_has_asymmetric_influence(self):
        opinions = {1: -.9, 2: -.7, 3: .5}
        uncertainties = {1: .1, 2: 1.4, 3: 1.4}
        i, j = select_pair(opinions, uncertainties)
        h = overlap(opinions[i], uncertainties[i], opinions[j], uncertainties[j])
        self.assertGreater(h, uncertainties[i])
        self.assertLess(h, uncertainties[j])
    def test_clock_uses_measured_stability_not_padding(self):
        self.assertEqual(outcome_clock({'stable_at': [0, 15806], 'max_change': [0.0]}, 20000), 'settled at 15,806')
        self.assertEqual(outcome_clock({'stable_at': [0], 'max_change': [.01]}, 20000), 'capped at 20,000')
    def test_nonzero_stable_at_does_not_establish_stability(self):
        self.assertEqual(outcome_clock({'stable_at': [200], 'max_change': [.01]}, 200, False), 'stopped at 200')
        self.assertEqual(outcome_clock({'stable_at': [20000], 'max_change': [.01]}, 20000, True), 'capped at 20,000')
    def test_marker_length_tracks_uncertainty(self):
        self.assertEqual(marker_length(.8), 8 * marker_length(.1))
    def test_diagram_uses_true_period(self):
        self.assertEqual(diagram_point(20, -.5, 100), (.2, .25))
    def test_uncertainty_legend_has_fixed_endpoints(self):
        self.assertEqual(uncertainty_color(0), (1.0, .2, .15))
        for actual, expected in zip(uncertainty_color(2), (.2, .9, .35)):
            self.assertAlmostEqual(actual, expected)
    def test_fourteen_beats_and_four_protocol_comparison(self):
        beats = episode.load_episode('agreement')
        self.assertEqual(len(beats), 14)
        self.assertIn('agreement-results', next(b for b in beats if b.name == 'readings').overlays)
    def test_final_credit_floats_above_single_host_without_title_scrim(self):
        end = episode.load_episode('agreement')[-1]
        self.assertEqual(end.name, 'end')
        self.assertIsNone(end.shot)
        self.assertFalse(end.title)
        self.assertEqual(end.caption_y, .45)
        self.assertEqual(end.caption, 'How extremists win - After Deffuant et al., 2002\nndouglas.github.io/SugarScape')
    def test_score_is_six_eighths_every_bar(self):
        tune = episode.load_module('agreement', 'tune').TUNE
        self.assertEqual((tune.meter, tune.beats_per_bar), ('6/8', 3))
        for parts in tune.sections.values():
            for line in parts.values():
                self.assertEqual([music.eighths(b) for b in music._bars(line)], [6] * 8)
        music.score(tune, sum(b.seconds for b in episode.load_episode('agreement')) - 13 * .4)

if __name__ == '__main__': unittest.main()
