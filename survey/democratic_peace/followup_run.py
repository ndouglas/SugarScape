"""Explicit follow-up activation and deterministic between-history checkpoints."""
from pathlib import Path
import subprocess
from . import followup, records
from .analysis import validate_output_destinations
from .followup_registration import verify_activation, verify_inventory
from .historical import historical_dataset
from .run import sha256_file


def batch_keys(value,seen,maximum=None):
    keys=followup.expected_keys(value)
    if set(seen)-set(keys):raise ValueError('undeclared resume keys')
    if maximum is not None and (type(maximum) is not int or not 1<=maximum<=10800):
        raise ValueError('checkpoint bound must be1..10800new histories')
    pending=[key for key in keys if key not in set(seen)]
    selected=pending if maximum is None else pending[:maximum]
    return {'keys':selected,'attempted_before':len(seen),'pending_after':len(pending)-len(selected),
        'status':'pending' if len(selected)<len(pending) else 'complete_attempted_census'}


def run_native(binary,manifest_path,resolved_out,out,receipt_path,source_root,*,
    declaration_path,review_path,runtime_path,historical_study_root,historical_inventory,
    historical_source_archive,historical_binary,literal_sessions=None,max_new_histories=None):
    value=records.strict_json(Path(manifest_path).read_bytes());followup.expected_keys(value)
    protected=[manifest_path,binary,receipt_path,declaration_path,review_path,runtime_path,
        historical_inventory,historical_source_archive,historical_binary]
    protected.extend(Path(historical_study_root).glob('*'))
    protected.extend(Path(source_root)/e['path'] for e in value.get('source_inventory') or [])
    if literal_sessions is not None:protected.append(literal_sessions)
    validate_output_destinations(protected,[resolved_out,out])
    old=historical_dataset(study_root=historical_study_root,inventory_path=historical_inventory,
        source_archive=historical_source_archive,binary=historical_binary)
    if old['historical_input']!=value['historical_input']:raise ValueError('wrong historical input')
    inventory=verify_inventory(source_root,value);receipt=Path(receipt_path).read_bytes()
    # The committed declaration binds resolved payload before the native export
    # is regenerated, so its previously validate-only export is an explicit input.
    resolved=records.strict_json(Path(resolved_out).read_bytes());followup.validate_resolved(value,resolved)
    binding={'manifest_sha256':sha256_file(manifest_path),'binary_sha256':sha256_file(binary),
        'source_inventory_sha256':inventory,'build_receipt_sha256':followup.sha(receipt),
        'resolved_configs_sha256':resolved['resolved_configs_sha256']}
    records.validate_build_receipt(records.strict_json(receipt),binding['manifest_sha256'],binding['binary_sha256'],inventory,source_root)
    verify_activation(value,binding,source_root,declaration_path=declaration_path,review_path=review_path,runtime_path=runtime_path)
    if value['phase']=='prose_precision' and literal_sessions is None:raise ValueError('full literal attempted census required before prose')
    seen=records.read_sessions(out,value,resolved,binding) if Path(out).exists() else {}
    batch_keys(value,seen,max_new_histories)
    command=[str(Path(binary).resolve())]
    options={'--manifest':manifest_path,'--resolved':resolved_out,'--out':out,'--receipt':receipt_path,
        '--repo':source_root,'--declaration':declaration_path,'--declaration-review':review_path,
        '--runtime-receipt':runtime_path,'--historical-study-root':historical_study_root,
        '--historical-inventory':historical_inventory,'--historical-source-archive':historical_source_archive,
        '--historical-binary':historical_binary}
    for option,path in options.items():command.extend([option,str(Path(path).resolve())])
    if literal_sessions is not None:command.extend(['--literal-sessions',str(Path(literal_sessions).resolve())])
    if max_new_histories is not None:command.extend(['--max-new-histories',str(max_new_histories)])
    return subprocess.run(command,capture_output=True,text=True,check=True)


def main():
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('binary','manifest','resolved','out','receipt','source-root','declaration',
        'declaration-review','runtime-receipt','historical-study-root','historical-inventory',
        'historical-source-archive','historical-binary'):
        parser.add_argument('--'+name,required=True,type=Path)
    parser.add_argument('--literal-sessions',type=Path);parser.add_argument('--max-new-histories',type=int)
    a=parser.parse_args()
    result=run_native(a.binary,a.manifest,a.resolved,a.out,a.receipt,a.source_root,
        declaration_path=a.declaration,review_path=a.declaration_review,runtime_path=a.runtime_receipt,
        historical_study_root=a.historical_study_root,historical_inventory=a.historical_inventory,
        historical_source_archive=a.historical_source_archive,historical_binary=a.historical_binary,
        literal_sessions=a.literal_sessions,max_new_histories=a.max_new_histories)
    print(result.stdout,end='')


if __name__=='__main__':main()
