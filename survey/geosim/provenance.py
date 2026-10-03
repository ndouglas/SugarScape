"""Comprehensive scientific source inventory and an explicit post-writer freeze."""
from copy import deepcopy
import hashlib
from pathlib import Path
from .methods import canonical_bytes

TABLE='docs/superpowers/specs/2026-10-03-geosim-source-table.json'
SPEC='docs/superpowers/specs/2026-10-03-geosim-design.md'
REQUIRED_FILES=(
    'Cargo.toml','Cargo.lock','crates/sugarscape-core/Cargo.toml',
    'crates/sugarscape-core/assets/sugar-map.txt',
    'crates/sugarscape-core/tests/geosim_discovery.rs',
    'crates/sugarscape-core/tests/geosim_host_sweep.rs',
    'crates/sugarscape-core/tests/geosim_invalid_sweep.rs',
    'survey/Cargo.toml','survey/Cargo.lock','survey/src/bin/geosim.rs',
    'survey/tests/geosim_native.rs','survey/tests/fixtures/geosim-registered-manifest.json',
    'survey/geosim/.python-version','survey/geosim/requirements.txt',
    'survey/geosim/README.md','survey/geosim/NUMERICAL_METHODS.md',
    'survey/geosim/reference/extract.py','survey/geosim/reference/test_extract.py',
    'survey/geosim/reference/test_launch.py','survey/geosim/reference/GeoSimReferenceProbe.java',
    TABLE,SPEC,
    *('docs/superpowers/specs/2026-10-03-geosim-'+name+'.md' for name in (
        'source-extraction','reading-notes','author-code-reading-notes','mechanics-reading-notes',
        'defender-threshold-amendment','damage-incidence-audit','artifact-audit',
        'finite-technology-amendment','portability-amendment')),
)


def safe_source_path(root,name):
    root=Path(root).resolve();path=Path(name)
    if not isinstance(name,str) or not name or path.is_absolute() or '..' in path.parts or path.as_posix()!=name:
        raise ValueError(f'unsafe inventory path: {name}')
    current=root
    for component in path.parts:
        current=current/component
        if current.is_symlink():raise ValueError(f'source symlink is forbidden: {name}')
    if not current.is_file():raise ValueError(f'missing inventory source: {name}')
    return current


def required_inventory_paths(root):
    """Sorted required roster; process ledgers, raw downloads and outputs excluded."""
    root=Path(root).resolve();required=set(REQUIRED_FILES)
    for directory,suffixes in (
        ('crates/sugarscape-core/src',{'.rs'}),('survey/geosim',{'.py','.java'}),
        ('crates/sugarscape-core/tests',{'.rs'}),('survey/tests',{'.rs'})):
        base=root/directory
        if not base.is_dir():raise ValueError(f'missing scientific source directory: {directory}')
        for path in base.rglob('*'):
            relative=path.relative_to(root).as_posix()
            if relative.startswith('survey/geosim/reference/raw/') or '__pycache__' in path.parts:continue
            if path.is_symlink():raise ValueError(f'source symlink is forbidden: {relative}')
            if path.is_file() and path.suffix in suffixes:required.add(relative)
    for name in ('.cargo/config','.cargo/config.toml','rust-toolchain','rust-toolchain.toml',
                 'build.rs','crates/sugarscape-core/build.rs','survey/build.rs'):
        if (root/name).exists() or (root/name).is_symlink():required.add(name)
    for name in required:safe_source_path(root,name)
    return sorted(required)


def source_inventory(root):
    entries=[]
    for name in required_inventory_paths(root):
        data=safe_source_path(root,name).read_bytes()
        entries.append({'path':name,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
    return entries


def freeze_manifest(provisional,root,source_bytes):
    """Bind current required source bytes; call only after all writers stop."""
    from .manifest import build_manifest
    from .records import strict_json
    source=strict_json(source_bytes)
    if provisional!=build_manifest(source,source_bytes):raise ValueError('provisional manifest differs from exact registered contract')
    if safe_source_path(root,TABLE).read_bytes()!=source_bytes:raise ValueError('source table/root bytes mismatch')
    result=deepcopy(provisional);inventory=source_inventory(root)
    result.update(provenance_status='frozen',source_inventory=inventory,
                  source_inventory_sha256=hashlib.sha256(canonical_bytes(inventory)).hexdigest())
    return result


def main():
    import argparse,json
    from .records import strict_json
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('manifest','source','source-root','output'):parser.add_argument('--'+name,required=True,type=Path)
    args=parser.parse_args()
    result=freeze_manifest(strict_json(args.manifest.read_bytes()),args.source_root,args.source.read_bytes())
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print(f"Frozen {len(result['source_inventory'])} scientific sources; source SHA256 {result['source_inventory_sha256']}")

if __name__=='__main__':main()
