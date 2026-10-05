"""Read-only retained workload/build artifacts, separate from forecast arithmetic."""
from pathlib import Path
import shlex
from . import followup, records
from .methods import canonical_bytes
from .provenance import safe_source_path
from .run import sha256_file

BUILD_FIELDS = {'build_command','build_exit_code','binary_sha256','source_inventory_sha256',
    'toolchain','effective_build_flags','features','cargo_configuration'}
TOOLCHAIN_FIELDS = {'target','rustc_version','cargo_version'}
ARTIFACT_FIELDS = {'path','bytes','sha256'}


def strings(value, label, *, nonempty=False):
    if not isinstance(value,list) or (nonempty and not value) or any(not isinstance(x,str) or not x for x in value):
        raise ValueError(f'invalid {label}')


def validate_toolchain(value):
    records._fields(value,TOOLCHAIN_FIELDS,'preparation toolchain')
    if any(not isinstance(v,str) or not v.strip() for v in value.values()):
        raise ValueError('complete native toolchain required')


def hash_value(value, label):
    if not isinstance(value,str) or len(value)!=64 or any(c not in '0123456789abcdef' for c in value):
        raise ValueError(f'invalid {label} SHA256')


def read_absolute_artifact(entry):
    records._fields(entry,ARTIFACT_FIELDS,'explicit context artifact')
    if not isinstance(entry['path'],str) or not entry['path']:
        raise ValueError('invalid build context artifact path')
    path=Path(entry['path'])
    if not path.is_absolute() or '..' in path.parts or path.is_symlink() or not path.is_file():
        raise ValueError('unsafe or missing build context artifact')
    data=path.read_bytes()
    if type(entry['bytes']) is not int or len(data)!=entry['bytes'] or followup.sha(data)!=entry['sha256']:
        raise ValueError('build context artifact identity mismatch')
    return data


def validate_build_evidence(value):
    records._fields(value,BUILD_FIELDS,'preparation build evidence')
    if type(value['build_exit_code']) is not int or value['build_exit_code']!=0:
        raise ValueError('successful preparation build required')
    strings(value['build_command'],'build command',nonempty=True)
    command=value['build_command']
    if Path(command[0]).name!='cargo' or len(command)<2 or command[1]!='build' or '--locked' not in command:
        raise ValueError('unrelated preparation build command')
    for flag,argument in (('--manifest-path','survey/Cargo.toml'),('--bin','democratic_peace')):
        if flag not in command or command.index(flag)+1>=len(command) or command[command.index(flag)+1]!=argument:
            raise ValueError('build command does not select declared native recorder')
    strings(value['effective_build_flags'],'effective build flags')
    strings(value['features'],'effective features')
    if len(value['features'])!=len(set(value['features'])):
        raise ValueError('duplicate effective features')
    validate_toolchain(value['toolchain'])
    for key in ('binary_sha256','source_inventory_sha256'):hash_value(value[key],key)
    data=read_absolute_artifact(value['cargo_configuration'])
    context=records.strict_json(data)
    records._fields(context,{'config_inputs','environment'},'effective Cargo/environment configuration')
    if not isinstance(context['config_inputs'],list) or not isinstance(context['environment'],dict):
        raise ValueError('malformed Cargo/environment build context')
    names=set();configured_flags=[];target_flags=False
    for entry in context['config_inputs']:
        records._fields(entry,{'path','sha256','parsed'},'captured Cargo configuration input')
        if not isinstance(entry['path'],str) or not entry['path']:
            raise ValueError('invalid captured Cargo configuration path')
        path=Path(entry['path'])
        if not path.is_absolute() or '..' in path.parts or entry['path'] in names or not isinstance(entry['parsed'],dict):
            raise ValueError('invalid captured Cargo configuration input')
        hash_value(entry['sha256'],'Cargo configuration input');names.add(entry['path'])
        # Retained parsed inputs are auditable even if a later host changes.
        build=entry['parsed'].get('build',{})
        targets=entry['parsed'].get('target',{})
        if not isinstance(build,dict) or not isinstance(targets,dict) or any(not isinstance(t,dict) for t in targets.values()):
            raise ValueError('malformed captured Cargo build/target configuration')
        flags=build.get('rustflags',[])
        if isinstance(flags,str):flags=shlex.split(flags)
        strings(flags,'captured configuration rustflags');configured_flags.extend(flags)
        target_flags=target_flags or any('rustflags' in t for t in targets.values())
    if any(not isinstance(k,str) or not k or not isinstance(v,str) for k,v in context['environment'].items()):
        raise ValueError('malformed captured compiler environment')
    # Cargo's direct compiler environment overrides configuration rustflags.
    env=context['environment']
    if 'CARGO_ENCODED_RUSTFLAGS' in env:
        expected=env['CARGO_ENCODED_RUSTFLAGS'].split('\x1f') if env['CARGO_ENCODED_RUSTFLAGS'] else []
        if expected!=value['effective_build_flags']:raise ValueError('effective encoded compiler flags differ from context')
    elif 'RUSTFLAGS' in env:
        if shlex.split(env['RUSTFLAGS'])!=value['effective_build_flags']:
            raise ValueError('effective compiler flags differ from captured environment')
    elif not target_flags and '--config' not in command:
        # Inputs are captured in low-to-high precedence order, as Cargo merges
        # configuration arrays. Target/cfg overrides remain explicitly auditable.
        expected=shlex.split(env['CARGO_BUILD_RUSTFLAGS']) if 'CARGO_BUILD_RUSTFLAGS' in env else configured_flags
        if expected!=value['effective_build_flags']:
            raise ValueError('effective compiler flags differ from captured configuration')
    return value


def artifact(path, root):
    path=Path(path);root=Path(root).resolve()
    try:name=(path.parent.resolve()/path.name).relative_to(root).as_posix()
    except ValueError as exc:raise ValueError('retained artifact must be inside benchmark directory') from exc
    data=safe_source_path(root,name).read_bytes()
    return {'path':name,'bytes':len(data),'sha256':followup.sha(data)}


def artifact_data(entry, root):
    records._fields(entry,ARTIFACT_FIELDS,'retained workload artifact')
    data=safe_source_path(root,entry['path']).read_bytes()
    if type(entry['bytes']) is not int or len(data)!=entry['bytes'] or followup.sha(data)!=entry['sha256']:
        raise ValueError('retained workload artifact identity mismatch')
    return data


def bind_measurement(measurement, *, phase, kind, index, paths, root):
    value=dict(measurement)
    value['workload']={'phase':phase,'kind':kind,'probe_index':index}
    value['artifacts']={role:artifact(path,root) for role,path in paths.items()}
    value['output_sha256']=value['artifacts']['primary']['sha256']
    return value


def synthetic_cases():
    return [('varied',[i/99 for i in range(100)]),('ties',[.5]*100),
        ('complete_extinction',[None]*100),('sparse_survivors',[1.,2.]+[None]*98)]


def writer_payload(phase):
    from .source import slot_roster
    return {**synthetic_header('writer',phase),'classification':'synthetic_writer_timing_only',
        'histories':[{'arm':f'synthetic.{i//100}','seed':i,'status':'completed','complete':True,
            'metrics':dict.fromkeys(records.METRIC_FIELDS,.5),'census':dict.fromkeys(records.COUNTER_FIELDS,0)} for i in range(10800)],
        'targets':[{'id':s['id'],'result':{'defined_draws':100000,'predictive_interval':[.1,.9]},'p':.5,'holm_p':1.} for s in slot_roster()]}


def synthetic_header(kind, phase):
    from .methods import method_contract
    return {'schema_version':1,'study_protocol':followup.PROTOCOL,'kind':kind,'phase':phase,
        'runtime':method_contract()['runtime'],'analysis_jobs_sha256':followup.sha(canonical_bytes(followup.analysis_jobs(phase)))}


def interval(value,label):
    if not isinstance(value,list) or len(value)!=2 or any(not records._number(v) for v in value) or value[0]>value[1]:
        raise ValueError(f'invalid {label} interval')


def validate_synthetic_payload(payload,kind,phase):
    header=synthetic_header(kind,phase)
    if any(not records._same_json_types(payload.get(k),v) for k,v in header.items()):
        raise ValueError('synthetic phase/kind/runtime/job identity mismatch')
    common=set(header)|{'classification'}
    if kind=='writer':
        if not records._same_json_types(payload,writer_payload(phase)):
            raise ValueError('synthetic full-phase writer workload changed')
        return
    if payload['classification']!='synthetic_analysis_timing_only':raise ValueError('wrong synthetic analysis classification')
    if kind=='predictive':
        records._fields(payload,common|{'draws','workloads'},'predictive timing workload')
        if type(payload['draws']) is not int or payload['draws']!=100000 or not isinstance(payload['workloads'],list) or len(payload['workloads'])!=4:
            raise ValueError('four complete100000draw predictive cases required')
        for case,(name,values) in zip(payload['workloads'],synthetic_cases()):
            records._fields(case,{'case','input_values_sha256','job_index','result'},'synthetic predictive case')
            if case['case']!=name or case['input_values_sha256']!=followup.sha(canonical_bytes(values)) or type(case['job_index']) is not int or case['job_index']!=0:
                raise ValueError('synthetic predictive case/input/job mismatch')
            result=case['result'];surviving=sum(v is not None for v in values)
            base={'status','reason','requested_draws','attempted_draws','defined_draws','undefined_draws',
                'surviving_precision_histories','predictive_interval','conditional','undefined_fraction'}
            available=surviving>0
            extra={'p','maximizer_kind','maximizer','tested_point_count','tested_open_interval_count'} if available else set()
            records._fields(result,base|extra,'retained predictive result')
            for key in ('requested_draws','attempted_draws'):
                if type(result[key]) is not int or result[key]!=100000:raise ValueError('wrong predictive draw count')
            defined=result['defined_draws'];undefined=result['undefined_draws']
            if not records._uint(defined) or not records._uint(undefined) or defined+undefined!=100000 or type(result['surviving_precision_histories']) is not int or result['surviving_precision_histories']!=surviving or type(result['conditional']) is not bool or result['conditional']!=(surviving!=100) or not records._number(result['undefined_fraction']) or result['undefined_fraction']!=undefined/100000:
                raise ValueError('predictive availability/draw census mismatch')
            if surviving==100 and defined!=100000:raise ValueError('complete synthetic reference contains undefined draws')
            if available:
                if defined<10000 or result['status']!='Available' or result['reason'] is not None:raise ValueError('incomplete synthetic predictive reference')
                interval(result['predictive_interval'],'predictive')
                finite=[v for v in values if v is not None]
                if not min(finite)<=result['predictive_interval'][0]<=result['predictive_interval'][1]<=max(finite):raise ValueError('predictive interval outside declared synthetic support')
                if not records._number(result['p']) or not 0<=result['p']<=1 or not records._uint(result['tested_point_count']) or result['tested_point_count']<2 or not records._uint(result['tested_open_interval_count']) or result['tested_open_interval_count']!=result['tested_point_count']-1:
                    raise ValueError('missing interval-maximization workload result')
                if result['maximizer_kind']=='point':
                    if not records._number(result['maximizer']) or not 0<=result['maximizer']<=1:raise ValueError('invalid interval maximizer')
                elif result['maximizer_kind']=='open_interval':
                    interval(result['maximizer'],'maximizer')
                    if not 0<=result['maximizer'][0]<result['maximizer'][1]<=1:raise ValueError('invalid open interval maximizer')
                else:raise ValueError('unknown interval maximizer')
            elif defined!=0 or result['status']!='Unavailable' or result['reason']!='insufficient_conditional_reference' or result['predictive_interval'] is not None:
                raise ValueError('complete-extinction timing case differs from declared workload')
    elif kind=='contrast_pair':
        records._fields(payload,common|{'draws_per_job','analysis_jobs','job_indices','input_values_sha256','result'},'contrast-pair timing workload')
        if type(payload['draws_per_job']) is not int or payload['draws_per_job']!=100000 or type(payload['analysis_jobs']) is not int or payload['analysis_jobs']!=2 or not records._same_json_types(payload['job_indices'],[105,106]) or payload['input_values_sha256']!=followup.sha(canonical_bytes([[0.]*50+[1.]*50,[.25]*100])):
            raise ValueError('declared contrast-pair inputs/jobs/draws changed')
        result=payload['result']
        records._fields(result,{'estimate','p','permutation_exceedances','permutations','bootstrap_draws','interval','resampling'},'retained contrast-pair result')
        if result['resampling']!='independent_whole_history':
            raise ValueError('contrast-pair resampling contract mismatch')
        if not records._number(result['estimate']) or result['estimate']!=.25 or any(type(result[k]) is not int or result[k]!=100000 for k in ('permutations','bootstrap_draws')) or not records._uint(result['permutation_exceedances']) or result['permutation_exceedances']>100000 or not records._number(result['p']) or result['p']!=(1+result['permutation_exceedances'])/100001:
            raise ValueError('contrast-pair result/draw arithmetic mismatch')
        interval(result['interval'],'contrast bootstrap')
    else:raise ValueError('unknown synthetic workload kind')


def retained_workload(measurement,evidence,root,phase,kind,index,build):
    from .followup_runtime import MEASUREMENT_FIELDS,validate_measurement
    records._fields(measurement,MEASUREMENT_FIELDS|{'workload','artifacts'},'bound runtime measurement')
    validate_measurement(measurement)
    expected={'phase':phase,'kind':kind,'probe_index':index}
    if not records._same_json_types(measurement['workload'],expected):raise ValueError('duplicate/substituted declared workload identity')
    if not records._same_json_types(measurement['toolchain'],build['toolchain']) or any(measurement[k]!=evidence[k] for k in ('source_inventory_sha256','binary_sha256')):
        raise ValueError('mixed workload toolchain/source/binary identities')
    roles={'primary','stdout','stderr'}|({'manifest','receipt','resolved'} if kind=='native_probe' else set())
    records._fields(measurement['artifacts'],roles,'workload artifacts')
    prefix=f'{phase}.probe{index}' if kind=='native_probe' else f'{phase}.{kind}'
    suffixes={'manifest':'manifest.json','receipt':'receipt.json','resolved':'resolved.json','primary':'sessions.jsonl' if kind=='native_probe' else 'synthetic.json','stdout':'stdout.log','stderr':'stderr.log'}
    data={};paths={}
    for role,entry in measurement['artifacts'].items():
        if entry.get('path')!=prefix+'.'+suffixes[role]:raise ValueError('workload artifact path substitution')
        data[role]=artifact_data(entry,root);paths[role]=safe_source_path(root,entry['path'])
    if measurement['output_sha256']!=measurement['artifacts']['primary']['sha256']:
        raise ValueError('primary workload output hash differs from summary')
    if kind=='native_probe':
        expected_command=[evidence['binary_path'],'--manifest',str(paths['manifest']),'--receipt',str(paths['receipt']),
            '--repo',evidence['source_root'],'--resolved',str(paths['resolved']),'--out',str(paths['primary'])]
    else:
        expected_command=[evidence['python_runtime']['executable'],'-m','survey.democratic_peace.followup_runtime',
            '--synthetic-kind',kind,'--phase',phase,'--output',str(paths['primary'])]
    if measurement['command']!=expected_command:raise ValueError('unrelated or substituted workload command')
    return data,paths


def validate_native_probe(data,paths,evidence,build,phase,index):
    from . import manifest,provenance
    from .followup_runtime import probe_manifest
    from .followup_registration import verify_inventory
    source_root=Path(evidence['source_root']);source_data=(source_root/provenance.TABLE).read_bytes();table=records.strict_json(source_data)
    if followup.sha(source_data)!=followup.SOURCE_TABLE_SHA256:raise ValueError('probe source table identity changed')
    expected_arm=probe_manifest(table,source_data,phase)['arms'][index]
    value=records.strict_json(data['manifest']);manifest.expected_keys(value)
    expected=manifest.build_unregistered_manifest(table,source_data,configurations=[expected_arm['config_overrides']],mode='runtime_probe')
    expected['arms'][0]['first_seed']=expected_arm['first_seed']
    expected.update(source_inventory=value['source_inventory'],source_inventory_sha256=evidence['source_inventory_sha256'])
    if not records._same_json_types(value,expected):raise ValueError('probe index/seed/coordinates/reading/defaults changed')
    verify_inventory(source_root,value)
    receipt=records.strict_json(data['receipt'])
    binding={'manifest_sha256':followup.sha(data['manifest']),'binary_sha256':evidence['binary_sha256'],
        'source_inventory_sha256':evidence['source_inventory_sha256'],'build_receipt_sha256':followup.sha(data['receipt'])}
    records.validate_build_receipt(receipt,binding['manifest_sha256'],binding['binary_sha256'],binding['source_inventory_sha256'],source_root)
    for key,expected in {'build_command':build['build_command'],'features':build['features'],'build_flags':build['effective_build_flags'],
        'cwd':evidence['source_root'],**build['toolchain']}.items():
        if not records._same_json_types(receipt[key],expected):raise ValueError('native probe receipt lost build context')
    resolved=records.strict_json(data['resolved'])
    records._fields(resolved,{*records.BINDING_FIELDS,'schema_version','model','arms','resolved_configs_json'},'probe resolved export')
    records.resolved_payload(resolved)
    base=records.strict_json(Path(followup.__file__).with_name('followup-default-config.json').read_bytes())
    configs=[{'id':value['arms'][0]['id'],'config':{**base,**expected_arm['config_overrides']}}]
    if type(resolved['schema_version']) is not int or resolved['schema_version']!=1 or resolved['model']!='democratic_peace' or not records._same_json_types(resolved['arms'],configs):
        raise ValueError('probe resolved defaults/reading/config changed')
    binding['resolved_configs_sha256']=resolved['resolved_configs_sha256']
    rows=records.read_sessions(paths['primary'],value,resolved,binding)
    if len(rows)!=1 or not records.history_metrics(next(iter(rows.values())))['complete']:
        raise ValueError('native probe must contain one complete full1000period history')
    if data['stdout']!=b'recorded 1 attempted histories\n' or data['stderr']!=b'':
        raise ValueError('native probe completion log mismatch')


def validate_retained_evidence(evidence,root):
    """Reread every bound primary artifact before approving any forecast."""
    from .followup_runtime import ANALYSIS_TYPES
    root=Path(root).resolve()
    for name in ('source_root','binary_path'):
        if not isinstance(evidence[name],str) or not Path(evidence[name]).is_absolute() or '..' in Path(evidence[name]).parts:
            raise ValueError('explicit absolute preparation authority paths required')
    if sha256_file(evidence['binary_path'])!=evidence['binary_sha256']:raise ValueError('preparation native binary changed')
    runtime=evidence['python_runtime']
    records._fields(runtime,{'executable','python','numpy','prefix'},'selected numerical runtime evidence')
    from .methods import method_contract
    if {k:runtime[k] for k in ('python','numpy')}!=method_contract()['runtime'] or any(not isinstance(runtime[k],str) or not Path(runtime[k]).is_absolute() for k in ('executable','prefix')):
        raise ValueError('invalid selected numerical runtime evidence')
    build=records.strict_json(artifact_data(evidence['build_evidence'],root));validate_build_evidence(build)
    if any(build[k]!=evidence[k] for k in ('source_inventory_sha256','binary_sha256')):raise ValueError('retained build identity differs from preparation')
    if set(evidence['phases'])!=set(followup.PHASES):raise ValueError('both complete preparation phases required')
    seen_files=[]
    for phase,payload in evidence['phases'].items():
        records._fields(payload,{'native_probes','analysis','writer','forecast'},'phase benchmark evidence')
        if not isinstance(payload['native_probes'],list) or len(payload['native_probes'])!=8 or set(payload['analysis'])!=set(ANALYSIS_TYPES):
            raise ValueError('eight distinct probes and complete synthetic job kinds required')
        workloads=[(m,'native_probe',i) for i,m in enumerate(payload['native_probes'])]
        workloads.extend((payload['analysis'][kind],kind,None) for kind in ANALYSIS_TYPES)
        workloads.append((payload['writer'],'writer',None))
        for measurement,kind,index in workloads:
            data,paths=retained_workload(measurement,evidence,root,phase,kind,index,build)
            for path in paths.values():
                if any(path.samefile(other) for other in seen_files):raise ValueError('retained workload artifacts alias another workload')
                seen_files.append(path)
            if kind=='native_probe':validate_native_probe(data,paths,evidence,build,phase,index)
            else:
                validate_synthetic_payload(records.strict_json(data['primary']),kind,phase)
                logged=records.strict_json(data['stdout'])
                records._fields(logged,{'classification','cpu_seconds','wall_seconds','peak_rss_bytes','output_sha256','kind','phase','runtime'},'synthetic child completion log')
                if logged['classification']!='synthetic_analysis_timing_only' or logged['kind']!=kind or logged['phase']!=phase or logged['runtime']!=method_contract()['runtime'] or logged['output_sha256']!=measurement['output_sha256'] or data['stderr']!=b'':
                    raise ValueError('synthetic completion log workload/output mismatch')
                if any(not records._number(logged[k]) or logged[k]<=0 for k in ('cpu_seconds','wall_seconds')) or type(logged['peak_rss_bytes']) is not int or logged['peak_rss_bytes']<=0:
                    raise ValueError('invalid synthetic child resource completion log')
    return build
