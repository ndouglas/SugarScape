import unittest

import episode

ROW = dict(third_2=0.325, third_10=0.325, third_25=0.325, ahead=1.0, frontier=0.49, tempt_10=0.75, tempt_25=0.84,
           tempt_40=0.88, all_d_at=97, below_one_min=0.99,
           below_async=0.72, below_sync=0.88, kaleidoscope=(1.0, 0.99))
FACTS = dict(scores_follow=True, copies_follow=True, b=1.9, board_frames=4, start=(1, 9800), symmetric=True,
             symmetric_to=260, edge=49, same_every_seed=True, min_c=0.18, still=0, changed=0.22,
             kaleidoscope_update="synchronous", clock_update="asynchronous")


def rows(**changes):
    return {s: dict(ROW, **changes) for s in range(1, 21)}


class SpatialVerdictTest(unittest.TestCase):
    claims = episode.load_module("spatial", "claims")

    def verdicts(self, r=None, **facts):
        return {c: holds for c, holds, _ in self.claims.verdicts(r or rows(), dict(FACTS, **facts))}

    def test_the_measured_world_supports_every_caption(self):
        self.assertTrue(all(self.verdicts().values()), self.verdicts())

    def test_a_score_off_the_rule_breaks_the_rules_claim(self):
        v = self.verdicts(scores_follow=False)
        self.assertFalse(v["every Flump plays each of its neighbors; two helpers earn 1 each, a cheat facing a helper "
                           "almost 2, and the helper nothing"])

    def test_one_settled_generation_breaks_never_settles(self):
        self.assertFalse(self.verdicts(still=1)["cheating spreads, but never wins, and never settles"])

    def test_a_broken_symmetry_breaks_the_kaleidoscope(self):
        self.assertFalse(self.verdicts(symmetric=False)["cheating spreads, but never wins, and never settles"])

    def test_one_run_far_from_a_third_breaks_about_a_third(self):
        r = rows()
        r[4]["third_25"] = 0.0
        self.assertFalse(self.verdicts(r)["scatter a few cheats anywhere: about a third of the Flumps end up helping"])

    def test_one_generation_where_cheats_earn_more_breaks_every_generation(self):
        r = rows()
        r[9]["ahead"] = 0.0
        self.assertFalse(self.verdicts(r)["on average, helpers earn more than cheats, every generation"])

    def test_a_surviving_helper_breaks_20_of_20(self):
        r = rows()
        r[3]["all_d_at"] = None
        self.assertFalse(self.verdicts(r)["one cheat takes the whole world, in 20 of 20"])

    def test_helpers_losing_below_the_chaos_break_either_way(self):
        r = rows()
        r[5]["below_async"] = 0.3
        claim = "Nowak, Bonhoeffer and May replied in 1994: make cheating less tempting, and helpers survive either way"
        self.assertFalse(self.verdicts(r)[claim])


if __name__ == "__main__":
    unittest.main()
