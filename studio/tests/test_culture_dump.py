import pathlib
import unittest

import dump

# Made by `sugarscape shot tests/fixtures/culture.json` (ac-sample-run, seed 1,
# 1,200 steps filmed every 100th: stable before the end).
HERE = pathlib.Path(__file__).parent / "fixtures"


class CultureDumpTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(HERE / "culture.frames.json")

    def test_a_culture_dump_is_a_board_of_still_sites(self):
        d = self.d
        self.assertEqual((d.model, d.width, d.height, d.ticks), ("culture", 10, 10, 12))
        self.assertEqual(len(d.frames[0].agents), 100)
        for f in d.frames:
            self.assertEqual({(a.x, a.y) for a in f.agents.values()}, {(x, y) for x in range(10) for y in range(10)})

    def test_each_site_carries_its_traits(self):
        f = self.d.frames[0]
        self.assertEqual(len(f.traits[0]), 5)
        self.assertTrue(all(0 <= t < 10 for ts in f.traits.values() for t in ts))

    def test_surviving_cultures_get_colors_by_size_and_the_rest_none(self):
        d = self.d
        last = d.frames[-1]
        sizes = {}
        for ts in last.traits.values():
            sizes[ts] = sizes.get(ts, 0) + 1
        ranked = sorted(sizes, key=lambda c: (-sizes[c], c))
        for i, ts in last.traits.items():
            self.assertEqual(last.groups[i], ranked.index(ts) + 1)
        first = d.frames[0]
        for i, ts in first.traits.items():
            self.assertEqual(first.groups[i], ranked.index(ts) + 1 if ts in sizes else 0)
