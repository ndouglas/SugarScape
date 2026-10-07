"""Read-only original preservation, source tar and dataset validation."""
from pathlib import Path
import tarfile
from . import manifest as baseline, records
from .methods import canonical_bytes
from .provenance import safe_source_path
from .followup import sha, ORIGINAL_INVENTORY_SHA256, ORIGINAL_SOURCE_SHA256, ORIGINAL_ARCHIVE_SHA256, ORIGINAL_BINARY_SHA256, ORIGINAL_RAW_SHA256

def verify_preservation(root, inventory):
    if not isinstance(inventory, list) or not inventory:
        raise ValueError('empty preservation inventory')
    prior = ''
    for entry in inventory:
        records._fields(entry, {'path', 'bytes', 'sha256'}, 'preservation entry')
        name = entry['path']; data = safe_source_path(root, name).read_bytes()
        if name <= prior or type(entry['bytes']) is not int or len(data) != entry['bytes'] or sha(data) != entry['sha256']:
            raise ValueError('preservation byte identity/sort mismatch')
        prior = name
    return sha(canonical_bytes(inventory))


def historical_dataset(*, study_root, inventory_path, source_archive, binary):
    """Read the original with its own bindings and frozen tar, never current sources."""
    root = Path(study_root)
    inventory_bytes = Path(inventory_path).read_bytes()
    inventory = records.strict_json(inventory_bytes)
    if sha(inventory_bytes) != ORIGINAL_INVENTORY_SHA256 or len(inventory) != 13:
        raise ValueError('wrong original preservation inventory')
    verify_preservation(root, inventory)
    manifest_bytes = (root/'manifest.json').read_bytes(); value = records.strict_json(manifest_bytes)
    if value['schema_version'] != 1 or value['precision_registered'] or len(baseline.expected_keys(value)) != 3240 or value['source_inventory_sha256'] != ORIGINAL_SOURCE_SHA256:
        raise ValueError('wrong historical manifest/population')
    from .run import sha256_file
    if sha256_file(source_archive) != ORIGINAL_ARCHIVE_SHA256 or sha256_file(binary) != ORIGINAL_BINARY_SHA256 or sha256_file(root/'sessions.jsonl') != ORIGINAL_RAW_SHA256:
        raise ValueError('wrong historical archive/binary/raw identity')
    with tarfile.open(source_archive, 'r:') as archive:
        members = archive.getmembers(); names = [m.name for m in members]
        if len(names) != len(set(names)):
            raise ValueError('duplicate frozen source member')
        for entry in value['source_inventory']:
            member = archive.getmember(entry['path'])
            if not member.isfile() or member.size != entry['bytes'] or sha(archive.extractfile(member).read()) != entry['sha256']:
                raise ValueError('frozen original source inventory mismatch')
        if sha(canonical_bytes(value['source_inventory'])) != ORIGINAL_SOURCE_SHA256:
            raise ValueError('frozen source inventory digest mismatch')
        receipt_bytes = (root/'build-receipt.json').read_bytes(); receipt = records.strict_json(receipt_bytes)
        records.validate_build_receipt(receipt, sha(manifest_bytes), ORIGINAL_BINARY_SHA256, ORIGINAL_SOURCE_SHA256)
        for name, digest in receipt['lockfile_hashes'].items():
            if sha(archive.extractfile(name).read()) != digest:
                raise ValueError('historical lockfile identity mismatch')
    resolved = records.strict_json((root/'resolved.json').read_bytes())
    binding = {'manifest_sha256': sha(manifest_bytes), 'binary_sha256': ORIGINAL_BINARY_SHA256,
        'source_inventory_sha256': ORIGINAL_SOURCE_SHA256, 'build_receipt_sha256': sha(receipt_bytes),
        'resolved_configs_sha256': resolved['resolved_configs_sha256']}
    rows = records.read_sessions(root/'sessions.jsonl', value, resolved, binding)
    if len(rows) != 3240 or any(not records.history_metrics(r)['complete'] for r in rows.values()):
        raise ValueError('historical complete census mismatch')
    if verify_preservation(root, inventory) != sha(canonical_bytes(inventory)):
        raise ValueError('historical input changed during validation')
    if Path(inventory_path).read_bytes() != inventory_bytes or sha256_file(source_archive) != ORIGINAL_ARCHIVE_SHA256 or sha256_file(binary) != ORIGINAL_BINARY_SHA256:
        raise ValueError('historical authorities changed during validation')
    historical_input = {'study_id': 'democratic-peace-original-2026-10-04', **binding,
        'raw_sha256': ORIGINAL_RAW_SHA256, 'preservation_inventory_sha256': sha(inventory_bytes)}
    return {'manifest': value, 'resolved': resolved, 'binding': binding, 'records': rows,
        'data_sha256': ORIGINAL_RAW_SHA256, 'historical_input': historical_input}
