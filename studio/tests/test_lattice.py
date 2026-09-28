import pathlib
import unittest

import animate
import dump
import lattice

# Made by `sugarscape shot tests/fixtures/lattice.json`: a hand-made 7 × 7
# board, b = 1.9, fixed edges, with scores.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "lattice.frames.json"


def timing(tps):
    return animate.Timing(tps, 0, 3, 0.0, 30)


class LatticeDumpTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_a_spatial_dump_loads_as_a_lattice(self):
        self.assertIsInstance(self.d, dump.Lattice)
        self.assertEqual((self.d.width, self.d.height, self.d.ticks, len(self.d.frames)), (7, 7, 3, 4))
        self.assertEqual(self.d.frames[0].strategies[:7], "CCCCDDD")
        self.assertEqual(len(self.d.frames[0].scores), 49)
        self.assertEqual(self.d.frame(2.6).tick, 3)

    def test_scores_follow_the_rule(self):
        # A cheat earns b = 1.9 from each helper beside it; a helper earns 1
        # from each helper beside it, and itself.
        f, w = self.d.frames[0], 7
        for i, c in enumerate(f.strategies):
            x, y = i % w, i // w
            helpers = sum(f.strategies[ny * w + nx] == "C" for dx, dy in lattice.NEIGHBORS
                          for nx, ny in [(x + dx, y + dy)] if 0 <= nx < w and 0 <= ny < w)
            self.assertAlmostEqual(f.scores[i], helpers + 1 if c == "C" else 1.9 * helpers)


class SquaresTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)
        self.switch = next(i for i, (a, b) in enumerate(zip(self.d.frames[0].strategies, self.d.frames[1].strategies))
                           if a == "C" and b == "D")

    def at(self, tps, frame):
        return lattice.squares(self.d, lattice.moment(timing(tps), frame, self.d.ticks))

    def test_a_switch_hops_and_changes_side_at_the_top(self):
        # One tick a second: 30 frames a tick; the hop takes the last 0.6 s.
        before, rising, after = self.at(1, 1)[self.switch], self.at(1, 1 + 30 * 0.6)[self.switch], self.at(1, 1 + 30 * 0.9)[self.switch]
        self.assertTrue(before.helper)
        self.assertEqual(before.height, 0.0)
        self.assertTrue(rising.helper)
        self.assertGreater(rising.height, 0.0)
        self.assertFalse(after.helper)
        self.assertEqual(after.glow, "D")
        self.assertGreater(after.strength, 0.5)

    def test_the_glow_fades_over_the_next_tick(self):
        early, late = self.at(1, 1 + 30 * 1.0)[self.switch], self.at(1, 1 + 30 * 1.35)[self.switch]
        self.assertEqual((early.glow, late.glow), ("D", "D"))
        self.assertGreater(early.strength, late.strength)

    def test_squares_that_keep_their_side_neither_hop_nor_glow(self):
        keep = next(i for i, (a, b) in enumerate(zip(self.d.frames[0].strategies, self.d.frames[1].strategies)) if a == b)
        s = self.at(1, 1 + 30 * 0.7)[keep]
        self.assertEqual((s.height, s.glow), (0.0, None))

    def test_a_tick_a_frame_does_not_hop(self):
        self.assertTrue(all(s.height == 0.0 for s in self.at(30, 2)))

    def test_the_last_generation_holds(self):
        s = self.at(1, 1 + 30 * 5)
        self.assertEqual("".join("C" if q.helper else "D" for q in s), self.d.frames[3].strategies)


class MeasuresTest(unittest.TestCase):
    def test_symmetry_needs_quarter_turns_and_mirrors(self):
        self.assertTrue(lattice.symmetric("CCCCDCCCC", 3, 3))
        self.assertFalse(lattice.symmetric("DCCCCCCCC", 3, 3))
        self.assertFalse(lattice.symmetric("CDCCCCCCC", 3, 3))

    def test_edges(self):
        self.assertFalse(lattice.reaches_edge("CCCCDCCCC", 3, 3))
        self.assertTrue(lattice.reaches_edge("CCCDCCCCC", 3, 3))

    def test_boundary_counts_each_helper_cheat_pair(self):
        f = dump.LatticeFrame(0, "CCCCDCCCC", [3, 5, 3, 5, 15.2, 5, 3, 5, 3])
        self.assertEqual(lattice.boundary(f, 3, 3), (8, 1.0))
        f = dump.LatticeFrame(0, "CD", [0.0, 2.0])
        self.assertEqual(lattice.boundary(f, 2, 1), (1, 1.0))

    def test_positions_center_the_board(self):
        self.assertEqual(lattice.position(0, 3, 3), (-1.0, 1.0))
        self.assertEqual(lattice.position(4, 3, 3), (0.0, 0.0))


if __name__ == "__main__":
    unittest.main()
