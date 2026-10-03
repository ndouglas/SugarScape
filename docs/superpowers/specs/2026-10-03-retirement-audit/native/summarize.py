from pathlib import Path
import csv,statistics,math,collections
p=Path(__file__).parent
metrics=['switch','first95','first99','first100','sustained65','sustained62_post','proxy_new','group_a','group_b','final_share','final_mode','imitator_events','dips95','dips99','dips100','previous_tick_mode_at_switch']
rows=[]
for f in [p/'selected-seeds.csv']:
 groups=collections.defaultdict(list)
 for r in csv.DictReader(f.open()): groups[r['case']].append(r)
 for case,rs in groups.items():
  for m in metrics:
   vals=[float(r[m]) for r in rs if r[m] and math.isfinite(float(r[m]))]
   rows.append(dict(case=case,metric=m,n=len(rs),attained=len(vals),censored=len(rs)-len(vals),conditional_mean=statistics.mean(vals) if vals else '',conditional_sample_sd=statistics.stdev(vals) if len(vals)>1 else '',conditional_median=statistics.median(vals) if vals else '',conditional_min=min(vals) if vals else '',conditional_max=max(vals) if vals else ''))
with (p/'summary.csv').open('w') as f:
 w=csv.DictWriter(f,fieldnames=list(rows[0]));w.writeheader();w.writerows(rows)
for r in rows:
 if r['metric'] in ['proxy_new','sustained62_post'] and 'policy' in r['case'] or r['metric'] in ['first95','first99','first100'] and 'trajectory' in r['case'] or r['metric'] in ['group_a','group_b'] and 'groups' in r['case']:
  print(r['case'],r['metric'],r['attained'],r['n'],r['conditional_mean'],r['conditional_sample_sd'])
