"""Strict evidence integrity and successful-period censuses; no engine defaults."""
import hashlib
import json
import math
from pathlib import Path
from .methods import canonical_bytes


def strict_json(text):
    def pairs(items):
        result={}
        for key,value in items:
            if key in result:raise ValueError(f'duplicate JSON field: {key}')
            result[key]=value
        return result
    def constant(value):raise ValueError(f'nonfinite JSON number: {value}')
    def parsed_float(value):
        result=float(value)
        if not math.isfinite(result):raise ValueError(f'nonfinite JSON numeric exponent: {value}')
        return result
    try:return json.loads(text,object_pairs_hook=pairs,parse_constant=constant,parse_float=parsed_float)
    except (json.JSONDecodeError,UnicodeDecodeError) as exc:raise ValueError(f'invalid JSON: {exc}') from exc


def resolved_payload(export):
    payload=export.get('resolved_configs_json')
    if not isinstance(payload,str):raise ValueError('missing exact native config payload')
    if hashlib.sha256(payload.encode('utf-8')).hexdigest()!=export.get('resolved_configs_sha256'):
        raise ValueError('native resolved config payload SHA256 mismatch')
    parsed=strict_json(payload)
    if parsed!=[{'id':a['id'],'config':a['config']} for a in export['arms']]:
        raise ValueError('native resolved payload/config equality mismatch')
    return parsed


def verify_source_inventory(root,inventory):
    from .provenance import required_inventory_paths,safe_source_path
    root=Path(root).resolve();seen=set();previous=None
    if not isinstance(inventory,list) or not inventory:raise ValueError('source inventory must be nonempty and sorted')
    for entry in inventory:
        if set(entry)!= {'path','bytes','sha256'}:raise ValueError('unknown source inventory field')
        name=entry['path']
        if previous is not None and previous>=name:raise ValueError('source inventory must be unique and path sorted')
        previous=name;seen.add(name);file=safe_source_path(root,name)
        if type(entry['bytes']) is not int or entry['bytes']<0:raise ValueError('invalid inventory byte count')
        data=file.read_bytes()
        if len(data)!=entry['bytes'] or hashlib.sha256(data).hexdigest()!=entry['sha256']:
            raise ValueError(f'source bytes/SHA256 mismatch: {name}')
    if not set(required_inventory_paths(root))<=seen:raise ValueError('source inventory omits required scientific sources')
    return hashlib.sha256(canonical_bytes(inventory)).hexdigest()


def _uint(value):return type(value) is int and 0<=value<=2**64-1

def _finite_nonnegative(value):
    return type(value) in (int,float) and math.isfinite(value) and value>=0


def history_availability(row,arm=None):
    attempt=row.get('attempt',{});status=attempt.get('status');out=row.get('outcome')
    if status in ('construction_panic','implementation_panic') and (not isinstance(attempt.get('panic_context'),str) or not attempt['panic_context'].strip()):
        raise ValueError('panic attempt needs nonempty panic_context')
    if out is None:
        if status not in ('construction_error','construction_panic','incomplete'):
            raise ValueError('missing Outcome without explicit unavailable attempt status')
        if status=='incomplete' and (not isinstance(attempt.get('recorder_error'),str) or not attempt['recorder_error'].strip()):
            raise ValueError('incomplete attempt needs nonempty recorder_error')
        return {'status':status,'complete':False,'state_available':False,'reason':attempt.get('recorder_error') or attempt.get('panic_context') or attempt.get('construction_errors')}
    config=row.get('config',out['config'])
    if out['config']!=config or ('seed' in row and out['seed']!=row['seed']):raise ValueError('Outcome config/seed mismatch')
    periods=out['periods'];attempted=out['attempted_period'];horizon=config['initialization_periods']+config['observation_periods']
    if not _uint(periods) or not _uint(attempted) or not periods<=attempted<=min(horizon,periods+1):
        raise ValueError('invalid completed/attempted clocks')
    if type(out['valid']) is not bool or type(out['state_available']) is not bool:raise ValueError('invalid availability flags')
    if out['finish_reason']=='implementation_panic' and status not in (None,'implementation_panic'):
        raise ValueError('inner panic requires outer panic attempt')
    if out['valid']:
        if periods!=horizon or attempted!=horizon or not out['state_available'] or out['finish_reason']!='horizon' or out['invalid_reason'] is not None:
            raise ValueError('valid outcome is not a complete registered horizon')
        if status=='implementation_panic':
            return {'status':'implementation_panic','complete':False,'state_available':True,'reason':attempt['panic_context']}
        if status not in (None,'completed'):raise ValueError('completed Outcome/attempt status mismatch')
        return {'status':'completed','complete':True,'state_available':True,'reason':None}
    if status not in (None,'invalid','implementation_panic'):raise ValueError('invalid Outcome/attempt status mismatch')
    return {'status':'implementation_panic' if status=='implementation_panic' else 'invalid_partial','complete':False,
            'state_available':out['state_available'],'reason':attempt.get('panic_context') if status=='implementation_panic' else out['invalid_reason']}


def completed_censuses(outcome,availability=None):
    intrinsic=history_availability({'outcome':outcome})
    availability=intrinsic if availability is None else availability
    if availability['complete'] and not intrinsic['complete']:raise ValueError('attempt cannot promote an invalid Outcome')
    c=outcome['config'];successful=outcome['periods'];attempted=outcome['attempted_period'];boundary=outcome['counting_start']
    expected_boundary=max(1,c['initialization_periods']) if c['count_boundary']=='at_initialization' else c['initialization_periods']+1
    if boundary!=expected_boundary:raise ValueError('counting boundary differs from resolved config')
    seen=set();accepted=[];excluded=[]
    for war in outcome['completed_wars']:
        wid=war['id']
        if not _uint(wid) or wid in seen:raise ValueError('invalid or duplicate completed war id')
        seen.add(wid);reasons=[];start=war['start_period'];end=war['end_period']
        clocks=_uint(start) and _uint(end) and start<=end<=attempted
        if not clocks:reasons.append('invalid_completion_clock')
        elif end>successful:reasons.append('invalid_partial_period_completion')
        if not _uint(start) or start<boundary:reasons.append('before_counting_start')
        raw=war['raw_severity'];exported=war['exported_severity']
        if not _finite_nonnegative(raw):reasons.append('invalid_raw_severity')
        if not _finite_nonnegative(exported):reasons.append('invalid_exported_severity')
        elif _finite_nonnegative(raw):
            expected=raw if c['severity_export']=='raw_damage' else float(min(2147483647,math.trunc(raw*100))) if raw<=2147483647/100 else 2147483647.
            if exported!=expected:reasons.append('resolved_export_mismatch')
        if not war.get('end_cause'):reasons.append('missing_end_cause')
        participants=set()
        for p in war['participants']:
            state=p['state'];identity=(state['capital_cell'],state['sovereignty_generation'])
            if not all(_uint(v) for v in identity) or identity[0]>=c['width']*c['height']:
                reasons.append('invalid_generation_identity')
            if identity in participants:reasons.append('duplicate_participant_generation_identity')
            participants.add(identity)
            if not _uint(p['last_fighting_period']) or not clocks or not start<=p['last_fighting_period']<=end:
                reasons.append('invalid_participant_clock')
        fights=war['fighting_periods']
        if not isinstance(fights,list) or any(not _uint(p) for p in fights) or fights!=sorted(set(fights)):
            reasons.append('invalid_fighting_periods')
        elif not clocks or any(not start<=p<=end for p in fights):reasons.append('invalid_fighting_periods')
        elif war['active_periods']!=len(fights) or (fights and war['last_active_period']!=fights[-1]):
            reasons.append('invalid_active_period_clock')
        if clocks and war['elapsed_periods']!=end-start+1:reasons.append('invalid_elapsed_period_clock')
        if reasons:excluded.append({'war':war,'reasons':sorted(set(reasons))})
        else:accepted.append(war)
    by_id={w['id']:w for w in outcome['completed_wars']};visible_ids=set();backlog_ids=set()
    for label,ids in [('legacy_visible_wars',visible_ids),('exporter_backlog',backlog_ids)]:
        for war in outcome[label]:
            if war['id'] in ids or by_id.get(war['id'])!=war:raise ValueError('export census duplicate or mismatched completed identity')
            ids.add(war['id'])
    if visible_ids&backlog_ids:raise ValueError('exporter backlog overlaps visible census')
    positive_ids={w['id'] for w in outcome['completed_wars'] if _finite_nonnegative(w['raw_severity']) and w['raw_severity']>0}
    queued_ids=visible_ids|backlog_ids
    # Null raw severity can represent a genuine nonfinite partial artifact; preserve
    # that unknown membership only on invalid rows, with its fit exclusion reason.
    unknown_ids={w['id'] for w in outcome['completed_wars'] if w['raw_severity'] is None and not intrinsic['complete']}
    if queued_ids-unknown_ids!=positive_ids:raise ValueError('positive raw collector partition is not conserved')
    if c['completed_export']=='all_completed' and backlog_ids:raise ValueError('all_completed retains exporter backlog')
    censored_ids=set()
    for war in outcome['censored_wars']:
        wid=war['id'];start=war['start_period'];last=war['last_active_period']
        if not _uint(wid) or wid in seen or wid in censored_ids:raise ValueError('duplicate or completed/censored war identity')
        censored_ids.add(wid)
        if war['end_period'] is not None or war['end_cause']!='horizon_censored' or not _uint(start) or not boundary<=start<=last<=attempted:
            raise ValueError('invalid censored war clocks/cause')
        identities=set()
        for part in war['participants']:
            identity=(part['state']['capital_cell'],part['state']['sovereignty_generation'])
            if not all(_uint(v) for v in identity) or identity[0]>=c['width']*c['height'] or identity in identities:
                raise ValueError('invalid censored participant generation identity')
            identities.add(identity)
            if not _uint(part['last_fighting_period']) or not start<=part['last_fighting_period']<=attempted:
                raise ValueError('invalid censored participant clock')
        fights=war['fighting_periods']
        if not isinstance(fights,list) or fights!=sorted(set(fights)) or any(not _uint(v) or not start<=v<=attempted for v in fights):
            raise ValueError('invalid censored fighting periods')
        if war['active_periods']!=len(fights) or (fights and last!=fights[-1]) or war['elapsed_periods']!=attempted-start+1:
            raise ValueError('invalid censored active/elapsed clocks')
        if intrinsic['complete'] and not _finite_nonnegative(war['raw_severity']):
            raise ValueError('invalid censored raw severity')
    if intrinsic['complete'] and excluded:raise ValueError('valid horizon contains invalid completed census')
    selected=accepted if c['completed_export']=='all_completed' else [w for w in accepted if w['id'] in visible_ids]
    return {'availability':availability,'primary_completed':accepted if availability['complete'] else [],
            'partial_completed':accepted if not availability['complete'] else [],
            'source_selected':selected if availability['complete'] else [],
            'partial_source_selected':selected if not availability['complete'] else [],
            'excluded':excluded,'completed_record_count':len(outcome['completed_wars']),
            'censored_count':len(outcome['censored_wars']),'backlog_count':len(outcome['exporter_backlog']),
            'export_diagnostics':{label:export_diagnostics(wars) for label,wars in {
                'complete_all':accepted if availability['complete'] else [],
                'complete_selected':selected if availability['complete'] else [],
                'partial_all':accepted if not availability['complete'] else [],
                'partial_selected':selected if not availability['complete'] else [],
                'completed_records':outcome['completed_wars'],'censored':outcome['censored_wars'],
                'backlog':outcome['exporter_backlog'],'legacy_visible':outcome['legacy_visible_wars']}.items()}}

def export_diagnostics(wars):
    known=[w['raw_severity'] for w in wars if _finite_nonnegative(w['raw_severity'])]
    return {'record_count':len(wars),'emitted_java_saturated':sum(w['java_saturated'] for w in wars),
            'emitted_java_subunit_zero':sum(w['java_subunit_zero'] for w in wars),
            'integer100_saturated':sum(raw>2147483647/100 for raw in known),
            'integer100_subunit_zero':sum(raw>0 and java_int100(raw)==0 for raw in known),
            'integer100_unavailable':len(wars)-len(known)}

BINDING_FIELDS=('manifest_sha256','binary_sha256','source_inventory_sha256','build_receipt_sha256','resolved_configs_sha256')
RECORD_FIELDS={'schema_version','model','arm','seed','arm_index','repeat_index','config','attempt','outcome',*BINDING_FIELDS}
OUTCOME_FIELDS={'config','seed','rng_mode','periods','attempted_period','counting_start','valid','state_available','finish_reason','invalid_reason','completed_wars','censored_wars','legacy_visible_wars','exporter_backlog','merges','retired_states','sovereign_count','states','cells','ledger','fronts','resource_updates','partial_period_fights'}


def validate_build_receipt(receipt,manifest_sha256,binary_sha256,source_inventory_sha256,source_root=None):
    required={'schema_version','model','manifest_sha256','source_inventory_sha256','prebuild_inventory_sha256',
              'postbuild_inventory_sha256','binary_sha256','build_command','cwd','target','rustc_version',
              'cargo_version','lockfile_hashes','features','build_flags'}
    if set(receipt)!=required or receipt['schema_version']!=1 or receipt['model']!='geosim':
        raise ValueError('unknown or missing build receipt/model fields')
    for name,want in [('manifest_sha256',manifest_sha256),('binary_sha256',binary_sha256),
                      ('source_inventory_sha256',source_inventory_sha256),('prebuild_inventory_sha256',source_inventory_sha256),
                      ('postbuild_inventory_sha256',source_inventory_sha256)]:
        if receipt[name]!=want:raise ValueError(f'build receipt {name} mismatch')
    if not receipt['build_command'] or not all(isinstance(s,str) for s in receipt['build_command']):
        raise ValueError('missing explicit build command')
    locks=receipt['lockfile_hashes']
    if not isinstance(locks,dict) or not {'Cargo.lock','survey/Cargo.lock'}<=set(locks):raise ValueError('receipt omits required Cargo locks')
    if source_root is not None:
        from .provenance import safe_source_path
        for name,digest in locks.items():
            if hashlib.sha256(safe_source_path(source_root,name).read_bytes()).hexdigest()!=digest:
                raise ValueError('build receipt lockfile SHA256 mismatch')
    return True


# Mirrors the native validate_outcome field checks and Rust Outcome field types.
# Nullable required floats are permitted only on invalid outcomes, in exactly the
# fields normalized by native normalize_invalid_floats; the evidence is not changed.
STATE_ID={'capital_cell':'uint','sovereignty_generation':'uint'}
WAR_SCHEMA={**dict.fromkeys(('id','start_period','last_active_period','active_periods','elapsed_periods'),'uint'),
    'parents':['uint'],'end_period':('optional','uint'),'raw_severity':'partial_nonnegative',
    'exported_severity':'partial_nonnegative','participants':[{'state':STATE_ID,'last_fighting_period':'uint'}],
    'end_cause':('optional','str'),'java_saturated':'bool','java_subunit_zero':'bool','fighting_periods':['uint']}
STATE_SCHEMA={'id':STATE_ID,'capacity':('optional','float'),'threshold':'float','alert':'bool',
    'campaign':('optional',STATE_ID),'previous_damage':'partial_float','newly_independent':'bool',
    'extracted_yield':'partial_float','recurrence_residual':'partial_float'}
LEDGER_SCHEMA={**dict.fromkeys(('attacks','fighting_front_periods','mutual_front_periods','conquests','collapses',
    'disconnections','stale_claims','locked_claims','double_successes','path_collisions','shocks'),'uint'),
    **dict.fromkeys(('damage','measured_damage','capacity_increase','capacity_decrease','clipping',
    'retirement_capacity','reemergence_capacity','recurrence_residual'),'partial_float')}
FRONT_SCHEMA={'states':('pair',STATE_ID),'previous':('pair','bool'),'actions':('pair','bool'),
    'old_commitments':('pair','partial_float'),'commitments':('pair','partial_float'),
    'path':('optional',('pair','uint')),'initiator':('optional','uint'),
    'last_damage':('pair','partial_float'),'last_victory_probabilities':('pair',('optional','float'))}
UPDATE_SCHEMA={'state':STATE_ID,'period':'uint','reset':'bool','clipping':'partial_float',
    **dict.fromkeys(('old_capacity','extracted_yield','applied_damage','target_capacity','new_capacity','residual'),('optional','float'))}


def _schema(value,schema,valid,path):
    if isinstance(schema,dict):
        if not isinstance(value,dict) or set(value)!=set(schema):
            raise ValueError(f'{path}: unknown or missing fields')
        for key,child in schema.items():_schema(value[key],child,valid,f'{path}.{key}')
        return
    if isinstance(schema,list):
        if not isinstance(value,list):raise ValueError(f'{path}: expected array')
        for i,item in enumerate(value):_schema(item,schema[0],valid,f'{path}[{i}]')
        return
    if isinstance(schema,tuple):
        kind,child=schema
        if kind=='optional':
            if value is not None:_schema(value,child,valid,path)
        else:
            if not isinstance(value,list) or len(value)!=2:raise ValueError(f'{path}: expected pair')
            for i,item in enumerate(value):_schema(item,child,valid,f'{path}[{i}]')
        return
    if schema.startswith('partial_'):
        if value is None and not valid:return
        schema=schema.removeprefix('partial_')
    good={'uint':lambda:_uint(value) and value<=2**64-1,
          'bool':lambda:type(value) is bool,'str':lambda:isinstance(value,str),
          'float':lambda:type(value) in (int,float) and math.isfinite(value),
          'nonnegative':lambda:_finite_nonnegative(value)}[schema]()
    if not good:raise ValueError(f'{path}: invalid {schema}')


def validate_outcome_schema(out):
    if not isinstance(out,dict) or set(out)!=OUTCOME_FIELDS:raise ValueError('unknown or missing Outcome fields')
    valid=out['valid'];c=out['config'];cells=c['width']*c['height']
    schema={**dict.fromkeys(('seed','periods','attempted_period','counting_start','sovereign_count'),'uint'),
        'valid':'bool','state_available':'bool','rng_mode':'str','finish_reason':'str','invalid_reason':('optional','str'),
        **dict.fromkeys(('completed_wars','censored_wars','legacy_visible_wars','exporter_backlog'),[WAR_SCHEMA]),
        'states':[STATE_SCHEMA],'cells':[{'id':'uint','owner':STATE_ID,'last_threshold':'float','next_generation':'uint'}],
        'retired_states':[STATE_ID],'merges':[{'period':'uint','survivor':'uint','absorbed':'uint'}],
        'ledger':LEDGER_SCHEMA,'fronts':[FRONT_SCHEMA],'resource_updates':[UPDATE_SCHEMA]}
    for key,child in schema.items():_schema(out[key],child,valid,f'Outcome.{key}')
    if out['rng_mode']!='portable_pcg64_mcg':raise ValueError('invalid Outcome RNG mode')
    if not valid and not out['invalid_reason']:raise ValueError('invalid Outcome lacks reason')
    def identities(value):
        if isinstance(value,dict):
            if set(value)==set(STATE_ID) and value['capital_cell']>=cells:raise ValueError('generation identity outside grid')
            for child in value.values():identities(child)
        elif isinstance(value,list):
            for child in value:identities(child)
    identities(out)
    states=[(state['id']['capital_cell'],state['id']['sovereignty_generation']) for state in out['states']]
    if len(set(states))!=len(states) or len(states)!=out['sovereign_count']:raise ValueError('duplicate sovereign identities/count mismatch')
    if sorted(cell['id'] for cell in out['cells'])!=list(range(cells)):raise ValueError('duplicate/missing grid cells')
    for field in ('merges','resource_updates'):
        if any(item['period']>out['attempted_period'] for item in out[field]):raise ValueError('event outside attempted clock')
    fights=out['partial_period_fights']
    if not isinstance(fights,list):raise ValueError('partial fights must be array')
    for fight in fights:
        if not isinstance(fight,list) or len(fight)!=3:raise ValueError('invalid partial fight tuple')
        _schema(fight[0],STATE_ID,valid,'partial fight state');_schema(fight[1],STATE_ID,valid,'partial fight state')
        identities(fight);_schema(fight[2],'partial_nonnegative',valid,'partial fight damage')
    for field in ('completed_wars','censored_wars'):
        for war in out[field]:
            start=war['start_period'];last=war['last_active_period']
            end=out['attempted_period'] if field=='censored_wars' else war['end_period']
            if end is None or not out['counting_start']<=start<=last<=end<=out['attempted_period']:
                raise ValueError('war clocks outside attempted history')
            fights=war['fighting_periods']
            if fights!=sorted(set(fights)) or any(not start<=p<=end for p in fights):raise ValueError('invalid fighting period order/range')
            if war['active_periods']!=len(fights) or (fights and fights[-1]!=last) or war['elapsed_periods']!=end-start+1:
                raise ValueError('war active/elapsed clock mismatch')
            participants=set()
            for part in war['participants']:
                identity=(part['state']['capital_cell'],part['state']['sovereignty_generation'])
                if identity in participants or not start<=part['last_fighting_period']<=end:raise ValueError('invalid participant identity/clock')
                participants.add(identity)
            if field=='completed_wars' and not war['end_cause']:raise ValueError('completed war has no cause')
            raw=war['raw_severity']
            if raw is not None:
                expected=raw if c['severity_export']=='raw_damage' else java_int100(raw)
                if war['exported_severity']!=expected:raise ValueError('resolved severity export mismatch')


def java_int100(raw):
    return float(min(2147483647,math.trunc(raw*100))) if raw<=2147483647/100 else 2147483647.


def _same_json_types(value,expected):
    if type(value) is not type(expected):return False
    if isinstance(expected,dict):
        return set(value)==set(expected) and all(_same_json_types(value[k],v) for k,v in expected.items())
    if isinstance(expected,list):
        return len(value)==len(expected) and all(_same_json_types(v,e) for v,e in zip(value,expected))
    return value==expected


def validate_record(row,arm,binding):
    if set(row)!=RECORD_FIELDS or type(row['schema_version']) is not int or row['schema_version']!=1 or row['model']!='geosim':
        raise ValueError('unknown or missing record/model fields')
    if any(row[k]!=binding[k] for k in BINDING_FIELDS):raise ValueError('record provenance mismatch')
    seed=row['seed'];repeat=row['repeat_index']
    if row['arm']!=arm['id'] or not _uint(row['arm_index']) or row['arm_index']!=arm['index'] or not _uint(seed) or not _uint(repeat) or not 0<=repeat<arm['sessions'] or seed!=arm['first_seed']+repeat:
        raise ValueError('record key differs from registered arm')
    if not _same_json_types(row['config'],arm['config']):raise ValueError('record config differs from authoritative resolved config')
    attempt=row['attempt']
    if set(attempt)!= {'status','construction_errors','panic_context','recorder_error'}:
        raise ValueError('unknown or missing attempt fields')
    if attempt['status'] not in ('completed','invalid','construction_error','construction_panic','implementation_panic','incomplete'):
        raise ValueError('unknown attempt status')
    _schema(attempt,{'status':'str','construction_errors':[{'field':'str','message':'str'}],
        'panic_context':('optional','str'),'recorder_error':('optional','str')},True,'attempt')
    if attempt['status']=='completed' and attempt['panic_context'] is not None:raise ValueError('completed attempt contains panic')
    if attempt['status']=='construction_error' and not attempt['construction_errors']:raise ValueError('construction_error lacks errors')
    if row['outcome'] is not None:
        if not isinstance(row['outcome'],dict) or not _same_json_types(row['outcome'].get('config'),arm['config']):raise ValueError('Outcome config differs from authoritative resolved config')
        validate_outcome_schema(row['outcome'])
    history_availability(row,arm)
    if row['outcome'] is not None:
        completed_censuses(row['outcome'],history_availability(row,arm))
    return row['arm'],seed


def read_sessions(path,manifest,resolved,binding):
    resolved_payload(resolved)
    if any(resolved[k]!=binding[k] for k in BINDING_FIELDS):raise ValueError('resolved export provenance mismatch')
    configs={a['id']:a['config'] for a in resolved['arms']}
    if len(configs)!=len(resolved['arms']):raise ValueError('duplicate resolved arm id')
    if set(configs)!={a['id'] for a in manifest['arms']}:raise ValueError('resolved arm family mismatch')
    arms={a['id']:{**a,'config':configs[a['id']]} for a in manifest['arms']}
    result={}
    with Path(path).open(encoding='utf-8',newline='') as file:
        for number,line in enumerate(file,1):
            try:
                if not line.endswith('\n'):raise ValueError('interrupted append: missing final newline')
                row=strict_json(line)
                if row['arm'] not in arms:raise ValueError('unregistered raw arm')
                key=validate_record(row,arms[row['arm']],binding)
                if key in result:raise ValueError('duplicate registered key')
                result[key]=row
            except (KeyError,TypeError,ValueError) as exc:
                raise ValueError(f'raw line {number}: {exc}') from exc
    return result
