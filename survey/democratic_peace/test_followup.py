"""Seconds-scale protocol fixtures; no registered world construction."""
import copy
import hashlib
from pathlib import Path
import tempfile
import unittest
from survey.democratic_peace import manifest, records, source, historical
try:
    from survey.democratic_peace import followup
except ImportError:
    followup = None
from survey.democratic_peace.methods import canonical_bytes
from survey.democratic_peace.test_records import science

HISTORICAL = followup.EXPECTED_HISTORICAL if followup is not None else {}


class FollowupProtocolTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(followup, 'follow-up protocol is not implemented')

    def make(self, phase='literal_precision'):
        table = source.fixture_table()
        return followup.build_manifest(table, canonical_bytes(table), phase, HISTORICAL)

    def test_exact_fresh_rosters_disjoint_from_original_and_each_other(self):
        literal, prose = self.make(), self.make('prose_precision')
        lkeys, pkeys = map(manifest.expected_keys, (literal, prose))
        self.assertEqual((len(lkeys), len(pkeys)), (10800, 10800))
        self.assertFalse({s for _, s in lkeys} & {s for _, s in pkeys})
        self.assertFalse({s for _, s in lkeys} & {s for _, s in manifest.expected_keys(manifest.build_manifest(source.fixture_table()))})
        self.assertEqual(lkeys[0], ('literal_precision.precision.mobile0_15.tagging.density0', 390000001))
        self.assertEqual(pkeys[-1][1], 411070100)

    def test_only_explicit_probability_override_differs(self):
        left, right = self.make(), self.make('prose_precision')
        for larm, rarm in zip(left['arms'], right['arms']):
            lconfig, rconfig = larm['config_overrides'], rarm['config_overrides']
            self.assertEqual({k for k in lconfig if lconfig[k] != rconfig[k]}, {'probability_direction'})
            self.assertEqual((larm['preset'], rarm['preset']), ('printed_2001', 'printed_2001'))

    def test_fixed_methods_and_phase_streams_are_separately_bound(self):
        from survey.democratic_peace.methods import contract_sha256
        left, right = self.make(), self.make('prose_precision')
        self.assertEqual(left['method_contract'], right['method_contract'])
        self.assertNotEqual(left['method_contract_sha256'], contract_sha256())
        self.assertEqual(left['method_contract']['baseline_method_sha256'], contract_sha256())
        self.assertEqual((left['analysis_seed'], right['analysis_seed']), (2026100302, 2026100402))
        self.assertEqual(len(left['analysis_jobs']), 129)
        self.assertNotEqual(left['analysis_jobs'][0]['state_u32'], right['analysis_jobs'][0]['state_u32'])

    def test_unknown_missing_drift_and_boolean_indices_rejected(self):
        mutations = [('phase', 'unknown'), ('reading', 'prose_increasing'),
            ('analysis_seed', 2026100402), ('execution_mode', 'fixture'), ('runtime_gate', 'full_precision')]
        for key, wrong in mutations:
            changed = self.make(); changed[key] = wrong
            with self.assertRaises(ValueError): manifest.expected_keys(changed)
        for field, wrong in [('sessions', 99), ('first_seed', 390000002), ('index', False), ('family', 'original')]:
            changed = self.make(); changed['arms'][0][field] = wrong
            with self.assertRaises(ValueError): manifest.expected_keys(changed)
        changed = self.make(); changed['extra'] = True
        with self.assertRaises(ValueError): manifest.expected_keys(changed)
        changed = self.make(); del changed['historical_input']['raw_sha256']
        with self.assertRaises(ValueError): manifest.expected_keys(changed)

    def test_schema2_rows_validate_without_broadening_schema1(self):
        value = self.make(); arm = value['arms'][0]
        config = science()['config']; binding = dict.fromkeys(records.BINDING_FIELDS, 'a' * 64)
        indexed = {**arm, 'config': config, 'schema_version': 2,
            'phase': value['phase'], 'study_protocol': value['study_protocol'], 'execution_mode': 'registered'}
        row = {**binding, 'schema_version': 2, 'model': 'democratic_peace', 'arm': arm['id'],
            'seed': arm['first_seed'], 'arm_index': 0, 'canonical_index': 0, 'repeat_index': 0,
            'family': 'precision', 'execution_mode': 'registered', 'config': config,
            'study_protocol': value['study_protocol'], 'phase': value['phase'],
            'attempt': {'status': 'completed', 'construction_errors': [], 'panic_context': None, 'recorder_error': None},
            'outcome': science(seed=arm['first_seed'])}
        self.assertEqual(records.validate_record(row, indexed, binding), (arm['id'], arm['first_seed']))
        for field, wrong in [('phase', 'prose_precision'), ('schema_version', 1), ('binary_sha256', 'b'*64)]:
            changed = copy.deepcopy(row); changed[field] = wrong
            with self.assertRaises(ValueError): records.validate_record(changed, indexed, binding)
        with self.assertRaises(ValueError): records.validate_record(row, {**indexed, 'schema_version': 1}, binding)

    def test_archive_inventory_rejects_tampering_and_unsafe_paths(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); (root/'file').write_bytes(b'original')
            inventory = [{'path': 'file', 'bytes': 8, 'sha256': hashlib.sha256(b'original').hexdigest()}]
            self.assertEqual(historical.verify_preservation(root, inventory), hashlib.sha256(canonical_bytes(inventory)).hexdigest())
            (root/'file').write_bytes(b'changed!')
            with self.assertRaises(ValueError): historical.verify_preservation(root, inventory)
            inventory[0]['path'] = '../file'
            with self.assertRaises(ValueError): historical.verify_preservation(root, inventory)
