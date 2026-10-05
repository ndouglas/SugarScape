import unittest
import episode
import music


class ThresholdsEpisodeTest(unittest.TestCase):
    def test_approved_storyboard_ends_with_standard_closing_card(self):
        beats=episode.load_episode('thresholds')
        self.assertEqual(len(beats),14)
        self.assertEqual(beats[-1].caption,'The riot that needs one person - After Granovetter, 1978; Watts, 2002\nndouglas.github.io/SugarScape')
        self.assertEqual(beats[-1].caption_y,.45)
        self.assertIsNone(beats[-1].shot)
        self.assertEqual(beats[12].compare,'dense-large')

    def test_missing_rung_has_complete_bars_and_fits_the_cut(self):
        tune=episode.load_module('thresholds','tune').TUNE
        self.assertEqual(tune.title,'The Missing Rung')
        for parts in tune.sections.values():
            for body in parts.values():
                bars=[bar for bar in body.split('|') if bar.strip()]
                self.assertEqual(len(bars),8)
                self.assertTrue(all(music.eighths(bar)==8 for bar in bars))
        seconds=sum(b.seconds for b in episode.load_episode('thresholds'))-13*.4
        form=music.form_for(tune,seconds)
        bpm=music.tempo_for(tune,len(form)*8,seconds)
        self.assertTrue(tune.bpm[0]<=bpm<=tune.bpm[2])

    def test_key_music_cues_start_at_the_actual_story_beats_and_fit(self):
        tune=episode.load_module('thresholds','tune').TUNE
        beats=episode.load_episode('thresholds')
        names=[b.name for b in beats]
        frames=[b.frames for b in beats]
        cues=music.cue_times(tune.cues,names,frames)
        keyed={beat:sting for beat,sting,*_ in tune.cues}
        self.assertEqual(set(keyed),{'chain','change','stalled','ceilings','sparse','middle','dense'})
        for beat,sting in keyed.items():
            index=names.index(beat)
            start=sum(f-12 for f in frames[:index])/30
            cue_start=dict(cues)[sting]
            self.assertAlmostEqual(cue_start,start)
            parts,bpm=tune.stings[sting]
            durations=[sum(music.eighths(bar) for bar in body.split('|'))/2*60/bpm for body in parts.values()]
            self.assertEqual(len(set(durations)),1)
            # FluidSynth release tail is separately verified from actual WAVs.
            self.assertLessEqual(durations[0]+2.5,(frames[index]-12)/30)
            self.assertIn(sting,tune.handoffs)

    def test_chain_and_missing_answer_cues_have_different_audible_entrances(self):
        tune=episode.load_module('thresholds','tune').TUNE
        chain=tune.stings['chain'][0]
        missing=tune.stings['change'][0]
        for voice in ('flute','vibes','pizz'):
            self.assertRegex(chain[voice],r'[A-Ga-g]')
        for voice in ('vibes','pizz'):
            self.assertEqual(missing[voice],'z8 | z8 |')

    def test_cued_textures_show_drop_gaps_fill_and_dense_late_answer(self):
        tune=episode.load_module('thresholds','tune').TUNE
        bars=lambda cue,voice:[s.strip() for s in tune.stings[cue][0][voice].split('|') if s.strip()]
        for voice in ('vibes','pizz'):
            self.assertRegex(bars('ceilings',voice)[0],r'[A-Ga-g]')
            self.assertEqual(bars('ceilings',voice)[1],'z8')
            self.assertNotIn('z',bars('middle',voice)[1])
            self.assertEqual(bars('dense',voice)[0],'z8')
        self.assertEqual(bars('sparse','pizz'),['z8','z8'])
        self.assertRegex(bars('dense','pizz')[-1],r'E A B c')

    def test_handoffs_silence_backing_at_onset_and_restore_it_after_score(self):
        tune=episode.load_module('thresholds','tune').TUNE
        cues=[('chain',12.2),('change',19.8),('stalled',25.4),
              ('ceilings',52.8),('sparse',67.),('middle',73.6),('dense',81.2)]
        # No WAV paths exist: handoffs must not consult synthesized tail length.
        ducks,fades=music.sting_envelopes(tune,cues,[('absent.wav',at) for _,at in cues])
        for i,((name,at),duration) in enumerate(zip(cues,[4.8,3.,4.,4.8,3.75,4.,6.])):
            with self.subTest(cue=name):
                self.assertAlmostEqual(music.duck_gain(at,ducks),0.)
                self.assertAlmostEqual(music.duck_gain(at-.4,ducks),.5)
                self.assertAlmostEqual(music.duck_gain(at+duration+.3,ducks),0.)
                self.assertAlmostEqual(music.duck_gain(at+duration+1.1,ducks),1.)
                self.assertEqual(fades[i],(duration+.3,.8))
