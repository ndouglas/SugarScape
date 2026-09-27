import unittest

import episode

HOLDS = dict(lender_age=60, borrower_age=37, past=0.66, both=25, both_share=0.12, depth=5, max_depth=10,
             births=5335.0, births_off=4465.0, pop=450.0, pop_off=378.0, gini=0.18, gini_off=0.16)


def rows(**changes):
    out = {s: dict(HOLDS, **changes) for s in range(1, 21)}
    for s in (19, 20):  # two worlds where credit is not less equal
        out[s]["gini"] = 0.15
    return out


class CreditVerdictTest(unittest.TestCase):
    claims = episode.load_module("credit", "claims")

    def verdicts(self, r=None):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows())}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_young_lenders_break_the_old_to_young_claim(self):
        self.assertFalse(self.verdicts(rows(lender_age=30))["loans flow from old to young: lenders about 60, borrowers about 37"])

    def test_a_shallow_network_breaks_the_ten_levels_claim(self):
        self.assertFalse(self.verdicts(rows(max_depth=5))["the book saw five levels; ours reach about ten"])

    def test_no_extra_births_breaks_the_fifth_more_claim(self):
        self.assertFalse(self.verdicts(rows(births=4500.0))["with credit, about a fifth more Flumps are born, in all 20 worlds"])

    def test_the_caption_must_name_the_measured_count(self):
        r = rows()
        r[18]["gini"] = 0.15
        self.assertFalse(self.verdicts(r)["and they're a little less equal, in 18 of 20"])


if __name__ == "__main__":
    unittest.main()
