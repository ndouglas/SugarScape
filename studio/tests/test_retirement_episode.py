"""The approved film cut uses native examples and protects its closing motion."""
import json
import pathlib
import re
import unittest
import episode
import cut
ROOT=pathlib.Path(__file__).resolve().parents[2]

class RetirementEpisodeTests(unittest.TestCase):
    def test_approved_captions_durations_and_real_shots(self):
        beats=episode.load_episode('retirement')
        spec=(ROOT/'docs/superpowers/specs/2026-10-03-retirement-spike.md').read_text()
        captions=re.findall(r'\| \d+ \| \d+ \| \w+ \| “(.*?)” \|',spec)
        self.assertEqual([b.caption for b in beats],[c.replace('\\n','\n')for c in captions])
        self.assertEqual(cut.total_frames([b.frames for b in beats],12),2940)
        self.assertEqual([round(b.seconds-.4)for b in beats[:-1]]+[round(beats[-1].seconds)],[7,6,8,8,7,7,7,7,7,7,6,8,7,6])
        self.assertEqual((beats[3].shot,beats[3].start_tick),('teaching',3))
        self.assertEqual((beats[7].shot,beats[7].start_tick),('all',1))
        self.assertEqual((beats[-1].shot,beats[-1].start_tick),('teaching',3))
        measured=json.loads((ROOT/'studio/episodes/retirement/measurements.json').read_text())
        for b in beats:
            self.assertIn(b.shot,measured['selected'])
            self.assertLessEqual(b.timing(len(measured['selected'][b.shot]['frame_periods'])-1).tick_at(b.frames),len(measured['selected'][b.shot]['frame_periods'])-1)
