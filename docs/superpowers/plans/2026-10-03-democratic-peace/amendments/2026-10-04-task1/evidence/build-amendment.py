import copy, hashlib, importlib.util, json, os, shutil, subprocess, sys, uuid
from pathlib import Path
root=Path('/Users/nathan/.config/superpowers/worktrees/SugarScape/democratic-peace')
original=root/'docs/superpowers/plans/2026-10-03-democratic-peace'
assets=original/'amendments/2026-10-04-task1'
assert not assets.exists(), 'Amendment must be newly owned'
assets.mkdir(parents=True)
sha=lambda data:hashlib.sha256(data).hexdigest()
checks=[]
def run(args,cwd=root):
 p=subprocess.run(args,cwd=cwd,capture_output=True)
 checks.append({'command':args,'cwd':str(cwd),'exit_code':p.returncode,'stdout':p.stdout.decode(errors='replace'),'stderr':p.stderr.decode(errors='replace')})
 assert p.returncode==0, checks[-1]
 return p.stdout
manifest=json.loads((original/'manifest.json').read_text());amended=copy.deepcopy(manifest)
orig_inventory={p.name:{'sha256':sha(p.read_bytes()),'bytes':p.stat().st_size,'mode':'100755' if p.stat().st_mode&0o100 else '100644'} for p in original.iterdir() if p.is_file()}
original_checkpoint2=run(['git','rev-parse','64636d7']).decode().strip()
corrected=run(['git','rev-parse','4f6e791']).decode().strip()
fixes=[run(['git','rev-parse',ref]).decode().strip() for ref in ['39d4164','4f6e791']]
paths=run(['git','diff','--name-only',original_checkpoint2,corrected]).decode().splitlines()
expected=['crates/sugarscape-core/src/democratic_peace/claims.rs','crates/sugarscape-core/src/democratic_peace/territory.rs','crates/sugarscape-core/src/democratic_peace/tests.rs','crates/sugarscape-core/tests/golden.rs']
assert sorted(paths)==expected
for patch in manifest['patches'][2:]:assert not set(paths)&set(patch['changes']), patch['name']
for name in ['replay.py','test_replay.py','prepare_checks.py','register_study.py','publish_findings.py',*[p['name'] for p in manifest['patches'] if p['name']!='02-core.patch']]:shutil.copy2(original/name,assets/name)
for name in ['preparation-verification.json','replay-verification.json','plan-self-review.json','README.md']:shutil.copy2(original/name,assets/('original-'+name))
(assets/'original-packet-inventory.json').write_text(json.dumps(orig_inventory,sort_keys=True,indent=2)+'\n')
work=root.parent/('democratic-peace-task1-amend-checkpoint1-'+uuid.uuid4().hex[:12])
run(['git','worktree','add','--detach',str(work),manifest['base']])
run(['python3',str(original/'replay.py'),'--target',str(work),'--through','1'])
cp1changes=[p for p in manifest['checkpoints'][1] if manifest['checkpoints'][0].get(p)!=manifest['checkpoints'][1][p]]
run(['git','add','--',*cp1changes],work)
cp1tree=run(['git','write-tree'],work).decode().strip()
run(['python3',str(original/'replay.py'),'--target',str(work),'--through','2'])
for name in paths:
 data=run(['git','show',corrected+':'+name]);target=work/name;target.write_bytes(data)
 mode=run(['git','ls-tree',corrected,'--',name]).decode().split()[0];target.chmod(0o755 if mode=='100755' else 0o644)
 image={'sha256':sha(data),'mode':mode}
 for cp in amended['checkpoints'][2:]:cp[name]=image.copy()
changes={p:{'before':amended['checkpoints'][1].get(p),'after':amended['checkpoints'][2].get(p)} for p in sorted(amended['checkpoints'][1].keys()|amended['checkpoints'][2].keys()) if amended['checkpoints'][1].get(p)!=amended['checkpoints'][2].get(p)}
run(['git','add','--',*changes],work)
patch=run(['git','diff','--cached','--binary','--full-index',cp1tree],work)
(assets/'02-core.patch').write_bytes(patch)
amended['patches'][1]['changes']=changes;amended['patches'][1]['sha256']=sha(patch)
(assets/'manifest.json').write_text(json.dumps(amended,sort_keys=True,indent=2)+'\n')
evidence=assets/'evidence';evidence.mkdir()
logs=['dp-task1-fix-red','dp-task1-fix-focused-all','dp-task1-fix-discovery','dp-task1-fix-golden','dp-task1-fix-fmt','dp-task1-fix-clippy','dp-task1-fix2-red','dp-task1-fix2-census','dp-task1-fix2-domain','dp-task1-fix2-golden','dp-task1-fix2-fmt','dp-task1-fix2-clippy']
for name in logs:shutil.copy2(Path('/tmp')/(name+'.log'),evidence/(name+'.log'))
for name in ['task-1-report.md','task-1-review.md','task-1-fix1-review.md','task-1-fix2-review.md']:shutil.copy2(root/'.superpowers/sdd/2026-10-03-democratic-peace'/name,evidence/name)
shutil.copy2(Path(__file__),evidence/'build-amendment.py')
receipt={'classification':'premesurement_metadata_packaging_no_worlds_or_product_suite_reruns','source_base':manifest['base'],'original_checkpoint2':original_checkpoint2,'fix_commits':fixes,'corrected_checkpoint2':corrected,'changed_paths':paths,'checkpoint1_tree':cp1tree,'checkpoint1_worktree':str(work),'manifest_sha256':sha((assets/'manifest.json').read_bytes()),'patch_sha256':{p['name']:p['sha256'] for p in amended['patches']},'later_patch_overlap':[], 'checks':checks}
(assets/'construction-verification.json').write_text(json.dumps(receipt,sort_keys=True,indent=2)+'\n')
print(json.dumps({'assets':str(assets),'manifest_sha256':receipt['manifest_sha256'],'patch02_sha256':sha(patch),'checkpoint1_worktree':str(work)},indent=2))
