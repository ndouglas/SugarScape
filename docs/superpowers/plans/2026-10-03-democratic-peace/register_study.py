"""Freeze a prospectively selected population and validate it; never run worlds."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys


def write(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    parser.add_argument('--population', required=True, choices=['original_only', 'full_precision'])
    parser.add_argument('--runtime-receipt', required=True, type=Path)
    args = parser.parse_args()
    root = args.repo.resolve(strict=True)
    sys.path.insert(0, str(root))
    os.chdir(root)
    import numpy as np
    from survey.democratic_peace.methods import method_contract
    from survey.democratic_peace.records import strict_json, verify_source_inventory
    from survey.democratic_peace.manifest import build_manifest, expected_keys
    from survey.democratic_peace.provenance import freeze_manifest, required_inventory_paths
    from survey.democratic_peace.run import make_build_receipt, run_native, sha256_file
    if {'python': platform.python_version(), 'numpy': np.__version__} != method_contract()['runtime']:
        raise ValueError('registered runtime must match pinned methods')
    runtime_path = args.runtime_receipt.resolve(strict=True)
    runtime = strict_json(runtime_path.read_bytes())
    if runtime['classification'] != 'unregistered_runtime_probe':
        raise ValueError('requires runtime-only receipt')
    full_allowed = runtime['full_scheduling_gate_passed']
    if (args.population == 'full_precision') != full_allowed:
        raise ValueError('population differs from the prospectively declared runtime gate')
    if type(full_allowed) is not bool or runtime['registered_histories'] != 0:
        raise ValueError('runtime receipt may not contain registered results')
    # Every normative file must already have a clean committed source identity.
    required = required_inventory_paths(root)
    changed = subprocess.check_output(['git', 'diff', 'HEAD', '--name-only', '--', *required], text=True).splitlines()
    tracked = set(subprocess.check_output(['git', 'ls-files', '--', *required], text=True).splitlines())
    if changed or set(required) - tracked:
        raise ValueError('commit all normative sources before registration')
    out = args.out.resolve()
    if out.exists() and any(out.iterdir()):
        raise ValueError('registration requires an empty new output directory')
    subprocess.run(['git', 'check-ignore', str(out / 'sessions.jsonl')], check=True, capture_output=True)
    out.mkdir(parents=True, exist_ok=True)
    table = root / 'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json'
    source_bytes = table.read_bytes()
    provisional = build_manifest(strict_json(source_bytes), source_bytes,
                                 precision_registered=args.population == 'full_precision')
    frozen = freeze_manifest(provisional, root, source_bytes)
    docs_manifest = root / 'docs/superpowers/specs/2026-10-03-democratic-peace-manifest.json'
    if docs_manifest.exists():
        raise ValueError('existing scientific declaration must be preserved as a separate study')
    write(docs_manifest, frozen)
    manifest_path = out / 'manifest.json'
    write(manifest_path, frozen)
    write(out / 'registration-decision.json', {
        'classification': 'prospective_registration_no_periods', 'population': args.population,
        'runtime_receipt_sha256': sha256_file(runtime_path), 'runtime_receipt': runtime,
        'registered_attempts': len(expected_keys(frozen)),
        'git_source_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()})
    pre = verify_source_inventory(root, frozen['source_inventory'])
    command = ['cargo', 'build', '--release', '--locked', '--manifest-path', 'survey/Cargo.toml', '--bin', 'democratic_peace']
    rustc = subprocess.check_output(['rustc', '-vV'], text=True)
    host = next(line.split(': ', 1)[1] for line in rustc.splitlines() if line.startswith('host: '))
    from prepare_checks import effective_flags
    flags, config_inputs = effective_flags(root, host)
    write(out / 'cargo-configuration.json', config_inputs)
    with (out / 'build.log').open('w') as log:
        built = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
    built.check_returncode()
    toolchain = {'target': next(line.split(': ', 1)[1] for line in rustc.splitlines() if line.startswith('host: ')),
                 'rustc_version': rustc, 'cargo_version': subprocess.check_output(['cargo', '--version'], text=True).strip()}
    binary = root / 'survey/target/release/democratic_peace'
    if os.environ.get('CARGO_TARGET_DIR'):
        binary = Path(os.environ['CARGO_TARGET_DIR']).resolve() / 'release/democratic_peace'
    receipt = make_build_receipt(manifest_path, binary, root,
        prebuild_inventory_sha256=pre, build_command=command, build_exit_code=built.returncode,
        toolchain=toolchain, lockfile_paths=['Cargo.lock', 'survey/Cargo.lock'], build_flags=flags)
    write(out / 'build-receipt.json', receipt)
    checked = run_native(binary, manifest_path, validate=True, resolved_out=out / 'resolved.json',
                         build_receipt=out / 'build-receipt.json', source_root=root)
    (out / 'validation.stdout.log').write_text(checked.stdout)
    (out / 'validation.stderr.log').write_text(checked.stderr)
    print(checked.stdout, end='')
    print('No worlds constructed. Commit the docs declaration, then run the plan command with these exact bindings.')


if __name__ == '__main__':
    main()
