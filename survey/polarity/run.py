"""Run the Rust survey binary, binding raw sessions to exact manifest bytes.

Build first: cargo build --release --manifest-path survey/Cargo.toml --bin polarity
Then run with --manifest survey/polarity/studies.json --out survey/out/polarity-sessions.jsonl.
Use --validate to check the complete manifest without executing a single period.
"""
import argparse
import hashlib
from pathlib import Path
import subprocess
import sys


def run_native(binary,manifest,out,arm=None,validate=False):
    manifest=Path(manifest).resolve()
    out=Path(out).resolve()
    digest=hashlib.sha256(manifest.read_bytes()).hexdigest()
    argv=[str(Path(binary).resolve()),'--manifest',str(manifest),
          '--manifest-sha256',digest,'--out',str(out)]
    if arm is not None:
        argv+=['--arm',arm]
    if validate:
        argv.append('--validate')
    return subprocess.run(argv,text=True,capture_output=True,check=False)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',required=True,type=Path)
    parser.add_argument('--out',required=True,type=Path)
    parser.add_argument('--binary',type=Path,default=Path(__file__).resolve().parents[1]/'target/release/polarity')
    parser.add_argument('--arm')
    parser.add_argument('--validate',action='store_true')
    args=parser.parse_args()
    try:
        result=run_native(args.binary,args.manifest,args.out,args.arm,args.validate)
    except OSError as error:
        parser.exit(2,f'{error}\nBuild the native binary before running the survey.\n')
    print(result.stdout,end='')
    print(result.stderr,end='',file=sys.stderr)
    raise SystemExit(result.returncode)

if __name__=='__main__':
    main()
