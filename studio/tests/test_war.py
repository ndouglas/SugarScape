import pathlib
import unittest
from types import SimpleNamespace

import dump
import war

WAR = pathlib.Path(__file__).parent / "fixtures" / "war.frames.json"


def frame(agents, groups, kills=()):
    """`agents`: {id: (x, y, sugar)}; `groups`: {id: tribe}."""
    a = {i: dump.Agent(i, x, y, s, 3, 1, 1) for i, (x, y, s) in agents.items()}
    return dump.Frame(0, a, [], {}, [], [], {}, groups=groups, kills=list(kills))


def run(*frames):
    return SimpleNamespace(frames=list(frames))


class WarTest(unittest.TestCase):
    def test_the_war_starts_when_a_tenth_of_its_kills_have_happened(self):
        k = dump.Kill(1, 2, 5.0)
        d = run(frame({}, {}), frame({}, {}, [k]), frame({}, {}), frame({}, {}, [k] * 9))
        self.assertEqual(war.burst_start(d), 1)
        self.assertEqual(war.burst_start(d, 0.5), 3)
        self.assertIsNone(war.burst_start(run(frame({}, {}))))

    def test_the_top_killers_share_and_its_nth_kill(self):
        d = run(frame({}, {}, [dump.Kill(1, 5, 1.0)]), frame({}, {}, [dump.Kill(1, 6, 1.0), dump.Kill(2, 7, 1.0)]),
                frame({}, {}, [dump.Kill(1, 8, 1.0)]))
        self.assertEqual(war.killers(d)[0], (1, 3))
        self.assertAlmostEqual(war.top_killer_share(d), 0.75)
        self.assertEqual(war.nth_kill_tick(d, 1, 2), 1)
        self.assertIsNone(war.nth_kill_tick(d, 2, 2))

    def test_unstoppable_means_no_richer_enemy_until_it_dies(self):
        groups = {1: 0, 2: 1, 3: 0}
        safe = frame({1: (0, 0, 50), 2: (1, 0, 10), 3: (2, 0, 90)}, groups)  # 3 is richer, but a friend
        threatened = frame({1: (0, 0, 50), 2: (1, 0, 60)}, groups)
        self.assertTrue(war.unstoppable_from(run(safe, safe), 1, 0))
        self.assertFalse(war.unstoppable_from(run(safe, threatened), 1, 0))
        self.assertTrue(war.unstoppable_from(run(safe, frame({2: (1, 0, 60)}, groups)), 1, 0))  # dead: stays true

    def test_quarter_shares_place_each_victim_where_it_stood(self):
        before = frame({2: (1, 1, 5), 3: (9, 9, 5)}, {2: 1, 3: 1})
        after = frame({}, {}, [dump.Kill(1, 2, 5.0), dump.Kill(1, 3, 5.0)])
        self.assertEqual(war.quarter_shares(run(before, after), 10, 10), {"NW": 0.5, "NE": 0.0, "SW": 0.0, "SE": 0.5})
        self.assertEqual(war.victim_ages(run(before, after)), [3, 3])

    def test_on_the_war_fixture_the_rich_blue_does_all_the_killing(self):
        d = dump.load(WAR)
        self.assertEqual(war.killers(d)[0][0], d.placed[0])
        self.assertEqual(war.top_killer_share(d), 1.0)


if __name__ == "__main__":
    unittest.main()
