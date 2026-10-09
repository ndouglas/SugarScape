"""Descriptive per-cell tables from retained native arrays, no paired re-estimation."""
from pathlib import Path
from collections import Counter,defaultdict
from statistics import median
import json,hashlib,argparse

def distribution(values):
    present=[v for v in values if v is not None]
    return {'n':len(values),'available':len(present),'null':len(values)-len(present),'min':min(present) if present else None,'median':median(present) if present else None,'max':max(present) if present else None}
def triple(values):
    d=distribution(values)
    return 'unavailable' if not d['available'] else '/'.join(f'{d[k]:.9g}' for k in ['min','median','max'])
def main():
    p=argparse.ArgumentParser();p.add_argument('--data',type=Path,required=True);args=p.parse_args();r=args.data
    b=(r/'analysis.json').read_bytes();a=json.loads(b);raw=json.loads((r/'raw-endpoints.json').read_text());times=json.loads((r/'raw-timing-distributions.json').read_text());matched=json.loads((r/'raw-matched-checks.json').read_text())
    assert raw==a['endpoints'];assert len(matched)==640 and all(x['equal_rng'] and x['equal_physical_task'] and x['frames']==65 for x in matched)
    episodes=defaultdict(list);rt={(x['condition'],x['seed']):x for x in times}
    for x in raw:episodes[x['condition']].append(x)
    sections=['# All 96 cells: observed endpoints, work and timing\n\nAll cells retain exactly 40 seeds 30001–30040. Ranges below are min / median / max within the named cell; these are secondary descriptive diagnostics, not new paired estimates or intervals. Exact native arrays remain in [analysis.json](analysis.json), all endpoints in [raw-endpoints.json](raw-endpoints.json), and every measured duration in [raw-timing-distributions.json](raw-timing-distributions.json). Nulls remain explicit.\n', '## Endpoints\n\n| Condition | n | Attained | First completion available; null | Restricted ticks min/med/max | Gross food min/med/max | Living ticks min/med/max | Alive at 64 |\n|---|---:|---:|---|---|---|---|---:|']
    descriptions=[];workkeys=['node_visits','candidate_evaluations','target_selections','path_queries','search_expansions','fallback_short','fallback_limit']
    for c in a['cells']:
        es=episodes[c['condition']];assert len(es)==40 and [x['seed'] for x in es]==list(range(30001,30041))
        rows=[rt[c['condition'],s] for s in c['seeds']]
        assert [t['active_calls'] for t in rows]==c['task_active_calls'] and [t['hold_calls'] for t in rows]==c['hold_calls']
        assert [t['controller_seconds'] for t in rows]==c['controller_seconds_per_invocation'] and [t['episode_seconds'] for t in rows]==c['episode_seconds']
        for t in rows:assert t['active_calls']+t['hold_calls']+t['dead_ticks']==64 and len(t['controller_seconds'])==t['active_calls']
        completion=[e['first_completion'] for e in es];nulls=completion.count(None)
        sections.append(f"| {c['condition']} | 40 | {sum(e['quota_attained'] for e in es)} | {triple(completion)}; null {nulls} | {triple([e['restricted_completion_ticks'] for e in es])} | {triple([e['gross_gathered'] for e in es])} | {triple([e['living_ticks'] for e in es])} | {sum(e['alive_at_horizon'] for e in es)} |")
        descriptions.append({'condition':c['condition'],'seeds':c['seeds'],'endpoints':{key:distribution([x[key] for x in es]) for key in ['first_completion','restricted_completion_ticks','gross_gathered','living_ticks']},'work':{k:distribution([w[k] if w is not None else None for w in c['work']]) for k in workkeys},'active_invocation_seconds':distribution([v for row in c['controller_seconds_per_invocation'] for v in row]),'controller_seconds_total':distribution(c['controller_seconds_total']),'episode_seconds':distribution(c['episode_seconds']),'active_calls':sum(c['task_active_calls']),'hold_calls':sum(c['hold_calls']),'dead_ticks':sum(t['dead_ticks'] for t in rows)})
    sections+=['\n## Work per episode\n\nNative diagnostic counters, min / median / max across all 40 episodes. Work covers actual evaluation stages, not unique candidate sites. Search zero is an observed zero; no unavailable count is converted to zero. Holds contribute zero work. Internal work is instrumented, not independently reconstructed operation by operation.\n','| Condition | Node visits | Candidate evaluations | Target selections | Path queries | Search expansions | Short fallback | Limit fallback |\n|---|---|---|---|---|---|---|---|']
    for c in a['cells']:sections.append('| '+c['condition']+' | '+' | '.join(triple([w[k] if w is not None else None for w in c['work']]) for k in workkeys)+' |')
    sections+=['\n## Measured timing and denominators\n\nEvery duration is in **seconds**. Each min / median / max uses the full named cell. Active distributions use every active invocation; controller totals and episode envelopes use 40 episodes. Holds and dead ticks have no measured controller invocation. These observational host timings make no machine-isolation or controller-speed ranking claim.\n','| Condition | Active / hold / dead ticks | Active seconds min/med/max | Controller total seconds/episode min/med/max | Envelope seconds/episode min/med/max |\n|---|---|---|---|---|']
    for c,d in zip(a['cells'],descriptions):
        sections.append(f"| {c['condition']} | {d['active_calls']}/{d['hold_calls']}/{d['dead_ticks']} | {triple([v for row in c['controller_seconds_per_invocation'] for v in row])} | {triple(c['controller_seconds_total'])} | {triple(c['episode_seconds'])} |")
    sections+=['\n## Unrestricted completion availability\n\nA summary is unavailable if any registered pair is unattained. No successful-only subset is summarized. The full native completion rows preserve all unavailable seed/reason entries.\n','| Native family/stratum ID | n | Mean | Native 95% interval | Unavailable seed pairs / reason |\n|---|---:|---|---|---|']
    for c in a['completion']:
        s=c['summary'];reason=Counter(reason for seed,reason in c['unavailable'])
        sections.append(f"| {c['id']} | {c['denominator']} | {s['mean'] if s else 'unavailable'} | {s['ci95'] if s else 'unavailable'} | {len(c['unavailable'])}; {dict(reason) if reason else 'none'} |")
    sections+=['\n## Physical aliases\n\nThe 40 physical-trajectory groups and 28 cross-condition alias groups remain exact in analysis.json. Representation, RNG and timing are excluded from physical duplicate signatures; distinct RNG fingerprints do not establish independent replication.\n','| Conditions | Native reason |\n|---|---|']
    sections+=['| '+', '.join(c['conditions'])+' | '+c['reason']+' |' for c in a['aliases']]
    summary={'analysis_sha256':hashlib.sha256(b).hexdigest(),'method':'per-cell descriptive n/null/min/median/max, no scientific paired estimate or CI recomputation','cells':descriptions,'counts':{'episodes':len(raw),'attained':sum(x['quota_attained'] for x in raw),'null_completion':sum(x['first_completion'] is None for x in raw),'alive_at_64':sum(x['alive_at_horizon'] for x in raw),'lifetime_at_64':sum(x['living_ticks']==64 for x in raw),'active':sum(x['active_calls'] for x in times),'hold':sum(x['hold_calls'] for x in times),'dead':sum(x['dead_ticks'] for x in times),'native_zero_estimates':sum(x['summary']['mean']==0 for x in a['estimates']),'zero_width_intervals':sum(x['summary']['ci95'][0]==x['summary']['ci95'][1] for x in a['estimates'])}}
    for name,obj in [('diagnostic-summary.json',summary),('completion-availability.json',a['completion']),('physical-aliases.json',{'duplicate_groups':a['duplicate_groups'],'aliases':a['aliases']})]:
        with (r/name).open('x') as f:json.dump(obj,f,indent=2);f.write('\n')
    with (r/'all-cell-diagnostics.md').open('x') as f:f.write('\n'.join(sections)+'\n')
    print(json.dumps(summary['counts']))
if __name__=='__main__':main()
