"""Schema 2 follow-up populations; schema 1 identities never become fresh rows."""
from copy import deepcopy
import hashlib
from pathlib import Path
import re
import tarfile
import numpy as np
from . import manifest as baseline, records, source
from .methods import canonical_bytes, method_contract, contract_sha256
from .provenance import safe_source_path

PROTOCOL = 'democratic-peace-followup-v1'
SPEC = 'docs/superpowers/specs/2026-10-04-democratic-peace-precision-design.md'
PHASES = {'literal_precision': ('printed_decreasing', 390000001, 2026100302),
          'prose_precision': ('prose_increasing', 410000001, 2026100402)}
HISTORICAL_FIELDS = {'study_id', 'manifest_sha256', 'source_inventory_sha256',
    'binary_sha256', 'build_receipt_sha256', 'resolved_configs_sha256', 'raw_sha256',
    'preservation_inventory_sha256'}
FIELDS = baseline.MANIFEST_FIELDS | {'study_protocol', 'phase', 'reading', 'historical_input'}
ORIGINAL_INVENTORY_SHA256 = '18da5bc191a00f1ed199dc1b54639a925f733b63db5c677a756e3c9bb30215bd'
ORIGINAL_SOURCE_SHA256 = 'fad57148ae9ca5f29d99df7777c235bd03b44187d9254d9d45d7dbd01bd856e2'
ORIGINAL_ARCHIVE_SHA256 = '69f7bbf5acd3a61190eea49df0657c3e72694d987c3de43f109f9bd70b3c70fa'
ORIGINAL_BINARY_SHA256 = '8bfac59be856cd68cf22186eff69c2f070026622fc7f81cfc2d0520f031461a5'
ORIGINAL_RAW_SHA256 = '753a1d5ff6236ba4720e0c52e46ab061643a5d27ef5c5d00175e0ff3f6d6c492'
EXPECTED_HISTORICAL = {'study_id':'democratic-peace-original-2026-10-04',
    'manifest_sha256':'8a2f365365ae1b5b5f71d8319225c6e565fb60ffe52e7d1008e5e04f296bb10a',
    'source_inventory_sha256':ORIGINAL_SOURCE_SHA256,'binary_sha256':ORIGINAL_BINARY_SHA256,
    'build_receipt_sha256':'239c1b10ac022564d9c2a86f7cbe13f63c17c24e47f548942124a170fe86f965',
    'resolved_configs_sha256':'ffb684d35cc0bfe0630c63aeef1e53306a0bdccdb0133a9ae08090338a228455',
    'raw_sha256':ORIGINAL_RAW_SHA256,'preservation_inventory_sha256':ORIGINAL_INVENTORY_SHA256}
SOURCE_TABLE_SHA256 = '221e944eefc22b166ce907c7bafc5acda95a76e3dbd84b2db240b5eb92366ed0'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def validate_historical_input(value):
    records._fields(value, HISTORICAL_FIELDS, 'historical input')
    if value['study_id'] != 'democratic-peace-original-2026-10-04' or any(
        not isinstance(value[k], str) or re.fullmatch('[0-9a-f]{64}', value[k]) is None
        for k in HISTORICAL_FIELDS - {'study_id'}):
        raise ValueError('invalid historical identity')
    if value != EXPECTED_HISTORICAL:
        raise ValueError('wrong historical archive binding')


def followup_contract():
    return {'revision': PROTOCOL, 'baseline_method_sha256': contract_sha256(),
        'baseline': method_contract(), 'population': {'fresh_histories': 100, 'arms': 108,
            'historical_literal_original': 30, 'pooling': 'none',
            'literal_eligibility': 'fresh_100_and_original_30_complete',
            'prose_eligibility': 'fresh_100_and_verified_historical_archive_no_prose_original'},
        'phase_roots': {p: v[2] for p, v in PHASES.items()}}


def canonical_arms(phase):
    if phase not in PHASES:
        raise ValueError('unsupported follow-up phase')
    reading, first, _ = PHASES[phase]
    arms = baseline.canonical_arms(True)[108:]
    for arm in arms:
        arm.update(id=phase + '.' + arm['id'], index=arm['canonical_index'],
            first_seed=first + arm['canonical_index'] * 10000)
        arm['config_overrides']['probability_direction'] = reading
    return arms


def analysis_jobs(phase):
    root = PHASES[phase][2]
    result = baseline.analysis_jobs(None)
    for job in result:
        job.update(root_entropy=root, state_u32=np.random.SeedSequence(root,
            spawn_key=(job['index'],)).generate_state(4).tolist())
    return result


def job_rng(job, phase):
    expected = analysis_jobs(phase)
    if type(job.get('index')) is not int or not 0 <= job['index'] < len(expected) or not records._same_json_types(job, expected[job['index']]):
        raise ValueError('follow-up analysis job seed/payload mismatch')
    return np.random.Generator(np.random.PCG64(np.random.SeedSequence(PHASES[phase][2],
        spawn_key=(job['index'],))))


def build_manifest(table, source_bytes, phase, historical_input):
    validate_historical_input(historical_input)
    if phase not in PHASES:
        raise ValueError('unsupported follow-up phase')
    value = baseline.build_manifest(table, source_bytes)
    contract = followup_contract()
    value.update(schema_version=2, study_protocol=PROTOCOL, phase=phase,
        reading=PHASES[phase][0], historical_input=deepcopy(historical_input), spec=SPEC,
        precision_registered=True, runtime_gate='followup_full_precision',
        analysis_seed=PHASES[phase][2], arms=canonical_arms(phase),
        analysis_jobs=analysis_jobs(phase), method_contract=contract,
        method_contract_json=canonical_bytes(contract).decode(),
        method_contract_sha256=sha(canonical_bytes(contract)))
    return value


def expected_keys(value):
    records._fields(value, FIELDS, 'follow-up manifest')
    if type(value['schema_version']) is not int or value['schema_version'] != 2 or value['study_protocol'] != PROTOCOL or value['phase'] not in PHASES:
        raise ValueError('unsupported follow-up schema/protocol/phase')
    phase = value['phase']; contract = followup_contract()
    fixed = {'model': 'democratic_peace', 'execution_mode': 'registered', 'spec': SPEC,
        'precision_registered': True, 'runtime_gate': 'followup_full_precision',
        'reading': PHASES[phase][0], 'analysis_seed': PHASES[phase][2],
        'source_draws': 100000, 'permutation_draws': 100000, 'contrast_draws': 100000,
        'method_contract': contract, 'method_contract_json': canonical_bytes(contract).decode(),
        'method_contract_sha256': sha(canonical_bytes(contract)),
        'arms': canonical_arms(phase), 'analysis_jobs': analysis_jobs(phase)}
    if any(not records._same_json_types(value[k], v) for k, v in fixed.items()):
        raise ValueError('fixed follow-up protocol changed')
    if value['provenance_status'] not in ('provisional_unfrozen', 'frozen'):
        raise ValueError('invalid follow-up provenance status')
    validate_historical_input(value['historical_input'])
    keys = [(a['id'], a['first_seed'] + r) for a in value['arms'] for r in range(100)]
    if len(keys) != 10800 or len({s for _, s in keys}) != 10800:
        raise ValueError('incomplete/duplicate fresh population')
    return keys


def validate_record(row, arm, binding):
    records._fields(row, records.RECORD_FIELDS | {'study_protocol', 'phase'}, 'follow-up record')
    if type(row['schema_version']) is not int or row['schema_version'] != 2 or row['study_protocol'] != PROTOCOL or row['phase'] not in PHASES:
        raise ValueError('unsupported follow-up row')
    if arm.get('schema_version') != 2 or row['phase'] != arm.get('phase') or row['study_protocol'] != arm.get('study_protocol') or not row['arm'].startswith(row['phase'] + '.precision.'):
        raise ValueError('mixed follow-up record phase')
    # Reuse science/attempt validation after this separate envelope is accepted.
    payload = {k: v for k, v in row.items() if k not in ('study_protocol', 'phase')}
    payload['schema_version'] = 1
    return records.validate_record(payload, {k:v for k,v in arm.items() if k != 'schema_version'}, binding)



def validate_resolved(value, resolved):
    expected_keys(value)
    records.resolved_payload(resolved)
    base = records.strict_json(Path(__file__).with_name('followup-default-config.json').read_bytes())
    expected = [{'id':arm['id'],'config':{**base,**arm['config_overrides']}} for arm in value['arms']]
    if not records._same_json_types(resolved['arms'],expected):
        raise ValueError('resolved follow-up defaults/reading/config drift')
    if resolved.get('schema_version') != 2 or resolved.get('phase') != value['phase'] or resolved.get('study_protocol') != PROTOCOL:
        raise ValueError('resolved follow-up envelope mismatch')
    return expected
