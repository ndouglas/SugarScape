"""Explicit prospective phase bindings; validate-only, no worlds/declaration."""
import argparse, json, subprocess
from pathlib import Path
from survey.democratic_peace import followup, records
from survey.democratic_peace.historical import historical_dataset
from survey.democratic_peace.followup_registration import freeze_manifest,phase_binding,new_outputs,source_inventory
from survey.democratic_peace.followup_runtime import prepare_build_receipt
p=argparse.ArgumentParser();p.add_argument('--build-evidence',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
a=p.parse_args();root=Path.cwd().resolve();out=a.output.resolve()
archive=Path('/Users/nathan/.local/share/sugarscape/evidence/democratic-peace-2026-10-04')
source=root/'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json';data=source.read_bytes();table=records.strict_json(data)
build=records.strict_json(a.build_evidence.read_bytes());binary=root/'survey/target/release/democratic_peace'
outputs=[out/'phase-bindings.json']
for phase in followup.PHASES:outputs.extend(out/f'{phase}.{s}' for s in ('manifest.json','receipt.json','resolved.json','validate.stdout.log','validate.stderr.log'))
new_outputs([source,a.build_evidence,binary,archive/'study-preservation-inventory.json',archive/'frozen-study-source.tar',archive/'democratic_peace',*(archive/'study').glob('*'),*[root/e['path'] for e in source_inventory(root)]],outputs)
old=historical_dataset(study_root=archive/'study',inventory_path=archive/'study-preservation-inventory.json',source_archive=archive/'frozen-study-source.tar',binary=archive/'democratic_peace')
out.mkdir(parents=True,exist_ok=True);bindings=[];resolved=[]
for phase in followup.PHASES:
    value=freeze_manifest(followup.build_manifest(table,data,phase,old['historical_input']),root,data)
    mp=out/f'{phase}.manifest.json';rp=out/f'{phase}.receipt.json';cp=out/f'{phase}.resolved.json'
    mp.write_text(json.dumps(value,indent=2)+'\n');rp.write_text(json.dumps(prepare_build_receipt(mp,binary,root,build),indent=2)+'\n')
    command=[str(binary),'--manifest',str(mp),'--receipt',str(rp),'--repo',str(root),'--resolved',str(cp),'--validate-only']
    with (out/f'{phase}.validate.stdout.log').open('wb') as stdout,(out/f'{phase}.validate.stderr.log').open('wb') as stderr:subprocess.run(command,stdout=stdout,stderr=stderr,check=True)
    bindings.append(phase_binding(mp,cp,rp,binary,root));resolved.append(records.strict_json(cp.read_bytes()))
for left,right in zip(resolved[0]['arms'],resolved[1]['arms']):
    assert {k for k in left['config'] if left['config'][k]!=right['config'][k]}=={'probability_direction'}
(out/'phase-bindings.json').write_text(json.dumps(bindings,indent=2)+'\n')
print(json.dumps({'phases':[b['phase'] for b in bindings],'arms_per_phase':[len(r['arms']) for r in resolved],'only_changed_rule':'probability_direction','registered_histories':0}))
