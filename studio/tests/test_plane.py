import pathlib
import unittest

import dump
import plane

# Made by `sugarscape shot tests/fixtures/norms.json`: ax-metanorms, seed 2,
# 4 generations, with events.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "norms.frames.json"


class PlaneTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_a_norms_dump_loads_as_a_plane(self):
        self.assertIsInstance(self.d, dump.Plane)
        self.assertEqual((self.d.ticks, self.d.every, len(self.d.frames[1].agents)), (4, 1, 20))
        self.assertTrue(self.d.frames[1].cheats)
        self.assertEqual([f.tick for f in self.d.frames], [0, 1, 2, 3, 4])

    def test_boldness_runs_right_and_vengefulness_back(self):
        self.assertLess(plane.square_center(0, 0)[0], plane.square_center(7, 0)[0])
        self.assertLess(plane.square_center(0, 0)[1], plane.square_center(0, 7)[1])

    def test_the_regions_are_galan_and_izquierdos(self):
        self.assertTrue(plane.established_square(2, 5) and not plane.established_square(3, 5))
        self.assertTrue(plane.collapsed_square(6, 1) and not plane.collapsed_square(6, 2))

    def test_each_flump_stands_on_its_square_and_none_share_a_spot(self):
        f = self.d.frames[2]
        spots = plane.layout(f)
        self.assertEqual(len(set(spots)), len(spots))
        for a, (x, y) in zip(f.agents, spots):
            cx, cy = plane.square_center(a[0], a[1])
            self.assertLess(abs(x - cx), plane.CELL / 2)
            self.assertLess(abs(y - cy), plane.CELL / 2 + 0.5)

    def test_a_flump_changing_square_hops_and_the_last_frame_holds(self):
        now, nxt = self.d.frames[1], self.d.frames[2]
        i = next(i for i, (a, b) in enumerate(zip(now.agents, nxt.agents)) if a[:2] != b[:2])
        self.assertGreater(plane.poses(self.d, 1.7, hop=0.6)[i].z, 0.0)
        self.assertEqual(plane.poses(self.d, 1.1, hop=0.6)[i].z, 0.0)
        self.assertTrue(all(p.z == 0.0 for p in plane.poses(self.d, 9, hop=0.6)))


if __name__ == "__main__":
    unittest.main()
