"""Complete membership expansion of retained occupancy/departure records.

Source occupancy counts here cover retained positive-valued source choices; full
unfiltered raw choices remain in immutable analysis.json. This is not inference.
"""
from collections import defaultdict,Counter
import json
from pathlib import Path
counts=defaultdict(Counter)
for line in Path('group-diagnostics.jsonl').open():
    g=json.loads(line)
    occupied=sum(c['target_occupant'] is not None for c in g['receiver_source_choice_records'])
    departures=sum(v for k,v in g['action_counts'].items() if k.startswith('1:departure:'))
    failed=any(k.split(':',1)[1]!='arrived' for k in g['walk_outcome_counts'])
    for member in g['members']:
        ident,_=member.rsplit(' seed ',1)
        counts[ident]['positive_valued_source_occupied_episodes']+=occupied>0
        counts[ident]['owner_departure_episodes']+=departures>0
        counts[ident]['nonarrived_walk_episodes']+=failed
        counts[ident]['episode_count']+=1
assert len(counts)==96 and all(value['episode_count']==40 for value in counts.values())
data=json.loads(Path('cell-diagnostics.json').read_text())
for cell in data['cells']:
    cell['additional_occupancy_departure_counts']=dict(counts[cell['condition']['id']])
Path('cell-diagnostics.json').write_text(json.dumps(data,indent=2,sort_keys=True,allow_nan=False)+'\n')
print(json.dumps(dict(cells=len(counts),all_episode_counts40=True,
    positive_valued_source_occupied_episodes=sum(v['positive_valued_source_occupied_episodes'] for v in counts.values()),
    nonarrived_walk_episodes=sum(v['nonarrived_walk_episodes'] for v in counts.values()),
    preserved_reporting_counter_correction='earlier exploratory counter wrongly treated actor2:arrived as failure; '
    'correct criterion compares outcome suffix; original study records untouched')))
