"""Production corpus rules frozen October 3, before fresh native runs.

These rules follow the approved exploratory audit, not prospective paper tests.
Every failed seed remains in the corpus. No independent simulator is used.
"""
import concurrent.futures
import copy
import csv
import hashlib
import json
import math
import pathlib
import statistics
import subprocess
import time
from collections import Counter

import measure as m

RULES = dict(mean_margin=2, chance_center=.25, chance_margin=.05,
             paired_positive_fraction=.8, support_p=.01,
             selection='closest to ensemble median own-mean variance; smallest seed breaks ties',
             shared_event='first all-or-none round after burn-in in selected representative run',
             teaching='first played round after burn-in in selected run; actual decision snapshot supplied by exporter',
             window_rounds=240, bar_every=1, memory_every=5,
             revision='No post-production-result rule revisions')
BASE = dict(model='farol', game='el_farol', agents=100, strategies=12,
            behavior='inductive', capacity=60, scoring='error', decay=.9,
            at_capacity='stay', shared=False, memory=3,
            mixed_memory=dict(enabled=False,min=1,max=10), payoff='step',
            rounding='nearest', bias=.5, information='true',
            evolution=dict(enabled=False,every=100,strategy_mutation=.1,memory_mutation=0.),
            stop_at=0)
EPISODE = m.STUDIO/'episodes/farol'
CACHE = m.STUDIO/'out/farol/corpus'


def protocols():
    """Return full explicit configurations, independent of changing presets."""
    result={}
    for name in ('accuracy','advice','random','shared','m2','m6','m12'):
        c=copy.deepcopy(BASE)
        if name=='advice': c['scoring']='payoff'
        if name=='random': c['behavior']='random'
        if name=='shared': c['shared']=True
        memory=name.startswith('m')
        if memory: c.update(game='minority',agents=101,strategies=2,memory=int(name[1:]),capacity=50)
        result[name]=dict(config=c,seeds=list(range(2001,2033) if memory else range(1001,1021)),
                          ticks=10000 if memory else 2000,burn=2000 if memory else 400)
    return result


def digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def moments(values,center,population):
    """Population moments; lag-1 uses the full centered sum as denominator."""
    if not values: raise ValueError('No retained attendance outcomes')
    if any(not isinstance(v,int) or not 0<=v<=population for v in values):
        raise ValueError('Attendance outside population')
    mean=statistics.fmean(values)
    centered=[v-mean for v in values]
    ss=sum(v*v for v in centered)
    histogram=[0]*(population+1)
    for value in values: histogram[value]+=1
    return dict(samples=len(values),mean=mean,variance=ss/len(values),
                center_squared_deviation=statistics.fmean((v-center)**2 for v in values),
                variance_per_agent=ss/len(values)/population,
                center_squared_deviation_per_agent=statistics.fmean((v-center)**2 for v in values)/population,
                lag1=sum(a*b for a,b in zip(centered,centered[1:]))/ss if ss else None,
                histogram=histogram,all_or_none=sum(v in (0,population) for v in values),
                above_capacity=sum(v>center for v in values),at_center=sum(v==center for v in values))


def reduce_series(path,protocol):
    with path.open() as stream: rows=list(csv.DictReader(stream))
    clocks=[int(r['tick']) for r in rows]
    if clocks!=list(range(protocol['ticks']+1)):
        raise ValueError(f'{path}: expected every native clock 0..{protocol["ticks"]}')
    values=[int(r['attendance']) for r in rows[protocol['burn']+1:]]
    c=protocol['config']
    center=c['agents']/2 if c['game']=='minority' else c['capacity']
    row=moments(values,center,c['agents'])
    row.update(first_round=protocol['burn']+1,last_round=protocol['ticks'],
               initial_rows_excluded=1,burn_rows_excluded=protocol['burn'])
    return row,values


def select_seed(rows):
    median=statistics.median(r['variance'] for r in rows.values())
    return min(sorted(rows),key=lambda seed:abs(rows[seed]['variance']-median))


def near_mean(values,center):
    return abs(statistics.fmean(values)-center)<=RULES['mean_margin']


def near_chance(values):
    return abs(statistics.fmean(values)-RULES['chance_center'])<=RULES['chance_margin']+1e-12


def mw_greater(a,b):
    """Port of survey/src/stats.rs::mw_greater (exact <=20, else tie-normal)."""
    u=sum(1 if x>y else .5 if x==y else 0 for x in a for y in b)
    counts=Counter(a+b)
    na,nb=len(a),len(b)
    if len(counts)==na+nb and max(na,nb)<=20:
        maxu=na*nb
        previous=[[0]*(maxu+1) for _ in range(nb+1)]
        for k in range(nb+1): previous[k][0]=1
        for j in range(1,na+1):
            current=[[0]*(maxu+1) for _ in range(nb+1)]
            current[0][0]=1
            for k in range(1,nb+1):
                for w in range(j*k+1):
                    current[k][w]=(previous[k][w-k] if w>=k else 0)+current[k-1][w]
            previous=current
        return sum(previous[nb][math.ceil(u):])/sum(previous[nb])
    total=na+nb
    variance=na*nb/12*((total+1)-sum(t**3-t for t in counts.values())/(total*(total-1)))
    if variance<=0: return 1.
    z=(u-na*nb/2-.5)/math.sqrt(variance)
    return .5*math.erfc(z/math.sqrt(2))


def compare(a,b):
    if set(a)!=set(b): raise ValueError('Paired comparisons require identical seeds')
    seeds=sorted(a)
    av=[a[s]['variance'] for s in seeds]; bv=[b[s]['variance'] for s in seeds]
    diffs=[x-y for x,y in zip(av,bv)]
    positives=sum(d>0 for d in diffs)
    p=mw_greater(av,bv)
    return dict(holds=positives/len(seeds)>=RULES['paired_positive_fraction'] and p<RULES['support_p'],
                seeds=seeds,differences={str(s):d for s,d in zip(seeds,diffs)},
                positive_pairs=positives,pairs=len(seeds),mean_difference=statistics.fmean(diffs),
                mean_ratio=statistics.fmean(av)/statistics.fmean(bv) if statistics.fmean(bv) else None,
                mann_whitney_greater_p=p,
                support_method='survey stats::mw_greater; unpaired support accompanies paired >=80% ordering')


def shot_spec(name,seed,protocol):
    every=RULES['memory_every'] if name.startswith('m') else RULES['bar_every']
    start=protocol['burn']; end=start+RULES['window_rounds']
    shot=dict(config=protocol['config'],seed=seed,ticks=end,every=every)
    selected=dict(seed=seed,start_round=start,end_round=end,every=every,
                  start_frame=start//every,end_frame=end//every,
                  first_decision_round=start+1,source_config=protocol['config'],
                  source_config_sha256=digest(protocol['config']),rationale=RULES['selection'],
                  shot=f'shots/{name}.json',initial_tick_is_played_round=False)
    return shot,selected


def sample(job):
    name,seed,p,cache=job
    config=cache/f'{name}-{seed}.config.json'; series=cache/f'{name}-{seed}.csv'
    config.write_text(json.dumps(p['config'],indent=2)+'\n')
    subprocess.run([m.CLI,'run','--config',config,'--seed',str(seed),'--ticks',str(p['ticks']),
                    '--series-csv',series],check=True,stdout=subprocess.DEVNULL)
    row,trace=reduce_series(series,p)
    row.update(seed=seed,config_sha256=digest(p['config']),
               series_sha256=hashlib.sha256(series.read_bytes()).hexdigest(),
               provenance=dict(command=[str(m.CLI),'run','--config',str(config),'--seed',str(seed),
                                        '--ticks',str(p['ticks']),'--series-csv',str(series)],
                               raw_cache=str(series.relative_to(m.REPO)),
                               outcome_window=f'{p["burn"]+1}..{p["ticks"]}',sampling_interval=1))
    return name,seed,row,trace


def measure(tmp):
    started=time.monotonic()
    ps=protocols()
    protocol=dict(cases=ps,rules=copy.deepcopy(RULES),frozen_before_first_run=True,
                  provenance='Native CLI run only; fresh seed sets distinct from exploratory audit 1–32',
                  library='48 integer forecasts; 12 distinct held each (shared: all 48); random best ties',
                  statistics='Own-mean population variance and center squared deviation are separate; initial tick excluded',
                  retain_failures=True,protocol_sha256=digest(dict(cases=ps,rules=RULES)))
    cache=CACHE/protocol['protocol_sha256'][:16]; cache.mkdir(parents=True,exist_ok=True)
    # Persist the immutable protocol before launching any measured native process.
    (cache/'protocol.json').write_text(json.dumps(protocol,indent=2,sort_keys=True)+'\n')
    runs={name:{} for name in ps}; traces={name:{} for name in ps}
    jobs=[(name,seed,p,cache) for name,p in ps.items() for seed in p['seeds']]
    with concurrent.futures.ThreadPoolExecutor(4) as pool:
        for name,seed,row,trace in pool.map(sample,jobs):
            runs[name][seed]=row; traces[name][seed]=trace
    cases={}; histograms={}; selected={}
    for name,p in ps.items():
        rows=runs[name]; seed=select_seed(rows)
        histograms[name]=[sum(r['histogram'][i] for r in rows.values()) for i in range(p['config']['agents']+1)]
        cases[name]={key:statistics.fmean(r[key] for r in rows.values()) for key in
                     ('mean','variance','center_squared_deviation','variance_per_agent','center_squared_deviation_per_agent')}
        cases[name].update(seeds=len(rows),samples=sum(r['samples'] for r in rows.values()),
                           all_or_none=sum(r['all_or_none'] for r in rows.values()),
                           config=p['config'],histogram=histograms[name])
        shot,selection=shot_spec(name,seed,p)
        trace=traces[name][seed]
        selection['retained_trace']=dict(first_round=p['burn']+1,every=1,attendance=trace)
        selection['teaching_round']=p['burn']+1
        if name=='shared':
            selection['first_qualifying_round']=next((p['burn']+1+i for i,v in enumerate(trace) if v in (0,100)),None)
        (EPISODE/'shots'/f'{name}.json').write_text(json.dumps(shot,indent=2,sort_keys=True)+'\n')
        selected[name]=selection
    # An unsampled teaching horizon permits inspection of actual M=6 lookup at round 2001.
    teaching=dict(config=ps['m6']['config'],seed=selected['m6']['seed'],ticks=2001,every=1)
    (EPISODE/'shots/teaching.json').write_text(json.dumps(teaching,indent=2,sort_keys=True)+'\n')
    selected['teaching']=dict(seed=teaching['seed'],start_round=2000,end_round=2001,every=1,
                              start_frame=2000,end_frame=2001,source_config=teaching['config'],
                              rationale=RULES['teaching'],shot='shots/teaching.json')
    comparisons={label:compare(runs[a],runs[b]) for label,a,b in
                 [('accuracy_random','accuracy','random'),('accuracy_advice','accuracy','advice'),
                  ('short_middle','m2','m6'),('long_middle','m12','m6')]}
    for label,row in comparisons.items(): cases[label]=row
    checks=[]
    for name in ('accuracy','random'):
        holds=near_mean([r['mean'] for r in runs[name].values()],60)
        checks.append((f'{name}: attendance averages about 60',holds,f'ensemble mean {cases[name]["mean"]:.4f}; fixed ±2'))
    for label in comparisons:
        row=comparisons[label]
        checks.append((label.replace('_',' > ')+' variance',row['holds'],
                       f'{row["positive_pairs"]}/{row["pairs"]} positive paired differences; mean effect {row["mean_difference"]:.4f}; ratio {row["mean_ratio"]:.4f}; MW p={row["mann_whitney_greater_p"]:.3g}'))
    shared=cases['shared']; unanimous=shared['all_or_none']==shared['samples']
    checks.append(('Shared bank: all go or all stay in the measured windows',unanimous,
                   f'{shared["all_or_none"]}/{shared["samples"]} retained rounds unanimous; rounds 401–2000 only'))
    checks.append(('M12 coordination falls back near chance',near_chance([r['variance_per_agent'] for r in runs['m12'].values()]),
                   f'variance/N {cases["m12"]["variance_per_agent"]:.5f}; fixed .25 ± .05; center deviation/N {cases["m12"]["center_squared_deviation_per_agent"]:.5f}'))
    checks.append(('M6 splits almost evenly relative to the other tested memories',
                   comparisons['short_middle']['holds'] and comparisons['long_middle']['holds'] and
                   cases['m6']['variance_per_agent']<.2 and near_mean([r['mean'] for r in runs['m6'].values()],50.5),
                   f'mean {cases["m6"]["mean"]:.5f}; variance/N {cases["m6"]["variance_per_agent"]:.5f}; center deviation/N {cases["m6"]["center_squared_deviation_per_agent"]:.5f}; mean ±2 and below chance band'))
    lines=['## Frozen production protocol','',
           'All rules were written before these runs. Fresh bar seeds 1001–1020, rounds 401–2000; memory seeds 2001–2032, rounds 2001–10000. Native CLI only. Initial tick 0 and burn-in are excluded. All seeds, including failures, are retained.',
           '',f'Protocol SHA-256: `{protocol["protocol_sha256"]}`.',
           '', 'Mean margin ±2; chance variance/N margin .25 ± .05. Paired variance ordering requires ≥80% positive differences and survey Mann–Whitney one-sided p<.01. Own-mean variance differs from center squared deviation. These prospective production rules followed the exploratory audit; they are not tests of literal paper replication.',
           '', '| case | seeds | retained outcomes | mean | variance | center squared deviation | variance/N | selected seed |',
           '|---|---|---|---|---|---|---|---|']
    for name in ps:
        r=cases[name]
        lines.append(f'| {name} | {r["seeds"]} | {r["samples"]} | {r["mean"]:.5f} | {r["variance"]:.5f} | {r["center_squared_deviation"]:.5f} | {r["variance_per_agent"]:.5f} | {selected[name]["seed"]} |')
    lines+=['','Examples: closest to the ensemble median variance, smallest seed on ties. Film windows start at burn-in and include the next 240 rounds; start frame is context, not a newly played round. M=6 teaching uses round 2001 at every=1; all teaching readouts must come from the forthcoming actual decision exporter.',
            '', 'Shared-bank unanimity is restricted to the measured windows; selected event and complete retained selected traces are in measurements.json. Complete ensemble histogram counts sum to every retained outcome; each per-seed histogram and moment is retained. Raw CSV cache is under ignored studio/out.',
            '',f'Native collection/reduction runtime: {time.monotonic()-started:.2f} seconds.']
    protocol['runtime_seconds']=time.monotonic()-started
    protocol['cli_sha256']=hashlib.sha256(m.CLI.read_bytes()).hexdigest()
    data=dict(protocol=protocol,runs=runs,cases=cases,selected=selected,histograms=histograms,
              verdicts=[dict(claim=c,holds=h,evidence=e) for c,h,e in checks])
    return lines,checks,data
