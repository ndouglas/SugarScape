"""Prospective manifests and noncircular independent activation identities."""
from copy import deepcopy
from pathlib import Path
import json
import subprocess
from . import followup, records, provenance, followup_runtime as runtime
from .methods import canonical_bytes
from .run import sha256_file
from .analysis import validate_output_destinations

DECLARATION_FIELDS={'schema_version','study_protocol','spec','status','motivated_by_observed_original',
    'historical_input','method_contract_sha256','source_table_sha256','benchmark_evidence_sha256',
    'phase_order','registered_attempts','phases'}
PHASE_BINDING_FIELDS={'phase','reading',*records.BINDING_FIELDS,'analysis_jobs_sha256','native_keys_sha256'}
EXTRA_SOURCES=('survey/src/democratic_peace_followup.rs',followup.SPEC,
    'survey/democratic_peace/followup-probe-schedule.json','survey/democratic_peace/followup-default-config.json',
    'survey/democratic_peace/report-requirements.txt','survey/democratic_peace/FOLLOWUP.md')


def source_inventory(root):
    entries={e['path']:e for e in provenance.source_inventory(root)}
    for name in EXTRA_SOURCES:
        data=provenance.safe_source_path(root,name).read_bytes()
        entries[name]={'path':name,'bytes':len(data),'sha256':followup.sha(data)}
    return [entries[k] for k in sorted(entries)]


def verify_inventory(root,value):
    got=records.verify_source_inventory(root,value['source_inventory'])
    names={e['path'] for e in value['source_inventory']}
    if not set(EXTRA_SOURCES)<=names or got!=value['source_inventory_sha256']:
        raise ValueError('follow-up normative source inventory incomplete')
    return got


def freeze_manifest(value,root,source_bytes):
    from .source import validate_registered_table
    followup.expected_keys(value)
    table=records.strict_json(source_bytes);validate_registered_table(table);provenance.verify_source_review(root,table)
    if value!=followup.build_manifest(table,source_bytes,value['phase'],value['historical_input']):
        raise ValueError('provisional follow-up differs from exact contract')
    if followup.sha(source_bytes)!=followup.SOURCE_TABLE_SHA256 or provenance.safe_source_path(root,provenance.TABLE).read_bytes()!=source_bytes:
        raise ValueError('fixed audited source table/root mismatch')
    inventory=source_inventory(root);result=deepcopy(value)
    result.update(source_inventory=inventory,source_inventory_sha256=followup.sha(canonical_bytes(inventory)),provenance_status='frozen')
    return result


def phase_binding(manifest_path,resolved_path,receipt_path,binary,root):
    data=Path(manifest_path).read_bytes();value=records.strict_json(data);followup.expected_keys(value)
    if value['provenance_status']!='frozen':raise ValueError('declaration requires frozen phase')
    inventory=verify_inventory(root,value);resolved=records.strict_json(Path(resolved_path).read_bytes());followup.validate_resolved(value,resolved)
    receipt=Path(receipt_path).read_bytes();binding={'manifest_sha256':followup.sha(data),
        'binary_sha256':sha256_file(binary),'source_inventory_sha256':inventory,
        'build_receipt_sha256':followup.sha(receipt),'resolved_configs_sha256':resolved['resolved_configs_sha256']}
    if any(resolved[k]!=binding[k] for k in records.BINDING_FIELDS):raise ValueError('phase native binding mismatch')
    records.validate_build_receipt(records.strict_json(receipt),binding['manifest_sha256'],binding['binary_sha256'],inventory,root)
    return {'phase':value['phase'],'reading':value['reading'],**binding,
        'analysis_jobs_sha256':followup.sha(canonical_bytes(value['analysis_jobs'])),
        'native_keys_sha256':followup.sha(canonical_bytes(followup.expected_keys(value)))}


def validate_declaration(value):
    records._fields(value,DECLARATION_FIELDS,'declaration')
    fixed={'schema_version':1,'study_protocol':followup.PROTOCOL,'spec':followup.SPEC,
        'status':'prospective_frozen_not_blind','motivated_by_observed_original':True,
        'historical_input':followup.EXPECTED_HISTORICAL,'method_contract_sha256':followup.sha(canonical_bytes(followup.followup_contract())),
        'source_table_sha256':followup.SOURCE_TABLE_SHA256,'phase_order':['literal_precision','prose_precision'],'registered_attempts':21600}
    if any(not records._same_json_types(value[k],v) for k,v in fixed.items()):raise ValueError('declaration fixed contract mismatch')
    if not isinstance(value['benchmark_evidence_sha256'],str) or len(value['benchmark_evidence_sha256'])!=64 or any(c not in '0123456789abcdef' for c in value['benchmark_evidence_sha256']):raise ValueError('invalid benchmark evidence identity')
    if not isinstance(value['phases'],list) or [p.get('phase') for p in value['phases']]!=fixed['phase_order']:
        raise ValueError('both prospective phases must freeze beforeL')
    for phase in value['phases']:
        records._fields(phase,PHASE_BINDING_FIELDS,'declaration phase binding')
        if phase['reading']!=followup.PHASES[phase['phase']][0] or phase['analysis_jobs_sha256']!=followup.sha(canonical_bytes(followup.analysis_jobs(phase['phase']))):
            raise ValueError('phase declaration reading/job mismatch')
        keys=[(a['id'],a['first_seed']+r) for a in followup.canonical_arms(phase['phase']) for r in range(100)]
        if phase['native_keys_sha256']!=followup.sha(canonical_bytes(keys)):raise ValueError('phase declaration key mismatch')
        for k in records.BINDING_FIELDS:
            if not isinstance(phase[k],str) or len(phase[k])!=64 or any(c not in '0123456789abcdef' for c in phase[k]):raise ValueError('incomplete declaration binding')


def declaration(phases,benchmark_evidence_sha256):
    result={'schema_version':1,'study_protocol':followup.PROTOCOL,'spec':followup.SPEC,
        'status':'prospective_frozen_not_blind','motivated_by_observed_original':True,
        'historical_input':deepcopy(followup.EXPECTED_HISTORICAL),
        'method_contract_sha256':followup.sha(canonical_bytes(followup.followup_contract())),
        'source_table_sha256':followup.SOURCE_TABLE_SHA256,'benchmark_evidence_sha256':benchmark_evidence_sha256,
        'phase_order':['literal_precision','prose_precision'],'registered_attempts':21600,'phases':phases}
    validate_declaration(result);return result


def verify_committed(root,declaration_path,inventory):
    root=Path(root).resolve();path=Path(declaration_path).resolve()
    try:relative=path.relative_to(root).as_posix()
    except ValueError as exc:raise ValueError('declaration must be committed inside source repository') from exc
    if relative in {e['path'] for e in inventory}:raise ValueError('declaration cannot enter its own source inventory')
    if subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=root):raise ValueError('registered execution requires clean committed sources')
    for e in inventory:
        committed=subprocess.run(['git','show','HEAD:'+e['path']],cwd=root,capture_output=True,check=True).stdout
        if followup.sha(committed)!=e['sha256']:raise ValueError('normative source not committed atHEAD')
    committed=subprocess.run(['git','show','HEAD:'+relative],cwd=root,capture_output=True,check=True).stdout
    if committed!=path.read_bytes():raise ValueError('declaration bytes not committed atHEAD')


def verify_activation(value,binding,root,*,declaration_path,review_path,runtime_path):
    followup.expected_keys(value);verify_inventory(root,value)
    data=Path(declaration_path).read_bytes();d=records.strict_json(data);validate_declaration(d)
    review_data=Path(review_path).read_bytes();review=records.strict_json(review_data);runtime.validate_review(review)
    run_data=Path(runtime_path).read_bytes();receipt=records.strict_json(run_data);runtime.validate_runtime_receipt(receipt)
    if review['declaration_sha256']!=followup.sha(data) or receipt['declaration_sha256']!=followup.sha(data) or review['runtime_receipt_sha256']!=followup.sha(run_data) or receipt['benchmark_evidence_sha256']!=d['benchmark_evidence_sha256']:
        raise ValueError('activation external receipt chain mismatch')
    selected=next(p for p in d['phases'] if p['phase']==value['phase'])
    if any(selected[k]!=binding[k] for k in records.BINDING_FIELDS) or selected['analysis_jobs_sha256']!=followup.sha(canonical_bytes(value['analysis_jobs'])):
        raise ValueError('activation phase/source/binary/config binding mismatch')
    verify_committed(root,declaration_path,value['source_inventory'])
    return d


def main():
    import argparse
    from .historical import historical_dataset
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action',choices=('prepare','freeze','declare','probes','attest'))
    parser.add_argument('--phase',choices=tuple(followup.PHASES));parser.add_argument('--output',required=True,type=Path)
    for name in ('source','source-root','manifest','historical-study-root','historical-inventory','historical-source-archive','historical-binary','phase-bindings','benchmark-evidence','declaration'):
        parser.add_argument('--'+name,type=Path)
    args=parser.parse_args()
    inputs=[p for k,p in vars(args).items() if isinstance(p,Path) and k!='output']
    if args.historical_study_root is not None:inputs.extend(args.historical_study_root.glob('*'))
    if args.source_root is not None:inputs.extend(Path(args.source_root)/e['path'] for e in source_inventory(args.source_root))
    new_outputs(inputs,[args.output])
    if args.action=='attest':
        result=runtime.runtime_attestation(args.declaration,args.benchmark_evidence)
    elif args.action=='declare':
        phases=records.strict_json(args.phase_bindings.read_bytes())
        result=declaration(phases,sha256_file(args.benchmark_evidence))
    elif args.action=='freeze':
        result=freeze_manifest(records.strict_json(args.manifest.read_bytes()),args.source_root,args.source.read_bytes())
    else:
        data=args.source.read_bytes();table=records.strict_json(data)
        if args.action=='probes':result=runtime.probe_manifest(table,data,args.phase)
        else:
            old=historical_dataset(study_root=args.historical_study_root,inventory_path=args.historical_inventory,
                source_archive=args.historical_source_archive,binary=args.historical_binary)
            result=followup.build_manifest(table,data,args.phase,old['historical_input'])
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    print('Prospective preparation only; no periods executed or scientific declaration activated')



def new_outputs(inputs,outputs):
    """Reserve fresh destinations before directories, benchmarks or derived writes."""
    validate_output_destinations(inputs,outputs)
    if any(Path(p).exists() or Path(p).is_symlink() for p in outputs):
        raise ValueError('derived evidence destinations must be fresh')


if __name__=='__main__':main()
