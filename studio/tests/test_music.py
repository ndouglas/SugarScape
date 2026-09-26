import re
import unittest

import music


def bars(voice_body):
    return [b for b in voice_body.split("|") if b.strip()]


class MusicTest(unittest.TestCase):
    def test_tempo_fits_the_form_to_the_video(self):
        # 40 bars of 4 beats in 95.5 s (97 s less the ring-out) is about 100 bpm.
        self.assertAlmostEqual(music.tempo_for(40, 97.0), 40 * 4 * 60 / (97.0 - music.RING_OUT))

    def test_the_form_is_chosen_for_a_walking_pace(self):
        for seconds in (40, 60, 96, 120):
            form = music.form_for(seconds)
            bpm = music.tempo_for(len(form) * music.SECTION_BARS, seconds)
            self.assertTrue(music.MIN_BPM <= bpm <= music.MAX_BPM, (seconds, form, bpm))
            self.assertEqual(form[0], "A")
            self.assertEqual(form[-1], "A")

    def test_every_voice_has_every_bar(self):
        abc = music.score(96.0)
        self.assertIn("K:D", abc)
        tempo = int(re.search(r"Q:1/4=(\d+)", abc).group(1))
        self.assertTrue(music.MIN_BPM <= tempo <= music.MAX_BPM)
        voices = re.split(r"^V:\d.*$", abc, flags=re.M)[1:]
        self.assertEqual(len(voices), 3)
        expected = len(music.form_for(96.0)) * music.SECTION_BARS
        for v in voices:
            body = "\n".join(line for line in v.splitlines() if not line.startswith("%"))
            self.assertEqual(len(bars(body)), expected)

    def test_each_bar_of_the_tune_has_eight_eighths(self):
        for part in (music.MELODY, music.GUITAR):
            for name, section in part.items():
                for bar in bars(section):
                    self.assertEqual(music.eighths(bar), 8, (name, bar))


if __name__ == "__main__":
    unittest.main()
