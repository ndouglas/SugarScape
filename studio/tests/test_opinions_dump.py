import collections
import pathlib
import unittest

import dump

# Made by `sugarscape shot tests/fixtures/opinions.json` (hk-polarisation, seed 1, 10 periods).
HERE = pathlib.Path(__file__).parent / "fixtures"


class OpinionsDumpTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(HERE / "opinions.frames.json")

    def test_each_opinion_bin_is_a_block_filled_from_the_front(self):
        d = self.d
        self.assertEqual((d.model, d.width, d.ticks), ("opinions", dump.OPINION_COLUMNS * dump.BLOCK, 10))
        for f in d.frames:
            bins = collections.defaultdict(list)
            for a in f.agents.values():
                bins[a.x // dump.BLOCK].append((d.height - 1 - a.y) * dump.BLOCK + a.x % dump.BLOCK)
            for slots in bins.values():
                self.assertEqual(sorted(slots), list(range(len(slots))), "filled from the front, row by row")

    def test_the_board_is_as_deep_as_the_biggest_block(self):
        d = self.d
        biggest = max(max(collections.Counter(a.x // dump.BLOCK for a in f.agents.values()).values()) for f in d.frames)
        self.assertEqual(d.height, -(-biggest // dump.BLOCK))

    def test_a_block_is_ordered_by_start_and_camps_make_big_blocks(self):
        d = self.d
        f = d.frames[-1]
        by_bin = collections.defaultdict(list)
        for a in f.agents.values():
            by_bin[a.x // dump.BLOCK].append(a)
        for block in by_bin.values():
            block.sort(key=lambda a: ((d.height - 1 - a.y), a.x))
            ranks = [d.start_rank[a.id] for a in block]
            self.assertEqual(ranks, sorted(ranks))
        self.assertLessEqual(len(by_bin), 4, "three camps at the end (a stray or so)")
        self.assertGreater(max(len(v) for v in by_bin.values()), 150)

    def test_colors_bin_the_start_from_red_to_magenta(self):
        f = self.d.frames[0]
        self.assertEqual(set(f.groups.values()), set(range(10)))
        low = min(f.agents.values(), key=lambda a: self.d.start_rank[a.id])
        self.assertEqual(f.groups[low.id], 0)
