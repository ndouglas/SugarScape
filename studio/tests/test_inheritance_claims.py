import unittest

import episode


def rows(**values):
    return {s: dict(values) for s in range(1, 21)}


HOLDS = dict(
    pop=1364, pop_plain=372, gini=0.345, gini_plain=0.160, top10=0.306, top10_plain=0.161,
    persist=0.219, persist_plain=0.042, rich_stay=0.372, rich_stay_plain=0.277,
    poor_rise=0.154, poor_rise_plain=0.230, max_gen=59,
)


class InheritanceVerdictTest(unittest.TestCase):
    claims = episode.load_module("inheritance", "claims")

    def verdicts(self, **changes):
        return {claim: holds for claim, holds, _ in self.claims.verdicts(rows(**dict(HOLDS, **changes)))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_modest_gain_is_not_nearly_four_times(self):
        self.assertFalse(self.verdicts(pop=600)["kept in the family, sugar feeds nearly four times as many"])

    def test_an_equal_split_breaks_the_inequality_claim(self):
        self.assertFalse(self.verdicts(gini=0.17, top10=0.17)["it is shared far less evenly"])

    def test_no_persistence_breaks_the_fortunes_claim(self):
        self.assertFalse(self.verdicts(persist=0.04, rich_stay=0.28, poor_rise=0.23)["fortunes last"])

    def test_few_generations_break_the_sixty_generations_caption(self):
        self.assertFalse(self.verdicts(max_gen=20)["a thousand ticks, about sixty generations"])


if __name__ == "__main__":
    unittest.main()
