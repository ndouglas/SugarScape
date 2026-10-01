import pathlib
import unittest

import dump

# Made by `sugarscape shot tests/fixtures/schelling.json` (s71-board, seed 2,
# 3 rounds), `… line.json` (s71-line, seed 3, 3 rounds) and
# `… schelling-every.json` (s71-board, seed 2, 8 rounds filmed every 4th).
HERE = pathlib.Path(__file__).parent / "fixtures"


class SchellingDumpTest(unittest.TestCase):
    def setUp(self):
        self.board = dump.load(HERE / "schelling.frames.json")
        self.line = dump.load(HERE / "line.frames.json")

    def test_a_schelling_dump_loads_as_a_board_of_moving_agents(self):
        d = self.board
        self.assertIsInstance(d, dump.Dump)
        self.assertEqual((d.model, d.width, d.height, d.ticks), ("schelling", 16, 13, 3))
        self.assertEqual(len(d.frames[0].agents), 138)
        self.assertEqual(sorted(d.placed), sorted(d.frames[0].agents))

    def test_red_is_group_one_and_content_is_its_sugar(self):
        f = self.board.frames[0]
        self.assertEqual(sum(f.groups.values()), 69, "Schelling's 69 stars are Red")
        unhappy = sum(1 for a in f.agents.values() if a.sugar == 0.0)
        self.assertEqual(unhappy / 138, self.board.stats["unsatisfied"][0])

    def test_nobody_is_born_or_dies_and_moves_match_the_stats(self):
        d = self.board
        for k in range(1, d.ticks + 1):
            a, b = d.frames[k - 1].agents, d.frames[k].agents
            self.assertEqual(set(a), set(b))
            self.assertEqual(d.frames[k].born, [])
            moved = sum(1 for i in a if (a[i].x, a[i].y) != (b[i].x, b[i].y))
            self.assertEqual(moved, d.stats["moves"][k])

    def test_the_line_is_a_row_of_squares_with_no_gaps(self):
        d = self.line
        self.assertEqual((d.model, d.width, d.height), ("line", 70, 1))
        for f in d.frames:
            self.assertEqual(sorted(a.x for a in f.agents.values()), list(range(70)))

    def test_a_strided_dump_counts_its_frames_as_ticks(self):
        d = dump.load(HERE / "schelling-every.frames.json")
        self.assertEqual(d.ticks, 2)
        self.assertEqual([f.tick for f in d.frames], [0, 1, 2])
        self.assertEqual(len(d.stats["segregation"]), 3)
