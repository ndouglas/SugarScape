import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from survey.democratic_peace.manifest import contrast_definitions

HELPER = Path(__file__).resolve().parents[2] / 'docs/superpowers/plans/2026-10-03-democratic-peace/amendments/2026-10-04-task4-reporting/derive_findings.py'


def synthetic_report():
    report = {'precision_registered': False, 'histories': [{'complete': True, 'value': .25}],
              'source': {'interval': [.1, .2], 'original_census': {'complete_count': 30}},
              'draws': {'source': 100000}, 'analysis_jobs': [{'index': 0}]}
    for family in ('primary', 'secondary'):
        components = []
        for definition in contrast_definitions():
            if definition['family'] != family:
                continue
            components.append({**definition, 'left_census': {'registered_count': 100,
                'missing_count': 100, 'received_count': 0, 'complete_count': 0},
                'right_census': {'registered_count': 100, 'missing_count': 100,
                'received_count': 0, 'complete_count': 0}, 'estimate': None,
                'p': None, 'holm_p': None, 'interval': None, 'result': None,
                'unavailable_reason': 'precision_not_registered', 'verdict': 'Unresolved'})
        report[family + '_contrasts'] = {'components': components, 'aggregate_verdict': 'Unresolved'}
    return report


class ReportingAmendmentTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        spec = importlib.util.spec_from_file_location('dp_reporting_amendment', HELPER)
        cls.helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.helper)

    def test_derivation_corrects_exactly_24_references_without_mutating_input(self):
        original = synthetic_report(); retained = copy.deepcopy(original)
        amended = self.helper.derive_report(original, {'date': '2026-10-04'})
        self.assertEqual(original, retained)
        count = 0
        for family in ('primary', 'secondary'):
            for component in amended[family + '_contrasts']['components']:
                self.assertIsNone(component['left_census'])
                self.assertIsNone(component['right_census'])
                self.assertEqual(component['required_precision_sample_size'], 100)
                self.assertEqual(component['unavailable_reason'], 'precision_not_registered')
                count += 2
        self.assertEqual(count, 24)
        self.assertEqual(self.helper.prove_invariance(original, amended)['corrected_census_references'], 24)

    def test_deterministic_serialization_keeps_every_empirical_value(self):
        original = synthetic_report(); metadata = {'date': '2026-10-04'}
        first = self.helper.derive_report(original, metadata)
        second = self.helper.derive_report(original, metadata)
        self.assertEqual(json.dumps(first, sort_keys=True), json.dumps(second, sort_keys=True))
        self.assertEqual(first['histories'], original['histories'])
        self.assertEqual(first['source'], original['source'])
        self.assertEqual(first['draws'], original['draws'])
        self.assertEqual(first['analysis_jobs'], original['analysis_jobs'])

    def test_empirical_changes_outside_the_census_fields_are_rejected(self):
        original = synthetic_report(); amended = self.helper.derive_report(original, {})
        amended['histories'][0]['value'] = .75
        with self.assertRaisesRegex(ValueError, 'empirical'):
            self.helper.prove_invariance(original, amended)

    def test_registered_precision_and_nonempty_reference_census_are_rejected(self):
        original = synthetic_report(); original['precision_registered'] = True
        with self.assertRaises(ValueError):
            self.helper.derive_report(original, {})
        original['precision_registered'] = False
        original['primary_contrasts']['components'][0]['left_census']['received_count'] = 1
        with self.assertRaises(ValueError):
            self.helper.derive_report(original, {})

    def test_aliases_reject_before_any_output_or_directory_write(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); protected = root/'original.json'; protected.write_bytes(b'sentinel')
            for kind in ('direct', 'symlink', 'hardlink'):
                alias = root/(kind+'.json')
                if kind == 'direct': alias = protected
                elif kind == 'symlink': alias.symlink_to(protected)
                else: os.link(protected, alias)
                output = root/'uncreated/report.md'
                with self.subTest(kind=kind), self.assertRaisesRegex(ValueError, 'alias'):
                    self.helper.validate_output_paths([protected], [output, alias])
                self.assertFalse(output.parent.exists())
                self.assertEqual(protected.read_bytes(), b'sentinel')

    def test_new_outputs_inside_original_study_are_rejected_before_writes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); study = root/'study'; study.mkdir()
            output = study/'new/report.json'
            with self.assertRaisesRegex(ValueError, 'immutable'):
                self.helper.validate_output_paths([], [output], [study])
            self.assertFalse(output.parent.exists())

    def test_two_output_paths_cannot_alias_each_other(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'new.json'
            with self.assertRaisesRegex(ValueError, 'alias'):
                self.helper.validate_output_paths([], [path, path])
            self.assertFalse(path.exists())
