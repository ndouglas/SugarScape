import unittest

import episode


def rows(**values):
    return {s: dict(values) for s in range(1, 21)}


HOLDS = dict(
    pop=177, pop_clean=228, pol_hill=173, pol_plain=116, pol_mean=140, pol_under=155,
    hill_share49=0.90, hill_share=0.61, hill_share_clean=0.92,
    met_hi_alive=0.14, met_hi_alive_clean=0.83, vis_hi_alive=0.71, vis_hi_alive_clean=1.0,
    vision=3.68, vision_clean=3.81, gini=0.335, gini_clean=0.385, pol300=79,
)


class PollutionVerdictTest(unittest.TestCase):
    claims = episode.load_module("pollution", "claims")

    def verdicts(self, **changes):
        return {claim: holds for claim, holds, _ in self.claims.verdicts(rows(**dict(HOLDS, **changes)))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_clean_hills_break_the_best_land_claim(self):
        self.assertFalse(self.verdicts(pol_hill=100)["the best land gets the dirtiest"])

    def test_staying_on_the_hills_breaks_the_flight_claim(self):
        self.assertFalse(self.verdicts(hill_share=0.88)["Flumps leave the hills"])

    def test_escaping_to_clean_ground_breaks_the_follows_claim(self):
        self.assertFalse(self.verdicts(pol_under=120)["the mess goes wherever they go"])

    def test_a_small_loss_is_not_a_quarter(self):
        self.assertFalse(self.verdicts(pop=215)["pollution costs about a quarter of the Flumps"])

    def test_sparing_the_far_sighted_breaks_the_who_dies_claim(self):
        self.assertFalse(self.verdicts(vis_hi_alive=1.0)["the hungry go first, then the far-sighted"])

    def test_less_equal_survivors_break_the_leveling_claim(self):
        self.assertFalse(self.verdicts(gini=0.40)["the survivors are more equal"])


if __name__ == "__main__":
    unittest.main()
