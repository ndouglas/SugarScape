"""Auditable complete census and fixed families; registered draws cannot shrink."""
from collections import Counter
import hashlib
import json
from pathlib import Path
from . import manifest as population, source, contrasts
from .methods import canonical_bytes, method_contract, contract_sha256
from .records import history_metrics, validate_record, validate_science, resolved_payload


def report(manifest, table, records, resolved=None, binding=None, *, fixture_draws=None, data_sha256=None):
    mode = manifest['execution_mode']
    fixture = mode == 'fixture'
    if fixture_draws is not None and not fixture:
        raise ValueError('fixture draw overrides require explicit fixture manifest')
    if fixture_draws is not None and (type(fixture_draws) is not int or not 1 <= fixture_draws <= 1000):
        raise ValueError('synthetic fixture draw count exceeds bounded gate')
    if manifest['method_contract'] != method_contract() or manifest['method_contract_sha256'] != contract_sha256() or manifest['method_contract_json'] != canonical_bytes(method_contract()).decode('utf-8'):
        raise ValueError('method contract differs from frozen implementation')
    if any(manifest[name] != 100000 for name in ('source_draws', 'permutation_draws', 'contrast_draws')):
        raise ValueError('fixed registered draw counts changed')
    source.validate_table(table)
    if mode == 'registered':
        source.validate_registered_table(table)
        if manifest['provenance_status'] != 'frozen' or resolved is None or binding is None or data_sha256 is None:
            raise ValueError('registered analysis requires frozen complete provenance')
        if table['extraction'].get('classification') == 'synthetic_fixture':
            raise ValueError('synthetic source cannot activate registered judgments')
    keys = population.expected_keys(manifest)
    if set(records) - set(keys):
        raise ValueError('unregistered raw session keys')
    expected_jobs = population.analysis_jobs(manifest)
    if manifest['analysis_jobs'] != expected_jobs:
        raise ValueError('fixed analysis job payload mismatch')
    jobs = {job['id']: job for job in expected_jobs}
    indexed = {}
    if resolved is not None:
        resolved_payload(resolved)
        if binding is None or any(resolved[name] != binding[name] for name in binding):
            raise ValueError('resolved/config binding mismatch')
        configs = {arm['id']: arm['config'] for arm in resolved['arms']}
        if len(configs) != len(resolved['arms']) or set(configs) != {a['id'] for a in manifest['arms']}:
            raise ValueError('resolved arm family mismatch')
        indexed = {a['id']: {**a, 'config': configs[a['id']], 'execution_mode': mode} for a in manifest['arms']}
    histories = {arm['id']: [] for arm in manifest['arms']}
    history_rows = []
    for arm, seed in keys:
        row = records.get((arm, seed))
        if row is None:
            history_rows.append({'arm': arm, 'seed': seed, 'status': 'missing',
                'complete': False, 'metrics': None, 'partial_metrics': None})
            continue
        if indexed:
            if validate_record(row, indexed[arm], binding) != (arm, seed):
                raise ValueError('raw dictionary key differs from envelope')
        else:
            if mode == 'registered':
                raise ValueError('registered record validation requires native resolution')
            if row['arm'] != arm or row['seed'] != seed:
                raise ValueError('fixture key differs from envelope')
            if row.get('outcome') is not None:
                validate_science(row['outcome'], allow_partial=row['attempt']['status'] in ('implementation_panic', 'incomplete'))
        history = history_metrics(row)
        histories[arm].append(history)
        history_rows.append({'arm': arm, 'seed': seed, **history})
    draws = fixture_draws or (20 if fixture else 100000)
    if mode == 'runtime_probe':
        # Runtime evidence never becomes inference, even with source-arm labels.
        precision_registered = False
    else:
        precision_registered = manifest['precision_registered']
    source_result = source.source_findings(table, histories, jobs,
        precision_registered=precision_registered, draws=draws,
        minimum_defined=2 if fixture else 10000, fixture=fixture)
    comparisons = contrasts.findings(histories, jobs,
        precision_registered=precision_registered, draws=draws)
    return {'schema_version': 1, 'model': 'democratic_peace',
        'classification': 'synthetic_fixture' if fixture else 'unregistered_runtime_probe' if mode == 'runtime_probe' else 'registered_offline_findings',
        'execution_mode': mode, 'precision_registered': precision_registered,
        'registered_histories': len(keys) if mode == 'registered' else 0,
        'attempted_slots': len(keys), 'received_histories': len(records),
        'history_status_counts': dict(Counter(h['status'] for h in history_rows)),
        'histories': history_rows, 'source': source_result,
        'primary_contrasts': comparisons['primary'], 'secondary_contrasts': comparisons['secondary'],
        'analysis_jobs': expected_jobs, 'draws': {'source': draws, 'permutation': draws, 'contrast': draws},
        'method_contract_sha256': contract_sha256(), 'source_equivalence': 'Unresolved',
        'provenance': {'data_sha256': data_sha256, 'binding': binding,
            'source_table_sha256': manifest['source_table_sha256']},
        'limits': ['source_curves_are_inferred_values_not_raw_history_means',
            'dependent_lattice_histories_do_not_establish_historical_causality',
            'outside_bootstrap_support_is_not_impossible_model_event',
            'printed_probability_primary_prose_variant_unmeasured',
            'original_executable_rng_and_statistic_identity_unresolved']}


def main():
    import argparse
    import platform
    import numpy as np
    from .records import strict_json, verify_source_inventory, validate_build_receipt, read_sessions
    from .run import sha256_file
    from .reporting import serialize_report, markdown_report
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('manifest', 'source', 'sessions', 'resolved', 'build-receipt', 'binary', 'source-root', 'output'):
        parser.add_argument('--' + name, required=True, type=Path)
    args = parser.parse_args()
    manifest_bytes = args.manifest.read_bytes()
    value = strict_json(manifest_bytes)
    actual = {'python': platform.python_version(), 'numpy': np.__version__}
    if actual != method_contract()['runtime']:
        raise ValueError(f'analysis runtime differs from pinned contract: {actual}')
    source_bytes = args.source.read_bytes()
    if hashlib.sha256(source_bytes).hexdigest() != value['source_table_sha256']:
        raise ValueError('source table byte SHA256 mismatch')
    if value['execution_mode'] == 'registered':
        from .provenance import verify_source_review
        verify_source_review(args.source_root, strict_json(source_bytes))
    inventory = verify_source_inventory(args.source_root, value['source_inventory'])
    if inventory != value['source_inventory_sha256']:
        raise ValueError('source inventory digest mismatch')
    manifest_hash = hashlib.sha256(manifest_bytes).hexdigest()
    binary_hash = sha256_file(args.binary)
    receipt_bytes = args.build_receipt.read_bytes()
    validate_build_receipt(strict_json(receipt_bytes), manifest_hash, binary_hash, inventory, args.source_root)
    resolved = strict_json(args.resolved.read_bytes())
    binding = {'manifest_sha256': manifest_hash, 'binary_sha256': binary_hash,
        'source_inventory_sha256': inventory, 'build_receipt_sha256': hashlib.sha256(receipt_bytes).hexdigest(),
        'resolved_configs_sha256': resolved['resolved_configs_sha256']}
    data_hash = sha256_file(args.sessions)
    records = read_sessions(args.sessions, value, resolved, binding)
    if sha256_file(args.sessions) != data_hash:
        raise ValueError('raw dataset changed during validation')
    result = report(value, strict_json(source_bytes), records, resolved, binding, data_sha256=data_hash)
    if verify_source_inventory(args.source_root, value['source_inventory']) != inventory:
        raise ValueError('scientific source changed during analysis')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.with_suffix('.json').write_text(serialize_report(result))
    args.output.with_suffix('.md').write_text(markdown_report(result))


if __name__ == '__main__':
    main()
