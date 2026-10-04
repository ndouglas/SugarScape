import copy
import unittest
from survey.democratic_peace import manifest, source


class ManifestTests(unittest.TestCase):
    def test_complete_original_and_disjoint_precision_populations(self):
        table = source.fixture_table()
        original = manifest.build_manifest(table)
        full = manifest.build_manifest(table, precision_registered=True)
        self.assertEqual(len(manifest.expected_keys(original)), 3240)
        keys = manifest.expected_keys(full)
        self.assertEqual(len(keys), 14040)
        self.assertEqual(len({seed for _, seed in keys}), 14040)
        self.assertEqual(full['arms'][108]['canonical_index'], 0)
        self.assertEqual(full['arms'][108]['index'], 108)
        self.assertEqual(full['arms'][108]['first_seed'], 390000001)
        self.assertEqual(len(full['analysis_jobs']), 129)

    def test_reduced_or_changed_registered_arm_rejected(self):
        value = manifest.build_manifest(source.fixture_table())
        for field, wrong in [('sessions', 29), ('first_seed', 400000001), ('canonical_index', 1)]:
            mutated = copy.deepcopy(value)
            mutated['arms'][0][field] = wrong
            with self.assertRaises(ValueError):
                manifest.expected_keys(mutated)
        value['arms'][0]['config_overrides']['unknown_field'] = True
        with self.assertRaises(ValueError):
            manifest.expected_keys(value)

    def test_all_source_slots_remain_even_without_readings(self):
        table = source.fixture_table('absent_source_point')
        self.assertEqual(len(source.validate_table(table)['slots']), 105)
        table['slots'][1]['id'] = table['slots'][0]['id']
        with self.assertRaises(ValueError):
            source.validate_table(table)


if __name__ == '__main__':
    unittest.main()

class UnregisteredModeTests(unittest.TestCase):
    def test_native_fixture_is_bounded_and_never_registered(self):
        value = manifest.build_unregistered_manifest(source.fixture_table(), None,
            configurations=[{'width': 2, 'height': 2, 'horizon_periods': 2}], sessions=2)
        self.assertEqual(value['execution_mode'], 'fixture')
        self.assertEqual(len(manifest.expected_keys(value)), 2)
        self.assertGreaterEqual(value['arms'][0]['first_seed'], 400000000)
        with self.assertRaises(ValueError):
            manifest.build_unregistered_manifest(source.fixture_table(), None,
                configurations=[{'width': 5, 'height': 2, 'horizon_periods': 2}])


class NativeRosterBindingTests(unittest.TestCase):
    def test_native_registered_roster_digest_matches_independent_fixed_ids_and_seed_states(self):
        import hashlib
        import re
        from pathlib import Path
        import numpy as np
        from survey.democratic_peace.methods import canonical_bytes
        mechanisms = ('tagging', 'alliances', 'collective_security')
        densities = ('0', '0_05', '0_1', '0_15', '0_2', '0_25', '0_3', '0_4', '0_5', '0_6', '0_7', '1')
        names = [f'predictive.source.fig{figure}.{mechanism}.density{density}'
            for figure in (9, 10, 11) for mechanism in mechanisms
            for density in (densities[1:] if figure == 10 else densities)]
        contrasts = [f'primary.{contrast}.density{density}' for density in ('0_1', '0_3')
            for contrast in ('alliances_minus_tagging', 'security_minus_alliances', 'security_minus_tagging')]
        contrasts += [f'secondary.mobile0_15_minus_0_85.{mechanism}.density{density}'
            for density in ('0_1', '0_3') for mechanism in mechanisms]
        names += [prefix + name for name in contrasts for prefix in ('permutation.', 'bootstrap.')]
        independently_declared = [{'index': i, 'id': name, 'root_entropy': 2026100302,
            'spawn_key': [i], 'state_u32': np.random.SeedSequence(2026100302,
                spawn_key=(i,)).generate_state(4).tolist()} for i, name in enumerate(names)]
        self.assertEqual(len(independently_declared), 129)
        self.assertEqual(manifest.analysis_jobs(None), independently_declared)
        rust = (Path(__file__).resolve().parents[1] / 'src/bin/democratic_peace.rs').read_text()
        declared = re.search(r'const REGISTERED_ANALYSIS_JOBS_SHA256:\s*&str\s*=\s*"([0-9a-f]{64})"', rust)
        self.assertIsNotNone(declared)
        self.assertEqual(declared.group(1), hashlib.sha256(canonical_bytes(independently_declared)).hexdigest())
