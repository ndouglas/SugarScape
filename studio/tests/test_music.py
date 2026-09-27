import re
import unittest

import episode
import music

TUNES = {name: episode.load_module(name, "tune").TUNE for name in ("sugarscape", "seasons", "pollution", "inheritance", "tribes")}


def bars(voice_body):
    return [b for b in voice_body.split("|") if b.strip()]


class MusicTest(unittest.TestCase):
    def test_tempo_fits_the_form_to_the_video(self):
        walk = TUNES["sugarscape"]
        # 40 bars of 4 beats in 95.5 s (97 s less the ring-out) is about 100 bpm.
        self.assertAlmostEqual(music.tempo_for(walk, 40, 97.0), 40 * 4 * 60 / (97.0 - music.RING_OUT))
        waltz = TUNES["seasons"]
        self.assertAlmostEqual(music.tempo_for(waltz, 64, 70.0), 64 * 3 * 60 / (70.0 - music.RING_OUT))

    def test_each_tune_chooses_a_form_in_its_tempo_range(self):
        for name, tune in TUNES.items():
            for seconds in (60, 70, 96):
                form = music.form_for(tune, seconds)
                bpm = music.tempo_for(tune, len(form) * music.SECTION_BARS, seconds)
                low, _, high = tune.bpm
                self.assertTrue(low <= bpm <= high, (name, seconds, form, bpm))
                self.assertIn(form, tune.forms)

    def test_every_voice_has_every_bar(self):
        for name, tune in TUNES.items():
            abc = music.score(tune, 70.0)
            self.assertIn(f"K:{tune.key}", abc)
            self.assertIn(f"M:{tune.beats_per_bar}/4", abc)
            voices = re.split(r"^V:\d.*$", abc, flags=re.M)[1:]
            self.assertEqual(len(voices), len(tune.voices))
            expected = len(music.form_for(tune, 70.0)) * music.SECTION_BARS
            for v in voices:
                body = "\n".join(line for line in v.splitlines() if not line.startswith("%"))
                self.assertEqual(len(bars(body)), expected, name)

    def test_every_bar_of_every_tune_fills_its_meter(self):
        for name, tune in TUNES.items():
            per_bar = tune.beats_per_bar * 2
            for section, parts in tune.sections.items():
                self.assertEqual(set(parts), {v.name for v in tune.voices}, (name, section))
                for voice, text in parts.items():
                    self.assertEqual(len(bars(text)), music.SECTION_BARS, (name, section, voice))
                    for bar in bars(text):
                        self.assertEqual(music.eighths(bar), per_bar, (name, section, voice, bar))
            for voice, bar in tune.ending.items():
                self.assertEqual(music.eighths(bar), per_bar, (name, "ending", voice))

    def test_a_voice_can_be_placed_left_or_right(self):
        tune = TUNES["sugarscape"]
        panned = music.Tune(**{**tune.__dict__, "voices": (music.Voice("accordion", 21, 110, pan=20),) + tune.voices[1:]})
        abc = music.score(panned, 96.0)
        self.assertIn("%%MIDI control 10 20", abc)
        self.assertNotIn("%%MIDI control 10", music.score(tune, 96.0))

    def test_percussion_voices_play_on_channel_ten(self):
        abc = music.score(TUNES["seasons"], 70.0)
        self.assertIn("%%MIDI channel 10", abc)

    def test_stings_are_cued_to_their_beats(self):
        cues = music.cue_times(TUNES["seasons"].cues, ["reprise", "seasons", "slower"], [120, 180, 180], dissolve=12)
        # "seasons" starts at (120 − 12)/30 = 3.6 s; "slower" at (120 + 180 − 24)/30 = 9.2 s; each 0.3 s in.
        self.assertEqual(cues, [("summer", 3.9), ("winter", 9.5)])

    def test_a_cue_can_land_at_its_own_moment_in_the_beat(self):
        cues = music.cue_times((("a", "x", 2.5), ("b", "y")), ["a", "b"], [150, 90], dissolve=12)
        self.assertEqual(cues, [("x", 2.5), ("y", round((150 - 12) / 30 + music.CUE_DELAY, 3))])

    def test_the_sting_mix_delays_each_sting(self):
        argv = music.sting_mix_command("t.wav", [("s.wav", 3.9), ("w.wav", 9.5)], "o.wav")
        graph = argv[argv.index("-filter_complex") + 1]
        self.assertIn("[1:a]adelay=3900|3900[s0]", graph)
        self.assertIn("[2:a]adelay=9500|9500[s1]", graph)
        self.assertIn("amix=inputs=3", graph)


    def test_a_ducking_sting_lowers_the_track_while_it_plays(self):
        argv = music.sting_mix_command("t.wav", [("s.wav", 3.0)], "o.wav", ducks=[(3.0, 9.0, 0.2)])
        graph = argv[argv.index("-filter_complex") + 1]
        self.assertIn("[0:a]volume='", graph)
        self.assertIn(":eval=frame[main]", graph)
        self.assertIn("[main][s0]amix=inputs=2", graph)

    def test_the_duck_ramps_down_holds_and_ramps_back(self):
        gain = lambda t: music.duck_gain(t, [(3.0, 9.0, 0.2)])
        self.assertEqual(gain(2.0), 1.0)
        self.assertAlmostEqual(gain(3.0 + music.DUCK_RAMP / 2), 0.6)
        self.assertAlmostEqual(gain(6.0), 0.2)
        self.assertEqual(gain(9.5), 1.0)

    def test_the_expression_ffmpeg_gets_matches_the_gain(self):
        expr = music.duck_expression([(3.0, 9.0, 0.2)])
        for t in (2.0, 3.4, 6.0, 8.8, 9.5):
            python = expr.replace("clip", "_clip").replace("min(", "_min(")
            value = eval(python, {"_clip": lambda x, lo, hi: min(max(x, lo), hi), "_min": min, "t": t})
            self.assertAlmostEqual(value, music.duck_gain(t, [(3.0, 9.0, 0.2)]))


    def test_dynamics_and_other_decorations_take_no_time(self):
        self.assertEqual(music.eighths("!pp! D,3 D, !mf! D,2 !fff! D,2 A,,2"), 10)


if __name__ == "__main__":
    unittest.main()
