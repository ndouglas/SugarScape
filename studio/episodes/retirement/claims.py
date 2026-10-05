"""Frozen prospective production checks following the approved retirement audit.

Native CLI only. First95 is an eligible-retired proxy, not a persistent age norm.
All failed and right-censored seeds remain in the corpus.
"""
import concurrent.futures
import copy
import hashlib
import json
import pathlib
import statistics
import subprocess
import time

import dump
import measure as m

EPISODE = m.STUDIO / 'episodes/retirement'
CACHE = m.STUDIO / 'out/retirement/corpus'
BASE = dict(model='retirement', per_cohort=100, rational=.10, random=.05, p=.5,
            threshold=.5, spread=0., size=dict(min=10,max=25), extent=5,
            counts='eligible', renewal='slot', order='by_cohort', initial_deaths='literal',
            eligibility=65, mandatory=0, policy=dict(enabled=False,to=62),
            groups=dict(enabled=False,coupling=.1), norm=.95, stop_at_norm=False, stop_at=0)
RULES = dict(
    first95='First native period with eligible-retired share >= .95; no persistence claim',
    quick_slow='Median paired slow-first95 minus quick-first95 > 0; every pair attained',
    slow_wavering='Maximum adjacent pre-first95 eligible-share drop >= .02 in at least80% of50 confirmation runs',
    denominator='Eligible first95 attained count exceeds all-member count; report both distributions',
    minority='Positive imitator retirements over600 plus mode65/positive rolling10 events and simultaneous eligible-retired share <= .25 in at least80% of50 confirmation runs',
    groups='Every paired group crossing attained; median A(.20)-A(.05)<0 and median B(.20)-B(.05)>0',
    policy='Original post-switch first95 attainment count exceeds revised; revised has positive right-censors',
    selection='Closest to ensemble median first95; censor uses horizon+1 only for distance; smallest seed breaks ties',
    censored_selection='Smallest censored revised-policy seed',
    teaching='First recorded positive-denominator imitator retirement in selected eligible run, never change run to get event',
    rolling_window=10, mode_tie='youngest age', population=8100,
    revision='criterion revised after initial production results were known',
    wave_min_drop=.02,minority_max_share=.25,required_fraction=.8,confirmation_samples=50,
    confirmation_seeds=list(range(3101,3151)),native_end=600,
    policy_event_at=100, expected_switch=101, post_decisions=[102,201],
    samples=50, seeds=list(range(3001,3051)))


def protocols():
    result={}
    for name in ('quick','slow','eligible','all','groups05','groups20','policy_original','policy_revised'):
        c=copy.deepcopy(BASE)
        if name=='quick':c['rational']=.15
        if name=='slow':c['rational']=.05
        if name=='all':c['counts']='all'
        if name.startswith('groups'):
            c['groups']=dict(enabled=True,coupling=.05 if name=='groups05' else .20)
        policy=name.startswith('policy')
        if policy:
            c.update(rational=.05,mandatory=70)
            if name=='policy_revised':c.update(threshold=.75,spread=.14433756729740646)
        result[name]=dict(config=c,seeds=list(RULES['seeds']),ticks=201 if policy else 600,
                          retirement_policy_at=100 if policy else None,
                          every=201 if policy else 600,gifts=False)
    return result


def digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest()


def file_digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def crossing(values,start=0):
    return next((i for i in range(start,len(values)) if values[i] is not None and values[i]>=.95),None)


def reduce_run(periods,series,protocol):
    horizon=protocol['ticks']
    if [p['tick'] for p in periods]!=list(range(horizon+1)):
        raise ValueError('Production reduction requires every native clock through declared horizon')
    for key in ('retired','retired_a','retired_b'):
        if len(series[key])!=len(periods):raise ValueError('Native share series must have full period clock')
    events=[0]*81; exposure=[0]*81; trace=[]
    total_events=[0]*81; total_exposure=[0]*81
    for tick,p in enumerate(periods):
        for k,(e,x) in enumerate(zip(p['retirements_by_age'],p['working_exposure_by_age'])):
            if not 0<=e<=x:raise ValueError('Retirement events exceed working opportunities')
            events[k]+=e;exposure[k]+=x;total_events[k]+=e;total_exposure[k]+=x
        if tick>=RULES['rolling_window']:
            old=periods[tick-RULES['rolling_window']]
            for k in range(81):
                events[k]-=old['retirements_by_age'][k];exposure[k]-=old['working_exposure_by_age'][k]
        mode=20+max(range(81),key=lambda k:(events[k],-k)) if sum(events) else None
        trace.append(dict(tick=tick,eligibility=p['eligibility'],decision_eligibility=p['decision_eligibility'],
            share=series['retired'][tick],share_a=series['retired_a'][tick],share_b=series['retired_b'][tick],
            mode=mode,rolling_events=sum(events),events_by_age=events.copy(),exposure_by_age=exposure.copy(),
            age_rates=[e/x if x else None for e,x in zip(events,exposure)],
            imitator_retirements=p['imitator_retirements']))
    switches=[p['tick'] for p in periods if p['policy_switched']]
    if len(switches)>1:raise ValueError('Unexpected multiple native policy switches')
    switch=switches[0] if switches else None
    is_policy=protocol.get('retirement_policy_at') is not None
    if is_policy:
        if switch!=101 or any(p['decision_eligibility']!=65 for p in periods[1:102]) or any(
                p['decision_eligibility']!=62 for p in periods[102:202]):
            raise ValueError('Fixed100 policy did not switch at101 with100 decision periods102..201')
    post=crossing(series['retired'],switch+1) if switch is not None else None
    first=crossing(series['retired'])
    dip=next((t for t in range(1,first if first is not None else len(trace))
              if trace[t]['share']<trace[t-1]['share']),None)
    imitators=sum(p['imitator_retirements'] for p in periods)
    minority=next((t['tick'] for t in trace if t['mode']==65 and t['share']<.95 and t['rolling_events']>0),None)
    drops=[(max(0.,trace[t-1]['share']-trace[t]['share']),t)
           for t in range(1,first if first is not None else len(trace))]
    max_drop,max_drop_tick=max(drops,key=lambda pair:(pair[0],-pair[1])) if drops else (0.,None)
    small=next((t for t in trace if t['mode']==65 and t['share']<=RULES['minority_max_share']
                and t['rolling_events']>0),None) if imitators>0 else None
    window={k:small[k] for k in ('tick','mode','share','rolling_events','events_by_age','exposure_by_age')} if small else None
    return dict(max_pre_crossing_drop=max_drop,max_drop_tick=max_drop_tick,minority_window=window,
        horizon=horizon,observable_horizons={'policy_new95':100},first95=first,group_a95=crossing(series['retired_a']),
        group_b95=crossing(series['retired_b']),switch_tick=switch,
        policy_new95=post-switch if post is not None else None,
        post_decision_periods=sum(p['decision_eligibility']==62 for p in periods[1:]) if is_policy else None,
        final_share=series['retired'][-1],final_mode=trace[-1]['mode'],
        imitator_retirements=imitators,minority_tick=minority if imitators>0 else None,
        pre_crossing_dip=dip,events_by_age=total_events,exposure_by_age=total_exposure,trace=trace)


def summarize(rows,observable,horizon):
    values=[r[observable] for r in rows.values() if r[observable] is not None]
    return dict(total=len(rows),attained=len(values),censored=len(rows)-len(values),horizon=horizon,
                conditional_mean=statistics.fmean(values) if values else None,
                conditional_sd=statistics.stdev(values) if len(values)>1 else (0. if values else None),
                conditional_median=statistics.median(values) if values else None,
                min=min(values) if values else None,max=max(values) if values else None)


def select_seed(rows,observable):
    def value(r):return r[observable] if r[observable] is not None else r.get('observable_horizons',{}).get(observable,r['horizon'])+1
    median=statistics.median(value(r) for r in rows.values())
    return min(sorted(rows),key=lambda seed:abs(value(rows[seed])-median))


def select_censored(rows,observable):
    return next((s for s in sorted(rows) if rows[s][observable] is None),None)


def shot_spec(protocol,seed,ticks=None,every=None,gifts=False):
    spec=dict(config=protocol['config'],seed=seed,ticks=ticks or protocol['ticks'],
              every=every or protocol['every'],gifts=gifts)
    if protocol['retirement_policy_at'] is not None:spec['retirement_policy_at']=protocol['retirement_policy_at']
    return spec


def recorded(spec,cache,name):
    source=cache/f'{name}.shot.json'; target=cache/f'{name}.frames.json'
    source.write_text(json.dumps(spec,sort_keys=True,indent=2)+'\n')
    command=[str(m.CLI),'shot',str(source),'--out',str(target)]
    # Cache is reusable only for this exact protocol/spec/binary hash.
    if not target.exists():subprocess.run(command,check=True,stdout=subprocess.DEVNULL)
    raw=json.loads(target.read_text())
    return raw,dict(command=command,raw_cache=str(target.relative_to(m.REPO)),
                    raw_sha256=file_digest(target),shot_sha256=digest(spec),
                    config_sha256=digest(spec['config']))


def sample(job):
    name,seed,p,cache=job
    raw,provenance=recorded(shot_spec(p,seed),cache,f'{name}-{seed}')
    if raw['agents']!=8100 or len(raw['frames'])!=2 or any(
            len(f['agents'])!=8100 or {a['id'] for a in f['agents']}!=set(range(8100)) for f in raw['frames']):
        raise ValueError('Production compact dump must keep two full8100-slot populations')
    row=reduce_run(raw['periods'],raw['stats'],p);row.pop('trace')
    row.update(seed=seed,provenance=provenance,config_sha256=digest(p['config']))
    return name,seed,row


def paired_difference(runs,a,b,key):
    if set(runs[a])!=set(runs[b]):raise ValueError('Production comparisons require paired identical seeds')
    if any(r[key] is None for name in (a,b) for r in runs[name].values()):return None
    return statistics.median(runs[a][s][key]-runs[b][s][key] for s in sorted(runs[a]))


def qualifies_wave(row):
    return row['max_pre_crossing_drop']>=RULES['wave_min_drop']


def qualifies_minority(row):
    window=row['minority_window']
    return (row['imitator_retirements']>0 and window is not None and window['mode']==65
            and window['rolling_events']>0 and window['share']<=RULES['minority_max_share'])


def support(rows,predicate):
    successes=sum(bool(predicate(row)) for row in rows.values())
    total=len(rows)
    return dict(successes=successes,total=total,required_fraction=RULES['required_fraction'],
                holds=total==RULES['confirmation_samples'] and successes/total>=RULES['required_fraction'])


def caption_checks(runs,cases,selected,confirmation=None):
    quick=paired_difference(runs,'slow','quick','first95')
    group_a=paired_difference(runs,'groups20','groups05','group_a95')
    group_b=paired_difference(runs,'groups20','groups05','group_b95')
    slow=runs['slow'][selected['slow']['seed']]
    allrun=runs['all'][selected['all']['seed']]
    validation=confirmation if confirmation is not None else runs
    wave=support(validation['slow'],qualifies_wave);minority=support(validation['all'],qualifies_minority)
    original,revised=cases['policy_original']['policy_new95'],cases['policy_revised']['policy_new95']
    return [
        ('More early retirees: quick spreads faster',quick is not None and quick>0,f'median paired slow−quick first95={quick}'),
        ('Slow retirement wavers before spreading',wave['holds'] and qualifies_wave(slow),f'confirmation{wave["successes"]}/{wave["total"]} have max adjacent pre-first95 drop>=.02; selected primary seed{slow["seed"]} max drop{slow["max_pre_crossing_drop"]:.6f}; {RULES["revision"]}'),
        ('Counting younger friends changes cascade',cases['eligible']['first95']['attained']>cases['all']['first95']['attained'],f'eligible{cases["eligible"]["first95"]["attained"]}/50 versus all{cases["all"]["first95"]["attained"]}/50 attained by600'),
        ('Mode65 can describe a retiring minority',minority['holds'] and qualifies_minority(allrun),f'confirmation{minority["successes"]}/{minority["total"]} have positive imitator retirements and mode65/positive rolling events with share<=.25; selected primary seed{allrun["seed"]}; {RULES["revision"]}'),
        ('Contact helps A catch up while slowing B',group_a is not None and group_b is not None and group_a<0<group_b,f'median paired A(.20−.05)={group_a}, B(.20−.05)={group_b}'),
        ('Policy after fixed100: switch101 then100 decision periods',all(r['switch_tick']==101 and r['post_decision_periods']==100 for name in ('policy_original','policy_revised') for r in runs[name].values()),'decisions101 use65, decisions102–201 use62'),
        ('Raised thresholds leave many runs censored',original['attained']>revised['attained'] and revised['censored']>0,f'original{original["attained"]}/50; revised{revised["attained"]}/50; revised right-censors{revised["censored"]} at100 post-switch periods')]


PRIMARY_HASH='2fbcc46c9ce693d01b73fab501e5e6b0eb1dd2db744265d54e14e76a84dd5678'


def confirmation_protocols(primary):
    return {name:dict(copy.deepcopy(primary[name]),seeds=list(RULES['confirmation_seeds']))
            for name in ('slow','all')}


def confirmed_protocol(primary):
    frozen=dict(cases=confirmation_protocols(primary['cases']),rules=copy.deepcopy(RULES),
                revision=RULES['revision'],original_protocol_sha256=PRIMARY_HASH,
                cli_sha256=file_digest(m.CLI),reducer_sha256=file_digest(pathlib.Path(__file__)),
                frozen_before_confirmation_runs=True,retain_failures=True)
    frozen['protocol_sha256']=digest(frozen)
    return frozen


def cached_primary_row(name,seed,p,original):
    provenance=original['provenance'];path=m.REPO/provenance['raw_cache']
    if file_digest(path)!=provenance['raw_sha256'] or digest(shot_spec(p,seed))!=provenance['shot_sha256']:
        raise ValueError('Original native raw/spec provenance mismatch')
    raw=json.loads(path.read_text())
    row=reduce_run(raw['periods'],raw['stats'],p);row.pop('trace')
    row.update(seed=seed,provenance=copy.deepcopy(provenance),config_sha256=digest(p['config']))
    if any(row[k]!=v for k,v in original.items()):
        raise ValueError('New reduction unexpectedly changes an original400-run summary')
    return row


def primary_corpus():
    """Reuse verified history, or freeze a separately labeled reproducibility replay."""
    historical=CACHE/PRIMARY_HASH[:16]
    if (historical/'protocol.json').exists() and (historical/'reduced.json').exists():
        freeze=json.loads((historical/'protocol.json').read_text())
        if freeze['protocol_sha256']!=PRIMARY_HASH or digest({k:v for k,v in freeze.items() if k!='protocol_sha256'})!=PRIMARY_HASH:
            raise ValueError('Original immutable protocol hash mismatch')
        if digest(protocols())==digest(freeze['cases']) and file_digest(m.CLI)==freeze['cli_sha256']:
            saved=json.loads((historical/'reduced.json').read_text())
            if all((m.REPO/row['provenance']['raw_cache']).exists()
                   for rows in saved['runs'].values() for row in rows.values()):
                return freeze,saved,historical
    freeze=dict(cases=protocols(),rules=copy.deepcopy(RULES),
        confirmation_cases=confirmation_protocols(protocols()),
        cli_sha256=file_digest(m.CLI),reducer_sha256=file_digest(pathlib.Path(__file__)),
        historical_protocol_sha256=PRIMARY_HASH,revision=RULES['revision'],
        collection_kind='replay of known production outcomes',
        frozen_before_replay_runs=True,retain_failures=True)
    freeze['protocol_sha256']=digest(freeze)
    cache=CACHE/('replay-'+freeze['protocol_sha256'][:16]);cache.mkdir(parents=True,exist_ok=True)
    target=cache/'protocol.json';text=json.dumps(freeze,indent=2,sort_keys=True)+'\n'
    if target.exists() and target.read_text()!=text:raise ValueError('Replay freeze would change')
    if not target.exists():target.write_text(text)
    runs={name:{} for name in freeze['cases']}
    jobs=[(name,seed,p,cache) for name,p in freeze['cases'].items() for seed in p['seeds']]
    with concurrent.futures.ThreadPoolExecutor(4) as pool:
        for name,seed,row in pool.map(sample,jobs):runs[name][seed]=row
    return freeze,dict(protocol=copy.deepcopy(freeze),runs=runs),cache


def measure(tmp):
    started=time.monotonic()
    freeze,saved,cache=primary_corpus()
    replay=freeze.get('collection_kind')=='replay of known production outcomes'
    protocol=copy.deepcopy(saved['protocol']);ps=freeze['cases']
    # Freeze the revised criteria for only the100 fresh confirmation runs before
    # launching any new numerical process. The original400 freeze/cache stay intact.
    confirmation_protocol=(confirmed_protocol(freeze) if replay else
        json.loads((EPISODE/'measurements.json').read_text())['confirmation']['protocol'])
    if replay:
        confirmation_protocol.update(collection_kind='replay of known production outcomes',
            original_protocol_sha256=freeze['protocol_sha256'],frozen_before_confirmation_runs=False,
            frozen_before_replay_runs=True)
        confirmation_protocol.pop('protocol_sha256')
        confirmation_protocol['protocol_sha256']=digest(confirmation_protocol)
    confirmation_cache=CACHE/(('replay-confirmation-' if replay else 'confirmation-')+confirmation_protocol['protocol_sha256'][:16])
    confirmation_cache.mkdir(parents=True,exist_ok=True)
    target=confirmation_cache/'protocol.json'
    text=json.dumps(confirmation_protocol,indent=2,sort_keys=True)+'\n'
    if target.exists() and target.read_text()!=text:raise ValueError('Confirmation freeze would change')
    if not target.exists():target.write_text(text)
    (confirmation_cache/'RULES.md').write_text('# Frozen revised confirmation criteria\n\n'+
        RULES['revision']+'\n\n'+ '\n'.join(f'- {k}: {v}' for k,v in RULES.items())+'\n')
    runs={name:{int(seed):cached_primary_row(name,int(seed),p,row)
                for seed,row in saved['runs'][name].items()} for name,p in ps.items()}
    confirmation_runs={name:{} for name in confirmation_protocol['cases']}
    jobs=[(name,seed,p,confirmation_cache) for name,p in confirmation_protocol['cases'].items() for seed in p['seeds']]
    with concurrent.futures.ThreadPoolExecutor(4) as pool:
        for name,seed,row in pool.map(sample,jobs):confirmation_runs[name][seed]=row
    confirmation=dict(protocol=confirmation_protocol,runs=confirmation_runs,
        support=dict(slow=support(confirmation_runs['slow'],qualifies_wave),
                     all=support(confirmation_runs['all'],qualifies_minority)),
        cases={name:dict(first95=summarize(rows,'first95',600),
            max_drop_range=[min(r['max_pre_crossing_drop'] for r in rows.values()),max(r['max_pre_crossing_drop'] for r in rows.values())],
            minority_share_range=[min(r['minority_window']['share'] for r in rows.values() if r['minority_window'] is not None),
                                  max(r['minority_window']['share'] for r in rows.values() if r['minority_window'] is not None)]
                                  if any(r['minority_window'] is not None for r in rows.values()) else None)
               for name,rows in confirmation_runs.items()})
    cases={};selected={}
    for name,p in ps.items():
        rows=runs[name];observable='policy_new95' if name.startswith('policy') else 'first95'
        cases[name]={key:summarize(rows,key,100 if key=='policy_new95' else p['ticks']) for key in
                     ('first95','group_a95','group_b95','policy_new95')}
        cases[name].update(config=p['config'],config_sha256=digest(p['config']),
                           imitator_events_total=sum(r['imitator_retirements'] for r in rows.values()))
        seed=select_seed(rows,observable)
        horizon=201 if name.startswith('policy') else 120
        every=3 if name.startswith('policy') else 5
        spec=shot_spec(p,seed,horizon,every)
        (EPISODE/'shots'/f'{name}.json').write_text(json.dumps(spec,indent=2,sort_keys=True)+'\n')
        raw,provenance=recorded(spec,cache,f'film-{name}-{seed}')
        parsed=dump.parse(json.dumps(raw));trace=reduce_run(raw['periods'],raw['stats'],dict(p,ticks=horizon))['trace']
        selected[name]=dict(seed=seed,shot=f'shots/{name}.json',source_config=p['config'],
            config_sha256=digest(p['config']),observable=observable,rationale=RULES['selection'],
            first_period=0,last_period=horizon,every=every,frame_periods=[f.period for f in parsed.frames],
            retained_trace=trace,provenance=provenance)
        if name=='all':selected[name]['minority_tick']=rows[seed]['minority_tick']
        del parsed,raw
    censored=select_censored(runs['policy_revised'],'policy_new95')
    if censored is not None:
        p=ps['policy_revised'];spec=shot_spec(p,censored,201,3)
        (EPISODE/'shots/policy_censored.json').write_text(json.dumps(spec,indent=2,sort_keys=True)+'\n')
        raw,provenance=recorded(spec,cache,f'film-policy-censored-{censored}')
        selected['policy_censored']=dict(seed=censored,shot='shots/policy_censored.json',
            rationale=RULES['censored_selection'],first_period=0,last_period=201,every=3,
            frame_periods=[f['tick'] for f in raw['frames']],source_config=p['config'],
            config_sha256=digest(p['config']),provenance=provenance,
            retained_trace=reduce_run(raw['periods'],raw['stats'],p)['trace'])
        del raw
    # Inspect the fixed candidate's recorded compact events before exporting a short
    # unsampled window. This changes storage only, never the event/seed selection.
    p=ps['eligible'];seed=selected['eligible']['seed']
    probe=shot_spec(p,seed,120,120,True)
    raw,_=recorded(probe,cache,f'teaching-probe-{seed}')
    event=next((r['decision'] for r in raw['periods'] if r['decision'] is not None
                and r['decision']['counted']>0 and r['decision']['retired_after']),None)
    teaching_end=event['tick']+1 if event is not None else 2
    del raw
    spec=shot_spec(p,seed,teaching_end,1,True)
    (EPISODE/'shots/teaching.json').write_text(json.dumps(spec,indent=2,sort_keys=True)+'\n')
    raw,provenance=recorded(spec,cache,f'film-teaching-{seed}')
    parsed=dump.parse(json.dumps(raw))
    renewal=next((dict(tick=f.period,id=i,before_born=parsed.frames[k-1].members[i]['born'],after_born=f.members[i]['born'])
                  for k,f in enumerate(parsed.frames[1:],1) for i in sorted(f.born)),None)
    selected['teaching']=dict(seed=seed,shot='shots/teaching.json',first_period=0,last_period=teaching_end,every=1,
        decision=event,renewal=renewal,rationale=RULES['teaching'],provenance=provenance,
        source_config=p['config'],config_sha256=digest(p['config']),
        frame_periods=[f.period for f in parsed.frames])
    del parsed,raw
    selected['slow']['wavering_example']=dict(seed=selected['slow']['seed'],
        max_adjacent_drop=runs['slow'][selected['slow']['seed']]['max_pre_crossing_drop'],
        tick=runs['slow'][selected['slow']['seed']]['max_drop_tick'],criterion=RULES['slow_wavering'])
    allrow=runs['all'][selected['all']['seed']]
    filmed=next((t for t in selected['all']['retained_trace'] if t['tick'] in selected['all']['frame_periods']
                 and t['mode']==65 and t['share']<=RULES['minority_max_share'] and t['rolling_events']>0),None)
    selected['all']['minority_example']=dict(imitator_retirements_over600=allrow['imitator_retirements'],
        window={k:filmed[k] for k in ('tick','mode','share','rolling_events','events_by_age','exposure_by_age')} if filmed else None,
        criterion=RULES['minority'])
    checks=caption_checks(runs,cases,selected,confirmation_runs)
    checks.append(('Recorded actual imitation teaching event',event is not None,
                   f'fixed selected eligible seed{seed}; first recorded qualifying retirement at{event["tick"] if event else None}'))
    confirmation['runtime_seconds']=time.monotonic()-started
    lines=['## Frozen native production protocol','',f'Protocol SHA-256: `{protocol["protocol_sha256"]}`.',
        '', 'Fresh paired seeds3001–3050,50 per case; C100 and all8100 agents. Native CLI only. Explicit expanded configurations are retained in measurements.json and the ignored protocol freeze. Failed/censored observations are retained. These are prospective production checks following the audit, not source-defined norm tests.',
        '', 'First95 is the first eligible-retired95% crossing. Event mode uses rolling10 actual periods, youngest age on ties, null when no events. Age-specific rates use retirement events / actual working exposure (including mandatory retirement); zero exposure gives null. Native stock shares and event rates are separate.',
        '', 'Policy disabled through100, enabled after100: actual switch101; decisions101 still use65, decisions102–201 use62, exactly100 post-switch decision periods. Fixed warm-up does not prove an established age65 norm.',
        '', '| case | observable | attained/total | right-censored | horizon | conditional mean | sample SD | conditional median | selected seed |',
        '|---|---|---|---|---|---|---|---|---|']
    for name in ps:
        key='policy_new95' if name.startswith('policy') else 'first95';r=cases[name][key]
        lines.append(f'| {name} | {key} | {r["attained"]}/{r["total"]} | {r["censored"]} | {r["horizon"]} | {r["conditional_mean"]} | {r["conditional_sd"]} | {r["conditional_median"]} | {selected[name]["seed"]} |')
    lines+=['','For groups, both A/B outcomes and all seed values are retained in measurements.json. Example selection minimizes distance to the ensemble median; censored values use horizon+1 solely for selection, never in conditional timing means. Smallest seed breaks ties. The smallest censored revised-policy example is separate and explicitly labeled.',
        '',f'Full-population filming is sampled (every5, horizon120; policy every3 through201). Teaching/renewal uses unsampled0 through the first recorded qualifying event plus one from the selected eligible seed; a compact120-period teaching probe determines the fixed event, not a different seed. Actual frame-period lists and activation-local teaching inputs are retained; no decision is inferred from final neighbor states.',
        '',f'Ignored reproducibility cache: `{cache.relative_to(m.REPO)}`. Each raw-file/config/shot hash and exact command is retained. Current cached reduction/confirmation runtime: {confirmation["runtime_seconds"]:.2f}s.']
    lines+=['','## Revised criteria and fresh confirmation','',RULES['revision']+'.',
        '',f'Confirmation protocol SHA-256: `{confirmation_protocol["protocol_sha256"]}`; frozen before fresh paired seeds3101–3150,50 per slow/all case, horizon600. Original400-run freeze and all raw outputs are preserved. Only these100 new native cases were collected.',
        '', 'Wavering requires max adjacent pre-first95 eligible-share drop>=.02 in at least80% of50 runs. Small minority requires positive imitator retirement events over600 and a mode65/positive rolling10-event window with concurrent native eligible-retired share<=.25 in at least80% of50 runs. These are declared production bounds, not source-defined norm criteria. The original median-selected illustrations must also satisfy the stronger bounds.',
        '', '| confirmation | successes/total | first95 attained/total | right-censored | measured range |',
        '|---|---|---|---|---|']
    for name in ('slow','all'):
        summary=confirmation['cases'][name];gate=confirmation['support'][name];timing=summary['first95']
        observed=summary['max_drop_range'] if name=='slow' else summary['minority_share_range']
        lines.append(f'| {name} | {gate["successes"]}/{gate["total"]} | {timing["attained"]}/{timing["total"]} | {timing["censored"]} | {observed} |')
    lines+=['',f'Original slow selected seed{selected["slow"]["seed"]}: maximum adjacent pre-first95 drop{selected["slow"]["wavering_example"]["max_adjacent_drop"]:.6f} at{selected["slow"]["wavering_example"]["tick"]}.',
        '',f'Original all-member selected seed{selected["all"]["seed"]}: {selected["all"]["minority_example"]["imitator_retirements_over600"]} actual imitator retirement events over600; sampled illustration window `{selected["all"]["minority_example"]["window"]}`.',
        '',f'Confirmation raw/spec hashes and exact commands: `{confirmation_cache.relative_to(m.REPO)}`.']
    if replay:
        lines=['## Reproducibility replay', '', 'Replay of known production outcomes using retained seeds3001–3050 and3101–3150. Current producer, reducer, explicit configurations and revised rules were frozen before collection in a separate namespace. This replay is not fresh prospective confirmation; the historical400-run production and100-run confirmation provenance remains unchanged. Historical chronology is documented in the approved spike and original retained protocol.']+lines
        lines=[line.replace('Fresh paired seeds3001–3050','Retained paired seeds3001–3050').replace('These are prospective production checks following the audit','These are reproducibility replays of known production outcomes').replace('## Revised criteria and fresh confirmation','## Revised criteria: confirmation-seed replay').replace('frozen before fresh paired seeds3101–3150','frozen before replay of retained paired seeds3101–3150').replace('Only these100 new native cases were collected.','These100 confirmation-seed outcomes were replayed; no new prospective confirmation is claimed.') for line in lines]
    data=dict(protocol=protocol,current_rules=copy.deepcopy(RULES),rule_revision=RULES['revision'],
              confirmation=confirmation,runs=runs,cases=cases,selected=selected,
              verdicts=[dict(claim=c,holds=h,evidence=e) for c,h,e in checks])
    (confirmation_cache/'reduced.json').write_text(json.dumps(data,sort_keys=True,allow_nan=False)+'\n')
    return lines,checks,data
