"""Prospectively frozen episode 8 protocol; native model only.

Post-initial ticks are measured without burn-in; every outcome is retained.
Exact curves use the model's detailed-balance rates, not a proxy simulation.
"""
import concurrent.futures
import csv
import hashlib
import json
import math
import pathlib
import statistics
import subprocess
import time

import measure as m

SEEDS = list(range(200001, 200021))
SHORT_SEEDS = list(range(200001, 201001))
BASE = dict(model='ants', ants=100, rule='kirman', epsilon=.002, delta=.01,
            conversion='kirman', meetings=50, sources=2, pull=0., a=.5,
            **{'lambda':1.}, network='complete', degree=10, link=.1,
            independent=0., start='random', stop_at=0)
RULES = dict(strong_extreme_min=.65, strong_flip_runs_min=18,
             strong_tv_max=.03, strong_modes=[0,100], short_middle_min=.15,
             short_middle_max=.35, short_iqr_min=.20, pull_tv_max=.04,
             pull_modes=[18,82], large_extreme_ratio_max=.5,
             ring_variance_ratio_max=.15, random_variance_ratio_min=.7,
             random_variance_ratio_max=1.4, random_ring_ratio_min=5.,
             independent_variance_ratio_max=.5)
HERE = pathlib.Path(__file__).resolve().parent
CACHE = m.REPO / '.superpowers/sdd/2026-10-02-ants-episode/measure-cache'


def protocols():
    def p(ticks, **changes):
        cfg = dict(BASE, **changes)
        return dict(config=cfg, ticks=ticks, seeds=SEEDS[:], burn_in=0,
                    sampling='every post-initial true tick',
                    time_unit='sweep' if cfg['rule']=='alfarano' else ('meeting' if cfg['meetings']==1 else 'step'),
                    updates_per_tick=cfg['ants'] if cfg['rule']=='alfarano' else cfg['meetings'])
    out = dict(micro=p(1000, meetings=1), strong=p(200000), short=p(2000),
               pull=p(20000, epsilon=.15, delta=.3, pull=1.), large=p(20000, ants=1000, meetings=500))
    out['short']['seeds'] = SHORT_SEEDS[:]
    for network in ('ring','random'):
        for n in (50,550,1050):
            out[f'{network}_{n}'] = p(300000, rule='alfarano', network=network, ants=n)
    for q in (0.,.05):
        out[f'independent_{q:g}'] = p(300000, rule='alfarano', network='random', ants=1000, independent=q)
    return out


def judge_strong(extreme, flip_runs, tv, modes):
    return extreme > .65 and flip_runs >= 18 and tv <= .03 and modes == [0,100]


def judge_short(middle, iqr):
    return .15 <= middle <= .35 and iqr > .20


def judge_pull(tv, modes):
    return tv <= .04 and modes == [18,82]


def judge_growth(ring_ratio, random_ratio, random_ring):
    return ring_ratio < .15 and .7 <= random_ratio <= 1.4 and random_ring > 5


def judge_large(small_extreme, large_extreme, small_variance, large_variance):
    return large_extreme < small_extreme/2 and large_variance < small_variance


def judge_independent(ratio):
    return ratio < .5


def select_event(rows, kind):
    for seed in sorted(rows):
        eligible = [e for e in rows[seed] if e['kind']==kind]
        if eligible:
            return seed, min(eligible, key=lambda e:(e['tick'],e['update']))
    raise ValueError(f'no measured {kind} event')


def reduce_series(path, n, ticks):
    hist = [0]*(n+1)
    count = 0
    mean = ss = 0.
    extreme = flips = 0
    regime = None
    first_flip = None
    last_extreme_tick = None
    trace = []
    flip_trace = None
    # Keep the preceding 2001 actual ticks for a deterministic flip window.
    from collections import deque
    recent = deque(maxlen=2001)
    with open(path) as f:
        for row in csv.DictReader(f):
            tick = int(row['tick'])
            if tick == 0:
                continue
            if tick != count + 1:
                raise ValueError(f'held or missing tick in {path}: {tick}, expected {count+1}')
            x = float(row['share'])
            k = round(x*n)
            hist[k] += 1
            count += 1
            d = x-mean
            mean += d/count
            ss += d*(x-mean)
            recent.append([tick,k])
            new = 0 if k*5 <= n else 1 if k*5 >= 4*n else None
            if new is not None:
                extreme += 1
                if regime is not None and new != regime:
                    flips += 1
                    if first_flip is None:
                        first_flip = dict(from_regime=regime,to_regime=new,
                                          departure_tick=last_extreme_tick, arrival_tick=tick)
                        flip_trace = list(recent)
                regime = new
                last_extreme_tick = tick
            if tick % max(1,ticks//200)==0:
                trace.append([tick,k])
    if count != ticks:
        raise ValueError(f'incomplete horizon {path}: {count}/{ticks}')
    return dict(samples=count, actual_ticks=count, histogram=hist, mean=mean,
                variance=ss/count, extreme=extreme/count, flips=flips,
                first_flip=first_flip, flip_trace=flip_trace, trace=trace)


def exact(config):
    n, eps, delta, pull = (config[k] for k in ('ants','epsilon','delta','pull'))
    def rates(k):
        x = k/n
        def join(a,b):
            return min(1.,eps+min(1.,max(0.,(1-delta)*(1+pull*(b-a)))))-eps
        return ((1-x)*(eps+k/(n-1)*join(1-x,x)),
                x*(eps+(n-k)/(n-1)*join(x,1-x)))
    logs = [0.]
    for k in range(n):
        logs.append(logs[-1]+math.log(rates(k)[0]/rates(k+1)[1]))
    probs = [math.exp(v-max(logs)) for v in logs]
    total = sum(probs)
    return [v/total for v in probs]


def aggregate(rows):
    hist = [sum(r['histogram'][k] for r in rows.values()) for k in range(len(next(iter(rows.values()))['histogram']))]
    count = sum(hist)
    variances = [r['variance'] for r in rows.values()]
    return dict(runs=len(rows), samples=count, histogram=hist,
                distribution=[v/count for v in hist],
                mean_extreme=statistics.mean(r['extreme'] for r in rows.values()),
                mean_variance=statistics.mean(variances),
                variance_se=statistics.stdev(variances)/math.sqrt(len(rows)),
                median_variance=statistics.median(variances),
                flip_runs=sum(r['flips']>0 for r in rows.values()))


def run_one(name, p, seed):
    CACHE.mkdir(parents=True,exist_ok=True)
    key = hashlib.sha256(json.dumps([p,seed],sort_keys=True).encode()).hexdigest()[:20]
    cache = CACHE/f'{name}-{seed}-{key}.json'
    if cache.exists():
        result = json.loads(cache.read_text())
        if result['samples'] != p['ticks'] or result['actual_ticks'] != p['ticks'] or sum(result['histogram']) != p['ticks']:
            raise ValueError(f'incomplete cached native outcome: {cache}')
        result['trace'] = [pair for pair in result['trace'] if pair[0] % max(1,p['ticks']//200)==0]
        return seed,result
    cfg = CACHE/f'{name}-{seed}.config.json'
    csvpath = CACHE/f'{name}-{seed}.csv'
    cfg.write_text(json.dumps(p['config']))
    started = time.monotonic()
    subprocess.run([m.CLI,'run','--config',cfg,'--seed',str(seed),'--ticks',str(p['ticks']),
                    '--series-csv',csvpath],check=True,stdout=subprocess.DEVNULL)
    result = reduce_series(csvpath,p['config']['ants'],p['ticks'])
    result['elapsed_seconds'] = time.monotonic()-started
    cache.write_text(json.dumps(result))
    csvpath.unlink()
    return seed,result


def measure_micro(p, tmp):
    rows = {}
    counts = {}
    for seed in p['seeds']:
        spec = dict(preset='ants-2b', seed=seed,ticks=p['ticks'],every=1000,gifts=True,
                    set=p['config'])
        spec['set'] = {k:v for k,v in spec['set'].items() if k!='model'}
        src,out = tmp/f'micro-{seed}.json',tmp/f'micro-{seed}.frames.json'
        src.write_text(json.dumps(spec))
        subprocess.run([m.CLI,'shot',src,'--out',out],check=True,stdout=subprocess.DEVNULL)
        data = json.loads(out.read_text())
        events = [e for frame in data['frames'] for e in frame['ants_events']]
        rows[seed] = events
        counts[seed] = {kind:sum(e['kind']==kind for e in events) for kind in ('recruit','spontaneous')}
    return rows,counts


def write_shot(name, seed, ticks, every, cfg, gifts=False):
    spec = dict(preset='ants-2b',seed=seed,ticks=ticks,every=every,gifts=gifts,
                set={k:v for k,v in cfg.items() if k!='model'})
    (HERE/'shots'/f'{name}.json').write_text(json.dumps(spec,indent=2)+'\n')
    return dict(file=f'shots/{name}.json',seed=seed,ticks=ticks,every=every,
                frames=ticks//every+1, updates_per_tick=cfg['ants'] if cfg['rule']=='alfarano' else cfg['meetings'],
                time_unit='sweeps' if cfg['rule']=='alfarano' else ('meetings' if cfg['meetings']==1 else 'steps'),example=True)


def evaluate(data):
    a = data['aggregates']; ex=data['exact']; rows=data['runs']
    tv = lambda key: sum(abs(x-y) for x,y in zip(a[key]['distribution'],ex[key]['distribution']))/2
    modes = lambda key: ex[key]['modes']
    middle = sum(.4<=r['mean']<=.6 for r in rows['short'].values())/1000
    means = sorted(r['mean'] for r in rows['short'].values())
    # Median of lower/upper halves; frozen quantile convention.
    iqr = statistics.median(means[500:])-statistics.median(means[:500])
    ring = a['ring_1050']['mean_variance']/a['ring_50']['mean_variance']
    random = a['random_1050']['mean_variance']/a['random_50']['mean_variance']
    cross = a['random_1050']['mean_variance']/a['ring_1050']['mean_variance']
    independent = a['independent_0.05']['mean_variance']/a['independent_0']['mean_variance']
    values = dict(strong_tv=tv('strong'),pull_tv=tv('pull'),short_middle_count=round(middle*1000),
                  short_iqr=iqr,ring_ratio=ring,random_ratio=random,random_ring_ratio=cross,
                  independent_ratio=independent)
    data['judge_values']=values
    return [
        ('observed 80–20 premise is source attribution; model has two symmetric sources and actual recorded teaching events',
         all(p['config']['sources']==2 for p in data['protocols'].values()) and
         len(data['micro']['event_counts'])==20 and all(data['selected'][name]['event']['kind']==kind for name,kind in [('micro-recruit','recruit'),('micro-self','spontaneous')]),
         'Kirman real-ant premise is not a simulator verdict; two-source configs; 20 actual one-meeting event records; deterministic exporter recording tests'),
        ('strong recruitment, flips and exact all-or-nothing peaks',judge_strong(a['strong']['mean_extreme'],a['strong']['flip_runs'],tv('strong'),modes('strong')),f"extreme {a['strong']['mean_extreme']:.6f}; flips {a['strong']['flip_runs']}/20; TV {tv('strong'):.6f}; modes {modes('strong')}"),
        ('finite records need not average half and half',judge_short(middle,iqr),f'{round(middle*1000)}/1000 compatible means; IQR {iqr:.6f}'),
        ('our majority-pull formula favors 18–82',judge_pull(tv('pull'),modes('pull')),f"TV {tv('pull'):.6f}; modes {modes('pull')}"),
        ('ten times the agents spend less time crowded',judge_large(a['strong']['mean_extreme'],a['large']['mean_extreme'],a['strong']['median_variance'],a['large']['median_variance']),f"extreme {a['large']['mean_extreme']:.6f} versus {a['strong']['mean_extreme']:.6f}; median variances {a['large']['median_variance']:.6f}, {a['strong']['median_variance']:.6f}"),
        ('growth weakens rings; random networks keep swings',judge_growth(ring,random,cross),f'ring ratio {ring:.6f}; random ratio {random:.6f}; random/ring {cross:.6f}'),
        ('5% independent agents calm the crowd',judge_independent(independent),f'variance ratio {independent:.6f}'),
    ]


def measure(tmp):
    p = protocols()
    # Freeze the complete protocol on disk before the first episode run.
    CACHE.mkdir(parents=True,exist_ok=True)
    frozen = dict(protocols=p,rules=RULES,quantiles='medians of sorted lower/upper 500 means',
                  selected_rule='smallest eligible seed; earliest recorded event/flip')
    (CACHE/'frozen-protocol.json').write_text(json.dumps(frozen,indent=2,sort_keys=True)+'\n')
    data = dict(**frozen,runs={},aggregates={},exact={},selected={},micro={},
                provenance=dict(native_cli=str(m.CLI), native_cli_sha256=hashlib.sha256(m.CLI.read_bytes()).hexdigest(),
                                approved_spec='docs/superpowers/specs/2026-10-02-ants-spike.md',
                                command='python3 studio/measure.py ants'))
    micro, counts = measure_micro(p['micro'],tmp)
    data['micro']=dict(event_counts=counts,events=micro)
    for kind,name in [('recruit','micro-recruit'),('spontaneous','micro-self')]:
        seed,event=select_event(micro,kind)
        selected=write_shot(name,seed,event['tick'],1,p['micro']['config'],True)
        selected['event']=event
        selected['window']=[max(0,event['tick']-1),event['tick']]
        selected['selection']='smallest eligible measured seed, earliest actual event'
        data['selected'][name]=selected
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        for name, protocol in p.items():
            if name=='micro':
                continue
            print(f'Measuring {name}: {len(protocol["seeds"])} seeds × {protocol["ticks"]} actual ticks',flush=True)
            rows=dict(pool.map(lambda seed:run_one(name,protocol,seed),protocol['seeds']))
            data['runs'][name]=rows
            data['aggregates'][name]=aggregate(rows)
            (CACHE/'partial-measurements.json').write_text(json.dumps(data))
            print(f'Completed {name}',flush=True)
    for name in ('strong','pull'):
        dist=exact(p[name]['config'])
        data['exact'][name]=dict(distribution=dist,modes=[i for i,v in enumerate(dist) if abs(v-max(dist))<1e-12],
                                provenance='detailed balance of actual capped Kirman transition rates')
    verdicts=evaluate(data)
    print('\n'.join(f'{"HOLDS" if ok else "FAIL"}: {claim}: {why}' for claim,ok,why in verdicts),flush=True)
    # Select a real first regime flip from the retained macro corpus.
    seed=min(s for s,r in data['runs']['strong'].items() if r['first_flip'])
    flip=data['runs']['strong'][seed]['first_flip']
    end=math.ceil(flip['arrival_tick']/10)*10
    sel=write_shot('strong',seed,end,10,p['strong']['config'])
    sel.update(window=[max(0,flip['arrival_tick']-2000),end],flip=flip,
               selection='smallest eligible measured seed; earliest 80%-to-20% flip')
    data['selected']['strong']=sel
    short_seed=min(s for s,r in data['runs']['short'].items() if .4<=r['mean']<=.6)
    for name,key,seed,ticks,every in [('short','short',short_seed,2000,10),('pull','pull',SEEDS[0],2000,10),
                                    ('large','large',SEEDS[0],2000,10),('ring','ring_1050',SEEDS[0],3000,30),
                                    ('random','random_1050',SEEDS[0],3000,30),('independent','independent_0.05',SEEDS[0],3000,30)]:
        data['selected'][name]=write_shot(name,seed,ticks,every,p[key]['config'])
        data['selected'][name]['selection']='smallest compatible measured seed' if name=='short' else 'first predetermined measured seed'
    trace_keys = dict(strong='strong', short='short', pull='pull', large='large',
                      ring='ring_1050', random='random_1050', independent='independent_0.05')
    for name, selected in data['selected'].items():
        selected['clip_start_frame'] = 0
        selected['clip_end_frame'] = selected['frames']-1
        if name.startswith('micro-'):
            selected['clip_start_frame'] = selected['event']['tick']-1
        elif name == 'strong':
            selected['clip_start_frame'] = max(0,(selected['flip']['departure_tick']-1)//selected['every'])
        selected['clip_start_period'] = selected['clip_start_frame']*selected['every']
        selected['clip_end_period'] = selected['clip_end_frame']*selected['every']
        if name in trace_keys:
            run = data['runs'][trace_keys[name]][selected['seed']]
            selected['trace'] = run['flip_trace'] if name=='strong' else run['trace']
            selected['trace_schema'] = '[actual tick, count at source 1]'
    # Retain every scalar outcome and every histogram sample, while keeping
    # dense/event-window traces only for the documented filming examples.
    for rows in data['runs'].values():
        for run in rows.values():
            run.pop('trace',None)
            run.pop('flip_trace',None)
    data['trace_retention'] = 'Full post-initial scalar statistics and histograms for all 1220 native series; selected bounded traces only (up to 200 background samples, up to 2001 actual ticks preceding the selected first flip). No time-series decimation enters any judge.'
    data['cohorts']=dict(short_long='paired numeric seeds for first 20; do not pool as independent replicates',
                         network='same numeric seeds across configurations; graph equality not verified, no matched-graph claim',
                         observations='real-ant 80–20 premise attributed to Kirman, not a simulator verdict')
    lines=['## Frozen prospective protocol','', 'All post-initial ticks retained, no burn-in; stop_at=0. Core seeds 200001 upward are disjoint from audit seeds 1–1000. Native CLI; at most four concurrent workers. Full configurations, every run outcome, histograms, and selected traces are in measurements.json.', '',
           'AM units are sequential immediate-update sweeps. Kirman units are steps of the stated meeting count; the micro cohort has one meeting per tick. Samples across populations are compared at equal total meetings where stated. Matching seed numbers do not establish matching AM graphs.','',
           '| cohort | runs | ticks/run | updates/tick | mean variance ± independent-seed SE | extreme occupancy |',
           '|---|---:|---:|---:|---:|---:|']
    for name,a in data['aggregates'].items():
        lines.append(f"| {name} | {a['runs']} | {p[name]['ticks']} | {p[name]['updates_per_tick']} | {a['mean_variance']:.6f} ± {a['variance_se']:.6f} | {a['mean_extreme']:.6f} |")
    lines += ['', 'Selected films are examples drawn from the retained corpus. The strong film uses the first qualifying flip in the smallest eligible seed; micro events use the smallest eligible seed and earliest actual exported event. Exact distributions are separately labeled model theory. Numeric graph pairing is not asserted.']
    return lines,verdicts,data
