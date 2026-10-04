import unittest
from survey.democratic_peace import analysis, manifest, source


class AnalysisTests(unittest.TestCase):
    def test_registered_report_cannot_accept_small_fixture_draws(self):
        value = manifest.build_manifest(source.fixture_table())
        with self.assertRaisesRegex(ValueError, 'fixture'):
            analysis.report(value, source.fixture_table(), {}, fixture_draws=10)

    def test_missing_fixture_keys_are_retained_and_never_registered_findings(self):
        table = source.fixture_table('absent_source_point')
        value = manifest.build_unregistered_manifest(table, None,
            configurations=[{'width': 2, 'height': 2, 'horizon_periods': 2}], sessions=2)
        result = analysis.report(value, table, {}, fixture_draws=10)
        self.assertEqual(result['classification'], 'synthetic_fixture')
        self.assertEqual(result['history_status_counts'], {'missing': 2})
        self.assertEqual(len(result['histories']), 2)
        self.assertEqual(len(result['source']['targets']), 105)
        self.assertEqual(result['primary_contrasts']['family_size'], 6)
        self.assertEqual(result['secondary_contrasts']['family_size'], 6)

    def test_unregistered_keys_and_changed_jobs_are_rejected(self):
        table = source.fixture_table()
        value = manifest.build_unregistered_manifest(table, None,
            configurations=[{'width': 2, 'height': 2, 'horizon_periods': 2}])
        with self.assertRaises(ValueError):
            analysis.report(value, table, {('unknown', 1): {}}, fixture_draws=10)
        value['analysis_jobs'][0]['state_u32'][0] += 1
        with self.assertRaises(ValueError):
            analysis.report(value, table, {}, fixture_draws=10)


if __name__ == '__main__':
    unittest.main()


class OutputProtectionTests(unittest.TestCase):
    def test_both_output_destinations_are_checked_without_creating_directories(self):
        import os
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source = root/'input.json'; source.write_bytes(b'input sentinel')
            alias = root/'alias.json'; os.link(source, alias)
            untouched = root/'uncreated/report.md'
            with self.assertRaisesRegex(ValueError, 'output aliases'):
                analysis.validate_output_destinations([source], [untouched, alias])
            self.assertEqual(source.read_bytes(), b'input sentinel')
            self.assertFalse(untouched.parent.exists())
            analysis.validate_output_destinations([source], [root/'new/report.json', root/'new/report.md'])
            self.assertFalse((root/'new').exists())
