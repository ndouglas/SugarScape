import unittest

import episode


def rows(**values):
    return {s: dict(values) for s in range(1, 21)}


HOLDS = dict(
    pop=160, pop_calm=228, met_high_alive=0.0, met_high_alive_calm=0.25, met_low_alive=0.95,
    hill_alive=0.45, plain_alive=0.34, rich_alive=0.41, poor_alive=0.37,
    migrant_summer=0.65, met_alive=1.43, met_alive_calm=1.82,
)


class SeasonsVerdictTest(unittest.TestCase):
    claims = episode.load_module("seasons", "claims")

    def verdicts(self, **changes):
        return {claim: holds for claim, holds, _ in self.claims.verdicts(rows(**dict(HOLDS, **changes)))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_migrants_who_wander_at_random_do_not_follow_the_summer(self):
        self.assertFalse(self.verdicts(migrant_summer=0.5)["many Flumps follow the summer"])

    def test_a_hungry_survivor_breaks_the_winter_claim(self):
        self.assertFalse(self.verdicts(met_high_alive=0.1)["winter is hard on the hungry: none survive"])

    def test_a_small_loss_is_not_a_third(self):
        self.assertFalse(self.verdicts(pop=210)["seasons cost about a third of the Flumps"])

    def test_the_ranking_needs_need_to_matter_most(self):
        v = self.verdicts(met_high_alive=0.0, met_low_alive=0.05)
        self.assertFalse(v["starting rich helps least, a hill some, needing little most"])


if __name__ == "__main__":
    unittest.main()
