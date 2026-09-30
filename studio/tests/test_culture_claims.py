import unittest

import episode

CONFIG = {"sample": (10, 10, 5, 10), "rule": ("von_neumann", "random", "active", "random"), "survey": True}
ROW = dict(sample=4.0, sample_stable=800.0, q15=20.0, f15=1.0, four=4.0, twelve=1.4, small_map=21.0, big_map=5.0,
           q25_20=205.0, q25_30=360.0, no_drift=17.0, drift=5.0, wander=1.0, wander_stop=1800.0)
SAMPLE = [4.0] * 600 + [1.0] * 100 + [8.0] * 300


def rows(**changes):
    return {s: dict(ROW, **changes) for s in range(1, 21)}


class CultureVerdictTest(unittest.TestCase):
    claims = episode.load_module("culture", "claims")

    def verdicts(self, r=None, sample=None, **config):
        return {c: h for c, h, _ in self.claims.verdicts(r or rows(), sample or SAMPLE, dict(CONFIG, **config))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_run_that_never_stops_breaks_nothing_changes(self):
        r = rows()
        r[3]["sample_stable"] = float(self.claims.CAP)
        self.assertFalse(self.verdicts(r)["neighbors grow alike, until each pair is either the same or shares "
                                          "nothing; then nothing changes"])

    def test_a_sample_median_of_three_breaks_about_four_here(self):
        v = self.verdicts(sample=[3.0] * 1000)
        self.assertFalse(v["a few cultures survive, about four here; Axelrod's runs, about three"])

    def test_one_divided_run_at_fifteen_features_breaks_one_culture_wins(self):
        r = rows()
        r[7]["f15"] = 2.0
        self.assertFalse(self.verdicts(r)["more features to share, and one culture wins"])

    def test_a_big_map_as_divided_breaks_the_surprise(self):
        self.assertFalse(self.verdicts(rows(big_map=15.0))["the surprise: a bigger map ends with fewer cultures"])

    def test_a_big_map_that_holds_together_breaks_shatters(self):
        v = self.verdicts(rows(q25_30=150.0))
        self.assertFalse(v["unless there are enough traits; then a big map shatters"])

    def test_wanderers_keeping_two_cultures_break_takes_everyone(self):
        v = self.verdicts(rows(wander=2.0))
        self.assertFalse(v["let them wander a sugar mountain, and one culture takes everyone"])

    def test_a_failing_survey_claim_breaks_reproduces(self):
        self.assertFalse(self.verdicts(survey=False)["everything Axelrod reported, the Flumps reproduce"])
