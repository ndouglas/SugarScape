"""Select source-domain native cells without modifying measured values."""
import csv,sys,subprocess
from pathlib import Path
p=Path(sys.argv[1])
def keep(c):return c.startswith(('trajectory_','critical_Slot_','extent_','spread-source_','groups-source_','policy_','counts_','cohort-source_'))
rows=[]
for f in sorted(p.glob('*-seeds.csv')):
 if f.name=='selected-seeds.csv':continue
 with f.open() as stream:rows.extend(r for r in csv.DictReader(stream) if keep(r['case']))
keys=[(r['case'],r['seed']) for r in rows]
assert len(keys)==len(set(keys)), 'Duplicate case/seed rows'
with (p/'selected-seeds.csv').open('w') as stream:
 w=csv.DictWriter(stream,fieldnames=rows[0].keys());w.writeheader();w.writerows(rows)
# Use package summarizer against selected output directory, preserving its logic.
script=(Path(__file__).parent/'native/summarize.py').read_text().replace('p=Path(__file__).parent','p=Path('+repr(str(p.resolve()))+')')
exec(compile(script,'native/summarize.py','exec'),{'__file__':str(Path(__file__).resolve())})
