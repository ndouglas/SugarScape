import unittest

import episode


def rows(split_every=2, **values):
    """20 seeds; every `split_every`-th has the hills split, the rest one tribe."""
    out = {}
    for s in range(1, 21):
        split = s % split_every == 0
        out[s] = dict(values, one_tribe=not split, hills_differ=split, pop=225 + (s % 3) - 1, pop_calm=225)
    return out


HOLDS = dict(homog0=0.48, homog3000=1.0, ne_share=1.0, sw_share=1.0)


class TribesVerdictTest(unittest.TestCase):
    claims = episode.load_module("tribes", "claims")

    def verdicts(self, split_every=2, **changes):
        return {c: holds for c, holds, _ in self.claims.verdicts(rows(split_every, **dict(HOLDS, **changes)))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_neighbourhoods_that_stay_mixed_break_the_convergence_claim(self):
        self.assertFalse(self.verdicts(homog3000=0.6)["in most worlds, every neighbourhood ends up one color"])

    def test_mixed_hills_break_the_one_tribe_per_hill_claim(self):
        self.assertFalse(self.verdicts(ne_share=0.6)["each hill becomes one tribe"])

    def test_one_tribe_always_winning_breaks_the_half_the_time_claim(self):
        self.assertFalse(self.verdicts(split_every=100)["one tribe wins about half the time; otherwise the hills split"])

    def test_a_population_gap_breaks_the_no_advantage_claim(self):
        r = rows(**HOLDS)
        for v in r.values():
            v["pop"] = 260
        v = {c: holds for c, holds, _ in self.claims.verdicts(r)}
        self.assertFalse(v["being Red or Blue changes nothing else: just as many Flumps live either way"])


if __name__ == "__main__":
    unittest.main()
