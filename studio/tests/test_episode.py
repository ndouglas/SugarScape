import unittest

import episode


class EpisodeTest(unittest.TestCase):
    def test_every_episode_loads_and_its_beats_are_sane(self):
        names = [p.name for p in (episode.ROOT / "episodes").iterdir() if (p / "beats.py").exists()]
        self.assertIn("sugarscape", names)
        self.assertIn("seasons", names)
        for name in names:
            beats = episode.load_episode(name)
            self.assertTrue(beats, name)
            for b in beats:
                self.assertGreater(b.seconds, 0)
                self.assertTrue(b.camera, (name, b.name))
                self.assertEqual(b.frames, round(b.seconds * 30))
                for shot in (b.shot, b.compare):
                    if shot:
                        path = episode.episode_dir(name) / "shots" / f"{shot}.json"
                        self.assertTrue(path.exists(), (name, b.name, shot))

    def test_timing_ends_at_the_dumps_last_tick(self):
        b = episode.Beat("x", "", 4, shot="s", ticks_per_second=2, lead_in=1)
        t = b.timing(6)
        self.assertEqual((t.end_tick, t.lead_in, t.ticks_per_second), (6, 1, 2))


if __name__ == "__main__":
    unittest.main()
