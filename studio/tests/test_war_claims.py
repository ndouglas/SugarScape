import unittest

import episode


def rows(conquests=14, skirmishes=5, **changes):
    out = {}
    for s in range(1, 21):
        if s <= conquests:
            r = dict(kills=120, first=130, burst=600, top_share=0.94, unstoppable=True, conquest=True,
                     tribes_left=1, richest=110000.0, richest_peace=4400.0, blue_home=0.0, red_home=1.0)
        elif s <= conquests + skirmishes:
            r = dict(kills=15, first=130, burst=200, top_share=0.1, unstoppable=False, conquest=False,
                     tribes_left=2, richest=6000.0, richest_peace=4400.0, blue_home=1.0, red_home=1.0)
        else:
            r = dict(kills=87, first=200, burst=1290, top_share=0.86, unstoppable=True, conquest=False,
                     tribes_left=2, richest=160000.0, richest_peace=4400.0, blue_home=0.3, red_home=1.0)
        out[s] = dict(r, **changes)
    return out


FIXED = {s: {"quarters": {"NW": .25, "NE": .25, "SW": .25, "SE": .25}, "smallest": .25, "young": .68, "age": 2}
         for s in range(1, 21)}


class WarVerdictTest(unittest.TestCase):
    claims = episode.load_module("war", "claims")

    def verdicts(self, r=None, fixed=FIXED):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), fixed)}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_war_from_the_first_ticks_breaks_the_quiet_claim(self):
        self.assertFalse(self.verdicts(rows(burst=50))["for hundreds of ticks, almost nothing happens"])

    def test_a_catchable_warlord_breaks_the_unstoppable_claim(self):
        r = rows()
        r[3]["unstoppable"] = False
        self.assertFalse(self.verdicts(r)["then one Flump gets rich enough that no one can stop it"])

    def test_the_caption_must_name_the_measured_count(self):
        self.assertFalse(self.verdicts(rows(conquests=12, skirmishes=7))["in 14 of 20 worlds, one tribe is wiped out, or nearly"])

    def test_shared_killing_breaks_the_single_flump_claim(self):
        self.assertFalse(self.verdicts(rows(top_share=0.5))["nearly all the killing is done by a single Flump"])

    def test_a_front_breaks_the_no_front_claim(self):
        front = {s: dict(v, quarters={"NW": .05, "NE": .45, "SW": .45, "SE": .05}) for s, v in FIXED.items()}
        claim = "under its own rules there isn't a front: the killing is everywhere, and most of the dead are newcomers"
        self.assertFalse(self.verdicts(fixed=front)[claim])


if __name__ == "__main__":
    unittest.main()
