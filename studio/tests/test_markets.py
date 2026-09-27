import pathlib
import unittest

import dump
import markets

MARKET = pathlib.Path(__file__).parent / "fixtures" / "market.frames.json"


def frame(tick, cells, spice=None, trades=()):
    """`cells`: {id: (x, y, sugar)}; `spice`: {id: (spice, spice metabolism)}."""
    agents = {i: dump.Agent(i, x, y, s, 0, 1, 1) for i, (x, y, s) in cells.items()}
    return dump.Frame(tick, agents, [], {}, [], [], {}, spice_agents=spice or {}, trades=list(trades))


class SideTest(unittest.TestCase):
    # A 2 × 1 board: sugar grows at x = 0, spice at x = 1.
    SUGAR, SPICE = [4.0, 0.0], [0.0, 4.0]

    def test_a_site_is_on_the_side_of_the_good_it_grows_more_of(self):
        sides = markets.sides(self.SUGAR, self.SPICE)
        self.assertEqual(sides, ["sugar", "spice"])
        self.assertEqual(markets.sides([2.0], [2.0]), [None])

    def test_crossings_count_changes_of_side_ignoring_the_middle(self):
        sides = ["sugar", None, "spice"]
        # Sugar, middle, spice, middle, sugar: two crossings.
        self.assertEqual(markets.crossings([0, 1, 2, 1, 0], sides), 2)
        self.assertEqual(markets.crossings([0, 0, 1, 0], sides), 0)

    def test_shuttlers_are_the_share_of_flumps_alive_all_window_crossing_twice(self):
        sides = ["sugar", "spice"]
        frames = [
            frame(0, {1: (0, 0, 5), 2: (0, 0, 5), 3: (0, 0, 5)}),
            frame(1, {1: (1, 0, 5), 2: (0, 0, 5), 3: (1, 0, 5)}),
            frame(2, {1: (0, 0, 5), 2: (0, 0, 5)}),  # 3 dies
        ]
        # 1 crosses twice; 2 never; 3 isn't alive for the whole window.
        self.assertEqual(markets.shuttle_share(frames, 2, sides, 0, 2), 0.5)


class TradeDirectionTest(unittest.TestCase):
    def test_the_sugar_giver_held_relatively_more_sugar_for_its_needs(self):
        before = frame(0, {1: (0, 0, 40), 2: (1, 0, 5)}, spice={1: (5, 1), 2: (40, 1)})
        after = frame(1, {1: (0, 0, 30), 2: (1, 0, 15)}, spice={1: (15, 1), 2: (30, 1)},
                      trades=[dump.Trade(1, 2, 10.0, 10.0, 3)])
        self.assertEqual(markets.expected_direction(before, after), (1, 1))
        wrong = frame(1, {1: (0, 0, 30), 2: (1, 0, 15)}, trades=[dump.Trade(2, 1, 1.0, 1.0, 1)])
        self.assertEqual(markets.expected_direction(before, wrong), (0, 1))

    def test_on_a_real_dump_the_sugar_givers_are_the_sugar_rich(self):
        d = dump.load(MARKET)
        right, total = 0, 0
        for k in range(1, len(d.frames)):
            r, n = markets.expected_direction(d.frames[k - 1], d.frames[k])
            right, total = right + r, total + n
        self.assertGreater(total, 0)
        self.assertGreaterEqual(right / total, 0.8)


class PriceSeriesTest(unittest.TestCase):
    def test_price_is_the_geometric_mean_over_exchanges_and_spread_their_log_sd(self):
        frames = [
            frame(0, {}),
            frame(1, {}, trades=[dump.Trade(1, 2, 1.0, 2.0, 1), dump.Trade(3, 4, 2.0, 1.0, 1)]),
            frame(2, {}),  # no trades: the price holds
        ]
        prices, spreads = markets.price_series(frames)
        self.assertEqual(prices[0], None)
        self.assertAlmostEqual(prices[1], 1.0)  # √(2 · ½)
        self.assertAlmostEqual(spreads[1], 0.6931, places=3)  # ln 2
        self.assertEqual((prices[2], spreads[2]), (prices[1], spreads[1]))

    def test_a_pairs_exchanges_weight_its_price(self):
        frames = [frame(0, {}), frame(1, {}, trades=[dump.Trade(1, 2, 1.0, 2.0, 3), dump.Trade(3, 4, 2.0, 1.0, 1)])]
        prices, _ = markets.price_series(frames)
        self.assertAlmostEqual(prices[1], 2 ** 0.5)  # (2³ · ½)^¼

    def test_on_the_market_fixture_prices_start_at_the_first_trades(self):
        prices, spreads = markets.price_series(dump.load(MARKET).frames)
        self.assertIsNone(prices[0])
        self.assertTrue(all(p > 0 for p in prices[1:]))


class SmoothedTest(unittest.TestCase):
    def test_the_rolling_price_is_the_geometric_mean_of_the_window(self):
        prices, spreads = markets.smoothed([None, 2.0, 0.5, 4.0], [None, 0.2, 0.4, 0.6], 2)
        self.assertIsNone(prices[0])
        self.assertAlmostEqual(prices[1], 2.0)  # one tick so far
        self.assertAlmostEqual(prices[2], 1.0)  # √(2 · ½)
        self.assertAlmostEqual(prices[3], 2 ** 0.5)  # √(½ · 4)
        self.assertAlmostEqual(spreads[3], 0.5)


class ShuttlersTest(unittest.TestCase):
    def test_shuttlers_are_the_tracks_alive_all_window_that_cross_twice(self):
        sides = ["sugar", "spice"]
        tracks = {
            1: dump.Track(1, 0, [(0, 0), (1, 0), (0, 0)], [5] * 3, None, None),
            2: dump.Track(2, 0, [(0, 0), (0, 0), (0, 0)], [5] * 3, None, None),
            3: dump.Track(3, 1, [(1, 0), (0, 0)], [5] * 2, None, None),  # born too late
        }
        self.assertEqual(markets.shuttlers(tracks, 2, sides, 0, 2), {1})


if __name__ == "__main__":
    unittest.main()
