"""Read-only census of every old archived JSONL, never world construction."""
import hashlib,json,re
from pathlib import Path
from survey.democratic_peace import followup
root=Path('/Users/nathan/.local/share/sugarscape/evidence/democratic-peace-2026-10-04')
reserved={arm['first_seed']+r for phase in followup.PHASES for arm in followup.canonical_arms(phase) for r in range(100)}
seen={};files=[];overlap=[]
for path in sorted(root.rglob('*.jsonl')):
    data=path.read_bytes();digest=hashlib.sha256(data).hexdigest()
    files.append({'path':str(path.relative_to(root)),'bytes':len(data),'sha256':digest})
    if digest in seen:continue
    counts={'registered':0,'runtime_probe':0,'fixture':0,'other':0,'malformed':0}
    for i,line in enumerate(data.splitlines(),1):
        try:row=json.loads(line)
        except (ValueError,UnicodeDecodeError):
            counts['malformed']+=1
            for match in re.finditer(rb'"seed"\s*:\s*([0-9]+)',line):
                if int(match[1]) in reserved:overlap.append({'path':str(path),'line':i,'status':'ambiguous_reserved_seed'})
            continue
        mode=row.get('execution_mode') if isinstance(row,dict) else None
        counts[mode if mode in ('registered','runtime_probe','fixture') else 'other']+=1
        if isinstance(row,dict) and type(row.get('seed')) is int and row['seed'] in reserved:
            overlap.append({'path':str(path),'line':i,'seed':row['seed'],'mode':mode})
    seen[digest]=counts
result={'classification':'historical_archive_seed_audit_only','archived_files':files,'unique_payload_census':seen,'reserved_seed_count':len(reserved),'overlaps':overlap}
print(json.dumps(result,indent=2))
if overlap:raise SystemExit('Declared fresh seed observed or ambiguous in old archive')
