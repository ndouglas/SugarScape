import unittest

import episode

ROW = dict(own=0.767, everyone=0.129, none=0.085, others=0.023, cooperation=0.756, kin_help=0.864, relatives=0.752,
           population=1565.0, scattered_none=0.883, scattered_cooperation=0.049, scattered_kin_help=0.127, blind=0.416)
CONFIG = dict(colors=4, mutation=0.005, discrimination="same_other", allowed=["E", "H", "S", "T"], cost=0.01,
              benefit=0.03, base_ptr=0.12, start="empty", immigration=1.0, offspring="adjacent")


def rows(**changes):
    return {s: dict(ROW, **changes) for s in range(1, 21)}


class EthnoVerdictTest(unittest.TestCase):
    claims = episode.load_module("ethno", "claims")

    def verdicts(self, r=None, **config):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), dict(CONFIG, **config))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_benefit_other_than_three_times_the_cost_breaks_the_cost_claim(self):
        v = self.verdicts(benefit=0.02)
        self.assertFalse(v["helping costs a little of your chance to have a child, and gives the neighbor three times "
                           "as much"])

    def test_one_world_where_another_kind_leads_breaks_all_20(self):
        r = rows()
        r[6]["none"] = 0.8
        self.assertFalse(self.verdicts(r)["favoritism wins: about 3 in 4 Flumps help only their own color, in all 20 "
                                          "worlds"])

    def test_one_world_with_less_kin_help_breaks_over_8_in_10(self):
        r = rows()
        r[2]["kin_help"] = 0.7
        self.assertFalse(self.verdicts(r)["why? their neighbors are family: over 8 in 10 of all helps go to relatives"])

    def test_a_scattered_world_that_keeps_helping_breaks_the_collapse(self):
        r = rows()
        r[11]["scattered_none"] = 0.5
        self.assertFalse(self.verdicts(r)["put each child anywhere, and favoritism collapses: nine in ten Flumps help "
                                          "no one"])

    def test_the_caption_must_name_the_measured_share(self):
        self.assertFalse(self.verdicts(rows(blind=0.3))["the paper says color-blind Flumps, paying double to help, help "
                                                        "14% of the time; here, 42%"])


if __name__ == "__main__":
    unittest.main()
