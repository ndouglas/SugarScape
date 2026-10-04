import copy
import hashlib
import json
import unittest
from survey.democratic_peace import records, manifest, source


def science(config=None, seed=380000001, democratic_cells=0):
    config = config or {'width': 2, 'height': 2, 'horizon_periods': 2,
        'periods_per_tick': 1, 'initial_democratic_share': .1}
    total = config['width'] * config['height']
    extinct = democratic_cells == 0
    metric = {'democratic_cells': democratic_cells, 'total_cells': total,
        'democratic_share': democratic_cells / total, 'sovereign_count': total,
        'democratic_states': democratic_cells, 'predatory_states': total - democratic_cells,
        'democratic_mean_size': None if extinct else 1.,
        'predatory_mean_size': None if democratic_cells == total else 1.,
        'democratic_max_size': None if extinct else 1,
        'predatory_max_size': None if democratic_cells == total else 1,
        'democratic_size_reason': 'no_surviving_states' if extinct else None,
        'predatory_size_reason': 'no_surviving_states' if democratic_cells == total else None,
        'democratic_exposure': None if extinct else 1.,
        'clustering_ratio': None if extinct else 1 / config['initial_democratic_share'],
        'clustering_reason': 'undefined_extinction' if extinct else None,
        'conflict_fronts': 0, 'alliance_count': 0, 'pariah_count': 0,
        'democratic_extinction': extinct, 'all_democratic': democratic_cells == total,
        'first_extinction_period': 0 if extinct else None,
        'first_all_democratic_period': 0 if democratic_cells == total else None}
    counters = dict.fromkeys(records.COUNTER_FIELDS, 0)
    terminal = {'valid': True, 'finish_reason': 'complete', 'invalid_reason': None,
        'invalid_phase': None, 'attempted_period': config['horizon_periods'],
        'completed_periods': config['horizon_periods'], 'final_metrics': metric,
        'census': counters}
    return {'model': 'democratic_peace', 'config': config, 'seed': seed,
        'period': config['horizon_periods'], 'periods': config['horizon_periods'],
        'attempted_period': config['horizon_periods'], 'completed_periods': config['horizon_periods'],
        'tick': config['horizon_periods'], 'last_tick_periods': 1,
        'setup': {'initial_democratic_cells': democratic_cells,
            'initial_resourced_cells': 0, 'total_cells': total},
        'current_metrics': metric, 'census': counters, 'outcome': terminal,
        'final_state_hash': '0123456789abcdef'}


class RecordTests(unittest.TestCase):
    def test_duplicate_nonfinite_and_float_overflow_json_rejected(self):
        for text in ('{"a":1,"a":2}', '{"a":NaN}', '{"a":1e999}'):
            with self.assertRaises(ValueError):
                records.strict_json(text)

    def test_complete_extinction_remains_complete_with_undefined_clustering(self):
        value = science()
        records.validate_science(value)
        result = records.history_metrics({'attempt': {'status': 'completed'}, 'outcome': value})
        self.assertTrue(result['complete'])
        self.assertIsNone(result['metrics']['clustering_ratio'])

    def test_invalid_partial_never_supplies_final_metrics(self):
        value = science()
        value.update(period=1, periods=1, completed_periods=1)
        value['outcome'].update(valid=False, finish_reason='invalid', invalid_reason='fixture',
            invalid_phase='decision', completed_periods=1, final_metrics=None)
        records.validate_science(value)
        result = records.history_metrics({'attempt': {'status': 'invalid'}, 'outcome': value})
        self.assertFalse(result['complete'])
        self.assertIsNone(result['metrics'])
        self.assertIsNotNone(result['partial_metrics'])

    def test_forged_counts_and_undefined_reason_rejected(self):
        value = science()
        for field, wrong in [('democratic_share', .5), ('clustering_reason', None), ('democratic_cells', True)]:
            forged = copy.deepcopy(value)
            forged['current_metrics'][field] = wrong
            with self.assertRaises(ValueError):
                records.validate_science(forged)

    def test_first_passage_zero_is_preserved_and_unknown_null_is_not_zero(self):
        value = science(democratic_cells=1)
        records.validate_science(value)
        self.assertIsNone(value['current_metrics']['first_extinction_period'])
        forged = copy.deepcopy(value)
        forged['current_metrics']['first_extinction_period'] = 3
        with self.assertRaises(ValueError):
            records.validate_science(forged)

    def test_exact_resolved_payload_hash_and_type_equality(self):
        payload = '[{"id":"fixture","config":{"width":2}}]'
        value = {'arms': json.loads(payload), 'resolved_configs_json': payload,
            'resolved_configs_sha256': hashlib.sha256(payload.encode()).hexdigest()}
        records.resolved_payload(value)
        value['arms'][0]['config']['width'] = 2.
        with self.assertRaises(ValueError):
            records.resolved_payload(value)


if __name__ == '__main__':
    unittest.main()

class PartialAndEnvelopeTests(unittest.TestCase):
    def test_panic_preserves_last_committed_science_without_terminal_outcome(self):
        value = science()
        value.update(period=1, periods=1, completed_periods=1)
        value['outcome'] = None
        records.validate_science(value, allow_partial=True)
        history = records.history_metrics({'attempt': {'status': 'implementation_panic',
            'panic_context': 'fixture panic'}, 'outcome': value})
        self.assertFalse(history['complete'])
        self.assertIsNotNone(history['partial_metrics'])

    def test_regime_reemergence_retains_historical_first_extinction(self):
        value = science(democratic_cells=1)
        value['current_metrics']['first_extinction_period'] = 0
        records.validate_science(value)
        self.assertFalse(value['current_metrics']['democratic_extinction'])

    def test_record_mode_and_provenance_cannot_drift(self):
        value = science(seed=400000001)
        arm = {'id': 'fixture.case0', 'index': 0, 'canonical_index': 0, 'sessions': 1,
            'first_seed': 400000001, 'family': 'fixture', 'execution_mode': 'fixture',
            'config': value['config']}
        binding = dict.fromkeys(records.BINDING_FIELDS, 'a' * 64)
        row = {**binding, 'schema_version': 1, 'model': 'democratic_peace',
            'arm': arm['id'], 'arm_index': 0, 'canonical_index': 0, 'repeat_index': 0,
            'family': 'fixture', 'execution_mode': 'fixture', 'seed': 400000001,
            'config': value['config'], 'outcome': value,
            'attempt': {'status': 'completed', 'construction_errors': [],
                'panic_context': None, 'recorder_error': None}}
        records.validate_record(row, arm, binding)
        for field, wrong in [('execution_mode', 'registered'), ('binary_sha256', 'b' * 64), ('repeat_index', 1)]:
            forged = copy.deepcopy(row)
            forged[field] = wrong
            with self.assertRaises(ValueError):
                records.validate_record(forged, arm, binding)

class SessionFileTests(unittest.TestCase):
    def test_duplicate_and_interrupted_attempts_are_rejected(self):
        import tempfile
        from pathlib import Path
        table = source.fixture_table()
        value = manifest.build_unregistered_manifest(table, None,
            configurations=[{'width': 2, 'height': 2, 'horizon_periods': 2}])
        arm = value['arms'][0]
        payload = json.dumps([{'id': arm['id'], 'config': science()['config']}], separators=(',', ':'))
        binding = dict.fromkeys(records.BINDING_FIELDS, 'a' * 64)
        binding['resolved_configs_sha256'] = hashlib.sha256(payload.encode()).hexdigest()
        resolved = {**binding, 'arms': json.loads(payload), 'resolved_configs_json': payload}
        row = {**binding, 'schema_version': 1, 'model': 'democratic_peace',
            'arm': arm['id'], 'arm_index': 0, 'canonical_index': 0, 'repeat_index': 0,
            'family': 'fixture', 'execution_mode': 'fixture', 'seed': arm['first_seed'],
            'config': science()['config'], 'outcome': science(seed=arm['first_seed']),
            'attempt': {'status': 'completed', 'construction_errors': [], 'panic_context': None, 'recorder_error': None}}
        line = json.dumps(row) + '\n'
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'sessions.jsonl'
            path.write_text(line)
            self.assertEqual(len(records.read_sessions(path, value, resolved, binding)), 1)
            path.write_text(line + line)
            with self.assertRaisesRegex(ValueError, 'duplicate registered key'):
                records.read_sessions(path, value, resolved, binding)
            path.write_text(line.rstrip('\n'))
            with self.assertRaisesRegex(ValueError, 'missing final newline'):
                records.read_sessions(path, value, resolved, binding)
