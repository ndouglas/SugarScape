import unittest
import episode
from tests.test_retirement_dump import period

c = episode.load_module('retirement', 'claims') if (episode.episode_dir('retirement')/'claims.py').exists() else None


def fixture(ticks=3):
    ps=[period(t) for t in range(ticks+1)]
    return ps,{'retired':[0.]*len(ps),'retired_a':[0.]*len(ps),'retired_b':[0.]*len(ps)}


class RetirementClaimsTest(unittest.TestCase):
    def test_conditional_timing_preserves_censored_denominator(self):
        rows={3001:dict(first95=10),3002:dict(first95=None),3003:dict(first95=20)}
        result=c.summarize(rows,'first95',600)
        self.assertEqual(result['conditional_mean'],15)
        self.assertEqual(result['attained'],2)
        self.assertEqual(result['censored'],1)
        self.assertEqual(c.summarize({1:dict(first95=None)},'first95',600)['conditional_mean'],None)

    def test_null_mode_zero_exposure_and_youngest_tie_are_distinct(self):
        ps,ss=fixture();ps[1]['retirements_by_age'][45]=1;ps[1]['working_exposure_by_age'][45]=2
        ps[1]['retirements_by_age'][46]=1;ps[1]['working_exposure_by_age'][46]=1
        row=c.reduce_run(ps,ss,dict(ticks=3,config={},retirement_policy_at=None))
        self.assertIsNone(row['trace'][0]['mode'])
        self.assertEqual(row['trace'][1]['mode'],65)
        self.assertEqual(row['trace'][1]['age_rates'][45],.5)
        self.assertIsNone(row['trace'][1]['age_rates'][44])

    def test_group_crossings_and_policy_clocks_use_native_periods(self):
        ps,ss=fixture(201)
        for t in range(101,202):
            ps[t]['eligibility']=62;ps[t]['decision_eligibility']=65 if t==101 else 62
        ps[101]['policy_switched']=True
        ss['retired_a'][2]=.96;ss['retired_b'][3]=.97;ss['retired'][102]=.95
        row=c.reduce_run(ps,ss,dict(ticks=201,config={},retirement_policy_at=100))
        self.assertEqual(row['group_a95'],2)
        self.assertEqual(row['group_b95'],3)
        self.assertEqual(row['switch_tick'],101)
        self.assertEqual(row['post_decision_periods'],100)
        self.assertEqual(row['policy_new95'],1)

    def test_rolling_window_excludes_expired_events(self):
        ps,ss=fixture(11);ps[1]['retirements_by_age'][45]=2;ps[1]['working_exposure_by_age'][45]=2
        ps[2]['retirements_by_age'][46]=1;ps[2]['working_exposure_by_age'][46]=1
        row=c.reduce_run(ps,ss,dict(ticks=11,config={},retirement_policy_at=None))
        self.assertEqual(row['trace'][10]['mode'],65)
        self.assertEqual(row['trace'][11]['mode'],66)

    def test_seed_selection_uses_censors_only_for_distance_and_smallest_ties(self):
        rows={3002:dict(first95=12,horizon=20),3001:dict(first95=10,horizon=20),3003:dict(first95=None,horizon=20)}
        self.assertEqual(c.select_seed(rows,'first95'),3002)
        self.assertEqual(c.select_seed({3002:rows[3002],3001:rows[3001]},'first95'),3001)
        self.assertEqual(c.select_censored({3003:rows[3003],3004:dict(first95=None,horizon=20)},'first95'),3003)

    def gates(self, drop=.03, share=.15, favorable=50):
        runs={name:{} for name in ('quick','slow','eligible','all','groups05','groups20','policy_original','policy_revised')}
        for name,rows in runs.items():
            for seed in range(3101,3151):
                ok=seed<3101+favorable
                rows[seed]=dict(seed=seed,first95=20 if name=='slow' else 10,
                    group_a95=10 if name=='groups20' else 20,
                    group_b95=20 if name=='groups20' else 10,
                    pre_crossing_dip=2 if ok else None,minority_tick=2 if ok else None,
                    imitator_retirements=1 if ok else 0,switch_tick=101,post_decision_periods=100,
                    max_pre_crossing_drop=drop if ok else 0.,
                    minority_window=dict(tick=2,mode=65,share=share,rolling_events=5) if ok else None)
        cases=dict(eligible={'first95':{'attained':50}},all={'first95':{'attained':0}},
                   policy_original={'policy_new95':{'attained':50}},
                   policy_revised={'policy_new95':{'attained':19,'censored':31}})
        selected={name:dict(seed=3101) for name in runs}
        return {claim:holds for claim,holds,_ in c.caption_checks(runs,cases,selected)}

    def test_tiny_share_drop_cannot_establish_wavering(self):
        self.assertFalse(self.gates(drop=.000001)['Slow retirement wavers before spreading'])

    def test_ninety_four_percent_retired_is_not_a_small_minority(self):
        self.assertFalse(self.gates(share=.94)['Mode65 can describe a retiring minority'])

    def test_one_favorable_selected_run_cannot_pass_fifty_seed_caption_gate(self):
        gates=self.gates(favorable=1)
        self.assertFalse(gates['Slow retirement wavers before spreading'])
        self.assertFalse(gates['Mode65 can describe a retiring minority'])

    def test_eighty_percent_support_includes_exact_criterion_boundaries(self):
        gates=self.gates(drop=.02,share=.25,favorable=40)
        self.assertTrue(gates['Slow retirement wavers before spreading'])
        self.assertTrue(gates['Mode65 can describe a retiring minority'])

    def test_reduction_preserves_max_drop_before_crossing_and_concurrent_minor_window(self):
        ps,ss=fixture(6);ss['retired']=[0.,.20,.12,.99,.8,.9,.5]
        row=c.reduce_run(ps,ss,dict(ticks=6,config={},retirement_policy_at=None))
        self.assertAlmostEqual(row['max_pre_crossing_drop'],.08)
        self.assertEqual(row['max_drop_tick'],2)
        self.assertEqual(row['first95'],3)
        ps,ss=fixture();ss['retired']=[0.,.94,.25,.94]
        ps[1]['retirements_by_age'][45]=1;ps[1]['working_exposure_by_age'][45]=2
        ps[1]['imitator_retirements']=1
        row=c.reduce_run(ps,ss,dict(ticks=3,config={},retirement_policy_at=None))
        self.assertEqual(row['minority_window']['tick'],2)
        self.assertEqual(row['minority_window']['share'],.25)
        self.assertEqual(row['minority_window']['rolling_events'],1)
        self.assertEqual(row['minority_window']['exposure_by_age'][45],2)


class ReplayProtocolTest(unittest.TestCase):
    def test_empty_cache_freezes_replay_before_injected_collection(self):
        import tempfile
        from pathlib import Path
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as folder:
            cache=Path(folder)
            def collect(job):
                name,seed,protocol,target=job
                frozen=__import__('json').loads((target/'protocol.json').read_text())
                self.assertEqual(frozen['collection_kind'],'replay of known production outcomes')
                return name,seed,{'first95':None,'seed':seed}
            ps={'slow':{'seeds':[3001],'config':{},'ticks':600}}
            with patch.object(c,'CACHE',cache),patch.object(c,'confirmation_protocols',return_value={}),patch.object(c,'protocols',return_value=ps),patch.object(c,'file_digest',return_value='new-producer'),patch.object(c,'sample',side_effect=collect):
                freeze,saved,target=c.primary_corpus()
            self.assertTrue(target.name.startswith('replay-'))
            self.assertIsNone(saved['runs']['slow'][3001]['first95'])
            self.assertFalse((cache/c.PRIMARY_HASH[:16]).exists())

    def test_changed_producer_preserves_historical_cache(self):
        import tempfile,json
        from pathlib import Path
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as folder:
            cache=Path(folder);old=cache/c.PRIMARY_HASH[:16];old.mkdir()
            original={'protocol_sha256':c.PRIMARY_HASH,'cli_sha256':'old','cases':{}}
            (old/'protocol.json').write_text(json.dumps(original));(old/'reduced.json').write_text('{}')
            before=(old/'protocol.json').read_bytes()
            with patch.object(c,'CACHE',cache),patch.object(c,'confirmation_protocols',return_value={}),patch.object(c,'protocols',return_value={}),patch.object(c,'file_digest',return_value='new'),patch.object(c,'digest',side_effect=lambda value:c.PRIMARY_HASH if value=={k:v for k,v in original.items() if k!='protocol_sha256'} else 'different'):
                freeze,saved,target=c.primary_corpus()
            self.assertNotEqual(target,old)
            self.assertEqual((old/'protocol.json').read_bytes(),before)

    def test_matching_historical_producer_reuses_cache_without_collection(self):
        import tempfile,json
        from pathlib import Path
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as folder:
            cache=Path(folder);old=cache/c.PRIMARY_HASH[:16];old.mkdir()
            freeze={'protocol_sha256':c.PRIMARY_HASH,'cli_sha256':'same','cases':{}}
            (old/'protocol.json').write_text(json.dumps(freeze))
            (old/'reduced.json').write_text(json.dumps({'protocol':freeze,'runs':{}}))
            with patch.object(c,'CACHE',cache),patch.object(c,'protocols',return_value={}),patch.object(c,'file_digest',return_value='same'),patch.object(c,'digest',return_value=c.PRIMARY_HASH),patch.object(c,'sample') as collection:
                actual,saved,target=c.primary_corpus()
            self.assertEqual(target,old)
            collection.assert_not_called()
