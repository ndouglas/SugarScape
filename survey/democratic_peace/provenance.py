"""Complete scientific inventory and explicit post-writer identity binding."""
from copy import deepcopy
import hashlib
from pathlib import Path
from .methods import canonical_bytes

TABLE = 'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json'
SPEC = 'docs/superpowers/specs/2026-10-03-democratic-peace-design.md'
REQUIRED_FILES = ('Cargo.toml', 'Cargo.lock', 'crates/sugarscape-core/Cargo.toml',
    'crates/sugarscape-core/assets/sugar-map.txt', 'survey/Cargo.toml', 'survey/Cargo.lock',
    'survey/src/bin/democratic_peace.rs', TABLE, SPEC,
    'docs/superpowers/specs/2026-10-03-democratic-peace-source-review.json',
    'survey/democratic_peace/methods.json', 'survey/democratic_peace/.python-version',
    'survey/democratic_peace/requirements.txt', 'survey/democratic_peace/README.md',
    'survey/democratic_peace/NUMERICAL_METHODS.md',
    *('docs/superpowers/specs/2026-10-03-democratic-peace-' + name + '.md' for name in
      ('source-extraction', 'mechanics-reading-notes', 'author-and-followup-reading-notes', 'reading-notes')))


def safe_source_path(root, name):
    root = Path(root).resolve()
    if not isinstance(name, str) or not name or any(c in name for c in ('\\', '\0', '\n', '\r')):
        raise ValueError('unsafe source inventory path')
    path = Path(name)
    if path.is_absolute() or '..' in path.parts or path.as_posix() != name:
        raise ValueError('unsafe source inventory path')
    current = root
    for component in path.parts:
        current = current / component
        if current.is_symlink():
            raise ValueError(f'source symlink forbidden: {name}')
    if not current.is_file():
        raise ValueError(f'missing inventory source: {name}')
    return current


def required_inventory_paths(root):
    root = Path(root).resolve()
    result = set(REQUIRED_FILES)
    for directory, suffixes in (
        ('crates/sugarscape-core/src', {'.rs'}),
        ('crates/sugarscape-core/tests', {'.rs'}),
        ('survey/democratic_peace', {'.py', '.json'}),
        ('survey/tests', {'.rs'})):
        base = root / directory
        if not base.is_dir():
            raise ValueError(f'missing scientific source directory: {directory}')
        for path in base.rglob('*'):
            if '__pycache__' in path.parts:
                continue
            if path.is_symlink():
                raise ValueError('source symlink forbidden')
            if path.is_file() and path.suffix in suffixes:
                result.add(path.relative_to(root).as_posix())
    for name in ('.cargo/config', '.cargo/config.toml', 'rust-toolchain', 'rust-toolchain.toml',
                 'build.rs', 'crates/sugarscape-core/build.rs', 'survey/build.rs'):
        if (root / name).exists() or (root / name).is_symlink():
            result.add(name)
    for name in result:
        safe_source_path(root, name)
    return sorted(result)


def source_inventory(root):
    result = []
    for name in required_inventory_paths(root):
        data = safe_source_path(root, name).read_bytes()
        result.append({'path': name, 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
    return result


def verify_source_review(root, table):
    from .records import strict_json
    receipts = set()
    for figure in table['figures']:
        check = figure['independent_visual_check']
        name = check.get('review_receipt')
        data = safe_source_path(root, name).read_bytes()
        if hashlib.sha256(data).hexdigest() != check.get('review_receipt_sha256'):
            raise ValueError('source review receipt SHA256 mismatch')
        receipt = strict_json(data)
        if receipt.get('paper_sha256') != table['paper_sha256'] or receipt.get('slots_sha256') != hashlib.sha256(canonical_bytes(table['slots'])).hexdigest():
            raise ValueError('source review paper/slot digest mismatch')
        if receipt.get('decision') != 'accepted_as_inferred_curve_values_with_reported_envelopes' or receipt.get('registered_histories') != 0:
            raise ValueError('source review is not a premeasurement acceptance')
        reviewed = {item['id']: item for item in receipt['figures']}
        if figure['id'] not in reviewed or reviewed[figure['id']]['image_sha256'] != figure['image_sha256'] or reviewed[figure['id']]['axis_landmarks'] != figure['axis_landmarks']:
            raise ValueError('source review raster/calibration mismatch')
        receipts.add(name)
    return receipts


def _bind(manifest, root, source_bytes, registered):
    from .manifest import build_manifest, expected_keys
    from .records import strict_json
    from .source import validate_registered_table
    table = strict_json(source_bytes)
    expected_keys(manifest)
    if registered:
        validate_registered_table(table)
        verify_source_review(root, table)
        if manifest != build_manifest(table, source_bytes, precision_registered=manifest['precision_registered']):
            raise ValueError('provisional manifest differs from exact registered contract')
    else:
        expected_keys(manifest)
        if manifest['execution_mode'] not in ('fixture', 'runtime_probe'):
            raise ValueError('unregistered binding cannot activate registered population')
    if safe_source_path(root, TABLE).read_bytes() != source_bytes:
        raise ValueError('source table/root bytes mismatch')
    if hashlib.sha256(source_bytes).hexdigest() != manifest['source_table_sha256']:
        raise ValueError('source table SHA256 differs from manifest')
    result = deepcopy(manifest)
    inventory = source_inventory(root)
    result.update(source_inventory=inventory,
        source_inventory_sha256=hashlib.sha256(canonical_bytes(inventory)).hexdigest())
    if registered:
        result['provenance_status'] = 'frozen'
    return result


def freeze_manifest(provisional, root, source_bytes):
    """Registered freeze only after all writers stop and source audit is complete."""
    return _bind(provisional, root, source_bytes, True)


def bind_unregistered_manifest(manifest, root, source_bytes):
    """Identity binding for bounded fixtures/probes; never a scientific freeze."""
    return _bind(manifest, root, source_bytes, False)


def main():
    import argparse
    import json
    from .records import strict_json
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('manifest', 'source', 'source-root', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    value = strict_json(args.manifest.read_bytes())
    binder = freeze_manifest if value['execution_mode'] == 'registered' else bind_unregistered_manifest
    result = binder(value, args.source_root, args.source.read_bytes())
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    print(f"Bound {len(result['source_inventory'])} sources for {result['execution_mode']}; no histories executed")


if __name__ == '__main__':
    main()
