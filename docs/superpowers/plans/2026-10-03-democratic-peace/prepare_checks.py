#!/usr/bin/env python3
"""Build and check bounded unregistered evidence; never freeze or run a study.

Run from any directory with --repo pointing to a prepared scratch checkout and
--out pointing to a new, ignored directory. Requires the checkout's Python deps.
"""
import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import signal
import subprocess
import sys
import time
import tomllib


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + '\n')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def command(argv, repo, out, label, *, limit=None, reject=False):
    started = time.perf_counter()
    process = subprocess.Popen(list(map(str, argv)), cwd=repo, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, text=True, start_new_session=True)
    timed_out = False
    try:
        stdout, stderr = process.communicate(timeout=limit)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(process.pid, signal.SIGTERM)
        try:
            stdout, stderr = process.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
    result = dict(command=list(map(str, argv)), cwd=str(repo), returncode=process.returncode,
        elapsed_wall_seconds=time.perf_counter()-started, timed_out=timed_out)
    (out/(label+'.stdout.log')).write_text(stdout)
    (out/(label+'.stderr.log')).write_text(stderr)
    write_json(out/(label+'.command.json'), result)
    if limit is not None and timed_out:
        return result, stdout, stderr
    require(process.returncode != 0 if reject else process.returncode == 0,
        f'{label}: unexpected exit {process.returncode}; see retained logs')
    return result, stdout, stderr


def effective_flags(repo, host):
    """Capture Cargo's selected rustflags and the inputs used to select them."""
    cargo_home = Path(os.environ.get('CARGO_HOME', str(Path.home()/'.cargo')))
    paths = []
    for base in [cargo_home, *[p/'.cargo' for p in reversed([repo, *repo.parents])]]:
        chosen = next((base/name for name in ('config', 'config.toml') if (base/name).is_file()), None)
        if chosen is not None and chosen not in paths:
            paths.append(chosen)
    configs = [tomllib.loads(p.read_text()) for p in paths]
    inputs = [{'path': str(p), 'sha256': sha(p.read_bytes()), 'parsed': c}
        for p, c in zip(paths, configs)]
    env = {k: os.environ[k] for k in ('CARGO_ENCODED_RUSTFLAGS', 'RUSTFLAGS',
        'CARGO_BUILD_RUSTFLAGS', 'CARGO_BUILD_TARGET',
        'CARGO_TARGET_'+host.upper().replace('-', '_')+'_RUSTFLAGS') if k in os.environ}
    def split(value):
        return value if isinstance(value, list) else shlex.split(value)
    if 'CARGO_ENCODED_RUSTFLAGS' in env:
        flags = env['CARGO_ENCODED_RUSTFLAGS'].split('\x1f')
    elif 'RUSTFLAGS' in env:
        flags = shlex.split(env['RUSTFLAGS'])
    else:
        target_env = 'CARGO_TARGET_'+host.upper().replace('-', '_')+'_RUSTFLAGS'
        selected = []
        for c in configs:
            for key, value in c.get('target', {}).items():
                require(not key.startswith('cfg(') or 'rustflags' not in value,
                    'cfg target rustflags require explicit RUSTFLAGS for reproducible capture')
            selected.extend(split(c.get('target', {}).get(host, {}).get('rustflags', [])))
        if target_env in env:
            selected.extend(shlex.split(env[target_env]))
        flags = selected or [flag for c in configs for flag in split(c.get('build', {}).get('rustflags', []))]
        if not selected and 'CARGO_BUILD_RUSTFLAGS' in env:
            flags.extend(shlex.split(env['CARGO_BUILD_RUSTFLAGS']))
    require(not env.get('CARGO_BUILD_TARGET') or env['CARGO_BUILD_TARGET'] == host,
        'cross-target preparation is unsupported')
    require(all(c.get('build', {}).get('target', host) == host for c in configs),
        'cross-target Cargo config is unsupported')
    return flags, {'config_inputs': inputs, 'environment': env}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    repo, out = args.repo.resolve(), args.out.resolve()
    require(repo.is_dir(), 'repository does not exist')
    require(not out.exists() or not any(out.iterdir()), 'output must be new or empty; never overwrite evidence')
    require(out.is_relative_to(repo), 'output must be inside repository ignored evidence directory')
    ignored = subprocess.run(['git', 'check-ignore', str(out/'probe')], cwd=repo,
        capture_output=True, text=True)
    require(ignored.returncode == 0, 'output prefix must be ignored before preparation')
    out.mkdir(parents=True, exist_ok=True)
    sys.path.insert(0, str(repo))
    from survey.democratic_peace.manifest import build_manifest, build_unregistered_manifest, expected_keys
    from survey.democratic_peace.provenance import bind_unregistered_manifest, verify_source_review
    from survey.democratic_peace.records import strict_json, read_sessions, BINDING_FIELDS, verify_source_inventory
    from survey.democratic_peace.source import validate_registered_table
    from survey.democratic_peace.run import make_build_receipt
    source = (repo/'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json').read_bytes()
    table = strict_json(source)
    validate_registered_table(table)
    reviews = verify_source_review(repo, table)
    write_json(out/'source-audit.json', dict(table_sha256=sha(source), review_paths=sorted(reviews),
        slots=len(table['slots']), registered_histories=0))
    fixtures = [dict(width=3, height=2, horizon_periods=7, periods_per_tick=3,
        initial_democratic_share=d) for d in (0., 1., 0.)]
    fixtures[-1].update(initial_resourced_share=0., zero_ratio='reject_zero_denominator')
    manifest = bind_unregistered_manifest(build_unregistered_manifest(table, source,
        configurations=fixtures, sessions=2), repo, source)
    mp = out/'fixture-manifest.json'; write_json(mp, manifest)
    pre = verify_source_inventory(repo, manifest['source_inventory'])
    write_json(out/'prebuild-inventory.json', manifest['source_inventory'])
    _, rustc, _ = command(['rustc', '-Vv'], repo, out, 'rustc')
    _, cargo, _ = command(['cargo', '--version'], repo, out, 'cargo-version')
    host = re.search(r'^host: (.+)$', rustc, re.M).group(1)
    flags, config_inputs = effective_flags(repo, host)
    write_json(out/'cargo-configuration.json', config_inputs)
    build = ['cargo', 'build', '--release', '--locked', '--manifest-path', 'survey/Cargo.toml', '--bin', 'democratic_peace']
    build_result, _, _ = command(build, repo, out, 'build')
    binary = repo/'survey/target/release/democratic_peace'
    toolchain = dict(target=host, rustc_version=rustc.strip(), cargo_version=cargo.strip())
    def receipt(path):
        value = make_build_receipt(path, binary, repo, prebuild_inventory_sha256=pre,
            build_command=build, build_exit_code=build_result['returncode'], toolchain=toolchain,
            lockfile_paths=['Cargo.lock', 'survey/Cargo.lock'], build_flags=flags)
        dest = path.with_name(path.stem.replace('-manifest', '')+'-build-receipt.json')
        write_json(dest, value)
        return dest
    rp = receipt(mp)
    def native(path, resolved, raw=None, receipt_path=None):
        argv = [binary, '--manifest', path, '--resolved', resolved, '--repo', repo]
        argv += ['--validate-only'] if raw is None else ['--out', raw]
        if receipt_path is not None:
            argv += ['--receipt', receipt_path]
        return argv
    # Provisional registered configuration validation constructs no worlds and
    # performs no freeze, inference, registration, or history execution.
    provisional = build_manifest(table, source, precision_registered=True)
    expected_keys(provisional)
    provisional_path = out/'provisional-full-manifest.json'; write_json(provisional_path, provisional)
    command(native(provisional_path, out/'provisional-full-resolved.json'), repo, out, 'provisional-full-validate')
    resolved, raw = out/'fixture-resolved.json', out/'fixture-sessions.jsonl'
    argv = native(mp, resolved, raw, rp)
    command(argv, repo, out, 'fixture-run', limit=60)
    r = strict_json(resolved.read_bytes()); binding = {k:r[k] for k in BINDING_FIELDS}
    rows = read_sessions(raw, manifest, r, binding)
    statuses = [row['attempt']['status'] for row in rows.values()]
    require(statuses.count('completed') == 4 and statuses.count('invalid') == 2, 'fixture outcomes differ')
    require(all(row['outcome']['outcome']['attempted_period'] == 1 and
        row['outcome']['outcome']['completed_periods'] == 0 for row in rows.values()
        if row['attempt']['status'] == 'invalid'), 'strict-zero atomicity differs')
    original = raw.read_bytes()
    command(argv, repo, out, 'fixture-resume', limit=60)
    require(raw.read_bytes() == original, 'resume changed raw bytes')
    write_json(out/'fixture-receipt.json', dict(histories=6, completed=4, invalid=2,
        resume_bytes_unchanged=True, raw_sha256=sha(original), raw_bytes=len(original), registered_histories=0))
    checks = []
    def reject_raw(name, data):
        path = out/f'reject-{name}.jsonl'; path.write_bytes(data)
        try:
            read_sessions(path, manifest, r, binding)
        except ValueError as exc:
            error = str(exc)
        else:
            raise RuntimeError(f'Python accepted {name}')
        result, _, stderr = command(native(mp, out/f'reject-{name}-resolved.json', path, rp),
            repo, out, f'reject-{name}', limit=60, reject=True)
        require(not result['timed_out'] and path.read_bytes() == data, f'{name} altered evidence or timed out')
        expected_error = {'duplicate':'duplicate attempted key', 'truncated':'truncated final JSONL',
            'provenance':'resume evidence identity mismatch', 'config':'resume config/family mismatch',
            'clock':'resume source clock mismatch', 'panic-context':'panic lacks context'}[name]
        require(expected_error in stderr, f'{name} rejected for an unexpected reason')
        checks.append(dict(case=name, python_error=error, native_error=stderr.strip(), unchanged=True, **result))
    reject_raw('duplicate', original+original.splitlines(keepends=True)[0])
    reject_raw('truncated', original[:-1])
    baseline = [strict_json(line) for line in original.splitlines()]
    for name in ('provenance', 'config', 'clock', 'panic-context'):
        changed = deepcopy(baseline)
        if name == 'provenance': changed[0]['binary_sha256'] = '0'*64
        elif name == 'config': changed[0]['config']['width'] = 4
        elif name == 'clock': changed[0]['outcome']['completed_periods'] += 1
        else:
            changed[0]['attempt']['status'] = 'implementation_panic'
            changed[0]['attempt']['panic_context'] = None
        reject_raw(name, ''.join(json.dumps(x)+'\n' for x in changed).encode())
    forged = strict_json(rp.read_bytes()); forged['lockfile_hashes']['Cargo.lock'] = '0'*64
    forged_path = out/'forged-lock-receipt.json'; write_json(forged_path, forged)
    reject_path = out/'reject-forged-lock.jsonl'; reject_path.write_bytes(original)
    result, _, stderr = command(native(mp, out/'forged-lock-resolved.json', reject_path, forged_path),
        repo, out, 'reject-forged-lock', limit=60, reject=True)
    require(not result['timed_out'] and reject_path.read_bytes() == original, 'forged-lock check altered evidence')
    require('receipt lockfile SHA256 mismatch' in stderr, 'forged lock rejected for unexpected reason')
    checks.append(dict(case='forged-lock-receipt', native_error=stderr.strip(), unchanged=True, **result))
    protected = [mp, rp, binary, repo/'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json', resolved, raw]
    snapshots = {p: p.read_bytes() for p in protected}
    aliases = [('resolved-manifest', mp, out/'alias-manifest-raw.jsonl'),
        ('resolved-source-table', protected[3], out/'alias-table-raw.jsonl'),
        ('out-resolved', resolved, resolved),
        ('resolved-receipt', rp, out/'alias-receipt-raw.jsonl')]
    for name, destination, raw_destination in aliases:
        result, _, stderr = command(native(mp, destination, raw_destination, rp), repo, out,
            'reject-alias-'+name, limit=60, reject=True)
        require(not result['timed_out'] and all(p.read_bytes()==data for p,data in snapshots.items()),
            f'alias {name} modified protected inputs')
        require('output aliases input or another output' in stderr, 'alias rejected for unexpected reason')
        checks.append(dict(case='alias-'+name,native_error=stderr.strip(),unchanged=True,**result))
    write_json(out/'rejections.json', checks)
    # Scheduling thresholds and 2x multiplier are fixed before measuring.
    gate = dict(maximum_cpu_hours=8, maximum_rss_bytes=2*1024**3, forecast_multiplier=2)
    probes = []
    for i, (preset, density, mobile) in enumerate((p,d,m) for p in
            ('printed_2001','prose_probability') for d,m in ((.05,.15),(.3,.5),(.7,.85))):
        config = dict(width=15,height=15,horizon_periods=1000,periods_per_tick=100,
            initial_democratic_share=density,mobile_share=mobile,mechanism='collective_security')
        m = build_unregistered_manifest(table,source,configurations=[config], mode='runtime_probe', presets=[preset])
        m['arms'][0].update(id=f'runtime_probe.case{i}', first_seed=400100001+i*10000)
        m = bind_unregistered_manifest(m,repo,source)
        path=out/f'probe-{i}-manifest.json'; write_json(path,m)
        resolved=out/f'probe-{i}-resolved.json'; raw=out/f'probe-{i}-sessions.jsonl'
        result, _, stderr = command(['/usr/bin/time','-l',*native(path,resolved,raw,receipt(path))],
            repo,out,f'probe-{i}',limit=60)
        row=dict(case=i,preset=preset,config=config,seed=m['arms'][0]['first_seed'],**result)
        if result['timed_out']:
            row.update(status='timeout', elapsed_lower_bound_seconds=60)
        else:
            export = strict_json(resolved.read_bytes())
            rows=read_sessions(raw,m,export,{k:export[k] for k in BINDING_FIELDS})
            attempt=next(iter(rows.values())); terminal=attempt['outcome']['outcome']
            cpu=re.search(r'([\d.]+) real\s+([\d.]+) user\s+([\d.]+) sys',stderr)
            rss=re.search(r'(\d+)\s+maximum resident set size',stderr)
            require(cpu is not None and rss is not None, 'cannot parse per-probe CPU/RSS')
            row.update(status=attempt['attempt']['status'], completed_periods=terminal['completed_periods'],
                attempted_period=terminal['attempted_period'], invalid_reason=terminal['invalid_reason'],
                cpu_seconds=float(cpu[2])+float(cpu[3]), maximum_rss_bytes=int(rss[1]))
        probes.append(row)
        write_json(out/'runtime-probes.json',dict(prospective_gate=gate,probes=probes,registered_histories=0))
        if result['timed_out']: break
    complete=[p for p in probes if p['status']=='completed']
    all_complete=len(complete)==6
    worst=max((p['cpu_seconds'] for p in complete),default=None)
    peak=max((p.get('maximum_rss_bytes',0) for p in probes),default=None)
    runtime=dict(classification='unregistered_runtime_probe',probes=probes,prospective_gate=gate,probes_completed=len(complete),all_six_completed=all_complete,
        worst_cpu_seconds=worst, maximum_rss_bytes=peak, registered_histories=0,
        original_3240_cpu_hours=None if worst is None else 2*worst*3240/3600,
        full_14040_cpu_hours=None if worst is None else 2*worst*14040/3600,
        full_scheduling_gate_passed=all_complete and 2*worst*14040/3600<=8 and peak<=2*1024**3,
        limitation='Six collective-security cases; not a runtime guarantee. Forecast excludes unbenchmarked fixed-100000-draw analysis. Gate does not authorize registered execution.')
    write_json(out/'runtime-receipt.json',runtime)
    require(verify_source_inventory(repo,manifest['source_inventory'])==pre,'sources changed during preparation')
    write_json(out/'preparation-receipt.json',dict(status='checks_passed',binary_sha256=sha(binary.read_bytes()),
        source_inventory_sha256=pre,source_files=len(manifest['source_inventory']),build_returncode=build_result['returncode'],
        fixture_completed=4,fixture_invalid=2,rejection_cases=len(checks),runtime=runtime,registered_histories=0,
        source_comparisons=0,inferential_draws=0))
    print(json.dumps(runtime,indent=2))


if __name__=='__main__':
    main()
