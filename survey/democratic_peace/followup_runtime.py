"""Unregistered preparation evidence and conservative complete-phase forecasts."""
from pathlib import Path
import platform
import resource
import subprocess
import time
import math
import sys
import numpy as np
from . import followup, manifest, records, numerics
from .methods import canonical_bytes, method_contract

SCHEDULE = records.strict_json(Path(__file__).with_name('followup-probe-schedule.json').read_bytes())
ANALYSIS_TYPES = ('predictive', 'contrast_pair')
MEASUREMENT_FIELDS = {'cpu_seconds','wall_seconds','peak_rss_bytes','command','exit_code',
    'source_inventory_sha256','binary_sha256','toolchain','output_sha256','classification'}


def probe_manifest(table, data, phase):
    reading=followup.PHASES[phase][0]
    configs=[{'width':15,'height':15,'horizon_periods':1000,'periods_per_tick':100,
        'mobile_share':c['mobile'],'mechanism':c['mechanism'],
        'initial_democratic_share':c['density'],'probability_direction':reading}
        for c in SCHEDULE['coordinates']]
    value=manifest.build_unregistered_manifest(table,data,configurations=configs,mode='runtime_probe')
    for i,arm in enumerate(value['arms']):arm['first_seed']=SCHEDULE['seed_bases'][reading]+i
    manifest.expected_keys(value)
    return value


def validate_measurement(m):
    expected=MEASUREMENT_FIELDS | ({'workload','artifacts'} if 'workload' in m or 'artifacts' in m else set())
    records._fields(m,expected,'runtime measurement')
    if m['classification']!='unregistered_runtime_only' or type(m['exit_code']) is not int or m['exit_code']!=0:
        raise ValueError('failed or registered runtime evidence')
    if any(type(m[k]) not in (int,float) or not math.isfinite(m[k]) or m[k]<=0 for k in ('cpu_seconds','wall_seconds')) or type(m['peak_rss_bytes']) is not int or m['peak_rss_bytes']<=0:
        raise ValueError('invalid runtime resource measurement')
    if not isinstance(m['command'],list) or not m['command'] or any(not isinstance(x,str) or not x for x in m['command']):
        raise ValueError('missing exact benchmark command')
    for key in ('source_inventory_sha256','binary_sha256','output_sha256'):
        if not isinstance(m[key],str) or len(m[key])!=64 or any(c not in '0123456789abcdef' for c in m[key]):
            raise ValueError('missing benchmark identities')
    if not isinstance(m['toolchain'],dict) or not m['toolchain']:raise ValueError('missing toolchain')


def phase_forecast(probes, analysis, writer):
    if len(probes)!=8 or set(analysis)!=set(ANALYSIS_TYPES):
        raise ValueError('full predeclared probe/job workload required')
    for m in [*probes,*analysis.values(),writer]:validate_measurement(m)
    generation=10800*max(m['cpu_seconds'] for m in probes)
    # A paired measurement executes both original contrast jobs exactly once.
    inference=105*analysis['predictive']['cpu_seconds']+12*analysis['contrast_pair']['cpu_seconds']
    writing=writer['cpu_seconds']
    cpu=2*(generation+inference+writing)
    peak=max(m['peak_rss_bytes'] for m in [*probes,*analysis.values(),writer])
    return {'cpu_seconds':cpu,'cpu_hours':cpu/3600,'generation_cpu_seconds':generation,
        'analysis_cpu_seconds':inference,'analysis_writer_cpu_seconds':writing,
        'forecast_multiplier':2,'native_attempts':10800,'analysis_jobs':129,
        'draws_per_job':100000,'native_workers':1,'peak_rss_bytes':peak,
        'within_budget':cpu<=12*3600 and peak<=2**31,
        'assumptions':['maximum_eight_complete_probe_cpu_times_10800',
            'maximum_synthetic_predictive_with_interval_optimization_times105',
            'maximum_joint_permutation_bootstrap_cpu_times12_no_double_count',
            'fullphase_synthetic_analysis_serialization_fsync_measured_once',
            'native_probe_cpu_includes_native_row_writer','wall_time_not_guaranteed']}


def validate_forecast(forecast):
    expected={'cpu_seconds','cpu_hours','generation_cpu_seconds','analysis_cpu_seconds',
        'analysis_writer_cpu_seconds','forecast_multiplier','native_attempts','analysis_jobs',
        'draws_per_job','native_workers','peak_rss_bytes','within_budget','assumptions'}
    records._fields(forecast,expected,'phase forecast')
    if any(not records._same_json_types(forecast[k],v) for k,v in {
        'forecast_multiplier':2,'native_attempts':10800,'analysis_jobs':129,
        'draws_per_job':100000,'native_workers':1,'within_budget':True}.items()):
        raise ValueError('complete phase forecast failed scheduling gate')
    parts=[forecast[k] for k in ('generation_cpu_seconds','analysis_cpu_seconds','analysis_writer_cpu_seconds')]
    if any(type(x) not in (int,float) or not math.isfinite(x) or x<=0 for x in parts) or abs(forecast['cpu_seconds']-2*sum(parts))>1e-8 or not 0<forecast['cpu_seconds']<=43200 or forecast['cpu_hours']!=forecast['cpu_seconds']/3600:
        raise ValueError('forecast resource arithmetic mismatch')
    if type(forecast['peak_rss_bytes']) is not int or not 0<forecast['peak_rss_bytes']<=2**31:
        raise ValueError('forecast memory gate exceeded')


def validate_runtime_receipt(value):
    records._fields(value,{'schema_version','study_protocol','status','declaration_sha256',
        'benchmark_evidence_sha256','phases','combined_cpu_seconds'},'runtime receipt')
    if type(value['schema_version']) is not int or value['schema_version']!=1 or value['study_protocol']!=followup.PROTOCOL or value['status']!='approved_complete_phase_forecast' or set(value['phases'])!=set(followup.PHASES):
        raise ValueError('invalid runtime receipt')
    for forecast in value['phases'].values():validate_forecast(forecast)
    total=sum(f['cpu_seconds'] for f in value['phases'].values())
    if value['combined_cpu_seconds']!=total or total>86400:raise ValueError('combined forecast exceeds24CPUhours')


def validate_review(value):
    records._fields(value,{'schema_version','study_protocol','decision','independent','reviewer',
        'registered_histories','declaration_sha256','runtime_receipt_sha256'},'declaration review')
    if type(value['schema_version']) is not int or value['schema_version']!=1 or value['study_protocol']!=followup.PROTOCOL or value['decision']!='accepted_prospective_followup' or value['independent'] is not True or not isinstance(value['reviewer'],str) or not value['reviewer'].strip() or type(value['registered_histories']) is not int or value['registered_histories']!=0:
        raise ValueError('independent premeasurement review required')


def verify_activation(*args, **kwargs):
    from .followup_registration import verify_activation as verify
    return verify(*args,**kwargs)


def rss_bytes(value):
    return int(value if sys.platform=='darwin' else value*1024)


def measure_command(command, stdout_path, stderr_path, *, source_inventory_sha256, binary_sha256, toolchain, cwd=None):
    """Retain exact process resource usage; command is explicit and never inferred."""
    import os
    start=time.perf_counter()
    with Path(stdout_path).open('wb') as out,Path(stderr_path).open('wb') as err:
        child=subprocess.Popen(command,stdout=out,stderr=err,cwd=cwd)
        _,status,usage=os.wait4(child.pid,0);child.returncode=os.waitstatus_to_exitcode(status)
    return {'cpu_seconds':usage.ru_utime+usage.ru_stime,'wall_seconds':time.perf_counter()-start,
        'peak_rss_bytes':rss_bytes(usage.ru_maxrss),'command':list(command),'exit_code':child.returncode,
        'source_inventory_sha256':source_inventory_sha256,'binary_sha256':binary_sha256,
        'toolchain':toolchain,'output_sha256':followup.sha(Path(stdout_path).read_bytes()),
        'classification':'unregistered_runtime_only'}


def synthetic_benchmark(kind, phase, output):
    """Exactly100000draws on synthetic values; never empirical history inference."""
    import json,os
    actual={'python':platform.python_version(),'numpy':np.__version__}
    if actual!=method_contract()['runtime']:raise ValueError('benchmark numerical runtime mismatch')
    jobs=followup.analysis_jobs(phase)
    start=time.process_time();wall=time.perf_counter()
    if kind=='predictive':
        from .followup_evidence import synthetic_cases,synthetic_header
        results=[]
        for name,values in synthetic_cases():
            prediction=numerics.predictive(values,followup.job_rng(jobs[0],phase))
            result={k:v for k,v in prediction.items() if k!='replicates'}
            if prediction['status']=='Available':result.update(numerics.maximize_interval_p(prediction['replicates'],[0.,1.]))
            results.append({'case':name,'input_values_sha256':followup.sha(canonical_bytes(values)),'job_index':0,'result':result})
        payload={**synthetic_header(kind,phase),'classification':'synthetic_analysis_timing_only','draws':100000,'workloads':results}
        # Four workloads in one timing receipt is deliberately conservative.
    elif kind=='contrast_pair':
        result=numerics.contrast([0.]*50+[1.]*50,[.25]*100,
            followup.job_rng(jobs[105],phase),followup.job_rng(jobs[106],phase))
        from .followup_evidence import synthetic_header
        payload={**synthetic_header(kind,phase),'classification':'synthetic_analysis_timing_only','draws_per_job':100000,'analysis_jobs':2,
            'job_indices':[105,106],'input_values_sha256':followup.sha(canonical_bytes([[0.]*50+[1.]*50,[.25]*100])),'result':result}
    elif kind=='writer':
        from .followup_evidence import writer_payload
        payload=writer_payload(phase)
    else:raise ValueError('unknown synthetic benchmark kind')
    with Path(output).open('w') as file:
        json.dump(payload,file,sort_keys=True,allow_nan=False);file.write('\n');file.flush();os.fsync(file.fileno())
    return {'classification':'synthetic_analysis_timing_only','cpu_seconds':time.process_time()-start,
        'wall_seconds':time.perf_counter()-wall,'peak_rss_bytes':rss_bytes(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss),
        'output_sha256':followup.sha(Path(output).read_bytes()),'kind':kind,'phase':phase,'runtime':actual}


def main():
    import argparse,json
    parser=argparse.ArgumentParser(description=__doc__)
    mode=parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--synthetic-kind',choices=(*ANALYSIS_TYPES,'writer'))
    mode.add_argument('--preparation-benchmarks',action='store_true')
    parser.add_argument('--phase',choices=tuple(followup.PHASES))
    parser.add_argument('--output',required=True,type=Path)
    for name in ('binary','source-root','source','build-evidence','python'):
        parser.add_argument('--'+name,type=Path)
    args=parser.parse_args()
    from .followup_registration import new_outputs,source_inventory
    root=Path(__file__).resolve().parents[2]
    try:relative=args.output.resolve().relative_to(root/'survey/out')
    except ValueError as exc:raise ValueError('runtime evidence must use repo-local ignored survey/out/democratic-peace-* paths') from exc
    if not relative.parts or not relative.parts[0].startswith('democratic-peace-'):
        raise ValueError('runtime evidence must use ignored democratic-peace prefix')
    inputs=[root/e['path'] for e in source_inventory(root)]
    if args.preparation_benchmarks:
        result=benchmark_preparation(binary=args.binary,source_root=args.source_root,source_path=args.source,
            build_evidence=args.build_evidence,output_dir=args.output,python=args.python)
    else:
        if args.phase is None:raise ValueError('synthetic benchmark requires explicit phase')
        new_outputs(inputs,[args.output]);args.output.parent.mkdir(parents=True,exist_ok=True)
        result=synthetic_benchmark(args.synthetic_kind,args.phase,args.output)
    print(json.dumps(result,sort_keys=True))



def selected_python(path):
    """Keep a venv's final executable symlink and inspect it before any probe/write."""
    import os
    executable=os.path.abspath(os.fspath(path))
    script="import json, platform, sys, numpy; print(json.dumps({'python':platform.python_version(),'numpy':numpy.__version__,'prefix':sys.prefix}))"
    try:
        result=subprocess.run([executable,'-c',script],capture_output=True,text=True,check=True,
            env={**os.environ,'PYTHONDONTWRITEBYTECODE':'1'})
        value=records.strict_json(result.stdout)
        records._fields(value,{'python','numpy','prefix'},'selected Python identity')
    except (OSError,subprocess.CalledProcessError,ValueError) as exc:
        raise ValueError('selected numerical runtime preflight failed') from exc
    if {k:value[k] for k in ('python','numpy')}!=method_contract()['runtime'] or not isinstance(value['prefix'],str) or not Path(value['prefix']).is_absolute():
        raise ValueError('selected numerical runtime differs from pinned Python/NumPy')
    return {'executable':executable,**value}


def prepare_build_receipt(manifest_path,binary,root,evidence):
    from .followup_evidence import validate_build_evidence
    from .run import make_build_receipt
    validate_build_evidence(evidence)
    return make_build_receipt(manifest_path,binary,root,
        prebuild_inventory_sha256=evidence['source_inventory_sha256'],
        build_command=evidence['build_command'],build_exit_code=evidence['build_exit_code'],
        toolchain=evidence['toolchain'],lockfile_paths=['Cargo.lock','survey/Cargo.lock'],
        features=evidence['features'],build_flags=evidence['effective_build_flags'])


def benchmark_preparation(*,binary,source_root,source_path,build_evidence,output_dir,python):
    """Explicitly invoked full bounded preparation; never registered inference."""
    import json
    from . import provenance
    from .followup_registration import source_inventory
    from .run import sha256_file
    from .analysis import validate_output_destinations
    python_runtime=selected_python(python)
    root=Path(source_root).resolve();out=Path(output_dir).resolve()
    evidence=records.strict_json(Path(build_evidence).read_bytes())
    from .followup_evidence import validate_build_evidence,artifact,bind_measurement
    validate_build_evidence(evidence)
    inventory=source_inventory(root);inventory_sha=followup.sha(canonical_bytes(inventory));binary_sha=sha256_file(binary)
    if evidence['build_exit_code']!=0 or evidence['binary_sha256']!=binary_sha or evidence['source_inventory_sha256']!=inventory_sha:
        raise ValueError('benchmark needs exact successful unchanged source/binary build evidence')
    source_data=Path(source_path).read_bytes();table=records.strict_json(source_data)
    schedule_data=Path(__file__).with_name('followup-probe-schedule.json').read_bytes()
    # Reserve every destination before the first directory creation/native probe.
    outputs=[out/'benchmark-evidence.json',out/'build-evidence.json',out/'cargo-configuration.json']
    for phase in followup.PHASES:
        for i in range(8):outputs.extend(out/f'{phase}.probe{i}.{suffix}' for suffix in ('manifest.json','receipt.json','resolved.json','sessions.jsonl','stdout.log','stderr.log'))
        for kind in (*ANALYSIS_TYPES,'writer'):outputs.extend(out/f'{phase}.{kind}.{suffix}' for suffix in ('synthetic.json','stdout.log','stderr.log'))
    inputs=[binary,source_path,build_evidence,Path(evidence['cargo_configuration']['path']),*[root/e['path'] for e in inventory]]
    validate_output_destinations(inputs,outputs)
    if any(path.exists() or path.is_symlink() for path in outputs):raise ValueError('preparation evidence destinations must be fresh; probes cannot silently repeat')
    out.mkdir(parents=True,exist_ok=True)
    retained=__import__('copy').deepcopy(evidence)
    context_path=out/'cargo-configuration.json';context_path.write_bytes(Path(evidence['cargo_configuration']['path']).read_bytes())
    retained['cargo_configuration']={'path':str(context_path),'bytes':context_path.stat().st_size,'sha256':sha256_file(context_path)}
    retained_path=out/'build-evidence.json';retained_path.write_bytes(canonical_bytes(retained)+b'\n')
    result={'schema_version':1,'classification':'unregistered_runtime_preparation_only',
        'study_protocol':followup.PROTOCOL,'schedule_sha256':followup.sha(schedule_data),
        'source_inventory_sha256':inventory_sha,'binary_sha256':binary_sha,'source_root':str(root),
        'binary_path':str(Path(binary).resolve()),'python_runtime':python_runtime,
        'build_evidence':artifact(retained_path,out),'phases':{}}
    toolchain=evidence['toolchain']
    for phase in followup.PHASES:
        full=probe_manifest(table,source_data,phase);probes=[]
        for i,arm in enumerate(full['arms']):
            value=manifest.build_unregistered_manifest(table,source_data,configurations=[arm['config_overrides']],mode='runtime_probe')
            value['arms'][0]['first_seed']=arm['first_seed']
            value=provenance.bind_unregistered_manifest(value,root,source_data)
            value.update(source_inventory=inventory,source_inventory_sha256=inventory_sha)
            prefix=out/f'{phase}.probe{i}';mp=Path(str(prefix)+'.manifest.json');rp=Path(str(prefix)+'.receipt.json')
            mp.write_text(json.dumps(value,indent=2)+'\n')
            receipt=prepare_build_receipt(mp,binary,root,retained)
            rp.write_text(json.dumps(receipt,indent=2)+'\n')
            command=[str(Path(binary).resolve()),'--manifest',str(mp),'--receipt',str(rp),'--repo',str(root),
                '--resolved',str(prefix)+'.resolved.json','--out',str(prefix)+'.sessions.jsonl']
            measured=measure_command(command,str(prefix)+'.stdout.log',str(prefix)+'.stderr.log',
                source_inventory_sha256=inventory_sha,binary_sha256=binary_sha,toolchain=toolchain,cwd=root)
            resolved=records.strict_json(Path(str(prefix)+'.resolved.json').read_bytes())
            binding={k:resolved[k] for k in records.BINDING_FIELDS}
            rows=records.read_sessions(Path(str(prefix)+'.sessions.jsonl'),value,resolved,binding)
            if len(rows)!=1 or not records.history_metrics(next(iter(rows.values())))['complete']:
                raise ValueError('native runtime probe did not complete its full1000periods')
            measured=bind_measurement(measured,phase=phase,kind='native_probe',index=i,root=out,
                paths={role:Path(str(prefix)+'.'+suffix) for role,suffix in {
                    'manifest':'manifest.json','receipt':'receipt.json','resolved':'resolved.json',
                    'primary':'sessions.jsonl','stdout':'stdout.log','stderr':'stderr.log'}.items()})
            validate_measurement(measured)
            probes.append(measured)
        analysis={};writer=None
        for kind in (*ANALYSIS_TYPES,'writer'):
            prefix=out/f'{phase}.{kind}'
            command=[python_runtime['executable'],'-m','survey.democratic_peace.followup_runtime',
                '--synthetic-kind',kind,'--phase',phase,'--output',str(prefix)+'.synthetic.json']
            measured=measure_command(command,str(prefix)+'.stdout.log',str(prefix)+'.stderr.log',
                source_inventory_sha256=inventory_sha,binary_sha256=binary_sha,toolchain=toolchain,cwd=root)
            measured=bind_measurement(measured,phase=phase,kind=kind,index=None,root=out,
                paths={role:Path(str(prefix)+'.'+suffix) for role,suffix in {
                    'primary':'synthetic.json','stdout':'stdout.log','stderr':'stderr.log'}.items()})
            validate_measurement(measured)
            if kind=='writer':writer=measured
            else:analysis[kind]=measured
        result['phases'][phase]={'native_probes':probes,'analysis':analysis,'writer':writer,
            'forecast':phase_forecast(probes,analysis,writer)}
    result['combined_cpu_seconds']=sum(p['forecast']['cpu_seconds'] for p in result['phases'].values())
    result['within_budget']=all(p['forecast']['within_budget'] for p in result['phases'].values()) and result['combined_cpu_seconds']<=86400
    if source_inventory(root)!=inventory:raise ValueError('normative sources changed during preparation')
    validate_retained_evidence(result,out)
    (out/'benchmark-evidence.json').write_text(json.dumps(result,indent=2,allow_nan=False)+'\n')
    return result



def runtime_attestation(declaration_path,benchmark_path):
    """Attest only complete retained benchmark evidence, after declaration bytes exist."""
    from .followup_registration import validate_declaration
    dbytes=Path(declaration_path).read_bytes();declaration=records.strict_json(dbytes);validate_declaration(declaration)
    bbytes=Path(benchmark_path).read_bytes();evidence=records.strict_json(bbytes)
    records._fields(evidence,{'schema_version','classification','study_protocol','schedule_sha256',
        'source_inventory_sha256','binary_sha256','source_root','binary_path','python_runtime','build_evidence',
        'phases','combined_cpu_seconds','within_budget'},'benchmark evidence')
    if type(evidence['schema_version']) is not int or evidence['schema_version']!=1 or evidence['classification']!='unregistered_runtime_preparation_only' or evidence['study_protocol']!=followup.PROTOCOL or evidence['schedule_sha256']!=followup.sha(Path(__file__).with_name('followup-probe-schedule.json').read_bytes()) or set(evidence['phases'])!=set(followup.PHASES) or evidence['within_budget'] is not True or followup.sha(bbytes)!=declaration['benchmark_evidence_sha256']:
        raise ValueError('complete frozen benchmark evidence required')
    validate_retained_evidence(evidence,Path(benchmark_path).resolve().parent)
    forecasts={}
    for declared in declaration['phases']:
        phase=declared['phase'];payload=evidence['phases'][phase]
        records._fields(payload,{'native_probes','analysis','writer','forecast'},'phase benchmark evidence')
        for key in ('source_inventory_sha256','binary_sha256'):
            if evidence[key]!=declared[key]:raise ValueError('benchmark/declaration source or binary mismatch')
        measurements=[*payload['native_probes'],*payload['analysis'].values(),payload['writer']]
        for measurement in measurements:
            validate_measurement(measurement)
            if any(measurement[k]!=evidence[k] for k in ('source_inventory_sha256','binary_sha256')):raise ValueError('mixed benchmark identities')
        forecast=phase_forecast(payload['native_probes'],payload['analysis'],payload['writer'])
        if not records._same_json_types(forecast,payload['forecast']):raise ValueError('retained benchmark forecast differs from recomputation')
        validate_forecast(forecast);forecasts[phase]=forecast
    total=sum(f['cpu_seconds'] for f in forecasts.values())
    if total!=evidence['combined_cpu_seconds'] or total>86400:raise ValueError('benchmark total fails24CPUhour gate')
    receipt={'schema_version':1,'study_protocol':followup.PROTOCOL,'status':'approved_complete_phase_forecast',
        'declaration_sha256':followup.sha(dbytes),'benchmark_evidence_sha256':followup.sha(bbytes),
        'phases':forecasts,'combined_cpu_seconds':total}
    validate_runtime_receipt(receipt);return receipt


def validate_retained_evidence(evidence,root):
    from .followup_evidence import validate_retained_evidence as validate
    return validate(evidence,root)


if __name__=='__main__':main()
