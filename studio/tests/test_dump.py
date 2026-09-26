import pathlib
import unittest

import dump

# Made by `sugarscape shot tests/fixtures/tiny.json`.
FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "tiny.frames.json"


class DumpTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_header(self):
        self.assertEqual((self.d.width, self.d.height, self.d.ticks), (8, 8, 6))
        self.assertEqual(len(self.d.frames), 7)
        self.assertEqual(len(self.d.capacity), 64)
        self.assertEqual(len(self.d.placed), 3)

    def test_frames_index_agents_by_id(self):
        first = self.d.placed[0]
        a = self.d.frames[0].agents[first]
        self.assertEqual((a.x, a.y, a.vision, a.metabolism, a.sugar), (4, 4, 3, 1, 5.0))

    def test_deaths_are_keyed_by_id(self):
        starving = self.d.placed[1]
        self.assertEqual(self.d.frames[1].deaths, {starving: "starvation"})

    def test_tracks_cover_life_and_record_death(self):
        t = dump.tracks(self.d)
        starving = t[self.d.placed[1]]
        self.assertEqual(
            (starving.first, len(starving.cells), starving.death, starving.cause),
            (0, 1, 1, "starvation"),
        )
        late = t[self.d.placed[2]]
        self.assertEqual(late.first, 2)
        self.assertEqual(late.cells[0], (7, 7))
        self.assertIsNone(late.death)
        self.assertEqual(len(late.cells), 5)

    def test_wrong_format_is_refused(self):
        with self.assertRaisesRegex(ValueError, "format"):
            dump.parse('{"format": 99}')


if __name__ == "__main__":
    unittest.main()
