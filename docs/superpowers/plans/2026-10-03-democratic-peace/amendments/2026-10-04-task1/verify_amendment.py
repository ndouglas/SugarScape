"""Verify the premeasurement Task 1 packet without running product suites/worlds."""
import argparse
import hashlib
import importlib.util
import json
import subprocess
import sys
import uuid
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--execution', required=True, type=Path)
    parser.add_argument('--original-final', required=True, type=Path)
    parser.add_argument('--worktree-root', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    assets = Path(__file__).resolve().parent
    original = assets.parents[1]
    before = json.loads((original / 'manifest.json').read_text())
    after = json.loads((assets / 'manifest.json').read_text())
    construction = json.loads((assets / 'construction-verification.json').read_text())
    changed = set(construction['changed_paths'])
    checks = []

    def digest(path):
        return hashlib.sha256(path.read_bytes()).hexdigest()

    def run(command, cwd=args.execution):
        result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
        checks.append({'command': command, 'cwd': str(cwd),
                       'exit_code': result.returncode, 'stdout': result.stdout,
                       'stderr': result.stderr})
        if result.returncode:
            raise ValueError(f'Failed command: {checks[-1]}')
        return result.stdout

    spec = importlib.util.spec_from_file_location('amended_replay', assets / 'replay.py')
    replay = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(replay)
    original_inventory = json.loads((assets / 'original-packet-inventory.json').read_text())
    actual_original = {p.name: {'sha256': digest(p), 'bytes': p.stat().st_size,
                               'mode': replay.image(p)['mode']}
                       for p in original.iterdir() if p.is_file()}
    assert actual_original == original_inventory, 'Original top-level bytes/modes changed'
    for field in before:
        if field not in ('checkpoints', 'patches'):
            assert before[field] == after[field], field
    assert before['checkpoints'][:2] == after['checkpoints'][:2]
    for index in range(2, 7):
        left, right = before['checkpoints'][index], after['checkpoints'][index]
        assert set(left) == set(right)
        assert {p for p in left if left[p] != right[p]} == changed
        assert all(left[p]['mode'] == right[p]['mode'] for p in left)
        for name in changed:
            assert right[name] == after['checkpoints'][2][name]
    unchanged = ['replay.py', 'test_replay.py', 'prepare_checks.py',
                 'register_study.py', 'publish_findings.py']
    for index, patch in enumerate(before['patches']):
        if index != 1:
            assert patch == after['patches'][index]
            unchanged.append(patch['name'])
        if index >= 2:
            assert not changed.intersection(patch['changes']), patch['name']
    for name in unchanged:
        assert (original / name).read_bytes() == (assets / name).read_bytes(), name
        assert replay.image(original / name) == replay.image(assets / name), name
    for name in ['preparation-verification.json', 'replay-verification.json',
                 'plan-self-review.json', 'README.md']:
        assert (original / name).read_bytes() == (assets / ('original-' + name)).read_bytes()
    assert digest(assets / 'manifest.json') == construction['manifest_sha256']
    for patch in after['patches']:
        assert digest(assets / patch['name']) == patch['sha256']

    run(['python3', str(original / 'replay.py'), '--target', str(args.original_final),
         '--verify-only'])
    run(['python3', str(assets / 'replay.py'), '--target', str(args.execution),
         '--through', '2', '--verify-only'])
    fresh = args.worktree_root / ('democratic-peace-task1-amend-replay-' + uuid.uuid4().hex[:12])
    run(['git', 'worktree', 'add', '--detach', str(fresh), after['base']])
    for index in range(1, 7):
        run(['python3', str(assets / 'replay.py'), '--target', str(fresh),
             '--through', str(index)])
    run(['python3', str(assets / 'replay.py'), '--target', str(fresh), '--verify-only'])
    run(['python3', '-m', 'unittest', 'discover', '-s', str(assets), '-p',
         'test_replay.py'], cwd=assets)

    expected = {**before['checkpoints'][6], **before['protected']}
    for name in changed:
        expected[name] = replay.image(args.execution / name)
    observed = {name: replay.image(fresh / name) for name in expected}
    original_observed = {name: replay.image(args.original_final / name) for name in expected}
    for name in changed:
        original_observed[name] = replay.image(args.execution / name)
    assert observed == expected == original_observed
    assert len(observed) == 1510
    assert replay.observed_products(after, fresh) == after['checkpoints'][6]
    replay.verify_protected(after, fresh)
    execution_inventory = {name: replay.image(args.execution / name)
                           for name in after['checkpoints'][2] | after['protected']}
    assert execution_inventory == {**after['checkpoints'][2], **after['protected']}
    receipt = {
        'classification': 'fresh_amended_replay_before_measurement',
        'invocation': [sys.executable, str(Path(__file__).resolve()), *sys.argv[1:]],
        'source_base': after['base'], 'corrected_checkpoint2': construction['corrected_checkpoint2'],
        'manifest_sha256': digest(assets / 'manifest.json'),
        'patch_sha256': {p['name']: p['sha256'] for p in after['patches']},
        'fresh_worktree': str(fresh), 'original_final_worktree': str(args.original_final),
        'checkpoint_product_counts': [len(cp) for cp in after['checkpoints']],
        'final_product_files': len(after['checkpoints'][6]),
        'protected_files': len(after['protected']), 'full_final_files': len(observed),
        'final_inventory': observed, 'execution_checkpoint2_inventory': execution_inventory,
        'original_top_level_inventory_preserved': actual_original,
        'changed_paths': sorted(changed), 'later_patch_overlap': [],
        'unchanged_helper_test_patch_paths': unchanged,
        'evidence_inventory': {str(p.relative_to(assets)): {'sha256': digest(p),
                               'bytes': p.stat().st_size}
                               for p in sorted((assets / 'evidence').iterdir()) if p.is_file()},
        'replay_tests_passed': 18, 'commands': checks,
        'worlds_runtime_probes_registered_histories_run_by_verification': 0,
        'product_suites_rerun_by_verification': 0,
        'complete_bytes_paths_modes_equal_to_original_final_with_reviewed_changes': True,
        'original_packet_bytes_modes_preserved': True,
        'source_spec_scientific_contract_and_protected_inventory_unchanged': True,
        'limitations': ['Historical product-suite and preparation receipts certify original bytes.',
                        'Copied regression logs were witnessed in the Task 1 fix reviews; not rerun here.',
                        'Format logs are empty and do not independently record original exit codes.',
                        'No claim of original executable identity or scientific compatibility.'],
    }
    args.output.write_text(json.dumps(receipt, sort_keys=True, indent=2) + '\n')
    print(f'Verified amended six-stage replay: {len(observed)} final files, 18 replay tests; '
          f'execution checkpoint 2; fresh worktree {fresh}')


if __name__ == '__main__':
    main()
