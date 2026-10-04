"""Strict native envelopes, atomic completion and defined science availability."""
import hashlib
import json
import math
import re
from pathlib import Path
from .methods import canonical_bytes

BINDING_FIELDS = ('manifest_sha256', 'binary_sha256', 'source_inventory_sha256',
                  'build_receipt_sha256', 'resolved_configs_sha256')
RECORD_FIELDS = {'schema_version', 'model', 'arm', 'seed', 'arm_index', 'canonical_index',
    'repeat_index', 'family', 'execution_mode', 'config', 'attempt', 'outcome', *BINDING_FIELDS}
ATTEMPT_FIELDS = {'status', 'construction_errors', 'panic_context', 'recorder_error'}
ATTEMPT_STATUSES = {'completed', 'invalid', 'construction_error', 'construction_panic',
                    'implementation_panic', 'incomplete'}
COUNTER_FIELDS = {'initiated_fronts', 'mutual_d_front_periods', 'completed_victory_battles',
    'completed_stalemate_battles', 'opposing_claims', 'successful_claims', 'stale_claims',
    'locked_claims', 'retired_states', 'released_states'}
METRIC_FIELDS = {'democratic_cells', 'total_cells', 'democratic_share', 'sovereign_count',
    'democratic_states', 'predatory_states', 'democratic_mean_size', 'predatory_mean_size',
    'democratic_max_size', 'predatory_max_size', 'democratic_size_reason',
    'predatory_size_reason', 'democratic_exposure', 'clustering_ratio', 'clustering_reason',
    'conflict_fronts', 'alliance_count', 'pariah_count', 'democratic_extinction',
    'all_democratic', 'first_extinction_period', 'first_all_democratic_period'}
SCIENCE_FIELDS = {'model', 'config', 'seed', 'period', 'periods', 'attempted_period',
    'completed_periods', 'tick', 'last_tick_periods', 'setup', 'current_metrics',
    'census', 'outcome', 'final_state_hash'}
TERMINAL_FIELDS = {'valid', 'finish_reason', 'invalid_reason', 'invalid_phase',
    'attempted_period', 'completed_periods', 'final_metrics', 'census'}
RECEIPT_FIELDS = {'schema_version', 'model', 'manifest_sha256', 'source_inventory_sha256',
    'prebuild_inventory_sha256', 'postbuild_inventory_sha256', 'binary_sha256',
    'build_command', 'cwd', 'target', 'rustc_version', 'cargo_version',
    'lockfile_hashes', 'features', 'build_flags'}


def strict_json(text):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError(f'duplicate JSON field: {key}')
            result[key] = value
        return result
    def constant(value):
        raise ValueError(f'nonfinite JSON number: {value}')
    def finite_float(value):
        parsed = float(value)
        if not math.isfinite(parsed):
            raise ValueError('nonfinite JSON numeric exponent')
        return parsed
    try:
        return json.loads(text, object_pairs_hook=pairs, parse_constant=constant, parse_float=finite_float)
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise ValueError(f'invalid JSON: {exc}') from exc


def _uint(value):
    return type(value) is int and 0 <= value <= 2**64 - 1


def _number(value):
    return type(value) in (int, float) and math.isfinite(value)


def _same_json_types(value, expected):
    if type(value) is not type(expected):
        return False
    if isinstance(value, dict):
        return set(value) == set(expected) and all(_same_json_types(value[k], v) for k, v in expected.items())
    if isinstance(value, list):
        return len(value) == len(expected) and all(_same_json_types(a, b) for a, b in zip(value, expected))
    return value == expected


def _fields(value, expected, label):
    if not isinstance(value, dict) or set(value) != set(expected):
        raise ValueError(f'{label}: unknown or missing fields')


def validate_metrics(metrics, config, completed_periods):
    _fields(metrics, METRIC_FIELDS, 'metrics')
    integer_fields = ('democratic_cells', 'total_cells', 'sovereign_count', 'democratic_states',
        'predatory_states', 'conflict_fronts', 'alliance_count', 'pariah_count')
    if any(not _uint(metrics[k]) for k in integer_fields):
        raise ValueError('metrics: invalid integer census')
    total = config['width'] * config['height']
    democratic = metrics['democratic_cells']
    if metrics['total_cells'] != total or democratic > total or not 1 <= metrics['sovereign_count'] <= total:
        raise ValueError('metrics: grid/count mismatch')
    if metrics['democratic_states'] + metrics['predatory_states'] != metrics['sovereign_count']:
        raise ValueError('metrics: regime counts do not conserve sovereigns')
    if not _number(metrics['democratic_share']) or abs(metrics['democratic_share'] - democratic / total) > 1e-12:
        raise ValueError('metrics: share differs from authoritative integer cells')
    for regime, cells in [('democratic', democratic), ('predatory', total - democratic)]:
        count = metrics[regime + '_states']
        mean = metrics[regime + '_mean_size']
        maximum = metrics[regime + '_max_size']
        reason = metrics[regime + '_size_reason']
        if count == 0:
            if cells != 0 or mean is not None or maximum is not None or reason != 'no_surviving_states':
                raise ValueError('metrics: missing-regime availability mismatch')
        elif not (count <= cells and _number(mean) and abs(mean - cells / count) <= 1e-12
                  and _uint(maximum) and mean <= maximum <= cells and reason is None):
            raise ValueError('metrics: invalid regime state-size census')
    for key, expected in [('democratic_extinction', democratic == 0), ('all_democratic', democratic == total)]:
        if type(metrics[key]) is not bool or metrics[key] != expected:
            raise ValueError('metrics: endpoint indicator disagrees with census')
    for time_field, indicator in [('first_extinction_period', 'democratic_extinction'),
                                  ('first_all_democratic_period', 'all_democratic')]:
        value = metrics[time_field]
        if value is not None and (not _uint(value) or value > completed_periods):
            raise ValueError('metrics: invalid first-passage clock')
        # Persistent latent tags can allow reemergence after first extinction;
        # an observed first passage need not equal the current endpoint status.
        if metrics[indicator] and value is None:
            raise ValueError('metrics: reached endpoint lacks first passage')
    exposure = metrics['democratic_exposure']
    ratio = metrics['clustering_ratio']
    density = config['initial_democratic_share']
    expected_reason = 'undefined_initial_density' if density == 0 else 'undefined_extinction' if democratic == 0 else None
    if metrics['clustering_reason'] != expected_reason:
        raise ValueError('metrics: invalid clustering availability reason')
    if democratic == 0:
        if exposure is not None:
            raise ValueError('metrics: extinct population has exposure')
    elif not _number(exposure) or not 0 <= exposure <= 1:
        raise ValueError('metrics: invalid democratic exposure')
    if expected_reason is not None:
        if ratio is not None:
            raise ValueError('metrics: undefined clustering was imputed')
    elif not _number(ratio) or abs(ratio - exposure / density) > 1e-12:
        raise ValueError('metrics: clustering ratio disagrees with exposure/configured density')


def validate_counters(counters):
    _fields(counters, COUNTER_FIELDS, 'counters')
    if any(not _uint(value) for value in counters.values()):
        raise ValueError('invalid battle/structural counter')


def validate_science(science, *, allow_partial=False):
    _fields(science, SCIENCE_FIELDS, 'science')
    if science['model'] != 'democratic_peace' or not _uint(science['seed']):
        raise ValueError('invalid science model or seed')
    config = science['config']
    for key in ('width', 'height', 'horizon_periods', 'periods_per_tick'):
        if not _uint(config.get(key)) or config[key] == 0:
            raise ValueError('invalid resolved science configuration')
    if not _number(config.get('initial_democratic_share')) or not 0 <= config['initial_democratic_share'] <= 1:
        raise ValueError('invalid configured democratic density')
    clocks = ('period', 'periods', 'completed_periods', 'attempted_period', 'tick', 'last_tick_periods')
    if any(not _uint(science[key]) for key in clocks):
        raise ValueError('invalid science clock')
    completed = science['completed_periods']
    attempted = science['attempted_period']
    horizon = config['horizon_periods']
    if science['period'] != completed or science['periods'] != completed or not completed <= attempted <= min(horizon, completed + 1):
        raise ValueError('atomic attempted/completed clocks disagree')
    if science['last_tick_periods'] > config['periods_per_tick']:
        raise ValueError('display tick exceeds configured grouping')
    setup = science['setup']
    _fields(setup, {'initial_democratic_cells', 'initial_resourced_cells', 'total_cells'}, 'setup')
    total = config['width'] * config['height']
    if any(not _uint(value) for value in setup.values()) or setup['total_cells'] != total or any(
        setup[key] > total for key in ('initial_democratic_cells', 'initial_resourced_cells')
    ):
        raise ValueError('invalid setup census')
    validate_metrics(science['current_metrics'], config, completed)
    validate_counters(science['census'])
    terminal = science['outcome']
    if terminal is None and allow_partial:
        if completed > horizon:
            raise ValueError('partial science exceeds source horizon')
        if not isinstance(science['final_state_hash'], str) or not re.fullmatch('[0-9a-f]{16}', science['final_state_hash']):
            raise ValueError('invalid partial committed state hash')
        return science
    _fields(terminal, TERMINAL_FIELDS, 'terminal outcome')
    if type(terminal['valid']) is not bool or terminal['completed_periods'] != completed or terminal['attempted_period'] != attempted:
        raise ValueError('terminal/science availability clocks disagree')
    if not _same_json_types(terminal['census'], science['census']):
        raise ValueError('terminal/science census differs')
    if terminal['valid']:
        if terminal['finish_reason'] != 'complete' or completed != horizon or attempted != horizon or terminal['invalid_reason'] is not None or terminal['invalid_phase'] is not None:
            raise ValueError('complete outcome is not a valid full horizon')
        if not _same_json_types(terminal['final_metrics'], science['current_metrics']):
            raise ValueError('complete final metrics differ from committed census')
    elif terminal['finish_reason'] != 'invalid' or not isinstance(terminal['invalid_reason'], str) or not terminal['invalid_reason'] or not isinstance(terminal['invalid_phase'], str) or not terminal['invalid_phase'] or terminal['final_metrics'] is not None:
        raise ValueError('invalid outcome must retain context and unavailable final metrics')
    if not isinstance(science['final_state_hash'], str) or not re.fullmatch('[0-9a-f]{16}', science['final_state_hash']):
        raise ValueError('invalid committed full state hash')
    return science


def history_metrics(row):
    status = row['attempt']['status']
    science = row.get('outcome')
    if science is None:
        return {'status': status, 'complete': False, 'metrics': None, 'partial_metrics': None}
    terminal = science['outcome']
    complete = status == 'completed' and terminal is not None and terminal['valid']
    return {'status': status, 'complete': complete,
        'metrics': terminal['final_metrics'] if complete else None,
        'partial_metrics': None if complete else science['current_metrics'],
        'setup': science['setup'], 'final_state_hash': science['final_state_hash'],
        'completed_periods': science['completed_periods'], 'attempted_period': science['attempted_period'],
        'census': science['census'], 'invalid_reason': terminal['invalid_reason'] if terminal else row['attempt'].get('panic_context') or row['attempt'].get('recorder_error'),
        'invalid_phase': terminal['invalid_phase'] if terminal else None}


def resolved_payload(export):
    payload = export.get('resolved_configs_json')
    if not isinstance(payload, str) or payload.endswith('\n'):
        raise ValueError('missing exact native resolved payload without newline')
    if hashlib.sha256(payload.encode('utf-8')).hexdigest() != export.get('resolved_configs_sha256'):
        raise ValueError('native resolved payload SHA256 mismatch')
    parsed = strict_json(payload)
    if not _same_json_types(parsed, [{'id': arm['id'], 'config': arm['config']} for arm in export['arms']]):
        raise ValueError('native resolved payload/config type equality mismatch')
    return parsed


def validate_record(row, arm, binding):
    _fields(row, RECORD_FIELDS, 'record')
    if type(row['schema_version']) is not int or row['schema_version'] != 1 or row['model'] != 'democratic_peace':
        raise ValueError('invalid record/model version')
    if any(row[key] != binding[key] for key in BINDING_FIELDS):
        raise ValueError('record provenance mismatch')
    repeat = row['repeat_index']
    if row['family'] != arm['family'] or row['execution_mode'] != arm.get('execution_mode', 'registered'):
        raise ValueError('record family or execution mode mismatch')
    if row['arm'] != arm['id'] or any(not _uint(row[key]) for key in ('seed', 'arm_index', 'canonical_index', 'repeat_index')) or row['arm_index'] != arm['index'] or row['canonical_index'] != arm['canonical_index'] or repeat >= arm['sessions'] or row['seed'] != arm['first_seed'] + repeat:
        raise ValueError('record key differs from exact registered arm')
    if not _same_json_types(row['config'], arm['config']):
        raise ValueError('record config differs from authoritative native config')
    attempt = row['attempt']
    _fields(attempt, ATTEMPT_FIELDS, 'attempt')
    if attempt['status'] not in ATTEMPT_STATUSES or not isinstance(attempt['construction_errors'], list):
        raise ValueError('invalid attempt status or construction errors')
    for error in attempt['construction_errors']:
        _fields(error, {'field', 'message'}, 'construction error')
        if any(not isinstance(error[key], str) or not error[key] for key in error):
            raise ValueError('invalid construction error context')
    for key in ('panic_context', 'recorder_error'):
        if attempt[key] is not None and (not isinstance(attempt[key], str) or not attempt[key]):
            raise ValueError('invalid failure context')
    status = attempt['status']
    if status == 'construction_error' and not attempt['construction_errors']:
        raise ValueError('construction error lacks context')
    if status.endswith('_panic') and not attempt['panic_context']:
        raise ValueError('panic lacks context')
    if status == 'incomplete' and not attempt['recorder_error']:
        raise ValueError('incomplete recording lacks context')
    science = row['outcome']
    if science is None:
        if status not in ('construction_error', 'construction_panic', 'implementation_panic', 'incomplete'):
            raise ValueError('missing science without explicit unavailable attempt')
    else:
        validate_science(science, allow_partial=status in ('implementation_panic', 'incomplete'))
        if not _same_json_types(science['config'], arm['config']) or science['seed'] != row['seed']:
            raise ValueError('science config or seed differs from envelope')
        if status == 'completed' and (not science['outcome']['valid'] or attempt['panic_context'] is not None or attempt['construction_errors'] or attempt['recorder_error'] is not None):
            raise ValueError('completed attempt contains unavailable state or error')
        if status == 'invalid' and science['outcome']['valid']:
            raise ValueError('invalid attempt contains complete outcome')
        if status in ('construction_error', 'construction_panic'):
            raise ValueError('construction failure unexpectedly contains science')
    return row['arm'], row['seed']


def read_sessions(path, manifest, resolved, binding):
    from .manifest import expected_keys
    resolved_payload(resolved)
    if any(resolved[key] != binding[key] for key in BINDING_FIELDS):
        raise ValueError('resolved export provenance mismatch')
    expected_keys(manifest)
    configs = {arm['id']: arm['config'] for arm in resolved['arms']}
    if len(configs) != len(resolved['arms']) or set(configs) != {arm['id'] for arm in manifest['arms']}:
        raise ValueError('resolved arm family mismatch or duplicate')
    arms = {arm['id']: {**arm, 'config': configs[arm['id']], 'execution_mode': manifest['execution_mode']} for arm in manifest['arms']}
    result = {}
    with Path(path).open(encoding='utf-8', newline='') as file:
        for number, line in enumerate(file, 1):
            try:
                if not line.endswith('\n'):
                    raise ValueError('interrupted append: missing final newline')
                row = strict_json(line)
                if row['arm'] not in arms:
                    raise ValueError('unregistered raw arm')
                key = validate_record(row, arms[row['arm']], binding)
                if key in result:
                    raise ValueError('duplicate registered key')
                result[key] = row
            except (KeyError, TypeError, ValueError) as exc:
                raise ValueError(f'raw line {number}: {exc}') from exc
    return result


def verify_source_inventory(root, inventory):
    from .provenance import required_inventory_paths, safe_source_path
    if not isinstance(inventory, list) or not inventory:
        raise ValueError('source inventory must be nonempty')
    previous = None
    seen = set()
    for entry in inventory:
        _fields(entry, {'path', 'bytes', 'sha256'}, 'inventory entry')
        name = entry['path']
        if previous is not None and previous >= name:
            raise ValueError('source inventory must be unique and sorted')
        previous = name
        seen.add(name)
        data = safe_source_path(root, name).read_bytes()
        if type(entry['bytes']) is not int or len(data) != entry['bytes'] or hashlib.sha256(data).hexdigest() != entry['sha256']:
            raise ValueError(f'source bytes/SHA256 mismatch: {name}')
    if not set(required_inventory_paths(root)) <= seen:
        raise ValueError('source inventory omits required scientific sources')
    return hashlib.sha256(canonical_bytes(inventory)).hexdigest()


def validate_build_receipt(receipt, manifest_sha256, binary_sha256, source_inventory_sha256, source_root=None):
    _fields(receipt, RECEIPT_FIELDS, 'build receipt')
    if type(receipt['schema_version']) is not int or receipt['schema_version'] != 1 or receipt['model'] != 'democratic_peace':
        raise ValueError('invalid build receipt model')
    for field, value in [('manifest_sha256', manifest_sha256), ('binary_sha256', binary_sha256),
        ('source_inventory_sha256', source_inventory_sha256), ('prebuild_inventory_sha256', source_inventory_sha256),
        ('postbuild_inventory_sha256', source_inventory_sha256)]:
        if receipt[field] != value:
            raise ValueError(f'build receipt {field} mismatch')
    if not receipt['build_command'] or any(not isinstance(x, str) for x in receipt['build_command']):
        raise ValueError('explicit build command required')
    if not {'Cargo.lock', 'survey/Cargo.lock'} <= set(receipt['lockfile_hashes']):
        raise ValueError('build receipt omits required Cargo locks')
    if source_root is not None:
        from .provenance import safe_source_path
        for name, digest in receipt['lockfile_hashes'].items():
            if hashlib.sha256(safe_source_path(source_root, name).read_bytes()).hexdigest() != digest:
                raise ValueError('build receipt lockfile SHA256 mismatch')
    return True
