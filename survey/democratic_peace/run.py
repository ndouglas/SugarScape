"""Exact native invocation and build receipts; never builds or silently runs."""
import argparse
import hashlib
from pathlib import Path
import subprocess
from .records import strict_json, verify_source_inventory


def sha256_file(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def run_native(binary, manifest, out=None, *, validate=False, resolved_out,
               build_receipt=None, source_root=None):
    command = [str(Path(binary).resolve()), '--manifest', str(Path(manifest).resolve()),
        '--resolved', str(Path(resolved_out).resolve()),
        '--repo', str(Path(source_root or '.').resolve())]
    if validate:
        command.append('--validate-only')
    else:
        if out is None or build_receipt is None:
            raise ValueError('native execution requires explicit output and bound build receipt')
        command += ['--out', str(Path(out).resolve())]
    if build_receipt is not None:
        command += ['--receipt', str(Path(build_receipt).resolve())]
    return subprocess.run(command, capture_output=True, text=True, check=True)


def make_build_receipt(manifest_path, binary_path, source_root, *, prebuild_inventory_sha256,
        build_command, build_exit_code, toolchain, lockfile_paths, features=None, build_flags=None):
    if build_exit_code != 0:
        raise ValueError('failed build cannot produce binding receipt')
    value = strict_json(Path(manifest_path).read_bytes())
    if value['model'] != 'democratic_peace' or (value['execution_mode'] == 'registered' and value['provenance_status'] != 'frozen'):
        raise ValueError('registered build receipt requires frozen democratic-peace sources')
    root = Path(source_root).resolve()
    post = verify_source_inventory(root, value['source_inventory'])
    if post != prebuild_inventory_sha256 or post != value['source_inventory_sha256']:
        raise ValueError('pre/post build source inventory changed')
    if not build_command or any(not isinstance(x, str) for x in build_command):
        raise ValueError('explicit successful build command required')
    if not {'Cargo.lock', 'survey/Cargo.lock'} <= set(lockfile_paths):
        raise ValueError('receipt omits required Cargo locks')
    from .provenance import safe_source_path
    locks = {name: sha256_file(safe_source_path(root, name)) for name in lockfile_paths}
    return {'schema_version': 1, 'model': 'democratic_peace',
        'manifest_sha256': sha256_file(manifest_path), 'source_inventory_sha256': post,
        'prebuild_inventory_sha256': prebuild_inventory_sha256, 'postbuild_inventory_sha256': post,
        'binary_sha256': sha256_file(binary_path), 'build_command': list(build_command),
        'cwd': str(root), 'target': toolchain['target'], 'rustc_version': toolchain['rustc_version'],
        'cargo_version': toolchain['cargo_version'], 'lockfile_hashes': locks,
        'features': features or [], 'build_flags': build_flags or []}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('binary', 'manifest', 'resolved', 'source-root'):
        parser.add_argument('--' + name, required=True, type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--receipt', type=Path)
    parser.add_argument('--validate-only', action='store_true')
    args = parser.parse_args()
    result = run_native(args.binary, args.manifest, args.out, validate=args.validate_only,
        resolved_out=args.resolved, build_receipt=args.receipt, source_root=args.source_root)
    print(result.stdout, end='')
    if result.stderr:
        import sys
        print(result.stderr, end='', file=sys.stderr)


if __name__ == '__main__':
    main()
