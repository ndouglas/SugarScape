import unittest

import episode

CONFIG = {"board": (16, 13, 138), "line": 70, "radius": 4, "line_preference": 0.5, "preference": 0.5}
ROW = dict(groups=7.0, mean_group=10.0, line_like=0.78, like=0.796, none=0.384, ratio=3.62, third_ratio=1.35,
           third_like=0.6, company_like=0.794, mixed_moves=96.0, mixed_left=0.087, same_moves=46.0)


def rows(**changes):
    return {s: dict(ROW, **changes) for s in range(1, 21)}


class NeighborsVerdictTest(unittest.TestCase):
    claims = episode.load_module("neighbors", "claims")

    def verdicts(self, r=None, **config):
        return {c: h for c, h, _ in self.claims.verdicts(r or rows(), dict(CONFIG, **config))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_bigger_clusters_break_about_seven_of_ten(self):
        v = self.verdicts(rows(groups=5.0, mean_group=14.0))
        self.assertFalse(v["a few rounds later: about seven clusters of ten; nobody asked for more than half"])

    def test_a_board_as_sorted_as_his_breaks_a_little_more_sorted(self):
        v = self.verdicts(rows(like=0.9, none=0.66))
        self.assertFalse(v["Schelling worked his boards by hand, and said they were too few to generalize; his came "
                           "out a little more sorted"])

    def test_a_third_that_sorts_breaks_slight(self):
        self.assertFalse(self.verdicts(rows(third_ratio=2.0))["ask for only a third, and the sorting is slight"])

    def test_integrationists_who_all_settle_break_never_satisfied(self):
        v = self.verdicts(rows(mixed_left=0.0))
        self.assertFalse(v["even Flumps who want a mixed street move more, and some are never satisfied"])

    def test_another_board_breaks_the_setup(self):
        self.assertFalse(self.verdicts(board=(50, 50, 2000))["then a checkerboard: 138 Flumps and 70 empty squares"])


if __name__ == "__main__":
    unittest.main()
