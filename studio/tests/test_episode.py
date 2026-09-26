import unittest

import episode


class EpisodeTest(unittest.TestCase):
    def test_the_pilot_loads_and_its_beats_are_sane(self):
        beats = episode.load_episode("sugarscape")
        self.assertTrue(beats)
        for b in beats:
            self.assertGreater(b.seconds, 0)
            self.assertTrue(b.camera, b.name)
            self.assertEqual(b.frames, round(b.seconds * 30))
            if b.shot:
                self.assertTrue((episode.episode_dir("sugarscape") / "shots" / f"{b.shot}.json").exists(), b.shot)

    def test_timing_ends_at_the_dumps_last_tick(self):
        b = episode.Beat("x", "", 4, shot="s", ticks_per_second=2, lead_in=1)
        t = b.timing(6)
        self.assertEqual((t.end_tick, t.lead_in, t.ticks_per_second), (6, 1, 2))


if __name__ == "__main__":
    unittest.main()
