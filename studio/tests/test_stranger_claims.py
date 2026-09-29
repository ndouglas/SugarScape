import unittest

import episode

SOURCES = dict(third_lo=0.3235, third_hi=0.3254, hg_lost=20.0, nbm_async=0.72, nbm_sync=0.884, own=0.767,
               kin_help=0.864, seeing2=0.666, seeing2_lo=0.591, gifts=0.737, twin_gifts=0.943, n20=0.889, n50=0.411,
               n100=0.164, everyone=7.0, meta=17.0, long_est=2.0, his_est=14.0, published=733.0, working_only=760.0,
               working=784.0, frn=2.476, frn_remain=0.936, rwr=1.084, rwr_remain=0.001, cra_worst=0.083, cra_order=True)


def run4(book=19, working=18, closest=18):
    return {k: {"extinct": n, "cooperators": 0, "defectors": 0, "both": 20 - n}
            for k, n in (("book", book), ("working", working), ("closest", closest))}


class StrangerVerdictTest(unittest.TestCase):
    claims = episode.load_module("stranger", "claims")

    def verdicts(self, counts=None, **changes):
        return {c: h for c, h, _ in self.claims.verdicts(dict(SOURCES, **changes), counts or run4())}

    def test_the_measured_series_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_row_off_the_paper_breaks_most_of_it_reproduces(self):
        self.assertFalse(self.verdicts(gifts=0.70)["most of it reproduces, closely"])
        self.assertFalse(self.verdicts(cra_order=False)["most of it reproduces, closely"])

    def test_run4_surviving_under_one_reading_breaks_didnt_come_back(self):
        self.assertFalse(self.verdicts(run4(working=12))["and two didn't come back, as written"])

    def test_a_seed_below_the_papers_56_breaks_didnt_come_back(self):
        self.assertFalse(self.verdicts(seeing2_lo=0.55)["and two didn't come back, as written"])

    def test_readings_that_agree_break_the_detail_claim(self):
        v = self.verdicts(his_est=4.0)
        self.assertFalse(v["two results depend on a detail the papers give two ways"])
        self.assertFalse(v["a rule everyone enforces: the norm holds; how long depends on one sentence"])

    def test_the_montage_needs_every_answer(self):
        v = self.verdicts(kin_help=0.4)
        self.assertFalse(v["relatives: most help goes to kin next door"])
        self.assertFalse(v["why help a stranger? The Flumps do, when something makes the stranger less strange"])

    def test_a_missing_source_claim_fails_loudly(self):
        with self.assertRaises(ValueError):
            self.claims.held("friends", "no such claim")


if __name__ == "__main__":
    unittest.main()
