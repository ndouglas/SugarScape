import unittest

import episode

CONFIG = {"crowd": (625, "random"), "rule": ("symmetric", "simultaneous", "all"), "edges": True,
          "middle_seed_two": True}
SURVEY = {"ok": True, "why": "18 of 19"}
ROW = dict(few=37.0, many=1.0, lean_mean=0.94, extremes_range=0.99, lattice_polar=0.0, everyone_polar=3.0,
           lattice_clusters=50.0, agree_50=0.0, agree_1000=1.0)


def rows(**changes):
    return {s: dict(ROW, **changes) for s in range(1, 21)}


def middle(two=16, mid=30):
    return {s: {"two": float(s <= two), "mid": float(two < s <= two + mid), "stable": 9.0} for s in range(1, 51)}


class OpinionsVerdictTest(unittest.TestCase):
    claims = episode.load_module("opinions", "claims")

    def verdicts(self, r=None, m=None, survey=None, **config):
        return {c: h for c, h, _ in self.claims.verdicts(r or rows(), m or middle(), dict(CONFIG, **config),
                                                         survey or SURVEY)}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_two_camps_as_the_rule_breaks_a_third_of_the_time(self):
        v = self.verdicts(m=middle(two=30, mid=15))
        self.assertFalse(v["here that happens about a third of the time; more often, a third camp holds the middle"])

    def test_a_split_at_wide_confidence_breaks_everyone_agrees(self):
        r = rows()
        r[4]["many"] = 2.0
        self.assertFalse(self.verdicts(r)["listen widely, and everyone agrees"])

    def test_polarized_lattices_break_two_camps_become_rare(self):
        v = self.verdicts(rows(lattice_polar=2.0))
        self.assertFalse(v["hear only your neighbors on a map, and two camps become rare: one crowd, with stranded "
                           "minorities"])

    def test_small_crowds_agreeing_as_often_breaks_numbers_matter(self):
        v = self.verdicts(rows(agree_50=1.0))
        self.assertFalse(v["and numbers matter: a big crowd agrees where a small one splits"])

    def test_another_failing_survey_claim_breaks_nearly_all(self):
        v = self.verdicts(survey={"ok": False, "why": ""})
        self.assertFalse(v["nearly all they reported, the Flumps reproduce"])
