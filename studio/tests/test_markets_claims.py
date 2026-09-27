import unittest

import episode

HOLDS = dict(shuttle=0.49, shuttle_trade=0.66, alive=120.0, exchanges1=340, right=95, trades=100,
             ln_price_late=0.006, sd_early=0.45, sd_late=0.026, gini_trade=0.365, gini=0.32)
SWEEP = (118, 120)


def rows(**changes):
    return {s: dict(HOLDS, **changes) for s in range(1, 21)}


class MarketsVerdictTest(unittest.TestCase):
    claims = episode.load_module("markets", "claims")

    def verdicts(self, sweep=SWEEP, **changes):
        return {c: holds for c, holds, _ in self.claims.verdicts(rows(**changes), sweep)}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_most_flumps_shuttling_breaks_the_about_half_claim(self):
        v = self.verdicts(shuttle=0.8, shuttle_trade=0.9)
        self.assertFalse(v["without trade, about half the Flumps walk back and forth between the hills"])

    def test_trades_the_wrong_way_break_the_swap_claim(self):
        self.assertFalse(self.verdicts(right=60)["neighbors swap what they have too much of for what they lack"])

    def test_a_drifting_price_breaks_the_near_one_claim(self):
        self.assertFalse(self.verdicts(ln_price_late=0.2)["nobody sets the price, yet it settles near one sugar per spice"])

    def test_the_caption_must_name_the_measured_count(self):
        self.assertFalse(self.verdicts(sweep=(120, 120))["trade feeds more Flumps: in 118 of 120 worlds"])

    def test_one_world_where_trade_is_fairer_breaks_all_20(self):
        r = rows()
        r[7]["gini_trade"] = 0.30
        v = {c: holds for c, holds, _ in self.claims.verdicts(r, SWEEP)}
        self.assertFalse(v["and it makes them less equal, in all 20 worlds"])


if __name__ == "__main__":
    unittest.main()
