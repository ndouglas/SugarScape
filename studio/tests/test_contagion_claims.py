import unittest

import episode

HOLDS = dict(v1_start=0.91, v1_well_by=8, v1_late=0.0, v1_end=0.0, v2_late=0.0, reach=0.55, ratio=1.0,
             plague_reach=1.0, plague_ratio=0.76, pop_calm=380.0, pop_plague=290.0)


def rows(**changes):
    out = {s: dict(HOLDS, **changes) for s in range(1, 21)}
    for s in range(1, 11):
        out[s]["ratio"] = 0.98  # a coin flip: lower in half the worlds
    return out


class ContagionVerdictTest(unittest.TestCase):
    claims = episode.load_module("contagion", "claims")
    RID = "the book says society rids itself of every disease; it does, in 20 of 20 worlds"
    CLEARS = ("give them more diseases than an immune system can hold: the book says disease stays; it clears here too, "
              "in 20 of 20")
    SPREADS = "the book expects a plague; it spreads to about half the Flumps, and costs almost nothing"

    def verdicts(self, r=None):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows())}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_slow_learning_breaks_the_few_ticks_claim(self):
        self.assertFalse(self.verdicts(rows(v1_well_by=40))["within a few ticks, nearly all are well"])

    def test_one_lingering_world_breaks_the_rid_claim(self):
        r = rows()
        r[3]["v1_end"] = 0.01
        self.assertFalse(self.verdicts(r)[self.RID])

    def test_an_endemic_world_breaks_the_clears_claim(self):
        r = rows()
        r[3]["v2_late"] = 0.02
        self.assertFalse(self.verdicts(r)[self.CLEARS])

    def test_a_toll_breaks_the_costs_almost_nothing_claim(self):
        self.assertFalse(self.verdicts(rows(ratio=0.8))[self.SPREADS])

    def test_a_harmless_long_disease_breaks_the_plague_claim(self):
        claim = "only a disease four times longer than any in the book sweeps through, and costs about a quarter of the Flumps"
        self.assertFalse(self.verdicts(rows(plague_ratio=1.0))[claim])


if __name__ == "__main__":
    unittest.main()
