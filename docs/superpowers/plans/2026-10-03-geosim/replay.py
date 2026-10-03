"""Apply verified test/implementation checkpoints without changing index or HEAD."""
import argparse
import hashlib
import json
import re
import stat
import subprocess
import sys
from pathlib import Path,PurePosixPath

class ReplayError(ValueError):
    pass


def relative(name):
    if not isinstance(name,str) or not name or any(c in name for c in ('\\','\0','\n','\r','\t')):
        raise ReplayError(f'Unsafe path: {name!r}')
    path=PurePosixPath(name)
    if path.is_absolute() or '..' in path.parts or '.' in path.parts or path.as_posix()!=name:
        raise ReplayError(f'Unsafe path: {name!r}')
    return path


def contained(root,name):
    current=root
    for part in relative(name).parts:
        current=current/part
        if current.is_symlink():raise ReplayError(f'Forbidden symlink in path: {name}')
    if not current.resolve().is_relative_to(root):raise ReplayError(f'Unsafe path: {name}')
    return current


def image(path):
    if not path.exists():return None
    info=path.lstat()
    if not stat.S_ISREG(info.st_mode):raise ReplayError(f'Nonregular inventory file: {path}')
    return {'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'mode':'100755' if info.st_mode & stat.S_IXUSR else '100644'}


def checked_spec(spec):
    if not isinstance(spec,dict) or set(spec)!= {'sha256','mode'} or spec['mode'] not in ('100644','100755') or not re.fullmatch(r'[a-f0-9]{64}',spec['sha256']):
        raise ReplayError(f'Invalid file image: {spec!r}')


def git(target,*args,data=None):
    proc=subprocess.run(['git',*args],cwd=target,input=data,capture_output=True)
    if proc.returncode:raise ReplayError(proc.stderr.decode(errors='replace').strip() or f'Git operation failed: {args}')
    return proc.stdout


def metadata(name,document):
    return name in document.get('metadata_files',[]) or any(name.startswith(p) for p in document.get('metadata_prefixes',[]))


def ignored(name,document):
    return PurePosixPath(name).name in document['excluded_names'] or any(name.startswith(p) for p in document['excluded_prefixes']) or metadata(name,document)


def product(name,document):
    return not ignored(name,document) and (name in document['product_files'] or any(name.startswith(p) for p in document['product_roots']))


def validate_document(document,target,assets):
    if document.get('version')!=1 or type(document.get('complete')) is not bool:raise ReplayError('Unsupported manifest version/completion')
    if not re.fullmatch(r'[a-f0-9]{40}',document.get('base','')):raise ReplayError('Invalid source base SHA')
    for field in ('product_roots','excluded_prefixes','metadata_prefixes'):
        for prefix in document.get(field,[]):
            if not prefix.endswith('/'):raise ReplayError(f'Invalid directory prefix: {prefix}')
            relative(prefix[:-1])
    for field in ('product_files','excluded_names','metadata_files'):
        for name in document.get(field,[]):relative(name)
    checkpoints=document['checkpoints'];patches=document['patches'];protected=document['protected']
    if len(checkpoints)!=len(patches)+1 or not patches:raise ReplayError('Invalid checkpoint count')
    for files in [protected,*checkpoints]:
        for name,spec in files.items():contained(target,name);checked_spec(spec)
    for files in checkpoints:
        if set(files)&set(protected):raise ReplayError('Product and protected inventories overlap')
        if any(not product(name,document) for name in files):raise ReplayError('Checkpoint contains a nonproduct path')
    for i,patch in enumerate(patches):
        path=contained(assets,patch['name'])
        if not path.is_file():raise ReplayError(f'Missing patch: {patch["name"]}')
        data=path.read_bytes()
        if hashlib.sha256(data).hexdigest()!=patch['sha256']:raise ReplayError(f'Changed patch SHA256: {patch["name"]}')
        modes=re.findall(rb'^(?:new file mode|deleted file mode|old mode|new mode) ([0-9]+)$',data,re.M)
        if any(m not in (b'100644',b'100755') for m in modes):raise ReplayError('Unsupported patch mode')
        if re.search(rb'^(?:rename from|rename to|copy from|copy to) ',data,re.M):raise ReplayError('Renames/copies are not part of this packet')
        before,after=checkpoints[i:i+2]
        expected={p:{'before':before.get(p),'after':after.get(p)} for p in sorted(before.keys()|after.keys()) if before.get(p)!=after.get(p)}
        if not expected or patch['changes']!=expected:raise ReplayError('Patch declared changes disagree with checkpoints')
        numstat=git(target,'apply','--numstat','-z','-',data=data)
        names=set()
        for row in numstat.split(b'\0'):
            if row:
                fields=row.split(b'\t',2)
                if len(fields)!=3:raise ReplayError('Invalid patch path record')
                name=fields[2].decode();contained(target,name);names.add(name)
        if names!=set(expected):raise ReplayError(f'Patch paths differ from declared changes: {patch["name"]}')


def verify_protected(document,target):
    for name,spec in document['protected'].items():
        if image(contained(target,name))!=spec:raise ReplayError(f'Changed protected source/base file: {name}')


def observed_products(document,target):
    for root in document['product_roots']:contained(target,root[:-1])
    names={p.decode() for p in git(target,'ls-files','--cached','--others','--exclude-standard','-z').split(b'\0') if p}
    result={}
    for name in names:
        if ignored(name,document):continue
        path=contained(target,name)
        if not path.exists():continue
        if product(name,document):result[name]=image(path)
        elif name not in document['protected']:raise ReplayError(f'Unexpected protected/base path: {name}')
    return result


def checkpoint(document,target):
    verify_protected(document,target)
    observed=observed_products(document,target)
    matches=[i for i,expected in enumerate(document['checkpoints']) if observed==expected]
    if len(matches)!=1:raise ReplayError('Target does not match a unique verified checkpoint (content/path/mode preimage mismatch)')
    return matches[0]


def replay(document,target,assets,through=None,verify_only=False):
    if through is None:
        if not document['complete']:raise ReplayError('Packet is incomplete; an explicit prefix is required')
        through=len(document['patches'])
    if not 0<=through<=len(document['patches']):raise ReplayError('Requested checkpoint is unavailable')
    validate_document(document,target,assets)
    top=Path(git(target,'rev-parse','--show-toplevel').decode().strip()).resolve()
    if top!=target:raise ReplayError('--target must be a Git worktree root')
    if not verify_only:git(target,'merge-base','--is-ancestor',document['base'],'HEAD')
    current=checkpoint(document,target)
    if current>through:raise ReplayError('Replay cannot rewind a verified checkpoint')
    if verify_only and current!=through:raise ReplayError('Target is not at the requested final checkpoint')
    for i in range(current,through):
        if checkpoint(document,target)!=i:raise ReplayError('Unexpected checkpoint before patch')
        patch=contained(assets,document['patches'][i]['name'])
        git(target,'apply','--check',str(patch));git(target,'apply',str(patch))
        if checkpoint(document,target)!=i+1:raise ReplayError('Postimage differs after patch')
    print(f'Verified checkpoint {through}/{len(document["patches"])}: {len(document["checkpoints"][through])} product files and {len(document["protected"])} protected files, bytes and Git modes.')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target',required=True,type=Path)
    parser.add_argument('--manifest',type=Path,default=Path(__file__).with_name('manifest.json'))
    parser.add_argument('--through',type=int)
    parser.add_argument('--verify-only',action='store_true')
    args=parser.parse_args()
    try:
        if args.target.is_symlink() or args.manifest.is_symlink():raise ReplayError('Target/manifest symlink is forbidden')
        target=args.target.resolve(strict=True);manifest=args.manifest.resolve(strict=True)
        document=json.loads(manifest.read_text())
        replay(document,target,manifest.parent,args.through,args.verify_only)
    except (ReplayError,OSError,KeyError,TypeError,UnicodeError,json.JSONDecodeError) as error:
        print(f'Replay rejected: {error}',file=sys.stderr);return 1
    return 0

if __name__=='__main__':sys.exit(main())
