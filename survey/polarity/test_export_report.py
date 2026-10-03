"""The report exporter preserves registered values and explicit censored causes."""
import json
import unittest
import subprocess
import sys
import tempfile
from pathlib import Path
import test_reporting

from export_report import serialize_report


class ReportExportTests(unittest.TestCase):
    def test_round_trip_retains_censored_and_named_end_causes_and_verdicts(self):
        report = {
            'arms': [{'episode_end_causes': {None: 2, 'victory': 3}}],
            'findings': [{'verdict': 'Fails', 'result': {'estimate': -0.5}}],
            'raw_records': [{'outcome': {'episodes': [{'end_cause': None}]}}],
        }
        actual = json.loads(serialize_report(report))
        self.assertEqual(actual, {
            'arms': [{'episode_end_causes': {'null': 2, 'victory': 3}}],
            'findings': report['findings'],
            'raw_records': report['raw_records'],
        })

    def test_rejects_nonfinite_output_instead_of_relaxing_strict_json(self):
        with self.assertRaises(ValueError):
            serialize_report({'value': float('nan')})

    def test_both_entrypoints_export_identical_complete_reports(self):
        manifest, raw, records = test_reporting.ReportingTests().fixture()
        records[0]['outcome']['episodes'] = [{'end_cause': None, 'censored': True}]
        records[1]['outcome']['episodes'] = [{'end_cause': 'victory', 'censored': False}]
        resolved, records = test_reporting.complete_fixture(manifest, raw, records)
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            inputs = {'manifest': raw, 'source': b'{}',
                'resolved': json.dumps(resolved).encode(),
                'sessions': ''.join(json.dumps(r)+'\n' for r in records).encode()}
            for name, data in inputs.items():
                (directory/name).write_bytes(data)
            exports = []
            for script in ['analysis.py', 'export_report.py']:
                destination = directory/script
                args = [sys.executable, str(Path(__file__).with_name(script))]
                for name in inputs:
                    args += ['--'+name, str(directory/name)]
                args += ['--output', str(destination)]
                subprocess.run(args, check=True, capture_output=True, text=True)
                exports.append(json.loads(Path(str(destination)+'.json').read_text()))
            self.assertEqual(exports[0], exports[1])
            self.assertEqual(exports[0]['arms'][0]['episode_end_causes'], {'null': 1, 'victory': 1})


if __name__ == '__main__':
    unittest.main()
