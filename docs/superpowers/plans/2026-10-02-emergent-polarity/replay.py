"""Replay the verified plan, checking preimages and final product bytes."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

assets = Path(__file__).resolve().parent
manifest = json.loads((assets / "manifest.json").read_text())
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--target", required=True, type=Path)
parser.add_argument("--verify-only", action="store_true")
args = parser.parse_args()
target = args.target.resolve()
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None

def verify(which):
    mismatches = [path for path, spec in manifest["files"].items()
                  if sha(target / path) != spec[which]]
    if mismatches:
        raise SystemExit(which + " content mismatch: " + ", ".join(mismatches))

for patch in manifest["patches"]:
    if sha(assets / patch["name"]) != patch["sha256"]:
        raise SystemExit("Plan patch content changed: " + patch["name"])
if not args.verify_only:
    subprocess.run(["git", "merge-base", "--is-ancestor", manifest["base"], "HEAD"],
                   cwd=target, check=True)
    verify("before")
    for patch in manifest["patches"]:
        subprocess.run(["git", "apply", "--check", str(assets / patch["name"])],
                       cwd=target, check=True)
        subprocess.run(["git", "apply", str(assets / patch["name"])],
                       cwd=target, check=True)
verify("after")
print("Verified " + str(len(manifest["files"])) + " product files byte for byte.")
