import pathlib
import unittest

import area
import dump

# Made by `sugarscape shot tests/fixtures/tipping.json`: tipping-fig19, 12 steps.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "tipping.frames.json"


class AreaTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_a_tipping_dump_loads_as_a_board_of_insiders_and_queues(self):
        d = self.d
        self.assertEqual((d.model, d.width, d.height), ("tipping", area.WIDTH, area.HEIGHT))
        self.assertEqual(len(d.frames[0].agents), 200, "everyone, inside or out")
        self.assertEqual(sum(d.frames[0].groups.values()), 100, "100 Red")

    def test_insiders_stand_in_the_area_and_outsiders_in_their_queue(self):
        f = self.d.frames[0]
        inside = [a for a in f.agents.values() if area.in_area(a.x, a.y)]
        self.assertEqual(len(inside), 100, "50 of each start inside")
        for i, a in f.agents.items():
            if not area.in_area(a.x, a.y):
                red = f.groups[i] == 1
                self.assertEqual(a.x < area.LEFT, red, "Red queue on the left, Blue on the right")

    def test_the_most_tolerant_outsider_stands_nearest_the_gate(self):
        near = area.queue_spot(True, 0)
        far = area.queue_spot(True, 60)
        self.assertLess(area.LEFT - near[0], area.LEFT - far[0] + 1)
        self.assertEqual(near[0], area.LEFT - area.GAP - 1)

    def test_an_insider_keeps_its_square_while_inside(self):
        a, b = self.d.frames[0].agents, self.d.frames[5].agents
        kept = [i for i in a if area.in_area(a[i].x, a[i].y) and i in b and area.in_area(b[i].x, b[i].y)]
        self.assertTrue(kept)
        self.assertTrue(all((a[i].x, a[i].y) == (b[i].x, b[i].y) for i in kept))

    def test_squares_are_never_shared(self):
        for f in self.d.frames:
            spots = [(a.x, a.y) for a in f.agents.values()]
            self.assertEqual(len(spots), len(set(spots)))

    def test_the_counts_match_the_stats(self):
        for k, f in enumerate(self.d.frames):
            reds = sum(1 for i, a in f.agents.items() if f.groups[i] == 1 and area.in_area(a.x, a.y))
            self.assertEqual(reds, self.d.stats["red_in"][k])
