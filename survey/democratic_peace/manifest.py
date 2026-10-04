"""Exact prospective populations; Rust resolves all model defaults."""
from copy import deepcopy
import hashlib
import json
import numpy as np
from .methods import canonical_bytes, contract_sha256, method_contract

MOBILES = (.15, .5, .85)
MECHANISMS = ('tagging', 'alliances', 'collective_security')
DENSITIES = (0., .05, .1, .15, .2, .25, .3, .4, .5, .6, .7, 1.)
ROOT_SEED = 2026100302
MANIFEST_FIELDS = {'schema_version', 'model', 'execution_mode', 'provenance_status', 'spec',
    'precision_registered', 'runtime_gate', 'analysis_seed', 'source_draws', 'permutation_draws',
    'contrast_draws', 'source_table_sha256', 'method_contract', 'method_contract_json',
    'method_contract_sha256', 'source_inventory', 'source_inventory_sha256', 'arms', 'analysis_jobs'}


def number_id(value):
    return format(value, 'g').replace('.', '_')


def arm_suffix(mobile, mechanism, density):
    return f'mobile{number_id(mobile)}.{mechanism}.density{number_id(density)}'


def canonical_arms(precision_registered):
    arms = []
    families = [('original', 30, 380000001)]
    if precision_registered:
        families.append(('precision', 100, 390000001))
    for family, sessions, seed_base in families:
        for mi, mobile in enumerate(MOBILES):
            for ki, mechanism in enumerate(MECHANISMS):
                for di, density in enumerate(DENSITIES):
                    canonical = ((mi * 3 + ki) * 12 + di)
                    arms.append({'id': f'{family}.{arm_suffix(mobile, mechanism, density)}',
                        'family': family, 'index': len(arms), 'canonical_index': canonical,
                        'mobile_index': mi, 'mechanism_index': ki, 'density_index': di,
                        'preset': 'printed_2001', 'config_overrides': {
                            'mobile_share': mobile, 'mechanism': mechanism,
                            'initial_democratic_share': density, 'periods_per_tick': 100},
                        'sessions': sessions, 'first_seed': seed_base + canonical * 10000})
    return arms


def contrast_definitions():
    result = []
    for density in (.1, .3):
        for name, left, right, metric in (
            ('alliances_minus_tagging', 'alliances', 'tagging', 'democratic_share'),
            ('security_minus_alliances', 'collective_security', 'alliances', 'democratic_share'),
            ('security_minus_tagging', 'collective_security', 'tagging', 'all_democratic')):
            result.append({'id': f'primary.{name}.density{number_id(density)}',
                'family': 'primary', 'metric': metric,
                'left': 'precision.' + arm_suffix(.5, left, density),
                'right': 'precision.' + arm_suffix(.5, right, density)})
    for density in (.1, .3):
        for mechanism in MECHANISMS:
            result.append({'id': f'secondary.mobile0_15_minus_0_85.{mechanism}.density{number_id(density)}',
                'family': 'secondary', 'metric': 'democratic_share',
                'left': 'precision.' + arm_suffix(.15, mechanism, density),
                'right': 'precision.' + arm_suffix(.85, mechanism, density)})
    return result


def analysis_jobs(manifest):
    from .source import slot_roster
    ids = ['predictive.source.' + slot['id'] for slot in slot_roster()]
    for contrast in contrast_definitions():
        ids += ['permutation.' + contrast['id'], 'bootstrap.' + contrast['id']]
    return [{'index': i, 'id': name, 'root_entropy': ROOT_SEED, 'spawn_key': [i],
        'state_u32': np.random.SeedSequence(ROOT_SEED, spawn_key=(i,)).generate_state(4).tolist()}
        for i, name in enumerate(ids)]


def job_rng(job):
    if job['root_entropy'] != ROOT_SEED or job['spawn_key'] != [job['index']]:
        raise ValueError('analysis seed derivation mismatch')
    seed = np.random.SeedSequence(ROOT_SEED, spawn_key=(job['index'],))
    if job['state_u32'] != seed.generate_state(4).tolist():
        raise ValueError('analysis child seed state mismatch')
    return np.random.Generator(np.random.PCG64(seed))


def build_manifest(source, source_bytes=None, *, precision_registered=False):
    from .source import validate_table
    validate_table(source)
    if type(precision_registered) is not bool:
        raise ValueError('precision registration must be an explicit boolean')
    from .records import strict_json, _same_json_types
    if source_bytes is not None and not _same_json_types(strict_json(source_bytes), source):
        raise ValueError('source table bytes differ from parsed table')
    result = {'schema_version': 1, 'model': 'democratic_peace',
        'execution_mode': 'registered', 'provenance_status': 'provisional_unfrozen',
        'spec': 'docs/superpowers/specs/2026-10-03-democratic-peace-design.md',
        'precision_registered': precision_registered,
        'runtime_gate': 'full_precision' if precision_registered else 'original_only',
        'analysis_seed': ROOT_SEED, 'source_draws': 100000,
        'permutation_draws': 100000, 'contrast_draws': 100000,
        'source_table_sha256': None if source_bytes is None else hashlib.sha256(source_bytes).hexdigest(),
        'method_contract': method_contract(),
        'method_contract_json': canonical_bytes(method_contract()).decode('utf-8'),
        'method_contract_sha256': contract_sha256(),
        'source_inventory': None, 'source_inventory_sha256': None,
        'arms': canonical_arms(precision_registered)}
    result['analysis_jobs'] = analysis_jobs(result)
    return result


def expected_keys(manifest):
    if set(manifest) != MANIFEST_FIELDS or type(manifest['schema_version']) is not int or manifest['schema_version'] != 1:
        raise ValueError('unknown or missing manifest fields/version')
    for key, value in [('analysis_seed', ROOT_SEED), ('source_draws', 100000), ('permutation_draws', 100000), ('contrast_draws', 100000)]:
        if type(manifest[key]) is not int or manifest[key] != value:
            raise ValueError('fixed manifest analysis contract changed')
    if manifest.get('model') != 'democratic_peace' or type(manifest.get('precision_registered')) is not bool:
        raise ValueError('invalid registered model or precision flag')
    if manifest.get('execution_mode') != 'registered':
        return unregistered_keys(manifest)
    expected = canonical_arms(manifest['precision_registered'])
    from .records import _same_json_types
    if not _same_json_types(manifest['arms'], expected):
        raise ValueError('registered arm/config/key contract changed')
    if manifest.get('runtime_gate') != ('full_precision' if manifest['precision_registered'] else 'original_only'):
        raise ValueError('runtime gate differs from registered population')
    keys = [(arm['id'], arm['first_seed'] + r) for arm in expected for r in range(arm['sessions'])]
    if len(keys) != (14040 if manifest['precision_registered'] else 3240) or len(set(keys)) != len(keys):
        raise ValueError('registered population is incomplete or duplicated')
    return keys



CONFIG_FIELDS = {'width', 'height', 'superiority_exponent', 'victory_exponent',
    'horizon_periods', 'periods_per_tick', 'event_limit', 'initial_democratic_share',
    'initial_resourced_share', 'mobile_share', 'superiority_threshold', 'victory_threshold',
    'stalemate_probability', 'tax_rate', 'distance_gradient', 'min_threat', 'event_recording',
    'mechanism', 'probability_direction', 'zero_ratio', 'assignment', 'distance_metric',
    'enemy_total', 'inactive_commitment', 'latent_regime', 'capital_capture', 'claim_locking',
    'opposing_victories', 'alliance_maintenance', 'threat_ties', 'obligation_observation',
    'security_scope', 'clustering_exposure', 'clustering_weights'}


def build_unregistered_manifest(source, source_bytes, *, configurations, mode='fixture', sessions=1, presets=None):
    result = build_manifest(source, source_bytes)
    if mode not in ('fixture', 'runtime_probe'):
        raise ValueError('unknown nonregistered mode')
    presets = presets or ['printed_2001'] * len(configurations)
    if len(presets) != len(configurations):
        raise ValueError('one named preset required per unregistered arm')
    result.update(execution_mode=mode, provenance_status=mode + '_unregistered', arms=[])
    for i, (config, preset) in enumerate(zip(configurations, presets)):
        result['arms'].append({'id': f'{mode}.case{i}', 'family': mode, 'index': i,
            'canonical_index': i, 'mobile_index': 0, 'mechanism_index': 0, 'density_index': 0,
            'preset': preset, 'config_overrides': deepcopy(config), 'sessions': sessions,
            'first_seed': 400000001 + i * 10000})
    unregistered_keys(result)
    return result


def unregistered_keys(manifest):
    mode = manifest.get('execution_mode')
    if mode not in ('fixture', 'runtime_probe') or manifest.get('provenance_status') != mode + '_unregistered' or manifest['precision_registered']:
        raise ValueError('nonregistered mode/provenance mismatch')
    arms = manifest['arms']
    maximum = 4 if mode == 'fixture' else 8
    if not 1 <= len(arms) <= maximum:
        raise ValueError('unregistered arm count exceeds bounded gate')
    keys = []
    for i, arm in enumerate(arms):
        if set(arm) != set(canonical_arms(False)[0]) or arm['index'] != i or arm['family'] != mode:
            raise ValueError('invalid unregistered arm fields or family')
        c = arm['config_overrides']
        if set(c) - CONFIG_FIELDS or arm['preset'] not in ('printed_2001', 'prose_probability'):
            raise ValueError('unknown configuration override or reading')
        if mode == 'fixture':
            bounded = 2 <= c.get('width', 15) <= 4 and 2 <= c.get('height', 15) <= 4 and 1 <= c.get('horizon_periods', 1000) <= 20
        else:
            bounded = c.get('width', 15) == 15 and c.get('height', 15) == 15 and c.get('horizon_periods', 1000) == 1000
        if any(type(c[key]) is not int for key in ('width', 'height', 'horizon_periods') if key in c):
            raise ValueError('nonregistered dimensions/horizon must be integers')
        if not bounded or type(arm['sessions']) is not int or not 1 <= arm['sessions'] <= (2 if mode == 'fixture' else 1) or type(arm['first_seed']) is not int or arm['first_seed'] < 400000000:
            raise ValueError('unregistered dimensions/horizon/seeds exceed bounded gate')
        keys.extend((arm['id'], arm['first_seed'] + r) for r in range(arm['sessions']))
    if len({seed for _, seed in keys}) != len(keys) or len({arm['id'] for arm in arms}) != len(arms):
        raise ValueError('duplicate unregistered arm or seed')
    return keys


def main():
    import argparse
    from pathlib import Path
    from .records import strict_json
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--precision-registered', action='store_true')
    args = parser.parse_args()
    data = args.source.read_bytes()
    result = build_manifest(strict_json(data), data, precision_registered=args.precision_registered)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, allow_nan=False) + '\n')
    print('Provisional manifest only; runtime gate and post-writer source freeze required before execution')


if __name__ == '__main__':
    main()
