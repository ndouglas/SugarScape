"""Construct only the reviewed Task3 metadata amendment; no product execution."""
import copy
import hashlib
import json
import shutil
import subprocess
import uuid
from pathlib import Path

root = Path('/Users/nathan/.config/superpowers/worktrees/SugarScape/democratic-peace')
original = root / 'docs/superpowers/plans/2026-10-03-democratic-peace'
prior = original / 'amendments/2026-10-04-task1'
assets = original / 'amendments/2026-10-04-task3'
assert not assets.exists(), 'New owned destination required'
checks = []
sha = lambda data: hashlib.sha256(data).hexdigest()
def run(command, cwd=root):
    result = subprocess.run(command, cwd=cwd, capture_output=True)
    checks.append(dict(command=command, cwd=str(cwd), exit_code=result.returncode,
                       stdout=result.stdout.decode(errors='replace'), stderr=result.stderr.decode(errors='replace')))
    assert result.returncode == 0, checks[-1]
    return result.stdout
def image(p):
    return dict(sha256=sha(p.read_bytes()), mode='100755' if p.stat().st_mode & 0o100 else '100644')
def inventory(directory, recursive):
    files = directory.rglob('*') if recursive else directory.iterdir()
    return {str(p.relative_to(directory)): dict(**image(p), bytes=p.stat().st_size)
            for p in sorted(files) if p.is_file()}
prior_inventory = inventory(prior, True)
original_inventory = inventory(original, False)
before = json.loads((prior / 'manifest.json').read_text())
after = copy.deepcopy(before)
old = run(['git', 'rev-parse', 'edb15b6']).decode().strip()
corrected = run(['git', 'rev-parse', 'fb41c0b']).decode().strip()
assert run(['git', 'rev-parse', 'HEAD']).decode().strip() == corrected
paths = run(['git', 'diff', '--name-only', old, corrected]).decode().splitlines()
expected = ['survey/democratic_peace/analysis.py', 'survey/democratic_peace/test_analysis.py',
            'survey/democratic_peace/test_manifest.py', 'survey/src/bin/democratic_peace.rs',
            'survey/tests/democratic_peace_native.rs']
assert paths == expected, paths
assets.mkdir()
helpers = ['replay.py', 'test_replay.py', 'prepare_checks.py', 'register_study.py', 'publish_findings.py']
for name in helpers + [p['name'] for p in before['patches'][:5]]:
    shutil.copy2(prior / name, assets / name)
for name in ['preparation-verification.json', 'replay-verification.json', 'plan-self-review.json', 'README.md']:
    shutil.copy2(original / name, assets / ('historical-original-' + name))
for name in ['construction-verification.json', 'replay-verification.json', 'metadata-final-checks.json', 'README.md']:
    shutil.copy2(prior / name, assets / ('historical-task1-' + name))
(assets / 'original-packet-inventory.json').write_text(json.dumps(original_inventory, sort_keys=True, indent=2) + '\n')
(assets / 'prior-task1-packet-inventory.json').write_text(json.dumps(prior_inventory, sort_keys=True, indent=2) + '\n')
(assets / 'historical-task1-manifest.json').write_bytes((prior / 'manifest.json').read_bytes())
work = root.parent / ('democratic-peace-task3-amend-checkpoint5-' + uuid.uuid4().hex[:12])
run(['git', 'worktree', 'add', '--detach', str(work), before['base']])
run(['python3', str(prior / 'replay.py'), '--target', str(work), '--through', '5'])
cp5changes = [p for p in before['checkpoints'][5] if before['checkpoints'][0].get(p) != before['checkpoints'][5][p]]
run(['git', 'add', '--', *cp5changes], work)
cp5tree = run(['git', 'write-tree'], work).decode().strip()
run(['python3', str(prior / 'replay.py'), '--target', str(work), '--through', '6'])
for name in paths:
    data = run(['git', 'show', corrected + ':' + name])
    mode = run(['git', 'ls-tree', corrected, '--', name]).decode().split()[0]
    assert (root / name).read_bytes() == data
    target = work / name
    target.write_bytes(data)
    target.chmod(0o755 if mode == '100755' else 0o644)
    after['checkpoints'][6][name] = dict(sha256=sha(data), mode=mode)
changes = {p: dict(before=after['checkpoints'][5].get(p), after=after['checkpoints'][6].get(p))
           for p in sorted(after['checkpoints'][5].keys() | after['checkpoints'][6].keys())
           if after['checkpoints'][5].get(p) != after['checkpoints'][6].get(p)}
run(['git', 'add', '--', *changes], work)
patch = run(['git', 'diff', '--cached', '--binary', '--full-index', cp5tree], work)
(assets / '06-protocol.patch').write_bytes(patch)
after['patches'][5]['changes'] = changes
after['patches'][5]['sha256'] = sha(patch)
(assets / 'manifest.json').write_text(json.dumps(after, sort_keys=True, indent=2) + '\n')
evidence = assets / 'evidence'
evidence.mkdir()
for name in ['task-3-report.md', 'task-3-review.md', 'task-3-fix1-review.md']:
    shutil.copy2(root / '.superpowers/sdd/2026-10-03-democratic-peace' / name, evidence / name)
shutil.copytree(root / '.superpowers/sdd/2026-10-03-democratic-peace/task3-fix1-evidence', evidence / 'task3-fix1-evidence')
shutil.copy2('/tmp/dp-task3-review-focused-evidence.json', evidence / 'dp-task3-review-focused-evidence.json')
shutil.copy2(__file__, evidence / 'build-amendment.py')
receipt = dict(classification='actual_metadata_construction_no_worlds_or_product_suite_reruns', source_base=before['base'],
               original_checkpoint6=old, fix_commits=[corrected], corrected_checkpoint6=corrected, changed_paths=paths,
               checkpoint5_tree=cp5tree, checkpoint5_worktree=str(work), invocation=['python3', str(Path(__file__).resolve())],
               manifest_sha256=sha((assets / 'manifest.json').read_bytes()),
               patch_sha256={p['name']: p['sha256'] for p in after['patches']}, checks=checks)
(assets / 'construction-verification.json').write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
print(json.dumps({k: receipt[k] for k in ['manifest_sha256', 'patch_sha256', 'checkpoint5_worktree', 'changed_paths']}, indent=2))
