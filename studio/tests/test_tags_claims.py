import unittest

import episode

ROW = dict(gifts=0.737, cluster=0.853, related=0.971, twins=0.829, tolerance=0.018, takeovers=29.0,
           per_takeover=1034.0, gap=400.0, steps=[0.005, 0.007, 0.01, 0.004], twin_gifts=0.94, strict=0.0138,
           literal=0.737, clones=0.753)
CONFIG = dict(agents=100, pairings=3, donation_test="at_most", cost=0.1, benefit=1.0, selection="tournament",
              tie_rule="current")


def rows(**changes):
    return {s: dict(ROW, **changes) for s in range(1, 21)}


class TagsVerdictTest(unittest.TestCase):
    claims = episode.load_module("tags", "claims")

    def verdicts(self, r=None, **config):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), dict(CONFIG, **config))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_the_caption_must_name_the_measured_rate(self):
        self.assertFalse(self.verdicts(rows(gifts=0.70))["it works: about 74% of meetings end in a gift, as the paper "
                                                         "says"])

    def test_scattered_shades_break_most_share_one(self):
        self.assertFalse(self.verdicts(rows(twins=0.4))["the paper saw it too: most Flumps share one exact shade"])

    def test_a_wide_tolerance_breaks_tiny(self):
        r = rows()
        r[8]["tolerance"] = 0.2
        self.assertFalse(self.verdicts(r)["their tolerance is tiny; they help almost no one but their twins"])

    def test_a_world_where_clones_give_less_breaks_even_more(self):
        r = rows()
        r[4]["clones"] = 0.73
        self.assertFalse(self.verdicts(r)["take tolerance away entirely, and they give even more: 75%"])

    def test_one_takeover_from_across_the_ring_breaks_next_door(self):
        r = rows()
        r[12]["steps"] = [0.005, 0.4]
        self.assertFalse(self.verdicts(r)["every few hundred generations, a new crowd takes over: always right next "
                                          "door"])

    def test_takeovers_every_few_thousand_generations_break_every_few_hundred(self):
        self.assertFalse(self.verdicts(rows(gap=3000.0))["every few hundred generations, a new crowd takes over: always "
                                                         "right next door"])

    def test_gifts_mostly_to_strangers_break_help_almost_no_one_but_their_twins(self):
        self.assertFalse(self.verdicts(rows(twin_gifts=0.6))["their tolerance is tiny; they help almost no one but "
                                                             "their twins"])

if __name__ == "__main__":
    unittest.main()
