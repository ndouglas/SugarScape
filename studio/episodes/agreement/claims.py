"""Prospective caption rules fixed in the approved agreement spike.
Counts are reconstructed from CSV roles and initial opinions, never y alone.
"""
import concurrent.futures
import csv
import subprocess
import json
import statistics
import measure as m
from agreement_visual import select_pair, overlap

CAP = 20000
BASE = dict(agents=200, mu=.2, extremists=.05, uncertainty=1.4,
            extremist_uncertainty=.1, delta=0., placement='drawn', extreme_margin=.1,
            stop_when_stable=True, stop_at=CAP)
CONFIGS = {
    'crowd': {}, 'middle': dict(extremists=.2, uncertainty=.4, mu=.5),
    'both': dict(extremists=.25, uncertainty=1.2, mu=.5), 'single': {},
    'early': dict(placement='band', extreme_margin=0., stop_when_stable=False, stop_at=200),
    'reply': dict(placement='band', extreme_margin=.1, stop_when_stable=False, stop_at=1200),
    'size-small': dict(extremists=.1, uncertainty=1.6),
    'size-big': dict(extremists=.1, uncertainty=1.6, agents=2000),
    'lean-balanced': dict(extremists=.1, uncertainty=1.6, agents=1000),
    'lean': dict(extremists=.1, uncertainty=1.6, agents=1000, delta=.1),
    'printed': dict(rule='bc', window='influencer', agents=1000, uncertainty=1., delta=.1),
    'neighbors': dict(agents=900, extremists=.2, uncertainty=1.4, placement='bounds',
                      network='lattice', **{'lattice.width':30, 'lattice.height':30,
                                          'lattice.neighborhood':'moore'}),
    'horizon-only': dict(placement='band', extreme_margin=0., stop_when_stable=False, stop_at=1200),
    'cutoff-only': dict(placement='band', extreme_margin=.1, stop_when_stable=False, stop_at=200),
}
CAPTIONS = [
 '200 Flumps, each with an opinion.\nA few at the ends are very sure of themselves.',
 'Each has an uncertainty range around its opinion.\nA short range means more confidence.',
 "Random pairs meet. Enough overlap lets one change\nthe other's opinion and uncertainty.",
 'A more certain crowd keeps a middle\nin 11 of our 20 runs.',
 'A less certain crowd can split between both extremes.',
 'Even with equal extremists at both ends,\na few confident Flumps can pull nearly everyone to one side.',
 "Stop early, and a crowd still drifting\nhasn't yet crossed the counting line.",
 "The authors' reply waited for opinions to settle,\nand after seeing the runs, moved the counting line inward.",
 'Together, those changes recover a frequent single extreme.',
 'At these settings, a bigger crowd\nreaches one extreme much less often.',
 'A small imbalance makes one side more likely to win.',
 "With their alternative's printed rule,\nthe moderates pull the extremists toward the middle.",
 'Hear only neighbors on this lattice,\nand neither extreme takes over the crowd.',
 'How extremists win\nA confident minority can move an uncertain crowd.\nAfter Deffuant et al., 2002',
]


def count_agents(agents, config):
    mods = [a for a in agents if a['role'] == 'moderate']
    plus = [float(a['start']) for a in agents if a['role'] == 'plus']
    minus = [float(a['start']) for a in agents if a['role'] == 'minus']
    placement = config['placement']
    pb = min(plus) if placement == 'drawn' else (config.get('band', .8) if placement == 'band' else 1.)
    mb = max(minus) if placement == 'drawn' else (-config.get('band', .8) if placement == 'band' else -1.)
    pc, mc = pb-config['extreme_margin'], mb+config['extreme_margin']
    pp = sum(float(a['opinion']) > pc for a in mods)/len(mods)
    pm = sum(float(a['opinion']) < mc for a in mods)/len(mods)
    ex = [a for a in agents if a['role'] != 'moderate']
    return dict(p_plus=pp, p_minus=pm, y=pp*pp+pm*pm,
                single=max(pp,pm)>=.7 and min(pp,pm)<.1, both=min(pp,pm)>=.25,
                near_all=max(pp,pm)>=.9, sign=1 if pp>pm else -1,
                middle=sum(abs(float(a['opinion']))<=.5 for a in mods)/len(mods),
                baseline_plus=sum(float(a['start'])>pc for a in mods)/len(mods),
                baseline_minus=sum(float(a['start'])<mc for a in mods)/len(mods),
                plus_cutoff=pc, minus_cutoff=mc, initial_plus=len(plus), initial_minus=len(minus),
                extremist_abs=statistics.mean(abs(float(a['opinion'])) for a in ex),
                drift=abs(statistics.mean(float(a['opinion']) for a in agents)))


def printed_holds(rows):
    return len(rows)>=50 and sum(r['extremist_abs']<.5 and r['y']==0 for r in rows.values())>=45


def sample(tmp, name, seed):
    changes = dict(BASE, **CONFIGS[name])
    ticks = changes['stop_at']
    config = m.with_changes(m.preset_config('ra-single', tmp), changes)
    source, series_path, agents_path = [tmp / f'{name}-{seed}{suffix}' for suffix in ('.json', '.csv', '-agents.csv')]
    source.write_text(json.dumps(config))
    subprocess.run([m.CLI, 'run', '--config', source, '--seed', str(seed), '--ticks', str(ticks),
                    '--series-csv', series_path, '--agents-csv', agents_path],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    with series_path.open() as f: series = list(csv.DictReader(f))
    with agents_path.open() as f: agents = list(csv.DictReader(f))
    r = count_agents(agents, config)
    r['initial_agents'] = len(agents)
    r['initial_extremists'] = sum(a['role'] != 'moderate' for a in agents)
    r['initial_uncertainties'] = [config['extremist_uncertainty'], config['uncertainty']]
    r['rule'] = config['rule']
    r['pair_update'] = config['pair_update']
    initial_x = {int(a['id']): float(a['start']) for a in agents}
    initial_u = {int(a['id']): config['uncertainty'] if a['role']=='moderate' else config['extremist_uncertainty'] for a in agents}
    i, j = select_pair(initial_x, initial_u)
    h = overlap(initial_x[i], initial_u[i], initial_x[j], initial_u[j])
    r['initial_pair_exists'] = h > initial_u[i] and h <= initial_u[j]
    r['initial_pair'] = dict(influencer=i, listener=j, overlap=h,
                             influencer_uncertainty=initial_u[i], listener_uncertainty=initial_u[j],
                             influence=(config['mu']*(h/initial_u[i]-1) if config['rule']=='ra' else config['mu'] if abs(initial_x[i]-initial_x[j])<initial_u[i] else 0.),
                             reverse_influence=(0. if config['rule']=='ra' else config['mu'] if abs(initial_x[i]-initial_x[j])<initial_u[j] else 0.))
    last = series[-1]
    r.update(tick=int(last['tick']), stable=float(last['max_change'])<=1e-6,
             max_change=float(last['max_change']))
    if abs(r['y']-float(last['y']))>1e-10:
        raise ValueError(f'independent counts disagree: {name}, seed {seed}')
    return name, seed, r


def verdicts(data):
    def n(name, key): return sum(r[key] for r in data[name].values())
    single = data['single']
    near = [r for r in single.values() if r['near_all']]
    lean = data['lean']
    favored = sum(r['single'] and r['sign']==(1 if r['initial_plus']>r['initial_minus'] else -1) for r in lean.values())
    opposite = n('lean','single')-favored
    tests = [all(r.get('initial_agents')==200 and r.get('initial_extremists')==10 for r in data['crowd'].values()),
             all(r.get('initial_uncertainties')==[.1,1.4] for r in data['crowd'].values()),
             all(r.get('rule')=='ra' and r.get('pair_update')=='simultaneous' and r.get('initial_pair_exists') for r in data['crowd'].values()),
             len(data['middle'])==20 and sum(r['middle']>=.5 for r in data['middle'].values())==11,
             n('both','both')>=40,
             len(near)>=40 and {r['sign'] for r in near}=={-1,1} and all(r['initial_plus']==r['initial_minus'] for r in single.values()),
             all(not r['single'] for r in data['early'].values()) and statistics.mean(r['drift'] for r in data['early'].values())>.5,
             all(r['tick']==1200 for r in data['reply'].values()),
             n('reply','single')>=40 and n('reply','single')>max(n('horizon-only','single'), n('cutoff-only','single')),
             n('size-big','single')<=n('size-small','single')/4 and all(r['stable'] for name in ('size-small','size-big') for r in data[name].values()),
             favored>opposite and favored>n('lean-balanced','single')/2,
             printed_holds(data['printed']),
             all(max(r['p_plus'],r['p_minus'])<.7 for r in data['neighbors'].values()),
             len(near)>=40]
    reasons = [
        '200 agents; 10 initial extremists with uncertainty 0.1 versus moderates 1.4, all 50 seeds',
        'uncertainty is the half-width of each measured opinion segment; initial values 0.1 versus 1.4',
        'RA simultaneous old-state updates; shared select_pair finds actual initial asymmetric-overlap pair in every seed',
        f"{sum(r['middle']>=.5 for r in data['middle'].values())}/20 seeds retain at least half the initial moderates within |x|≤0.5; caption revised after result was known, original prospective rule 16/20 failed",
        f"{n('both','both')}/50 both extremes; rule requires at least 40",
        f"{len(near)}/50 recruit at least 90% beyond one fixed initial cutoff; both signs observed; equal initial extremist counts",
        f"{n('early','single')}/50 single at period 200, mean physical drift {statistics.mean(r['drift'] for r in data['early'].values()):.3f}; all trajectories read before period 1200",
        'same band initialization and seed; prescribed horizons 200 and 1200; cutoffs ±0.8 and ±0.7, a later author measurement choice rather than a change in trajectories',
        f"single frequencies original {n('early','single')}/50, horizon only {n('horizon-only','single')}/50, cutoff only {n('cutoff-only','single')}/50, combined {n('reply','single')}/50; 1200 is a fixed horizon, not a guarantee of numerical stability",
        f"N200: {n('size-small','single')}/50 versus N2000: {n('size-big','single')}/50; every run stable",
        f"balanced singles {n('lean-balanced','single')}/50; imbalance favored-side wins {favored}/50, opposite {opposite}/50",
        f"{sum(r['extremist_abs']<.5 and r['y']==0 for r in data['printed'].values())}/50 with direct initial-extremist final mean absolute opinion <0.5 and y=0",
        f"0/20 sides recruit 70% beyond ±0.9; {sum(not r['stable'] for r in data['neighbors'].values())}/20 unsettled at the declared 20000-period cap",
        'model mechanism supported by the measured single-extreme setup; no empirical law about people',
    ]
    return list(zip(CAPTIONS,tests,reasons))


def measure(tmp):
    m.preset_config('ra-single',tmp)
    jobs = [(name,seed) for name in CONFIGS if name!='crowd' for seed in range(1,21 if name in ('middle','neighbors') else 51)]
    data={name:{} for name in CONFIGS}
    with concurrent.futures.ThreadPoolExecutor(4) as pool:
        for name,seed,row in pool.map(lambda j: sample(tmp,*j), jobs): data[name][seed]=row
    data['crowd']=data['single']
    summaries={name:dict(seeds=len(rows), single=sum(r['single'] for r in rows.values()),
                         both=sum(r['both'] for r in rows.values()),
                         mean_y=statistics.mean(r['y'] for r in rows.values()),
                         mean_drift=statistics.mean(r['drift'] for r in rows.values()),
                         unsettled=sum(not r['stable'] for r in rows.values())) for name,rows in data.items()}
    typical={name:m.typical_seed(rows,['y','drift','tick']) for name,rows in data.items()}
    # Each filmed example must demonstrate its caption rather than merely the median.
    for name,predicate in [('middle',lambda r:r['middle']>=.5), ('both',lambda r:r['both']), ('single',lambda r:r['near_all']), ('lean',lambda r:r['single'] and r['sign']==1)]:
        eligible={s:r for s,r in data[name].items() if predicate(r)}
        if eligible: typical[name]=m.typical_seed(eligible,['y','drift','tick'])
    typical['crowd']=typical['single']
    typical['early']=typical['reply']
    for name in CONFIGS:
        if name in ('horizon-only','cutoff-only'): continue
        seed=typical[name]
        ticks=data[name][seed]['tick']
        shot=dict(preset='ra-single',seed=seed,ticks=ticks,set=dict(BASE,**CONFIGS[name]))
        if ticks>1200: shot['every']=20
        elif ticks>300: shot['every']=5
        every = shot.get('every', 1)
        shot['ticks'] = ((ticks + every - 1) // every) * every
        (m.STUDIO/'episodes/agreement/shots'/f'{name}.json').write_text(json.dumps(shot,indent=2)+'\n')
    checks=verdicts(data)
    lines=['## Fixed measurement conventions','',
           'Seeds 1–50 for source-sized comparisons; seeds 1–20 for middle and Moore. Drawn cutoff is the innermost initial extremist minus 0.1; band cutoffs are ±0.8 or ±0.7; bounds cutoff ±0.9. Strict inequalities count initial moderates only. Period-zero shares are retained, not subtracted. y = p_plus² + p_minus². Single means one side ≥0.7 and opposite <0.1; both means each ≥0.25. Stable means maximum opinion/uncertainty change ≤1e-6, cap 20,000 periods. Capped outcomes do not establish asymptotic limits.',
           '', '**Caption revision after observing results:** original beat4 required at least16/20 seeds to retain≥50%initial moderates in|x|≤0.5. Observed11/20: original rule FAILED. Caption now states the exact observed11/20 frequency; the counting threshold was not changed.',
           '', 'The historical 1,000-seed disputed-example audit is not rerun or captioned here. Pair rule is RA with simultaneous old-state updates; confidence is uncertainty half-width.','',
           '| config | seeds | single | both | mean y | mean absolute population drift | unsettled | typical seed |',
           '|---|---|---|---|---|---|---|---|']
    for name,s in summaries.items(): lines.append(f"| {name} | {s['seeds']} | {s['single']} | {s['both']} | {s['mean_y']:.4f} | {s['mean_drift']:.4f} | {s['unsettled']} | {typical[name]} |")
    lines+=['','All individual agent-derived counts, baseline shares, cutoffs, stability diagnostics, exact configurations and caption verdicts are retained in measurements.json.']
    return lines, checks, dict(medians={name: {key: statistics.median(r[key] for r in rows.values())
                                             for key in ('y', 'drift', 'middle', 'extremist_abs', 'tick')}
                                        for name, rows in data.items()},
                              protocols=summaries, typical_seeds=typical, rows=data,
                              configs={name:dict(BASE,**changes) for name,changes in CONFIGS.items()},
                              caption_revisions=[dict(beat=4, original_caption='A more certain crowd can keep a middle.', original_required=16, observed=11, seeds=20, original_holds=False, revised_after_result=True, revised_caption=CAPTIONS[3])],
                              caption_verdicts=[dict(caption=c,holds=h,why=w) for c,h,w in checks])
