"""Exact native invocation and post-build binding; this module never builds Rust."""
import argparse
import hashlib
from pathlib import Path
import subprocess
import sys
from .records import strict_json,verify_source_inventory


def sha256_file(path):
    digest=hashlib.sha256()
    with Path(path).open('rb') as file:
        for chunk in iter(lambda:file.read(1024*1024),b''):digest.update(chunk)
    return digest.hexdigest()


def run_native(binary,manifest,out,*,arm=None,validate=False,resolved_out=None,build_receipt=None,source_root=None,allow_unfrozen_fixture=False):
    binary=Path(binary).resolve();manifest=Path(manifest).resolve();out=Path(out).resolve()
    command=[str(binary),'--manifest',str(manifest),'--manifest-sha256',sha256_file(manifest),
             '--out',str(out),'--source-root',str(Path(source_root or '.').resolve())]
    if arm is not None:command+=['--arm',arm]
    if resolved_out is not None:command+=['--resolved-out',str(Path(resolved_out).resolve())]
    if build_receipt is not None:command+=['--build-receipt',str(Path(build_receipt).resolve())]
    if validate:command+=['--validate']
    if allow_unfrozen_fixture:command+=['--allow-unfrozen-fixture']
    return subprocess.run(command,capture_output=True,text=True,check=True)


def make_build_receipt(manifest_path,binary_path,source_root,*,prebuild_inventory_sha256,build_command,build_exit_code,toolchain,lockfile_paths,features=None,build_flags=None):
    if build_exit_code!=0:raise ValueError('failed build cannot create binding receipt')
    manifest=strict_json(Path(manifest_path).read_bytes());root=Path(source_root).resolve()
    if manifest.get('model')!='geosim' or manifest.get('provenance_status')!='frozen':raise ValueError('build receipt requires frozen GeoSim sources')
    inventory=manifest['source_inventory'];post=verify_source_inventory(root,inventory)
    if prebuild_inventory_sha256!=post or manifest['source_inventory_sha256']!=post:
        raise ValueError('pre/post build source inventories changed')
    if not build_command or any(not isinstance(s,str) for s in build_command):raise ValueError('explicit build command required')
    if not {'Cargo.lock','survey/Cargo.lock'}<=set(lockfile_paths):raise ValueError('build receipt omits required Cargo locks')
    locks={}
    for name in lockfile_paths:
        path=(root/name).resolve()
        if Path(name).is_absolute() or '..' in Path(name).parts or not path.is_relative_to(root):raise ValueError('unsafe lockfile path')
        locks[name]=sha256_file(path)
    return {'schema_version':1,'model':'geosim','manifest_sha256':sha256_file(manifest_path),
            'source_inventory_sha256':post,'prebuild_inventory_sha256':prebuild_inventory_sha256,'postbuild_inventory_sha256':post,
            'binary_sha256':sha256_file(binary_path),'build_command':list(build_command),'cwd':str(root),
            'target':toolchain['target'],'rustc_version':toolchain['rustc_version'],'cargo_version':toolchain['cargo_version'],
            'lockfile_hashes':locks,'features':features or [],'build_flags':build_flags or []}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('binary','manifest','out','source-root'):parser.add_argument('--'+name,required=True,type=Path)
    parser.add_argument('--arm');parser.add_argument('--validate',action='store_true')
    parser.add_argument('--resolved-out',type=Path);parser.add_argument('--build-receipt',type=Path)
    parser.add_argument('--allow-unfrozen-fixture',action='store_true')
    args=parser.parse_args()
    result=run_native(args.binary,args.manifest,args.out,arm=args.arm,validate=args.validate,
                      resolved_out=args.resolved_out,build_receipt=args.build_receipt,source_root=args.source_root,
                      allow_unfrozen_fixture=args.allow_unfrozen_fixture)
    print(result.stdout,end='')
    if result.stderr:print(result.stderr,end='',file=sys.stderr)

if __name__=='__main__':main()
