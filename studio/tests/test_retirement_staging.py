"""The stage magnifies actual slots without inventing teaching state."""
import json
import pathlib
import unittest
import dump
from retirement_visual import teaching_members, nearby_ids, music_seconds, color, birth_visibility, contact_attainment, CastTimeline

ROOT = pathlib.Path(__file__).resolve().parents[1]

class StagingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.measured = json.loads((ROOT/'episodes/retirement/measurements.json').read_text())
        cls.d = dump.load(ROOT/'out/retirement/dumps/teaching.frames.json')

    def test_teaching_before_and_after_are_same_recorded_identities(self):
        event = self.measured['selected']['teaching']['decision']
        before = teaching_members(event)
        after = teaching_members(event, after=True)
        self.assertEqual(set(before), {event['id']} | {m['id'] for m in event['neighbors']})
        self.assertFalse(before[event['id']]['retired'])
        self.assertTrue(after[event['id']]['retired'])
        for m in event['neighbors']:
            self.assertEqual(before[m['id']], m)
            self.assertNotIn('kind', before[m['id']])
        self.assertEqual(color(before[7428]), 'slate')

    def test_habits_are_actual_eligible_members(self):
        ids = nearby_ids('habits', self.d, self.measured)
        self.assertEqual(ids, [4401, 4416, 4400])
        self.assertTrue(all(self.d.frames[1].members[i]['age'] >= 65 for i in ids))

    def test_late_cascade_cast_magnifies_current_eligible_slots(self):
        d=dump.load(ROOT/'out/retirement/dumps/quick.frames.json')
        for f in (d.frames[0],d.frames[len(d.frames)//2],d.frames[-1]):
            ids=nearby_ids('quick',d,self.measured,f)
            self.assertEqual(len(set(ids)),15)
            self.assertTrue(all(f.members[i]['age']>=f.retirement['eligibility'] for i in ids))

    def test_stable_group_seats_keep_the_current_holders_actual_contacts(self):
        import episode
        beat=next(b for b in episode.load_episode('retirement') if b.name=='groups')
        d=dump.load(ROOT/'out/retirement/dumps/groups05.frames.json')
        timing=beat.timing(d.ticks)
        timeline=CastTimeline(d,timing,beat.frames,nearby_ids('groups',d,self.measured,d.frames[0]),
                              lambda f:nearby_ids('groups',d,self.measured,f),groups=True)
        for index,ids in timeline.rows.items():
            f=d.frames[index];holder=f.members[ids[0]]
            contacts=list(dict.fromkeys(i for i in holder['network'] if i!=ids[0]))[:len(ids)-1]
            self.assertEqual(set(ids[1:1+len(contacts)]),set(contacts))
            self.assertTrue(holder['age']>=f.retirement['eligibility'])
            self.assertTrue(any(f.members[i]['group']!=holder['group'] for i in holder['network']))
            self.assertEqual(len(set(ids)),len(ids))

    def test_birth_transition_tracks_the_actual_source_change(self):
        import animate
        timing=animate.Timing(.22)
        boundary=timing.frame(.5)
        self.assertEqual(birth_visibility(self.d,timing,boundary,4047),0)
        self.assertEqual(birth_visibility(self.d,timing,boundary-12,4047),1)
        self.assertEqual(birth_visibility(self.d,timing,boundary+12,4047),1)
        self.assertAlmostEqual(birth_visibility(self.d,timing,boundary-6,4047),.5)
        self.assertAlmostEqual(birth_visibility(self.d,timing,boundary+6,4047),.5)

    def test_contact_context_covers_every_actual_group_and_treatment(self):
        cases=self.measured['cases']
        self.assertEqual(contact_attainment(cases),'All groups / contacts: 50/50 attained · 0 censored · horizon 600')
        import copy
        changed=copy.deepcopy(cases)
        changed['groups20']['group_b95']['censored']=1
        self.assertIn('group_b95: 50/50 attained · 1 censored at 600',contact_attainment(changed))

    def test_global_music_clock_has_no_beat_resets(self):
        names = ['ages','renewal','habits','decision','quick','slow','denominator','observable','groups','contact','policy','thresholds','meaning','end']
        starts = [0,7,13,21,29,36,43,50,57,64,71,77,85,92]
        self.assertEqual([music_seconds(n,1) for n in names], starts)
        self.assertAlmostEqual(music_seconds('decision',31),22)

if __name__ == '__main__': unittest.main()
