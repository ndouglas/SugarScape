import unittest

import episode

HOLDS = dict(vi2_late=837.0, vi3_late=821.0, vi2_dip=174.0, vi3_dip=123.0, vi2_min_late=725.0, vi3_max=914.0,
             vi3_period=50, ii6_travelers=0.02, iii12_mixed=0.0, iii14_pop=10.0, trades=31330.0,
             foresight_start=4.98, foresight_end=4.41)


def rows(**changes):
    out = {s: dict(HOLDS, **changes) for s in range(1, 21)}
    out[20]["vi3_max"] = 1005.0  # one world that doubles
    for s in range(13, 21):
        out[s]["foresight_end"] = 5.2  # eight where foresight rises
    return out


class FinaleVerdictTest(unittest.TestCase):
    claims = episode.load_module("finale", "claims")

    def verdicts(self, r=None):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows())}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_crash_breaks_the_no_crash_count(self):
        r = rows()
        r[3]["vi2_min_late"] = 100.0
        self.assertFalse(self.verdicts(r)["no crash in 20 of 20 worlds; no doubling in 19; no 115-year cycles"])

    def test_the_books_cycles_break_the_no_cycles_claim(self):
        self.assertFalse(self.verdicts(rows(vi3_period=115))["no crash in 20 of 20 worlds; no doubling in 19; no 115-year cycles"])

    def test_travelling_waves_break_the_stay_put_claim(self):
        self.assertFalse(self.verdicts(rows(ii6_travelers=0.5))["waves that sweep across the land (II-6): here the Flumps stay put"])

    def test_trade_making_a_difference_breaks_the_same_either_way_claim(self):
        self.assertFalse(self.verdicts(rows(vi3_late=1100.0))["here both dip, recover and settle near 800, with trade or without"])


if __name__ == "__main__":
    unittest.main()
