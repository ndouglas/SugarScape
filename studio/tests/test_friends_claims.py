import unittest

import episode

CONFIG = dict(agents=256, partners=4, moves=4, judge_error=0.1, mutation=0.1)
PAY = {"rwr": 1.084, "2dk": 2.555, "frn": 2.476, "ffr1": 2.406, "ffr3": 2.116, "ffr5": 1.340}
REMAIN = {"rwr": 0.001, "2dk": 0.999, "frn": 0.936, "ffr1": 0.843, "ffr3": 0.371, "ffr5": 0.061}


def rows(pay=None, remain=None, reached=20):
    pay, remain = dict(PAY, **(pay or {})), dict(REMAIN, **(remain or {}))
    out = {}
    for s in range(1, 21):
        row = {f"pay_{k}": v for k, v in pay.items()}
        row.update({f"remain_{k}": v for k, v in remain.items()})
        row.update({f"reached_{k}": float(s <= reached) for k in PAY})
        row["flicker"] = float(s <= 11)
        out[s] = row
    return out


class FriendsVerdictTest(unittest.TestCase):
    claims = episode.load_module("friends", "claims")

    def verdicts(self, r=None, **config):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), dict(CONFIG, **config))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_cooperating_strangers_break_almost_never(self):
        v = self.verdicts(rows(pay={"rwr": 1.6}))
        self.assertFalse(v["meet strangers every period, and cooperation almost never takes hold"])

    def test_a_torus_that_never_gets_there_breaks_every_world(self):
        v = self.verdicts(rows(reached=19))
        self.assertFalse(v["keep the same four neighbors, and it takes hold in every world, and stays"])

    def test_a_far_worse_fixed_network_breaks_nearly_as_good(self):
        v = self.verdicts(rows(pay={"frn": 2.3}))
        self.assertFalse(v["keep the same partners, scattered anywhere: nearly as good; no geography needed"])

    def test_the_dial_is_checked_at_each_setting(self):
        self.assertFalse(self.verdicts(rows(remain={"ffr1": 0.6}))["swap a tenth of them each period, and it mostly holds"])
        self.assertFalse(self.verdicts(rows(remain={"ffr3": 0.6}))["swap 30%, and it holds only about a third of the time"])
        self.assertFalse(self.verdicts(rows(pay={"ffr5": 1.9}))["half, and it collapses"])

    def test_a_row_off_the_table_breaks_row_by_row(self):
        v = self.verdicts(rows(pay={"ffr3": 1.95}))
        self.assertFalse(v["that's what Cohen, Riolo and Axelrod found in 2001, row by row"])

    def test_other_rules_break_the_rule_claims(self):
        self.assertFalse(self.verdicts(partners=8)["each period, each plays four rounds with four partners"])
        self.assertFalse(self.verdicts(judge_error=0.0)["then each copies its best-scoring partner, if it did better"])


if __name__ == "__main__":
    unittest.main()
