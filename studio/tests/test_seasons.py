import unittest

import dump
import seasons


def track(cells, first=0, death=None):
    return dump.Track(1, first, cells, [10.0] * len(cells), death, None)


class SeasonTest(unittest.TestCase):
    def test_the_north_has_summer_first_and_they_swap_each_period(self):
        # The engine's growback during tick t uses the season at t − 1.
        self.assertTrue(seasons.north_summer(0, 50))
        self.assertTrue(seasons.north_summer(49, 50))
        self.assertFalse(seasons.north_summer(50, 50))
        self.assertTrue(seasons.north_summer(100, 50))

    def test_the_north_is_the_rows_above_half_the_height(self):
        self.assertTrue(seasons.in_north(24, 50))
        self.assertFalse(seasons.in_north(25, 50))
        self.assertFalse(seasons.in_north(2, 5))  # odd height: the extra row is in the south

    def test_frost_follows_the_season_and_eases_across_a_swap(self):
        # Frost on the north: none in its summer, full in its winter, halfway mid-swap.
        self.assertEqual(seasons.frost(True, 10.0, 50), 0.0)
        self.assertEqual(seasons.frost(True, 70.0, 50), 1.0)
        self.assertEqual(seasons.frost(False, 10.0, 50), 1.0)
        self.assertAlmostEqual(seasons.frost(True, 50.0, 50, ease=2.0), 0.5)


class StrategyTest(unittest.TestCase):
    def test_switches_count_hemisphere_crossings_in_a_window(self):
        # Height 10: rows 0–4 are north. North, north, south, south, north.
        t = track([(0, 1), (0, 2), (0, 7), (0, 8), (0, 3)])
        self.assertEqual(seasons.switches(t, 0, 4, 10), 2)
        self.assertEqual(seasons.switches(t, 0, 1, 10), 0)

    def test_a_flump_not_alive_for_the_whole_window_has_no_count(self):
        self.assertIsNone(seasons.switches(track([(0, 1)] * 3, first=2), 0, 4, 10))
        self.assertIsNone(seasons.switches(track([(0, 1)] * 3, death=3), 0, 4, 10))

    def test_summer_share_is_the_share_of_ticks_spent_in_summer(self):
        # Period 2: north summer at ticks 0–1 and 4–5, winter at 2–3.
        t = track([(0, 1), (0, 1), (0, 8), (0, 8), (0, 1), (0, 1)])
        self.assertEqual(seasons.summer_share(t, 0, 5, 10, 2), 1.0)
        still = track([(0, 1)] * 6)
        self.assertAlmostEqual(seasons.summer_share(still, 0, 5, 10, 2), 4 / 6)


if __name__ == "__main__":
    unittest.main()
