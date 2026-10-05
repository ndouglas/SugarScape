import copy
import json
import unittest

import dump


def period(tick):
    return dict(tick=tick, eligibility=65, decision_eligibility=65,
                policy_switched=False, retirements_by_age=[0]*81,
                working_exposure_by_age=[0]*81, retired=0., retired_a=0.,
                retired_b=0., imitator_retirements=0, decision=None)


def recording():
    agents=[dict(id=i,born=-20,age=20,kind='imitator',retired=False,
                 retired_at=None,group=0,threshold_units=500000,network=[(i+1)%243])
            for i in range(243)]
    periods=[period(t) for t in range(4)]
    frames=[dict(periods[t],agents=copy.deepcopy(agents),cohort_counts={'20':243}) for t in (0,3)]
    frames[1]['agents'][0]['born']=3
    return dict(format=1,model='retirement',seed=3001,ticks=1,every=3,agents=243,
                age_min=20,age_max=100,config={},stats={'modal_age':[None]*4},
                periods=periods,frames=frames,retirement_policy_at=None,policy_switched_at=None)


class RetirementDumpTest(unittest.TestCase):
    def parse(self,raw):
        return dump.parse(json.dumps(raw))

    def test_slot_zero_renewal_large_cohort_and_actual_clock_survive(self):
        d=self.parse(recording())
        self.assertEqual(d.ticks,1)
        self.assertEqual(d.frames[1].period,3)
        self.assertEqual(d.frames[1].members[0]['born'],3)
        self.assertEqual(d.frames[0].members[0]['born'],-20)
        self.assertEqual(len(d.frames[1].agents),243)
        self.assertEqual(len({(a.x,a.y) for a in d.frames[1].agents.values()}),243)
        self.assertEqual(d.frames[1].retirement['cohort_counts'],{'20':243})
        self.assertIsNone(d.frames[1].retirement['decision'])
        self.assertEqual(len(d.periods),4)

    def test_duplicate_missing_population_and_network_ids_are_rejected(self):
        for kind in ('duplicate','missing','network','cohort'):
            raw=recording(); fr=raw['frames'][1]
            if kind=='duplicate': fr['agents'][1]['id']=0
            if kind=='missing': fr['agents'].pop()
            if kind=='network': fr['agents'][0]['network']=[243]
            if kind=='cohort': fr['cohort_counts']['20']=242
            with self.subTest(kind=kind),self.assertRaises(ValueError):self.parse(raw)

    def test_event_counts_cannot_exceed_actual_exposure(self):
        raw=recording();raw['periods'][2]['retirements_by_age'][45]=1
        with self.assertRaises(ValueError):self.parse(raw)

    def test_decision_numerator_must_match_recorded_neighbors(self):
        raw=recording();event=dict(tick=3,id=0,born=3,age=65,eligibility=65,
            counts='eligible',threshold_units=500000,threshold_scale=1000000,
            counted=2,retired_counted=1,retired_before=False,retired_after=True,
            neighbors=[dict(id=1,born=-65,age=65,eligible=True,retired=True,counted=True),
                       dict(id=2,born=-64,age=64,eligible=False,retired=False,counted=False)])
        raw['periods'][3]['decision']=event;raw['frames'][1]['decision']=event
        with self.assertRaises(ValueError):self.parse(raw)
        event['counted']=1
        self.assertEqual(self.parse(raw).frames[1].retirement['decision']['retired_counted'],1)

    def test_exact_threshold_equality_retires_and_empty_denominator_does_not(self):
        raw=recording()
        event=dict(tick=3,id=0,born=3,age=65,eligibility=65,counts='eligible',
                   threshold_units=500000,threshold_scale=1000000,counted=2,
                   retired_counted=1,retired_before=False,retired_after=True,
                   neighbors=[dict(id=i,born=-65,age=65,eligible=True,retired=(i==1),counted=True)
                              for i in (1,2)])
        raw['periods'][3]['decision']=event;raw['frames'][1]['decision']=event
        self.assertTrue(self.parse(raw).frames[1].retirement['decision']['retired_after'])
        event.update(neighbors=[],counted=0,retired_counted=0,threshold_units=0,retired_after=False)
        self.assertFalse(self.parse(raw).frames[1].retirement['decision']['retired_after'])

    def test_stats_native_clock_is_independent_of_sampled_frame_count(self):
        raw=recording();raw['stats']['modal_age']=[None,None]
        with self.assertRaises(ValueError):self.parse(raw)
