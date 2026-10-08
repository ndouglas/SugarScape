"""Read-only full-group rescan after empirical reporting correction; no inference."""
import hashlib
import json
from pathlib import Path
from diagnostics_p4 import summarize
from stream_p4 import load
import plot_p4

root=Path('recovery-correction-scan')
root.mkdir()
old=json.loads(Path('presentation-analysis.json').read_text())
source=Path(old['presentation_projection']['path'])
members={}
with (root/'group-diagnostics.jsonl').open('x') as stream:
    def save(group):
        assert [f['tick'] for f in group['frames']]==list(range(65))
        for member in group['members']:
            condition,seed=member.rsplit(' seed ',1)
            key=(condition,int(seed))
            assert key not in members
            members[key]=group['id']
        stream.write(json.dumps(summarize(group),sort_keys=True,allow_nan=False)+'\n')
    projection,binding=load(source,group_callback=save)
assert binding==old['presentation_projection']
assert plot_p4.canonical(projection)==Path('presentation-analysis.json').read_bytes()
assert members=={(e['condition'],e['seed']):e['biological_group'] for e in old['endpoints']}
assert len(members)==3840
result=dict(classification='complete_read_only_reporting_recovery_correction_scan',source=binding,
    all1760_groups_and3840_memberships_rechecked=True,presentation_analysis_actual_byte_equal=True,
    paired_estimates_recalculated=False,original_data_modified=False,
    correction='physical recovery from owner positive dug at originalsource; departureflag recorded separately; allphysical recoveryframes retained')
(root/'receipt.json').write_bytes(plot_p4.canonical(result))
print(json.dumps(result))
