"""Bind exact native paired estimates to the 16 registered presentation panels.

No estimate, interval, sign or completion endpoint is calculated here.
"""
import re
FAMILIES={'primary':'Reactive Utility','secondary':'Task GOAP','ablation':'Unguarded Tree','legacy_reference':'Legacy GOAP'}
METRICS=('quota_attained','restricted_completion_ticks','gross_gathered','living_ticks')
SCENARIOS=('stable','better-alternative','depleted-target','temporary-obstacle')
def bind_rows(chart, analysis, analysis_sha256):
    if chart['source']['sha256'] != analysis_sha256:
        raise ValueError('source analysis hash mismatch')
    if analysis['schema'] != 'minds-behavior-tree-measured-v1':
        raise ValueError('native measured schema mismatch')
    if len(chart['estimates']) != 256 or len(analysis['estimates']) != 256 or chart['rows'] != 256:
        raise ValueError('must retain exactly 256 native estimates')
    native={r['id']:(i,r) for i,r in enumerate(analysis['estimates'])}
    if len(native)!=256:raise ValueError('duplicate native ID')
    seen=set();groups={(f,m):[] for f in FAMILIES for m in METRICS}
    for index,row in enumerate(chart['estimates']):
        key=row['id']
        if key in seen:raise ValueError('duplicate chart ID')
        seen.add(key)
        if row['family'] not in FAMILIES or row['metric'] not in METRICS:
            raise ValueError('unregistered family or metric')
        if row['denominator']!=40 or row['summary']['n']!=40:
            raise ValueError('registered denominator must remain 40')
        if key not in native or row!=native[key][1]:raise ValueError('chart differs from native estimate')
        match=re.fullmatch(r'([^:]+):guarded-tree-(stable|better-alternative|depleted-target|temporary-obstacle)-quota(20|40)-m([01]):([^:]+)',key)
        if not match:raise ValueError('unregistered native stratum ID')
        f,s,q,o,m=match.groups()
        if (f,m)!=(row['family'],row['metric']):raise ValueError('ID family/metric mismatch')
        summary=row['summary']
        if sum(summary[k] for k in ['positive','zero','negative'])!=40:raise ValueError('sign denominator mismatch')
        ci=summary['ci95']
        if ci is not None and not ci[0]<=summary['mean']<=ci[1]:raise ValueError('native interval does not contain mean')
        groups[f,m].append({'chart_index':index,'analysis_index':native[key][0], 'scenario':s,'quota':int(q),'orientation':'base' if o=='0' else 'reflected','native':row})
    expected={(s,q,o) for s in SCENARIOS for q in [20,40] for o in ['base','reflected']}
    for rows in groups.values():
        if {(r['scenario'],r['quota'],r['orientation']) for r in rows}!=expected or len(rows)!=16:
            raise ValueError('incomplete 16-stratum panel')
        rows.sort(key=lambda r:(r['orientation']=='reflected',SCENARIOS.index(r['scenario']),r['quota']))
    return groups
