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
