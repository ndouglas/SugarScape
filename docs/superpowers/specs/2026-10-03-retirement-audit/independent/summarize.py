import csv,statistics as s,collections,json
from pathlib import Path
rows=list(csv.DictReader(open('seeds.csv')))
traces=collections.defaultdict(list)
for row in csv.DictReader(open('trace.csv')): traces[row['case'],row['seed']].append(row)
metrics=[]
for r in rows:
 ts=traces[r['case'],r['seed']];sw=int(r['switch']);elig=62 if sw>=0 else 65
 post=[x for x in ts if int(x['t'])>sw] if sw>=0 else ts
 streak=0;hit=-1;first=-1
 for x in post:
  good=int(x['mode10'])==elig
  streak=streak+1 if good else 0
  if good and first<0:first=int(x['t'])-(sw if sw>=0 else 0)
  if streak>=10 and hit<0:hit=int(x['t'])-(sw if sw>=0 else 0)
 pre=[x for x in ts if int(x['t'])<=sw][-10:] if sw>=0 else []
 def frac(xs):
  n=sum(int(x['event_count']) for x in xs)
  return sum(int(x['elig_age_events']) for x in xs)/n if n else 0
 r['switch10_elig_event_fraction']=frac(pre);r['first_target_mode10']=first;r['ten_consecutive_target_mode10']=hit
 r['last10_imitator_events']=sum(int(x['imitator_events']) for x in ts[-10:])
 r['last10_events']=sum(int(x['event_count']) for x in ts[-10:])
 r['final10_imitator_fraction']=r['last10_imitator_events']/r['last10_events'] if r['last10_events'] else 0
 r['post_last10_target_mode_rounds']=sum(int(x['mode10'])==elig for x in ts[-10:])
with open('seeds_enriched.csv','w') as f:
 w=csv.DictWriter(f,fieldnames=rows[0].keys());w.writeheader();w.writerows(rows)
summary=[]
for case in dict.fromkeys(r['case'] for r in rows):
 rr=[r for r in rows if r['case']==case]
 for metric in ('first95','first99','first100','switch','mode_at_switch','post95','post99','post100','final_share','final_mode10','switch10_elig_event_fraction','final10_elig_event_fraction','first_target_mode10','ten_consecutive_target_mode10','imitator_events','empty_decisions','decisions','births','truncated_birth_networks','empty_birth_networks','final10_imitator_fraction'):
  vals=[float(r[metric]) for r in rr if float(r[metric])>=0];sv=sorted(vals)
  q=lambda p: sv[int((len(sv)-1)*p)] if sv else ''
  summary.append(dict(case=case,metric=metric,n=len(rr),attained=len(vals),censored=len(rr)-len(vals),mean_conditional=s.mean(vals) if vals else '',sd_conditional=s.stdev(vals) if len(vals)>1 else '',min=q(0),q05=q(.05),median=s.median(vals) if vals else '',q95=q(.95),max=q(1)))
with open('summary.csv','w') as f:
 w=csv.DictWriter(f,fieldnames=summary[0].keys());w.writeheader();w.writerows(summary)
with open('selected_trace.csv','w') as f:
 w=csv.DictWriter(f,fieldnames=ts[0].keys());w.writeheader()
 for (case,seed),ts in traces.items():
  if seed=='2001':w.writerows(ts)
print('\n'.join(str(r) for r in summary if r['metric'] in ('first95','post95','mode_at_switch','ten_consecutive_target_mode10','final10_elig_event_fraction','final_share')))
