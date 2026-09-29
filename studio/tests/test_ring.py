import math
import pathlib
import unittest

import dump
import ring

# Made by `sugarscape shot tests/fixtures/tags.json`: rca-published, seed 4,
# 5 generations, with gifts.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "tags.frames.json"


class RingTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_a_tags_dump_loads_as_a_ring(self):
        self.assertIsInstance(self.d, dump.Ring)
        self.assertEqual((self.d.ticks, len(self.d.frames), len(self.d.frames[1].agents)), (5, 6, 100))
        f = self.d.frames[1]
        self.assertEqual(sum(a.given for a in f.agents), len(f.gifts))

    def test_tags_run_clockwise_from_the_top_leaving_a_gap_between_0_and_1(self):
        (x0, y0), (x1, y1) = ring.on_ring(0.0), ring.on_ring(1.0)
        self.assertGreater(x0, 0.5)
        self.assertLess(x1, -0.5)
        self.assertGreater(min(y0, y1), 0.9 * ring.RADIUS)
        x, y = ring.on_ring(0.5)
        self.assertAlmostEqual(x, 0.0)
        self.assertAlmostEqual(y, -ring.RADIUS)
        x, y = ring.on_ring(0.25)
        self.assertGreater(x, 0.9 * ring.RADIUS)

    def test_twins_share_a_pile_and_no_two_flumps_stand_together(self):
        f = dump.RingFrame(0, [dump.Tagger(i, 0, 0.5 if i < 30 else i / 100, 0.0, 0, 0) for i in range(100)])
        spots = ring.layout(f)
        pile = spots[:30]
        cx, cy = ring.on_ring(0.5)
        self.assertTrue(all(math.hypot(x - cx, y - cy) < 6 for x, y in pile))
        for i in range(len(spots)):
            for j in range(i):
                if f.agents[i].tag == f.agents[j].tag:
                    self.assertGreater(math.dist(spots[i], spots[j]), 0.9)

    def test_a_flump_that_copies_another_leaps_and_takes_its_shade_at_the_top(self):
        before, after = self.d.frames[1], self.d.frames[2]
        k = ring.leaps(before, after).index(True)
        rising = ring.poses(self.d, 1.3, hop=0.6)[k]
        top = ring.poses(self.d, 1.75, hop=0.6)[k]
        self.assertEqual(rising.tag, before.agents[k].tag)
        self.assertEqual(top.tag, after.agents[k].tag)
        self.assertGreater(top.z, 0.0)

    def test_the_last_generation_holds(self):
        last = ring.poses(self.d, 9.0, hop=0.6)
        self.assertEqual([p.tag for p in last], [a.tag for a in self.d.frames[5].agents])
        self.assertTrue(all(p.z == 0.0 for p in last))

    def test_shades_cover_the_wheel(self):
        self.assertEqual({ring.hue(t / 1000) for t in range(1000)}, set(range(ring.HUES)))


if __name__ == "__main__":
    unittest.main()
