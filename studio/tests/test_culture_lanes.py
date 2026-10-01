import unittest

import culture


class LanesTest(unittest.TestCase):
    def test_every_pair_of_side_neighbors_has_one_lane_on_the_shared_edge(self):
        lanes = culture.lanes(3, 2)
        self.assertEqual(len(lanes), 2 * 2 + 3 * 1, "side by side pairs, then above and below")
        pairs = {(i, j) for i, j, *_ in lanes}
        self.assertIn((0, 1), pairs)
        self.assertIn((0, 3), pairs)
        self.assertNotIn((2, 3), pairs, "no wrap: the map is bounded")
        # Squares (x, y) sit at (x − w/2 + 0.5, h/2 − y − 0.5): between (0, 0)
        # and (1, 0) the lane runs north–south at x = −0.5.
        i, j, x, y, along_y = next(l for l in lanes if l[:2] == (0, 1))
        self.assertEqual((x, y, along_y), (-0.5, 0.5, True))
        i, j, x, y, along_y = next(l for l in lanes if l[:2] == (0, 3))
        self.assertEqual((x, y, along_y), (-1.0, 0.0, False))

    def test_shared_counts_matching_features(self):
        traits = {0: (1, 2, 3), 1: (1, 5, 3)}
        self.assertEqual(culture.shared(traits, 0, 1), 2)
