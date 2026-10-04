"""Dated deterministic census-label correction; never execute worlds or inference."""
import argparse
from copy import deepcopy
import datetime
import hashlib
import json
from pathlib import Path
import platform
import stat
import subprocess
import sys
import tarfile
import time

BASE = 'a318d1f78fc7f0f8f13489f20d7e686737fc13d1'
ORIGINAL_FINDINGS_SHA = '558b0f8c482503dba1afd3b2a0b97cf49567871eb323c6c0a8a66f7f60e0e08b'
ORIGINAL_PROVENANCE_SHA = '3e804032a633b3bd41fce66bc42c1d8adac42b393ba6a892829799b5c2933c41'
ORIGINAL_RAW_SHA = '753a1d5ff6236ba4720e0c52e46ab061643a5d27ef5c5d00175e0ff3f6d6c492'
ORIGINAL_ARCHIVE_SHA = '69f7bbf5acd3a61190eea49df0657c3e72694d987c3de43f109f9bd70b3c70fa'
FAMILIES = ('primary_contrasts', 'secondary_contrasts')
DISCLOSURE = '\n## Reporting amendment — 2026-10-04\n\nThe 24 unregistered precision contrast references now have null censuses with `precision_not_registered`. The fixed required sample size is 100 per precision arm, separately recorded as `required_precision_sample_size`; it does not assert registration. This is a deterministic reporting derivation of the preserved original findings. All empirical histories, source censuses/intervals, methods, jobs, availability, p values and verdicts are unchanged. No worlds or inferential analysis were rerun. Original measurement bindings and amended reporting-code identities are separately retained in the provenance.\n'


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def file_identity(path, root=None):
    path = Path(path)
    return {'path': path.relative_to(root).as_posix() if root else str(path.resolve()),
            'bytes': path.stat().st_size, 'sha256': digest(path.read_bytes())}


def validate_output_paths(inputs, outputs, immutable_directories=()):
    protected = [Path(p).resolve() for p in inputs]
    immutable = [Path(p).resolve() for p in immutable_directories]
    seen = []
    for output in outputs:
        path = Path(output).resolve()
        if any(path == parent or path.is_relative_to(parent) for parent in immutable):
            raise ValueError('output enters an immutable original study directory')
        if path.is_dir():
            raise ValueError('output is a directory')
        for other in protected + seen:
            if path == other or (path.exists() and other.exists() and path.samefile(other)):
                raise ValueError('output aliases protected input or another output')
        seen.append(path)


def derive_report(original, metadata):
    if original.get('precision_registered') is not False or 'reporting_amendment' in original:
        raise ValueError('requires the unamended original-only findings')
    amended = deepcopy(original)
    for family in FAMILIES:
        components = amended[family]['components']
        if len(components) != 6:
            raise ValueError('requires both complete six-component families')
        for component in components:
            if component['unavailable_reason'] != 'precision_not_registered' or component['verdict'] != 'Unresolved':
                raise ValueError('cannot amend an available or differently judged contrast')
            for field in ('left_census', 'right_census'):
                census = component[field]
                if not isinstance(census, dict) or any(census.get(k) != v for k, v in
                        [('registered_count', 100), ('missing_count', 100), ('received_count', 0), ('complete_count', 0)]):
                    raise ValueError('unexpected original unregistered contrast census')
                component[field] = None
            if 'required_precision_sample_size' in component:
                raise ValueError('sample-size metadata already amended')
            component['required_precision_sample_size'] = 100
    amended['reporting_amendment'] = deepcopy(metadata)
    prove_invariance(original, amended)
    return amended


def prove_invariance(original, amended):
    before = deepcopy(original); after = deepcopy(amended)
    after.pop('reporting_amendment', None)
    references = 0
    for family in FAMILIES:
        for old, new in zip(before[family]['components'], after[family]['components']):
            for field in ('left_census', 'right_census'):
                if new.pop(field) is not None:
                    raise ValueError('corrected unregistered census must be null')
                old.pop(field); references += 1
            if new.pop('required_precision_sample_size') != 100:
                raise ValueError('required precision sample size changed')
    if references != 24 or before != after:
        raise ValueError('empirical content changed outside the permitted census metadata')
    return {'corrected_census_references': references,
            'unchanged_empirical_projection_sha256': digest(canonical(before)),
            'unchanged_content': 'every field outside 24 contrast censuses, required sample sizes and amendment provenance'}


def main():
    started = datetime.datetime.now(datetime.timezone.utc).isoformat(); clock = time.monotonic()
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('repo', 'study', 'original-provenance', 'source-archive', 'original-binary', 'out'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args(); root = args.repo.resolve(strict=True)
    sys.path.insert(0, str(root))
    from survey.democratic_peace.records import strict_json
    from survey.democratic_peace.reporting import serialize_report, markdown_report
    from survey.democratic_peace.manifest import contrast_definitions
    import numpy as np
    study = args.study.resolve(strict=True); out = args.out.resolve()
    code_paths = sorted(p for p in (root/'survey/democratic_peace').iterdir()
                        if p.is_file() and p.suffix in ('.py', '.json')) + [Path(__file__).resolve()]
    inputs = sorted(p for p in study.rglob('*') if p.is_file()) + [args.original_provenance,
             args.source_archive, args.original_binary, *code_paths]
    outputs = [out/'findings.json', out/'findings.md', out/'provenance.json']
    # Every destination is checked before mkdir, reads or any output write.
    validate_output_paths(inputs, outputs, [study])
    if any(p.exists() for p in outputs):
        raise ValueError('preserve existing amendment outputs; use a fresh destination')
    subprocess.run(['git', 'check-ignore', str(outputs[0])], cwd=root, check=True, capture_output=True)
    original_bytes = (study/'findings.json').read_bytes(); provenance_bytes = args.original_provenance.read_bytes()
    if digest(original_bytes) != ORIGINAL_FINDINGS_SHA or digest(provenance_bytes) != ORIGINAL_PROVENANCE_SHA:
        raise ValueError('original findings/provenance differ from preserved review identity')
    original = strict_json(original_bytes); provenance = strict_json(provenance_bytes)
    manifest = strict_json((study/'manifest.json').read_bytes())
    bindings = original['provenance']['binding']
    if provenance['bindings'] != bindings or original['provenance']['data_sha256'] != ORIGINAL_RAW_SHA:
        raise ValueError('original measurement bindings disagree')
    for entry in provenance['evidence']:
        path = root/entry['path']
        if not path.resolve().is_relative_to(study):
            raise ValueError('original study inventory points outside its immutable directory')
        identity = file_identity(path)
        if identity['bytes'] != entry['bytes'] or identity['sha256'] != entry['sha256']:
            raise ValueError('original study evidence bytes changed')
    for path, expected in [(study/'sessions.jsonl', ORIGINAL_RAW_SHA),
            (study/'manifest.json', bindings['manifest_sha256']),
            (study/'build-receipt.json', bindings['build_receipt_sha256']),
            (args.original_binary, bindings['binary_sha256']), (args.source_archive, ORIGINAL_ARCHIVE_SHA)]:
        if file_identity(path)['sha256'] != expected:
            raise ValueError('preserved original input binding changed')
    if digest(canonical(manifest['source_inventory'])) != bindings['source_inventory_sha256'] or manifest['source_inventory'] != provenance['source_inventory']:
        raise ValueError('original source inventory binding changed')
    with tarfile.open(args.source_archive) as archive:
        for entry in manifest['source_inventory']:
            member = archive.getmember(entry['path'])
            if not member.isfile():
                raise ValueError('original source archive member is not a regular file')
            data = archive.extractfile(member).read()
            if len(data) != entry['bytes'] or digest(data) != entry['sha256']:
                raise ValueError('original frozen source archive image changed')
    resolved = strict_json((study/'resolved.json').read_bytes())
    if any(resolved[name] != value for name, value in bindings.items()):
        raise ValueError('original resolved bindings changed')
    if digest(resolved['resolved_configs_json'].encode()) != bindings['resolved_configs_sha256']:
        raise ValueError('original resolved configuration payload changed')
    if {'python': platform.python_version(), 'numpy': np.__version__} != manifest['method_contract']['runtime']:
        raise ValueError('reporting runtime differs from retained pinned runtime')
    actual_definitions = [c for family in FAMILIES for c in original[family]['components']]
    definitions = contrast_definitions()
    if len(actual_definitions) != 12 or any(any(c[k] != v for k, v in d.items())
            for c, d in zip(actual_definitions, definitions)):
        raise ValueError('original contrast definitions changed')
    code_inventory = [dict(file_identity(p, root), mode=stat.S_IMODE(p.stat().st_mode)) for p in sorted(code_paths)]
    metadata = {'date': '2026-10-04', 'classification': 'deterministic_reporting_derivation_no_inference',
        'review_base': BASE, 'correction': '24 unregistered contrast census references set to null; required precision size100 stated separately',
        'corrected_census_references': 24, 'original_findings_sha256': ORIGINAL_FINDINGS_SHA,
        'original_provenance_sha256': ORIGINAL_PROVENANCE_SHA, 'original_measurement_binding': bindings,
        'measurement_source_scope': 'preserved original frozen source archive, not current reporting code',
        'reporting_code_inventory': code_inventory,
        'reporting_code_inventory_sha256': digest(canonical(code_inventory)),
        'original_source_archive': file_identity(args.source_archive),
        'original_native_executable': file_identity(args.original_binary)}
    amended = derive_report(original, metadata); invariance = prove_invariance(original, amended)
    metadata['invariance'] = invariance; amended['reporting_amendment'] = metadata
    json_bytes = serialize_report(amended).encode(); markdown_bytes = (markdown_report(amended) + DISCLOSURE).encode()
    if markdown_report(original).encode() != (study/'findings.md').read_bytes():
        raise ValueError('original Markdown differs from unchanged presentation code')
    if [dict(file_identity(p, root), mode=stat.S_IMODE(p.stat().st_mode)) for p in sorted(code_paths)] != code_inventory:
        raise ValueError('reporting code changed during derivation')
    corrected_provenance = deepcopy(provenance)
    corrected_provenance['classification'] = 'registered_democratic_peace_evidence_with_reporting_amendment'
    corrected_provenance['bindings_scope'] = 'original measurement only; current reporting code is bound separately'
    corrected_provenance['reporting_amendment'] = {**metadata,
        'original_provenance_input': file_identity(args.original_provenance),
        'original_findings_input': file_identity(study/'findings.json'),
        'outputs': [{'name': 'findings.json', 'bytes': len(json_bytes), 'sha256': digest(json_bytes)},
                    {'name': 'findings.md', 'bytes': len(markdown_bytes), 'sha256': digest(markdown_bytes)}],
        'execution': {'command': [sys.executable, *sys.argv], 'cwd': str(Path.cwd()),
            'runtime': {'python': platform.python_version(), 'numpy': np.__version__},
            'started_utc': started, 'report_preparation_wall_seconds': time.monotonic()-clock,
            'successful_exit_code': 0, 'worlds_executed': 0, 'inferential_analyses_executed': 0}}
    provenance_output = serialize_report(corrected_provenance).encode()
    out.mkdir(parents=True, exist_ok=True)
    for path, data in zip(outputs, (json_bytes, markdown_bytes, provenance_output)):
        with path.open('xb') as file:
            file.write(data)
    print('Corrected exactly24 census references; empirical projection unchanged; no worlds or inference.')


if __name__ == '__main__':
    main()
