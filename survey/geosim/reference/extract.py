"""Extract pinned third-party reference binaries, never author source.

Outputs belong under ignored survey/out, outside every product build path.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import zipfile

ARCHIVE_SHA256 = '2f20146bc34f9521302f251ffd6388443c4bbd21e8db47a527c120d220e98a5c'
MODEL_MEMBER = 'models/geosim2-0.9.0.zip'


def extract(archive, destination, expected_sha256=ARCHIVE_SHA256):
    archive, destination = Path(archive), Path(destination)
    archive_bytes = archive.read_bytes()
    actual = hashlib.sha256(archive_bytes).hexdigest()
    if actual != expected_sha256:
        raise ValueError(f'Archive SHA256 mismatch: expected {expected_sha256}, got {actual}')
    outputs = []

    def select(source, prefix, output_prefix, jar_only=False):
        for member in source.infolist():
            if member.is_dir() or not member.filename.startswith(prefix):
                continue
            path = PurePosixPath(member.filename)
            if path.is_absolute() or '..' in path.parts or '\\' in member.filename:
                raise ValueError(f'Unsafe archive member: {member.filename}')
            if jar_only and path.suffix != '.jar':
                continue
            relative = PurePosixPath(output_prefix) / path.relative_to(prefix.rstrip('/'))
            payload = source.read(member)
            outputs.append((relative, member.filename, payload))

    with zipfile.ZipFile(io.BytesIO(archive_bytes)) as outer:
        select(outer, 'lib/', 'lib', jar_only=True)
        nested_bytes = outer.read(MODEL_MEMBER)
        with zipfile.ZipFile(io.BytesIO(nested_bytes)) as model:
            select(model, 'classes/', 'geosim2/classes')
    if not any(str(p).startswith('lib/') for p, _, _ in outputs):
        raise ValueError('Archive contains no dependency JARs')
    if not any(str(p).endswith('.class') for p, _, _ in outputs):
        raise ValueError('Archive contains no model classes')
    if len({p for p, _, _ in outputs}) != len(outputs):
        raise ValueError('Duplicate extraction output paths')
    receipt = {'archive': str(archive), 'archive_sha256': actual,
               'model_member': MODEL_MEMBER, 'model_sha256': hashlib.sha256(nested_bytes).hexdigest(),
               'files': []}
    for relative, member, payload in outputs:
        target = destination / str(relative)
        # Reject pre-existing symlink parents before resolving the output path.
        parents = [destination, target, *(destination / str(p) for p in relative.parents if str(p) != '.') ]
        if any(parent.is_symlink() for parent in parents):
            raise ValueError(f'Symlink extraction destination: {target}')
    for relative, member, payload in outputs:
        target = destination / str(relative)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(payload)
        receipt['files'].append({'path': str(relative), 'member': member,
                                 'bytes': len(payload), 'sha256': hashlib.sha256(payload).hexdigest()})
    (destination / 'extraction-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('archive', type=Path)
    parser.add_argument('--destination', type=Path, required=True)
    args = parser.parse_args()
    receipt = extract(args.archive, args.destination)
    print(json.dumps({'archive_sha256': receipt['archive_sha256'], 'extracted_files': len(receipt['files']),
                      'receipt': str(args.destination / 'extraction-receipt.json')}))


if __name__ == '__main__':
    main()
