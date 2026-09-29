import unittest

import episode

CONFIG = dict(temptation=3.0, hurt=-1.0, punishment=-9.0, enforcement=-2.0, metanorms=True, meta_punishment=-9.0,
              meta_enforcement=-2.0, all_equal="drift")


def rows(held=17, long_col=18, his_est=14, norms_col=20, mild=20):
    out = {}
    for s in range(1, 21):
        i = s - 1
        out[s] = dict(norms_collapsed=float(i < norms_col), meta_established=float(i < held),
                      long_established=float(i >= long_col), long_collapsed=float(i < long_col),
                      his_established=float(i < his_est), his_collapsed=float(i >= his_est),
                      mild_collapsed=float(i < mild), mild_his_collapsed=float(i < mild))
    return out


class NormsVerdictTest(unittest.TestCase):
    claims = episode.load_module("norms", "claims")

    def verdicts(self, r=None, **config):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), dict(CONFIG, **config))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_surviving_norm_breaks_the_cheats_take_over(self):
        v = self.verdicts(rows(norms_col=19))
        self.assertFalse(v["Axelrod, 1986: on their own, punishments fade; the cheats take over, in 20 of 20 worlds"])

    def test_the_captions_report_the_measured_counts(self):
        v = self.verdicts(rows(held=16, long_col=17, his_est=13))
        self.assertIn("the norm holds, in 16 of 20 worlds, as he found", v)
        self.assertIn("in 2005, Galán and Izquierdo ran it a million generations; the norm collapsed, in 17 of 20", v)
        self.assertIn("read his way, the norm still holds after a million generations, in 13 of 20", v)

    def test_his_reading_losing_the_norm_breaks_still_holds(self):
        v = self.verdicts(rows(his_est=8))
        self.assertFalse(v["read his way, the norm still holds after a million generations, in 8 of 20"])

    def test_a_mild_world_that_holds_breaks_either_way(self):
        self.assertFalse(self.verdicts(rows(mild=19))["either way, make metapunishment milder, and the norm collapses"])

    def test_other_payoffs_break_the_payoff_claims(self):
        v = self.verdicts(punishment=-5.0)
        self.assertFalse(v["anyone who sees it may punish it, if vengeful enough: −9 to the cheat, −2 to the punisher"])


if __name__ == "__main__":
    unittest.main()
