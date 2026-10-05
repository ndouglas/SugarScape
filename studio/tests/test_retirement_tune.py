"""The approved retirement score stays continuous on one shared bar grid."""
import unittest

import episode
import music


SUBJECT = ['D2 FA dA', 'B2 AF ED', 'F2 Ad fd', 'e2 cA G2',
           'F A d A F D', 'G B e B G E', 'F2 E2 C2', 'D4 z2']


def bars(text):
    return [bar.strip() for bar in text.split('|') if bar.strip()]


class RetirementTuneTest(unittest.TestCase):
    def setUp(self):
        self.tune = episode.load_module('retirement', 'tune').TUNE

    def test_subject_retains_the_approved_lead(self):
        self.assertEqual(bars(self.tune.sections['A']['fiddle']), SUBJECT)

    def test_all_sections_and_cadences_fill_three_quarter_bars(self):
        for section, parts in self.tune.sections.items():
            for voice, text in parts.items():
                self.assertEqual(len(bars(text)), 8, (section, voice))
                for bar in bars(text):
                    self.assertEqual(music.eighths(bar), 6, (section, voice, bar))
        for bar in self.tune.ending.values():
            self.assertEqual(music.eighths(bar), 6)

    def test_approved_instruments_and_volumes(self):
        self.assertEqual([(v.name, v.program, v.volume) for v in self.tune.voices],
                         [('fiddle', 40, 88), ('accordion', 21, 45), ('guitar', 24, 58)])

    def test_the_98_second_cut_fits_56_bars_at_104_bpm(self):
        self.assertEqual(music.form_for(self.tune, 98), 'AABACBA')
        abc = music.score(self.tune, 98)
        self.assertIn('Q:1/4=104', abc)
        self.assertIn('M:3/4', abc)
        self.assertIn('K:D', abc)
        import re
        for voice in re.split(r'^V:\d.*$', abc, flags=re.M)[1:]:
            body = '\n'.join(line for line in voice.splitlines() if not line.startswith('%'))
            self.assertEqual(len(bars(body)), 56)

    def test_accompaniment_carries_every_strain_ending(self):
        for parts in self.tune.sections.values():
            self.assertEqual(bars(parts['accordion'])[-1], '[DFA]6')
            self.assertEqual(bars(parts['guitar'])[-1], 'D,6')

    def test_final_cadence_replaces_last_bar(self):
        self.assertEqual(self.tune.ending, {'fiddle': 'D6', 'accordion': '[DFA]6', 'guitar': 'D,6'})

    def test_continuous_score_has_no_cues_or_ducking(self):
        self.assertEqual((self.tune.cues, self.tune.stings, self.tune.ducks, self.tune.handoffs),
                         ((), {}, {}, ()))

    def test_variation_and_long_note_strain_keep_the_subject_turn(self):
        self.assertEqual(bars(self.tune.sections['B']['fiddle'])[:2], SUBJECT[:2])
        self.assertTrue(all('2' in bar or '4' in bar or '6' in bar
                            for bar in bars(self.tune.sections['C']['fiddle'])))
