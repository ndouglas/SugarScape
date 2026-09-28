import pathlib
import unittest

import dump
import street

# Made by `sugarscape shot tests/fixtures/image.json`: ns-fig-1, seed 3, 5
# generations, with meetings.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "image.frames.json"


class StreetTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_an_image_dump_loads_as_a_street(self):
        self.assertIsInstance(self.d, dump.Street)
        self.assertEqual((self.d.ticks, len(self.d.frames[1].agents), len(self.d.frames[1].meetings)), (5, 100, 125))

    def test_columns_run_from_help_everyone_on_the_left_to_help_no_one_on_the_right(self):
        self.assertLess(street.column_x(-5), street.column_x(0))
        self.assertLess(street.column_x(0), street.column_x(6))
        self.assertAlmostEqual(street.column_x(-5), -street.column_x(6))

    def test_each_flump_stands_in_its_thresholds_column_and_none_share_a_spot(self):
        f = self.d.frames[1]
        spots = street.layout(f)
        self.assertEqual(len(set(spots)), len(spots))
        for a, (x, _) in zip(f.agents, spots):
            self.assertLess(abs(x - street.column_x(a[1])), street.COLUMN / 2)

    def test_scores_run_from_red_through_neutral_to_green(self):
        r, g, b = street.score_color(-5)
        self.assertGreater(r, g)
        r, g, b = street.score_color(5)
        self.assertGreater(g, r)
        r, g, b = street.score_color(0)
        self.assertLess(max(r, g, b) - min(r, g, b), 0.15)

    def test_a_generation_takes_its_places_first_then_plays_its_meetings(self):
        now, nxt = self.d.frames[1], self.d.frames[2]
        i = next(i for i, (a, b) in enumerate(zip(now.agents, nxt.agents)) if a[1] != b[1])
        rising = street.poses(self.d, 1.2, hop=0.4)[i]
        self.assertGreater(rising.z, 0.0)
        self.assertEqual(rising.score, now.agents[i][2])
        # The meetings play out one by one; once all have, every score is
        # generation 2's.
        self.assertEqual(street.shown(self.d, 1.999, 0.4), 124)
        done = street.poses(self.d, 2.0, hop=0.4)
        self.assertEqual([p.score for p in done], [a[2] for a in nxt.agents])

    def test_meetings_replay_to_each_generations_scores(self):
        for f in self.d.frames[1:]:
            self.assertEqual(street.replay(f.meetings, len(f.meetings), 100), [a[2] for a in f.agents])

if __name__ == "__main__":
    unittest.main()
