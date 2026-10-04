"""Reference wrapper checks; no product mechanics or fitted statistics."""
import json
import os
import re
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[3]
RUNTIME = ROOT / 'survey/out/geosim-reference-runtime'
CP = os.pathsep.join([str(RUNTIME / 'wrapper-classes'), str(RUNTIME / 'lib/*'), str(RUNTIME / 'geosim2/classes')])

class ArchivedLaunchTests(unittest.TestCase):
    def run_probe(self, periods):
        proc = subprocess.run(['java', '-Djava.awt.headless=true', '-cp', CP, 'GeoSimReferenceProbe', str(periods)], text=True, capture_output=True, timeout=60)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        def unique_keys(pairs):
            result = {}
            for key, value in pairs:
                self.assertNotIn(key, result, f'Duplicate reference field: {key}')
                result[key] = value
            return result
        rows = [json.loads(value, object_pairs_hook=unique_keys) for value in re.findall(r'REFERENCE (\{[^\n]+\})', proc.stdout)]
        self.assertTrue(rows, 'No reference completion output')
        self.assertEqual((rows[-1]['type'], rows[-1]['time']), ('completed', periods))
        return rows

    def test_wrapper_propagates_lab_time_and_initializes_founders(self):
        rows = self.run_probe(1)
        self.assertEqual([(r['time'], r['sovereigns']) for r in rows if r['type'] == 'snapshot'], [(0, 200), (1, 200)])

    def test_wrapper_survives_first_export_without_gui_collector(self):
        rows = self.run_probe(600)
        self.assertEqual(rows[-1]['type'], 'completed')
        self.assertEqual(rows[-1]['time'], 600)
        self.assertGreater(sum(r['type'] == 'export' for r in rows), 0)

if __name__ == '__main__':
    unittest.main()
