"""Freeze native workloads from source extraction, before measuring.

The model is Rust; this script only records the registered arms and random prior.
"""
import argparse
import hashlib
import itertools
import json
import random
from pathlib import Path

SHARES=[0.,.05,.1,.2,.4,.6,.8,1.]
SPEC='docs/superpowers/specs/2026-10-02-emergent-polarity-design.md'


def build_manifest(source):
    arms=[]
    def add(family, name, config, sessions, **metadata):
        config={'model':'polarity','periods_per_tick':100,**config}
        arms.append({'id':f'{family}.{name}','family':family,'config':config,
                     'first_seed':(len(arms)+1)*1_000_000+1,'sessions':sessions,**metadata})

    for share,ratio,alliances in itertools.product(SHARES,[2,3],[False,True]):
        config={'predator_share':share,'superiority':ratio,'victory':ratio,'alliances':alliances}
        name=f'p{int(share*100):03}.r{ratio}.a{int(alliances)}'
        add('original',name,config,20)
        add('precision',name,config,200)
    for share,ratio in itertools.product(SHARES,[2,3]):
        base={'predator_share':share,'superiority':ratio,'victory':ratio,'alliances':False}
        name=f'p{int(share*100):03}.r{ratio}'
        add('radax',name,{**base,'action_memory':'war_until_victory'},20,
            evidence='Recovered design; no executable or exact output dataset recovered')
        add('pra',name,{**base,'source_profile':'chapter5','allocation':'pra'},20)
        for allocation,alliances in itertools.product(['equal','pra'],[False,True]):
            add('allocation',f'{name}.{allocation}.a{int(alliances)}',
                {**base,'source_profile':'chapter5','allocation':allocation,'alliances':alliances},200)
        switches={
            'schlieffen_gate':['previous_hostilities','unresolved_war'],
            'action_memory':['war_until_victory'],
            'victory_timing':['after_damage','after_harvest'],
            'resource_policy':['floor_zero','reject_nonpositive'],
            'capital_capture':['capture_and_fragment'],
            'province_transfer':['primitive_stock_only'],
            'locking':['affected_states'],'topology':['torus'],'update':['sequential']}
        for field,values in switches.items():
            for value in values:
                add('ambiguity',f'{name}.{field}.{value}',{**base,field:value},20,
                    baseline=f'original.{name}.a0')
        for field,values in {'trust_initial':[-1000,1000],
                             'threat_threshold':[-100,100],
                             'obligation_timing':['next_period']}.items():
            for value in values:
                add('alliance_ambiguity',f'{name}.{field}.{value}',
                    {**base,'alliances':True,field:value},20,
                    baseline=f'original.{name}.a1')

    tax_samples=[]
    for figure in source['figures']:
        if figure['id'].lower().replace(' ','').endswith('5.8'):
            tax_samples=[s for p in figure['panels'] for s in p['samples'] if 'tax_rate' in s]
    if not tax_samples:
        raise ValueError('No recoverable source Fig5.8 tax settings; extraction must precede manifest')
    provincial={'variant':'two_level','source_profile':'chapter5','allocation':'pra',
                'predator_share':1.,'superiority':2,'victory':2,'alliances':False,
                'horizon':1000,'stop_at_hegemony':True,'tax_discount':1.}
    for i,sample in enumerate(tax_samples):
        add('two_level',f'knot{i:02}',{**provincial,'tax_rate':sample['tax_rate']},20,
            source_sample=sample)
    for tax in [0.,.1,.2,.4,1.]:
        add('two_level_precision',f'tax{tax:g}',{**provincial,'tax_rate':tax},200)
    add('overextension','narrative_observation',
        {**provincial,'variant':'overextension','tax_rate':.4,'tax_discount':.7,
         'stochastic_threshold':3.,'stochastic_exponent':5.,'horizon':4000,
         'stop_at_hegemony':False},20,evidence='Descriptive; no unknown-seed trajectory judge')
    prior_rng=random.Random(2026100201)
    for i in range(110):
        initial=prior_rng.randint(5,200)
        harvest=prior_rng.randint(5,200)
        victory=prior_rng.randint(11,50)/10
        attack=prior_rng.randint(11,50)/10
        config={'initial_mean':initial,'initial_sd':prior_rng.randint(0,initial),
                'harvest_mean':harvest,'harvest_sd':prior_rng.randint(0,harvest),
                'victory':victory,'superiority':attack,
                'predator_share':prior_rng.randint(0,100)/100,
                'damage_rate':prior_rng.randint(1,100)/100,'alliances':True,
                'horizon':1000,'resource_distribution':'normal'}
        add('stoermer',f'configuration{i:03}',config,10,configuration=i,
            evidence='Adaptation of text prior; cardinal EPM, not appendix Python')
    for share,ratio in itertools.product(SHARES,[2,3]):
        add('support_control',f'p{int(share*100):03}.r{ratio}',
            {'predator_share':share,'superiority':ratio,'victory':ratio,
             'source_profile':'chapter5','allocation':'pra','alliances':True,
             'pra_alliance_support':'stocks'},20,
            evidence='Premasurement named pooling ambiguity; not part of main interaction family')
    return {'schema_version':1,'spec':SPEC,'analysis_draws':100_000,
            'analysis_seed':2026100202,'prior_seed':2026100201,
            'source_sha256':hashlib.sha256(json.dumps(source,sort_keys=True).encode()).hexdigest(),
            'seed_policy':'Nonoverlapping million-sized arm blocks; no replacement invalid runs',
            'unsupported_stoermer_dimensions':['trust/threat integer thresholds of different implementation',
                                               'spontaneous attack probability','stock tax','distance discount'],
            'arms':arms}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    manifest=build_manifest(json.loads(args.source.read_text()))
    args.output.write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(f"Frozen {len(manifest['arms'])} arms, {sum(a['sessions'] for a in manifest['arms'])} sessions")

if __name__=='__main__':
    main()
