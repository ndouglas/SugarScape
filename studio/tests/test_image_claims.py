import unittest

import episode

CONFIG = dict(clamp=5, strategies=["k"], information="perfect", c=0.1, b=1.0)
WINNERS = ["k=0"] * 5 + ["k=-3"] * 2 + ["k=6"] * 3 + ["k=3"] * 10


def rows(**changes):
    out = {}
    for s, fixed in enumerate(WINNERS, start=1):
        k = int(fixed.split("=")[1])
        row = dict(fixed=fixed, fixed_k=k, distinct=1, help_end=1.0 if k <= 0 else 0.0, collapses=3, recoveries=3,
                   fig2_cooperative=0.73, n20=0.89, n50=0.41, n100=0.16)
        out[s] = {**row, **changes}
    return out


class ImageVerdictTest(unittest.TestCase):
    claims = episode.load_module("image", "claims")

    def verdicts(self, r=None, replayed=True):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), CONFIG, replayed)}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_scores_that_dont_replay_break_the_scoring_claim(self):
        self.assertFalse(self.verdicts(replayed=False)["helping raises your score; refusing lowers it"])

    def test_a_world_still_mixed_breaks_the_counts(self):
        r = rows()
        r[9]["distinct"] = 2
        self.assertFalse(self.verdicts(r)["here, everyone ends up helping in 7 of 20 worlds; in 13, nobody helps anyone"])

    def test_the_caption_must_name_the_measured_counts(self):
        r = rows()
        r[20].update(fixed="k=0", fixed_k=0, help_end=1.0)
        self.assertFalse(self.verdicts(r)["here, everyone ends up helping in 7 of 20 worlds; in 13, nobody helps anyone"])

    def test_one_world_without_cycles_breaks_again_and_again(self):
        r = rows()
        r[4]["collapses"] = 1
        self.assertFalse(self.verdicts(r)["let rules mutate, and helping rises and collapses, again and again"])

    def test_bigger_groups_helping_more_breaks_the_ordering(self):
        self.assertFalse(self.verdicts(rows(n50=0.95))["it works best in small groups: the bigger the group, the less "
                                                       "anyone helps"])


if __name__ == "__main__":
    unittest.main()
