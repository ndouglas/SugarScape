import unittest,re
import episode,music,animate

class AntsEpisodeTest(unittest.TestCase):
    def test_continuous_score_has_no_secondary_music_stream(self):
        tune=episode.load_module('ants','tune').TUNE
        beats=episode.load_episode('ants')
        self.assertEqual(music.cue_times(tune.cues,[b.name for b in beats],
                                        [b.frames for b in beats]),[])
        self.assertEqual(tune.stings,{})

    def test_dance_backing_keeps_full_gain_across_the_entire_cut(self):
        tune=episode.load_module('ants','tune').TUNE
        self.assertEqual(tune.ducks,{})
        seconds=sum(b.seconds for b in episode.load_episode('ants'))-13*.4
        cues=music.cue_times(tune.cues,[b.name for b in episode.load_episode('ants')],
                             [b.frames for b in episode.load_episode('ants')])
        ducks=[(at,at+6.404,tune.ducks[sting])
               for sting,at in cues if sting in tune.ducks]
        samples=[i/10 for i in range(int(seconds*10)+1)]+[seconds]
        for at in samples:
            with self.subTest(seconds=at):
                self.assertEqual(music.duck_gain(at,ducks),1.0)

    def test_all_approved_captions_and_closing(self):
        captions=['Two identical food piles.\nReal ants sometimes crowded one, about 80–20.', "Kirman's Flumps choose between two sources.\nNeither is better.", 'Meet another Flump, and you may copy its choice.', 'Occasionally, a Flump switches on its own.', 'With strong recruiting,\nnearly everyone crowds one source.', 'Then the crowd can flip,\nwithout the food changing.', "Both sides get turns.\nA short run needn't average half and half.", 'The long-run peaks are at all-or-nothing.\n80–20 is not a preferred split.', 'Kirman suggested stronger attraction to the majority.\nOur version favors splits near 18–82.', 'Ten times the Flumps, with the same habits:\nless time crowded at one source.', 'Alfarano and Milaković counted neighbors,\nrather than one partner per meeting.', 'With their rule, growth weakens herding on rings.\nRandom networks keep their swings.', 'A few Flumps who never copy\ncan calm the crowd.', 'Ants at two food piles - After Kirman, 1993; Alfarano & Milaković, 2007\nndouglas.github.io/SugarScape']
        beats=episode.load_episode('ants')
        self.assertEqual([b.caption for b in beats],captions)
        self.assertEqual(beats[-1].caption_y,.45)
        shut=[f for f in range(1,beats[-1].frames+1) if animate.closing_blink(f,beats[-1].frames)<1]
        self.assertEqual(shut,list(range(min(shut),max(shut)+1)))
        self.assertGreater(min(shut),beats[-1].frames*.7)
    def test_micro_and_flip_show_actual_saved_windows(self):
        b={b.name:b for b in episode.load_episode('ants')}
        self.assertEqual((b['meet'].start_tick,b['self'].start_tick),(2,212))
        self.assertEqual(b['flip'].start_tick,17)
        self.assertEqual(b['flip'].timing(29).tick_at(b['flip'].frames),29)
    def test_dance_subject_sections_and_fitted_form(self):
        t=episode.load_module('ants','tune').TUNE
        self.assertEqual((t.key,t.meter,t.beats_per_bar),('Ddor','2/4',2))
        for parts in t.sections.values():
            for body in parts.values():
                bars=[b for b in body.split('|') if b.strip()]
                self.assertEqual(len(bars),8)
                self.assertTrue(all(music.eighths(b)==4 for b in bars))
        subject=t.sections['A']['oboe'].split('|')[:4]
        notes=re.findall(r'[A-Ga-g]',subject[0])
        self.assertEqual(notes[0],notes[1])  # repeated-note dance pickup
        self.assertEqual([b.strip() for b in t.sections['A']['clarinet'].split('|')[4:8]],
                         [b.strip() for b in t.sections['A']['oboe'].split('|')[:4]])
        beats=episode.load_episode('ants');seconds=sum(b.seconds for b in beats)-13*.4
        form=music.form_for(t,seconds)
        self.assertLessEqual(abs(music.tempo_for(t,len(form)*8,seconds)-110),5)

    @staticmethod
    def sounding_eighths(body):
        cursor=0; sounding=set()
        for token in re.findall(r"\[[^\]]+\]\d*|[A-Ga-gz][,']*\d*",body):
            length=int(music.eighths(token))
            if not token.startswith('z'):
                sounding.update(range(cursor,cursor+length))
            cursor+=length
        return sounding

    def test_reeds_overlap_then_answer_alone_with_equal_stereo_roles(self):
        t=episode.load_module('ants','tune').TUNE
        voices={v.name:v for v in t.voices}
        self.assertIn('oboe',voices)
        self.assertIn('clarinet',voices)
        self.assertNotEqual(voices['oboe'].program,voices['clarinet'].program)
        self.assertEqual(voices['oboe'].volume,voices['clarinet'].volume)
        self.assertLess(voices['oboe'].pan,64)
        self.assertEqual(voices['oboe'].pan+voices['clarinet'].pan,128)
        for section,caller,answer in [('A','oboe','clarinet'),('C','clarinet','oboe'),
                                      ('D','oboe','clarinet'),('E','oboe','clarinet'),
                                      ('F','oboe','clarinet'),('G','clarinet','oboe')]:
            with self.subTest(section=section):
                a=self.sounding_eighths(t.sections[section][caller])
                b=self.sounding_eighths(t.sections[section][answer])
                self.assertEqual(a & b,{14,15})  # Only the caller's final beat.
                self.assertEqual(b-a,set(range(16,32)))
        self.assertEqual(self.sounding_eighths(t.sections['B']['clarinet']),set())

    def test_continuous_arrangement_hands_the_phrase_between_reeds(self):
        tune=episode.load_module('ants','tune').TUNE
        beats=episode.load_episode('ants')
        seconds=sum(b.seconds for b in beats)-13*.4
        score=music.score(tune,seconds)
        voices=re.split(r'V:\d+ name="[^\"]+"\n',score)[1:]
        bodies=[' '.join(line for line in voice.splitlines()
                         if not line.startswith('%%')) for voice in voices]
        oboe=[bar.strip() for bar in bodies[0].split('|') if bar.strip()]
        clarinet=[bar.strip() for bar in bodies[1].split('|') if bar.strip()]
        self.assertEqual(len(oboe),88)
        self.assertEqual(len(clarinet),88)
        self.assertEqual(clarinet[4:8],oboe[:4])
        # Section C reverses the roles within the same uninterrupted score.
        self.assertEqual(oboe[36:40],clarinet[32:36])
        self.assertEqual(score.count('Q:'),1)
