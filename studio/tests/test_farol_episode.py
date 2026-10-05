"""Approved episode must consume measured examples and one continuous score."""
import pathlib
import re
import json
import unittest
import episode
import music
import cut
ROOT=pathlib.Path(__file__).resolve().parents[2]

class FarolEpisodeTests(unittest.TestCase):
    def test_all_approved_captions_and_recording_windows(self):
        self.assertTrue((ROOT/'studio/episodes/farol/beats.py').exists(),'approved beats missing')
        beats=episode.load_episode('farol')
        spec=(ROOT/'docs/superpowers/specs/2026-10-02-farol-spike.md').read_text()
        captions=re.findall(r'\| \d+ \| \w+ \| “(.*?)” \|',spec)
        self.assertEqual([b.caption for b in beats],[c.replace('\\n','\n') for c in captions])
        measured=json.loads((ROOT/'studio/episodes/farol/measurements.json').read_text())
        self.assertEqual(cut.total_frames([b.frames for b in beats],12)/30,95)
        self.assertEqual(beats[0].start_tick,0)
        for b in beats[1:-1]:
            selection=measured['selected'][b.shot]
            self.assertGreaterEqual(b.start_tick,selection['start_frame'])
            self.assertLessEqual(b.timing(selection['end_frame']).tick_at(b.frames),selection['end_frame'])
        self.assertEqual(beats[9].shot,'teaching')

    def test_one_fitted_score_fills_every_voice_without_cues(self):
        self.assertTrue((ROOT/'studio/episodes/farol/tune.py').exists(),'original score missing')
        tune=episode.load_module('farol','tune').TUNE
        self.assertEqual((tune.stings,tune.cues,tune.ducks,tune.handoffs),({},(),{},()))
        self.assertEqual(music.form_for(tune,95),'ABCDE')
        self.assertEqual([v.program for v in tune.voices],[71,21,24])
        for parts in tune.sections.values():
            for phrase in parts.values():
                self.assertEqual([music.eighths(bar) for bar in music._bars(phrase)],[8]*8)
        self.assertEqual([music.eighths(v) for v in tune.ending.values()],[8]*3)
        self.assertIn('Q:1/4=103',music.score(tune,95))
