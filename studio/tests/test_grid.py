import math
import pathlib
import unittest

import dump
import grid

# Made by `sugarscape shot tests/fixtures/structure.json`: cra-2dk, seed 2, 3
# periods, with partners.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "structure.frames.json"


class GridTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_a_structure_dump_loads_as_a_grid(self):
        self.assertIsInstance(self.d, dump.Grid)
        self.assertEqual((self.d.ticks, len(self.d.frames[1].agents), len(self.d.site)), (3, 256, 256))

    def test_on_the_torus_every_partner_stands_next_door_or_across_the_wrap(self):
        f = self.d.frames[1]
        for a, partners in enumerate(f.partners):
            for b in partners:
                d = math.dist(grid.position(a, self.d), grid.position(b, self.d))
                self.assertTrue(abs(d - grid.SPACING) < 1e-9 or abs(d - 15 * grid.SPACING) < 1e-9, d)

    def test_every_agent_has_its_own_square(self):
        spots = {grid.position(a, self.d) for a in range(256)}
        self.assertEqual(len(spots), 256)

    def test_shades_run_from_red_to_green(self):
        self.assertEqual((grid.shade(0.0), grid.shade(1.0)), (0, grid.SHADES - 1))
        r0, g0, _ = grid.shade_color(0)
        r9, g9, _ = grid.shade_color(grid.SHADES - 1)
        self.assertGreater(r0, g0)
        self.assertGreater(g9, r9)

    def test_ties_follow_the_chosen_agents(self):
        self.assertEqual(len(grid.ties(self.d, self.d.frames[1], [0, 5])), 8)
        self.assertEqual(grid.ties(self.d, self.d.frames[0], [0]), [])


if __name__ == "__main__":
    unittest.main()
