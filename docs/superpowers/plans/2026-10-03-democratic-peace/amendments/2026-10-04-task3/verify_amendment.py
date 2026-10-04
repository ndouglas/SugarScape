"""Verify the Task3 metadata packet without product suites, probes or worlds."""
import argparse
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import uuid
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execution', required=True, type=Path)
    parser.add_argument('--prior-final', required=True, type=Path)
    parser.add_argument('--worktree-root', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    assets = Path(__file__).resolve().parent
    original = assets.parents[1]
    prior = assets.parent / '2026-10-04-task1'
    before = json.loads((prior / 'manifest.json').read_text())
    after = json.loads((assets / 'manifest.json').read_text())
    construction = json.loads((assets / 'construction-verification.json').read_text())
    changed = set(construction['changed_paths'])
    checks = []

    def digest(path):
        return hashlib.sha256(path.read_bytes()).hexdigest()

    def run(command, cwd=args.execution):
        env = dict(os.environ, PYTHONDONTWRITEBYTECODE='1')
        result = subprocess.run(command, cwd=cwd, env=env, capture_output=True, text=True)
        checks.append(dict(command=command, cwd=str(cwd), exit_code=result.returncode,
                           stdout=result.stdout, stderr=result.stderr))
        if result.returncode:
            raise ValueError(f'Failed command: {checks[-1]}')
        return result.stdout

    spec = importlib.util.spec_from_file_location('amended_replay', assets / 'replay.py')
    replay = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(replay)

    def inventory(directory, recursive):
        files = directory.rglob('*') if recursive else directory.iterdir()
        return {str(p.relative_to(directory)): dict(**replay.image(p), bytes=p.stat().st_size)
                for p in sorted(files) if p.is_file()}

    original_inventory = json.loads((assets / 'original-packet-inventory.json').read_text())
    prior_inventory = json.loads((assets / 'prior-task1-packet-inventory.json').read_text())
    assert inventory(original, False) == original_inventory, 'Original packet changed'
    assert inventory(prior, True) == prior_inventory, 'Task1 packet changed'
    assert before['checkpoints'][:6] == after['checkpoints'][:6]
    for field in before:
        if field not in ('checkpoints', 'patches'):
            assert before[field] == after[field], field
    left, right = before['checkpoints'][6], after['checkpoints'][6]
    assert set(left) == set(right)
    assert {p for p in left if left[p] != right[p]} == changed
    assert all(left[p]['mode'] == right[p]['mode'] for p in left)
    unchanged = ['replay.py', 'test_replay.py', 'prepare_checks.py',
                 'register_study.py', 'publish_findings.py']
    for index, patch in enumerate(before['patches']):
        if index < 5:
            assert patch == after['patches'][index]
            unchanged.append(patch['name'])
    for name in unchanged:
        assert replay.image(prior / name) == replay.image(assets / name), name
    assert (assets / 'historical-task1-manifest.json').read_bytes() == (prior / 'manifest.json').read_bytes()
    for name in ['preparation-verification.json', 'replay-verification.json', 'plan-self-review.json', 'README.md']:
        assert (assets / ('historical-original-' + name)).read_bytes() == (original / name).read_bytes()
    for name in ['construction-verification.json', 'replay-verification.json', 'metadata-final-checks.json', 'README.md']:
        assert (assets / ('historical-task1-' + name)).read_bytes() == (prior / name).read_bytes()
    assert digest(assets / 'manifest.json') == construction['manifest_sha256']
    for patch in after['patches']:
        assert digest(assets / patch['name']) == patch['sha256']
    retained_prior_verification = json.loads((prior / 'replay-verification.json').read_text())
    assert str(args.prior_final) == retained_prior_verification['fresh_worktree']
    run(['python3', str(prior / 'replay.py'), '--target', str(args.prior_final), '--verify-only'])
    run(['python3', str(assets / 'replay.py'), '--target', str(args.execution), '--verify-only'])
    fresh = args.worktree_root / ('democratic-peace-task3-amend-replay-' + uuid.uuid4().hex[:12])
    run(['git', 'worktree', 'add', '--detach', str(fresh), after['base']])
    for index in range(1, 7):
        run(['python3', str(assets / 'replay.py'), '--target', str(fresh), '--through', str(index)])
    run(['python3', str(assets / 'replay.py'), '--target', str(fresh), '--verify-only'])
    test_output = run(['python3', '-m', 'unittest', 'discover', '-s', str(assets), '-p', 'test_replay.py'], cwd=assets)
    assert 'Ran 18 tests' in checks[-1]['stderr'] and '\nOK\n' in checks[-1]['stderr']
    expected = {**before['checkpoints'][6], **before['protected']}
    for name in changed:
        expected[name] = replay.image(args.execution / name)
    observed = {name: replay.image(fresh / name) for name in expected}
    substituted_prior = {name: replay.image(args.prior_final / name) for name in expected}
    for name in changed:
        substituted_prior[name] = replay.image(args.execution / name)
    assert observed == expected == substituted_prior
    assert len(observed) == 1510
    assert replay.observed_products(after, fresh) == after['checkpoints'][6]
    replay.verify_protected(after, fresh)
    execution_inventory = {name: replay.image(args.execution / name) for name in expected}
    assert execution_inventory == observed
    assert inventory(original, False) == original_inventory
    assert inventory(prior, True) == prior_inventory
    receipt = dict(
        classification='actual_fresh_task3_amended_replay_before_measurement',
        invocation=[sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]],
        source_base=after['base'], corrected_checkpoint6=construction['corrected_checkpoint6'],
        manifest_sha256=digest(assets / 'manifest.json'),
        patch_sha256={p['name']: p['sha256'] for p in after['patches']},
        fresh_worktree=str(fresh), prior_task1_final_worktree=str(args.prior_final),
        checkpoint_product_counts=[len(cp) for cp in after['checkpoints']],
        final_product_files=len(after['checkpoints'][6]), protected_files=len(after['protected']),
        full_final_files=len(observed), final_inventory=observed,
        execution_checkpoint6_inventory=execution_inventory, changed_paths=sorted(changed),
        original_top_level_inventory_preserved=original_inventory,
        prior_task1_packet_inventory_preserved=prior_inventory,
        unchanged_helper_test_patch_paths=unchanged,
        evidence_inventory=inventory(assets / 'evidence', True), replay_tests_passed=18,
        commands=checks, worlds_runtime_probes_registered_histories_run_by_verification=0,
        product_suites_rerun_by_verification=0,
        complete_bytes_paths_modes_equal_to_prior_task1_final_with_reviewed_changes=True,
        original_and_task1_packet_bytes_modes_preserved=True,
        checkpoints0_through5_source_spec_scientific_contract_and_protected_inventory_unchanged=True,
        limitations=['Inherited original/Task1 receipts are historical and do not certify corrected protocol bytes.',
                     'Task3 witnessed RED/GREEN and preparation receipts are retained, not rerun by packaging.',
                     'Six-case scheduling forecasts exclude unbenchmarked full analysis and are not runtime guarantees.',
                     'No registered histories, scientific comparison or original-code identity established.'])
    args.output.write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
    print(f'Verified all six checkpoints; {len(observed)} final path/byte/mode images; '
          f'18 replay tests; execution checkpoint 6; fresh worktree {fresh}')


if __name__ == '__main__':
    main()
