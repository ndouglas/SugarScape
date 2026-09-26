import unittest

import dump
import tribes


def frame(cells):
    """`cells`: {(x, y): group}."""
    agents = {i: dump.Agent(i, x, y, 1, 0, 1, 1) for i, (x, y) in enumerate(cells)}
    return dump.Frame(0, agents, [], {}, [], [], {}, groups={i: g for i, g in enumerate(cells.values())})


class TribesTest(unittest.TestCase):
    def test_neighbors_alike_counts_adjacent_pairs_in_the_same_tribe(self):
        f = frame({(0, 0): 0, (1, 0): 0, (2, 0): 1, (5, 5): 1})
        # Pairs: (0,0)-(1,0) same, (1,0)-(2,0) differ; (5,5) has no neighbors.
        self.assertEqual(tribes.neighbors_alike(f, 10, 10), 0.5)

    def test_the_board_wraps(self):
        f = frame({(0, 0): 1, (9, 0): 1})
        self.assertEqual(tribes.neighbors_alike(f, 10, 10), 1.0)

    def test_majority_reports_the_leading_tribe_and_its_share(self):
        f = frame({(0, 0): 0, (1, 0): 0, (2, 0): 1, (8, 8): 1})
        self.assertEqual(tribes.majority(f, lambda a: a.y < 5), (0, 2 / 3))
        self.assertEqual(tribes.majority(f, lambda a: a.y > 50), (None, None))


if __name__ == "__main__":
    unittest.main()
