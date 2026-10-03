"""Prospective rules from the approved thresholds spike, fixed before running.

All worlds are fresh, repeat=False. Every city draw has its own seed stream;
5,000 independent streams exceed the required minimum of twenty streams.
Final participation is read from actual agent states. Network comparisons use
integer 100*active_neighbors >= 18*degree, including isolated-node behavior.
"""
import concurrent.futures
import collections
import json
import math
import measure as m

CONFIGS = {
    'uniform': ('gr-uniform', {}),
    'perturbed': ('gr-perturbed', {}),
    'city': ('gr-city', {}),
    'friends': ('gr-friends', {'friends.acquaintance': .25, 'friends.weight': 2}),
    'rescue': ('gr-friends-perturbed', {'friends.acquaintance': .25, 'friends.weight': 5}),
    'ceilings': ('gr-ceilings', {'max_steps': 100000}),
    'sparse': ('watts-lower', {'actors': 1000}),
    'middle': ('watts-middle', {'actors': 1000}),
    'dense': ('watts-upper', {'actors': 1000}),
}
# Disjoint from all audit seeds (100001–101000), and between configurations.
SEEDS = {name: range(start, start + count) for name, start, count in (
    ('uniform', 200001, 20), ('perturbed', 201001, 20),
    ('city', 202001, 5000), ('friends', 208001, 1000),
    ('rescue', 210001, 1000), ('ceilings', 212001, 40),
    ('sparse', 214001, 1000), ('middle', 216001, 1000), ('dense', 218001, 1000))}
RULES = dict(city_draws=5000, city_expected=.5013584, city_tolerance=.03,
             friendship_graphs=1000, friends_below10=.8, rescue_above1=.2,
             rescue_atmost10=.9, ceiling_seeds=40, ceiling_window=[501,600],
             ceiling_range=.05, ceiling_required=20, network_graphs=1000,
             network_actors=1000, threshold=.18, small_strict=.01,
             large_inclusive=.9, sparse_small=.6, middle_large=.75,
             dense_small=.55, dense_large=.1, settle_cap=1000)


def city_holds(values):
    return len(values) == 5000 and abs(sum(v <= 1 for v in values)/len(values)-.5013584) <= .03


def outcome_counts(values, population):
    return sum(v < .01*population for v in values), sum(v >= .9*population for v in values)


def rescue_holds(values):
    return len(values) == 1000 and sum(v > 1 for v in values) >= 200 and sum(v <= 10 for v in values) >= 900


def sample(tmp, name, seed):
    preset, changes = CONFIGS[name]
    changes = dict(changes, repeat=False)
    ticks = 600 if name == 'ceilings' else 1000
    rows, agents = m.run(preset, seed, ticks, tmp, changes)
    counts = [round(float(r['acting'])*len(agents)) for r in rows]
    final = sum(int(a['acting']) for a in agents)
    step = int(rows[-1]['step'])
    episodes = int(rows[-1]['episodes'])
    expected = [i/100 for i in range(100)]
    if name == 'perturbed': expected[1] = .02
    thresholds = [float(a['threshold']) for a in agents]
    network = name in ('sparse','middle','dense')
    rule_ok = True
    if network:
        rule_ok = (len(agents)==1000 and sum(int(a['seed']) for a in agents)==1
                   and all(float(a['threshold'])==.18 and int(a['of'])==int(a['degree'])
                           and (bool(int(a['acting'])) == (bool(int(a['seed'])) or
                                (int(a['of'])>0 and 100*int(a['sees'])>=18*int(a['of']))))
                           for a in agents))
    result = dict(final=final, population=len(agents), step=step, episodes=episodes,
                  settled=(episodes==1), initial=counts[0], rule_ok=rule_ok,
                  trace=counts[:(601 if name=='ceilings' else step+1)])
    if name in ('uniform','perturbed'):
        result['thresholds_exact'] = thresholds == expected
        result['uniform_chain'] = counts[:101] == list(range(101))
    if name == 'ceilings': result['late_range'] = (max(counts[501:601])-min(counts[501:601]))/len(agents)
    return name, seed, result


def verdicts(data):
    vals = lambda name: [r['final'] for r in data[name].values()]
    pair = all(r['thresholds_exact'] and r['initial']==0 and r['final']==100 and r['uniform_chain']
               for r in data['uniform'].values()) and all(
                   r['thresholds_exact'] and r['initial']==0 and r['final']==1 for r in data['perturbed'].values())
    settled = lambda name: all(r['settled'] for r in data[name].values())
    small_s, large_s = outcome_counts(vals('sparse'),1000)
    small_m, large_m = outcome_counts(vals('middle'),1000)
    small_d, large_d = outcome_counts(vals('dense'),1000)
    pulse = sum(r['late_range']>=.05-1e-12 for r in data['ceilings'].values())
    city = vals('city'); friends = vals('friends'); rescue = vals('rescue')
    network_ok = all(r['rule_ok'] and r['settled'] for name in ('sparse','middle','dense') for r in data[name].values())
    checks = [
        ('Fixed-threshold uniform and perturbed pair (beats 1–5)', pair, '20 fresh seeds each; exact threshold lists, initial zero; uniform grows one per synchronous step to 100, perturbed to one; means differ by 0.01 percentage point'),
        ('Sampled city (beat 6)', city_holds(city) and settled('city'), f'{sum(v<=1 for v in city)}/5000 stop at zero or one; {sum(v==100 for v in city)}/5000 complete riots; fixed tolerance ±0.03 around exact 0.5013584'),
        ('Friends twice (beat 7)', len(friends)==1000 and sum(v<10 for v in friends)>=800 and settled('friends'), f'{sum(v<10 for v in friends)}/1000 below ten; our symmetric random-tie probability 0.25, weighted whole-crowd denominator'),
        ('Friendship rescue (beat 8)', rescue_holds(rescue) and settled('rescue'), f'{sum(v>1 for v in rescue)}/1000 above one; {sum(v<=10 for v in rescue)}/1000 at most ten; probability 0.25, weight five'),
        ('Constructed ceiling extension (beat 9)', len(data['ceilings'])==40 and pulse>=20, f'{pulse}/40 range at least 0.05 during actual steps 501–600; synchronous, ceiling share 0.1 above 0.9, max_steps 100000'),
        ('Neighbor-only network rule (beat 10)', network_ok, '3000 fresh 1000-agent networks, one forced seed, fixed threshold 18%; final states independently checked against actual degree and active-neighbor integer counts'),
        ('Sparse network (beat 11)', small_s>=600, f'{small_s}/1000 below 1%; {large_s}/1000 reach 90%; mean degree 1.05'),
        ('Middle network (beat 12)', large_m>=750, f'{large_m}/1000 reach 90%; mean degree 3'),
        ('Dense finite network (beat 13)', small_d>=550 and large_d>=100, f'{small_d}/1000 below 1%; {large_d}/1000 reach 90%; mean degree 6.14; does not reproduce the original printed Figure 3 frequency'),
    ]
    return checks


def measure(tmp):
    for preset, _ in CONFIGS.values(): m.preset_config(preset,tmp)
    data = {name:{} for name in CONFIGS}
    jobs = [(name,s) for name in CONFIGS for s in SEEDS[name]]
    with concurrent.futures.ThreadPoolExecutor(8) as pool:
        for name, seed, row in pool.map(lambda job:sample(tmp,*job),jobs): data[name][seed]=row
    protocols = {}
    typical = {}
    for name, rows in data.items():
        values = [r['final'] for r in rows.values()]
        small, large = outcome_counts(values,rows[next(iter(rows))]['population'])
        protocols[name] = dict(runs=len(rows), histogram=dict(sorted(collections.Counter(values).items())), small=small,
                               large=large, above_one=sum(v>1 for v in values), below_ten=sum(v<10 for v in values),
                               atmost_ten=sum(v<=10 for v in values), zero_or_one=sum(v<=1 for v in values))
        eligible = rows
        if name=='rescue': eligible={s:r for s,r in rows.items() if r['final']>1}
        if name=='middle': eligible={s:r for s,r in rows.items() if r['final']>=900}
        if name=='ceilings': eligible={s:r for s,r in rows.items() if r['late_range']>=.05-1e-12}
        if name in ('sparse','dense'): eligible={s:r for s,r in rows.items() if r['final']<10}
        if not eligible: raise ValueError(f'no eligible filmed example for {name}; keep fixed rules and revise caption')
        typical[name] = m.typical_seed(eligible,['final','step'])
    dense_large = {s:r for s,r in data['dense'].items() if r['final']>=900}
    if not dense_large: raise ValueError('no dense large example')
    typical['dense-large'] = m.typical_seed(dense_large,['final','step'])
    for name,seed in typical.items():
        source = 'dense' if name=='dense-large' else name
        preset, changes = CONFIGS[source]
        row = data[source][seed]
        ticks = 600 if source=='ceilings' else row['step']
        shot=dict(preset=preset,seed=seed,ticks=ticks,set=dict(changes,repeat=False))
        (m.STUDIO/'episodes/thresholds/shots'/f'{name}.json').write_text(json.dumps(shot,indent=2)+'\n')
    checks=verdicts(data)
    lines=['## Prospective protocol','',
           'Rules in claims.py were fixed before this episode corpus was run. All seeds are 200001 upward, disjoint from the earlier audit and between configurations. City: 5000 fresh worlds, each an independent one-draw seed stream (more than twenty streams). Friendship and each network setting: 1000 fresh worlds. Uniform and perturbed: twenty fresh seeds each. Ceilings: forty fresh seeds.', '',
           'Irreversible worlds run through 1000 true steps with repeat=false and must complete one episode; the agent-derived final count and actual settling step are retained. Ceilings are measured through step 600 with max_steps=100000 and repeat=false. Every outcome, trace, exact config, seed, stopping criterion and selected example is retained in measurements.json. Examples are selected from this measured corpus; frequency comes from all runs, not the showcased trajectory.', '',
           'Source scopes: Granovetter\'s exact 100-person example is reproduced; the city sampling tests his stated aggregation instability. Random friendship sampling and ceilings are our declared realizations. Watts panels show our finite 1000-node networks, not the unresolved original Figure 3 upper-frequency reconstruction. Fixed neighbor thresholds and asynchronous step timing are unchanged.', '',
           '| configuration | fresh worlds | below 1% | at least 90% | selected example seed |', '|---|---|---|---|---|']
    for name,p in protocols.items(): lines.append(f"| {name} | {p['runs']} | {p['small']} | {p['large']} | {typical[name]} |")
    return lines,checks,dict(medians={},rules=RULES,protocols=protocols,rows=data,typical_seeds=typical,
                            configs={name:m.with_changes(m.preset_config(preset,tmp),dict(changes,repeat=False)) for name,(preset,changes) in CONFIGS.items()},
                            caption_verdicts=[dict(caption=c,holds=h,why=w) for c,h,w in checks],caption_revisions=[])
