"""Copy reviewed generated findings and bind evidence; never rerun inference."""
import argparse
import json
import os
from pathlib import Path
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', required=True, type=Path)
    parser.add_argument('--study', required=True, type=Path)
    args = parser.parse_args()
    root = args.repo.resolve(strict=True)
    os.chdir(root)
    sys.path.insert(0, str(root))
    from survey.democratic_peace.records import strict_json, read_sessions, verify_source_inventory, validate_build_receipt
    from survey.democratic_peace.run import sha256_file
    from survey.democratic_peace.manifest import expected_keys
    study = args.study.resolve(strict=True)
    manifest_path = study / 'manifest.json'
    manifest = strict_json(manifest_path.read_bytes())
    receipt_path = study / 'build-receipt.json'
    receipt = strict_json(receipt_path.read_bytes())
    resolved = strict_json((study / 'resolved.json').read_bytes())
    binary = root / 'survey/target/release/democratic_peace'
    if os.environ.get('CARGO_TARGET_DIR'):
        binary = Path(os.environ['CARGO_TARGET_DIR']).resolve() / 'release/democratic_peace'
    inventory = verify_source_inventory(root, manifest['source_inventory'])
    validate_build_receipt(receipt, sha256_file(manifest_path), sha256_file(binary), inventory, root)
    binding = {'manifest_sha256': sha256_file(manifest_path), 'binary_sha256': sha256_file(binary), 'source_inventory_sha256': inventory, 'build_receipt_sha256': sha256_file(receipt_path), 'resolved_configs_sha256': resolved['resolved_configs_sha256']}
    rows = read_sessions(study / 'sessions.jsonl', manifest, resolved, binding)
    if set(rows) != set(expected_keys(manifest)):
        raise ValueError('publication requires all registered attempted keys, including failures')
    findings_path = study / 'findings.json'
    findings = strict_json(findings_path.read_bytes())
    if findings['classification'] != 'registered_offline_findings' or findings['provenance']['binding'] != binding:
        raise ValueError('findings do not match registered binding')
    if findings['provenance']['data_sha256'] != sha256_file(study / 'sessions.jsonl'):
        raise ValueError('raw data changed after analysis')
    prefix = root / 'docs/superpowers/specs/2026-10-03-democratic-peace'
    destinations = [Path(str(prefix) + '-findings.' + suffix) for suffix in ['json','md']] + [Path(str(prefix) + '-provenance.json')]
    if any(p.exists() for p in destinations):
        raise ValueError('preserve prior findings/provenance before a distinct amendment')
    for suffix, destination in zip(['json','md'], destinations):
        destination.write_bytes((study / ('findings.' + suffix)).read_bytes())
    evidence = [{'path': p.relative_to(root).as_posix(), 'bytes': p.stat().st_size, 'sha256': sha256_file(p)}
                for p in sorted(study.rglob('*')) if p.is_file()]
    provenance = {'classification': 'registered_democratic_peace_evidence',
        'bindings': binding, 'runtime_decision': strict_json((study / 'registration-decision.json').read_bytes()),
        'attempted_keys': len(rows), 'source_equivalence': 'Unresolved',
        'source_inventory': manifest['source_inventory'], 'evidence': evidence}
    Path(str(prefix) + '-provenance.json').write_text(json.dumps(provenance, indent=2, allow_nan=False) + '\n')
    print(f'Saved findings and evidence for {len(rows)} attempts; no inference rerun.')


if __name__ == '__main__':
    main()
