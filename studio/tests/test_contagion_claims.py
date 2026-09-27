import unittest

import episode

HOLDS = dict(v1_start=0.91, v1_well_by=7, v1_late=0.018, v1_end=0.018, v2_late=0.042, reach=0.047, ratio=0.98,
             plague_reach=1.0, plague_ratio=0.8, pop_calm=380.0, pop_plague=304.0)


def rows(**changes):
    out = {s: dict(HOLDS, **changes) for s in range(1, 21)}
    for s in (19, 20):
        out[s]["v1_end"] = 0.0  # two worlds rid of every disease
    for s in (17, 18, 19, 20):
        out[s]["v2_late"] = 0.01  # four where more diseases leave less
    return out


class ContagionVerdictTest(unittest.TestCase):
    claims = episode.load_module("contagion", "claims")

    def verdicts(self, r=None):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows())}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_slow_learning_breaks_the_few_dozen_ticks_claim(self):
        self.assertFalse(self.verdicts(rows(v1_well_by=80))["within a few dozen ticks at most, nearly all are well"])

    def test_a_world_rid_of_disease_everywhere_breaks_the_lingering_count(self):
        claim = "the book says society rids itself of every disease; here a little always lingers, in 18 of 20 worlds"
        self.assertFalse(self.verdicts(rows(v1_end=0.0))[claim])

    def test_a_spreading_outbreak_breaks_the_fizzle_claim(self):
        claim = "the book expects a plague; here it fizzles: about one Flump in twenty catches it"
        self.assertFalse(self.verdicts(rows(reach=0.6))[claim])

    def test_a_harmless_long_disease_breaks_the_plague_claim(self):
        claim = "only a disease four times longer than any in the book sweeps through, and costs a fifth of the Flumps"
        self.assertFalse(self.verdicts(rows(plague_ratio=1.0))[claim])


if __name__ == "__main__":
    unittest.main()
