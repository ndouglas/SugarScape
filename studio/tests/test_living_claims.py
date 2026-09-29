import unittest

import episode

ROW = dict(helpers=729.0, cheats=171.0, population=900.0, full_at=34, working=784.0, working_only=758.0, soup_gone=8.0,
           soup_end=1.0,
           soup_end_helpers=0.0, low_end=0.0, low_extinct=243.0)
FACTS = dict(walks_and_plays=True, vision=1, play="each_neighbor", payoffs=(6.0, 5.0, -5.0, -6.0), fission=11.0,
             endowment=6.0, clones=True, broke=26, deaths=26, start=(100, 0.5), shown=True, low=(1.0, 100))


def rows(**changes):
    r = {s: dict(ROW, **changes) for s in range(1, 21)}
    r[20].update(soup_gone=None, soup_end_helpers=1.0, low_end=192.0, low_extinct=None)
    return r


class LivingVerdictTest(unittest.TestCase):
    claims = episode.load_module("living", "claims")

    def verdicts(self, r=None, **facts):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or self.rows_spread(), dict(FACTS, **facts))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_other_payoffs_break_the_payoff_claim(self):
        v = self.verdicts(payoffs=(6.0, 5.0, 0.0, -6.0))
        self.assertFalse(v["two helpers earn 5 each; a cheat takes 6 from a helper; two cheats lose 5 each"])

    def rows_spread(self, **changes):
        r = rows(**changes)
        for i, seed in enumerate(r):
            r[seed]["helpers"] += (i % 5 - 2) * 8
            r[seed]["working"] += (i % 5 - 2) * 8
        return r

    def test_the_published_rules_matching_epstein_breaks_the_book_claim(self):
        claim = ("Epstein counted 779 helpers; his published rules give about 730; his working paper's rule, one game a "
                 "turn, comes close if the first Flumps start with nothing")
        self.assertTrue(self.verdicts(self.rows_spread())[claim])
        self.assertFalse(self.verdicts(self.rows_spread(helpers=779.0))[claim])
        self.assertFalse(self.verdicts(self.rows_spread(working=740.0))[claim])

    def test_the_close_up_must_show_its_clone_and_its_broke_cheat(self):
        self.assertFalse(self.verdicts(shown=False)["reach 11, and a Flump splits in two; go broke, and it's gone"])

    def test_a_board_left_unfilled_breaks_the_land_fills(self):
        r = rows()
        r[5]["population"] = 850.0
        self.assertFalse(self.verdicts(r)["the land fills up, mostly with helpers: about 730 to 170"])

    def test_the_caption_must_name_the_measured_count(self):
        r = rows()
        r[20]["soup_gone"] = 9.0
        self.assertFalse(self.verdicts(r)["the last helper is gone within 12 cycles, in 19 of 20 worlds"])

    def test_two_cheats_left_breaks_one_or_none(self):
        r = rows()
        r[2]["soup_end"] = 2.0
        self.assertFalse(self.verdicts(r)["then the cheats ruin each other; by cycle 500, one Flump is left, or none"])


if __name__ == "__main__":
    unittest.main()
