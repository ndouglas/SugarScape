"""Controller preparation receipt: compile exact stopped scratch sources."""
from pathlib import Path
import importlib.util, json, subprocess, hashlib, sys
from survey.democratic_peace.followup_registration import source_inventory
from survey.democratic_peace.methods import canonical_bytes
repo=Path.cwd().resolve();out=repo/'survey/out/democratic-peace-precision-corrected-build'
if out.exists():raise SystemExit('Build evidence destination already exists')
spec=importlib.util.spec_from_file_location('original_checks',repo/'docs/superpowers/plans/2026-10-03-democratic-peace/prepare_checks.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
version=subprocess.check_output(['rustc','-Vv'],text=True).strip()
host=next(line.removeprefix('host: ') for line in version.splitlines() if line.startswith('host: '))
flags,context=module.effective_flags(repo,host)
inventory=source_inventory(repo);before=hashlib.sha256(canonical_bytes(inventory)).hexdigest()
out.mkdir(parents=True)
(out/'prebuild-source-inventory.json').write_bytes(canonical_bytes(inventory)+b'\n')
contextpath=out/'cargo-configuration.json';contextpath.write_text(json.dumps(context,indent=2)+'\n')
command=['cargo','build','--release','--locked','--manifest-path','survey/Cargo.toml','--bin','democratic_peace']
with (out/'build.stdout.log').open('wb') as stdout,(out/'build.stderr.log').open('wb') as stderr:
    result=subprocess.run(command,stdout=stdout,stderr=stderr)
if result.returncode:raise SystemExit('Build failed: see retained logs')
if source_inventory(repo)!=inventory:raise SystemExit('Source changed during build')
binary=repo/'survey/target/release/democratic_peace'
evidence={'build_command':command,'build_exit_code':result.returncode,
'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'source_inventory_sha256':before,
'effective_build_flags':flags,'features':[],
'cargo_configuration':{'path':str(contextpath),'bytes':contextpath.stat().st_size,'sha256':hashlib.sha256(contextpath.read_bytes()).hexdigest()},
'toolchain':{'target':host,'rustc_version':version,'cargo_version':subprocess.check_output(['cargo','--version'],text=True).strip()}}
from survey.democratic_peace.followup_evidence import validate_build_evidence
validate_build_evidence(evidence)
(out/'build-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
receipt={'source_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
'prebuild_inventory_sha256':before,'postbuild_inventory_sha256':before,**evidence}
(out/'actual-build-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'source_count':len(inventory),'source_sha256':before,'binary_sha256':evidence['binary_sha256'],'flags':flags,'build_exit_code':result.returncode}))
