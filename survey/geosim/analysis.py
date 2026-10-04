"""Fixed source and modern matrices; no reduced registered resampling entry point."""
from collections import Counter
import hashlib
import json
from pathlib import Path
from . import manifest as population
from . import modern,source
from .methods import method_contract,contract_sha256


def _compact_parameter(result):
    return {**{k:v for k,v in result.items() if k not in ('replicates','eligibility_counts')},
            'eligibility_count_histogram':dict(sorted(Counter(result.get('eligibility_counts',[])).items()))}


def _compact_source(history):
    result={k:v for k,v in history.items() if k!='census'}
    census=history.get('census')
    if census is not None:
        result['census']={k:v for k,v in census.items() if k not in ('primary_completed','partial_completed','source_selected','partial_source_selected','excluded')}
        result['census']['exclusions']=[{'war_id':e['war']['id'],'reasons':e['reasons']} for e in census['excluded']]
    return result


def _compact_modern(history):
    result={k:v for k,v in history.items() if k not in ('raw_sizes','excluded')}
    result['exclusions']=[{'war_id':e['war']['id'],'reasons':e['reasons']} for e in history.get('excluded',[])]
    return result


def report(manifest,table,records,resolved=None,binding=None,*,fixture_draws=None,alternatives=True,data_sha256=None):
    if manifest['method_contract']!=method_contract() or manifest['method_contract_sha256']!=contract_sha256():
        raise ValueError('method contract differs from frozen implementation')
    payload=manifest.get('method_contract_json')
    if not isinstance(payload,str) or hashlib.sha256(payload.encode()).hexdigest()!=contract_sha256() or json.loads(payload)!=method_contract():
        raise ValueError('exact method contract payload mismatch')
    fixture=fixture_draws is not None
    if not fixture and manifest['provenance_status']!='frozen':raise ValueError('registered analysis requires frozen source provenance')
    if not fixture and (resolved is None or binding is None or data_sha256 is None):raise ValueError('registered analysis requires full data/resolved provenance')
    if not fixture and not alternatives:raise ValueError('registered analysis requires all four alternatives')
    draws={'source':100000,'ks':1000,'parameters':100000} if not fixture else fixture_draws
    if set(draws)!={'source','ks','parameters'} or any(type(v) is not int or v<=0 for v in draws.values()):raise ValueError('invalid fixed draw counts')
    keys=population.expected_keys(manifest);registered=set(keys)
    if set(records)-registered:raise ValueError('unregistered raw session keys')
    if resolved is not None:
        from .records import validate_record,resolved_payload
        resolved_payload(resolved);configs={a['id']:a['config'] for a in resolved['arms']}
        indexed={a['id']:{**a,'config':configs[a['id']]} for a in manifest['arms']}
        for key,row in records.items():
            if validate_record(row,indexed[key[0]],binding)!=key:raise ValueError('raw dictionary key differs from envelope')
    expected_jobs=population.analysis_jobs(manifest)
    if manifest['analysis_jobs']!=expected_jobs:raise ValueError('fixed analysis job payload mismatch')
    jobs={j['id']:j for j in expected_jobs}
    sh={a['id']:[] for a in manifest['arms']};mh={a['id']:[] for a in manifest['arms']};history_rows=[]
    for arm,seed in keys:
        if (arm,seed) not in records:
            history_rows.append({'arm':arm,'seed':seed,'status':'missing','source':None,'modern':None})
            continue
        row=records[(arm,seed)];s=source.history_source(row);m=modern.history_modern(row,alternatives=alternatives)
        sh[arm].append(s);mh[arm].append(m)
        history_rows.append({'arm':arm,'seed':seed,'status':s['availability']['status'],'source':_compact_source(s),'modern':_compact_modern(m)})
    source_result=source.source_findings(table,sh,jobs,draws['source']);contrasts=source.contrast_findings(sh,jobs,draws['source'])
    pools={};parameters={}
    for arm in manifest['arms']:
        aid=arm['id'];histories=mh[aid]
        if arm['family'] in ('original','precision'):
            pool=modern.fit_pool(histories,arm['sessions'],alternatives=alternatives)
            raw=[x for h in histories for x in h['raw_sizes']]
            pool['ks_test']=modern.ks_refit_test(pool,raw,population.job_rng(jobs[f'ks.{aid}']),draws['ks'])
            pools[aid]=pool
        parameters[aid]=modern.parameter_bootstrap(histories,population.job_rng(jobs[f'parameters.{aid}']),arm['sessions'],draws['parameters'])
    parameter_contrasts={name:modern.parameter_contrast(parameters['precision.base'],parameters[f'precision.{control}'])
                        for name,control in [('base_minus_shock0','shock0'),('base_minus_context_off','context_off')]}
    return {'schema_version':1,'model':'geosim','classification':'synthetic_fixture' if fixture else 'registered_offline_findings',
            'registered_histories':1490,'received_histories':len(records),'history_status_counts':dict(Counter(h['status'] for h in history_rows)),
            'histories':history_rows,'source':source_result,'source_contrasts':contrasts,'modern_pools':pools,
            'modern_parameters':{k:_compact_parameter(v) for k,v in parameters.items()},'modern_parameter_contrasts':parameter_contrasts,
            'analysis_jobs':manifest['analysis_jobs'],'method_contract_sha256':manifest['method_contract_sha256'],'draws':draws,
            'provenance':{'data_sha256':data_sha256,'binding':binding,'source_table_sha256':manifest['source_table_sha256']},
            'empirical':{'cow_v4':{'status':'Available_documented_preparation_input','runtime_input_verification':'Unresolved_not_supplied_to_analysis','unknown_fatalities':[{'war_id':170,'participant':'Thailand','code':-9}],
                                    'missing_policy':'unknown_not_zero; exact aggregation awaits source protocol'},
                         'clauset_s1':{'status':'Unresolved','reason':'Supplement_S1_estimator_bootstrap_missing_participant_handling_unavailable'},
                         'cederman_historical':{'status':'Unresolved','reason':'distinct1820_1997_input_unavailable_no_modern_substitution'}},
            'artifact_reference':{'execution':'Available_documented_isolated_later_port','runtime_trace_verification':'Unresolved_not_supplied_to_analysis','identity':'Unresolved','rng':'PCG_does_not_match_two_MT_streams_by_seed'}}


def main():
    import argparse
    import platform
    import numpy
    import scipy
    from .records import strict_json,verify_source_inventory,validate_build_receipt,read_sessions
    from .run import sha256_file
    from .reporting import serialize_report,markdown_report
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('manifest','source','sessions','resolved','build-receipt','binary','source-root','output'):
        parser.add_argument('--'+name,required=True,type=Path)
    args=parser.parse_args();manifest_bytes=args.manifest.read_bytes();m=strict_json(manifest_bytes)
    if m['provenance_status']!='frozen':raise ValueError('registered analysis requires frozen source provenance')
    wanted=method_contract()['runtime'];actual={'python':platform.python_version(),'numpy':numpy.__version__,'scipy':scipy.__version__}
    if actual!=wanted:raise ValueError(f'analysis runtime differs from pinned contract: {actual}')
    source_bytes=args.source.read_bytes()
    if hashlib.sha256(source_bytes).hexdigest()!=m['source_table_sha256']:raise ValueError('source table byte SHA256 mismatch')
    inventory=verify_source_inventory(args.source_root,m['source_inventory'])
    if inventory!=m['source_inventory_sha256']:raise ValueError('source inventory digest mismatch')
    manifest_hash=hashlib.sha256(manifest_bytes).hexdigest();binary_hash=sha256_file(args.binary)
    receipt_bytes=args.build_receipt.read_bytes();receipt=strict_json(receipt_bytes)
    validate_build_receipt(receipt,manifest_hash,binary_hash,inventory,args.source_root)
    resolved=strict_json(args.resolved.read_bytes())
    binding={'manifest_sha256':manifest_hash,'binary_sha256':binary_hash,'source_inventory_sha256':inventory,
             'build_receipt_sha256':hashlib.sha256(receipt_bytes).hexdigest(),'resolved_configs_sha256':resolved['resolved_configs_sha256']}
    data_hash=sha256_file(args.sessions);records=read_sessions(args.sessions,m,resolved,binding)
    if sha256_file(args.sessions)!=data_hash:raise ValueError('raw dataset changed during validation')
    result=report(m,strict_json(source_bytes),records,resolved,binding,data_sha256=data_hash)
    if verify_source_inventory(args.source_root,m['source_inventory'])!=inventory:raise ValueError('scientific source bytes changed during analysis')
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.with_suffix('.json').write_text(serialize_report(result),encoding='utf-8')
    args.output.with_suffix('.md').write_text(markdown_report(result),encoding='utf-8')

if __name__=='__main__':main()
