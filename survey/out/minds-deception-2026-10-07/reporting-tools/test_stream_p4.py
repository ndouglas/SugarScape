"""Static streaming/projection checks; no campaign input or simulation."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path('../figure-prep').resolve()))
from test_plot_p4 import synthetic_analysis
import stream_p4


class StreamingTests(unittest.TestCase):
    def test_projection_preserves_every_complete_chart_field_and_full_input_hash(self):
        fixture = synthetic_analysis()
        fixture['biological_groups'] = [dict(id='synthetic-group', members=['a'],
                                            frames=[dict(tick=0, unusual={'kept': ['all', 'diagnostics']})])]
        raw = json.dumps(fixture, indent=2).encode()
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / 'analysis.json'
            source.write_bytes(raw)
            groups = []
            result = stream_p4.load(source, group_callback=groups.append)
        self.assertIsInstance(result, tuple)
        projection, binding = result
        for key in ('schema', 'census', 'estimates', 'cells', 'endpoints', 'repeated_endpoints', 'effective_aliases'):
            self.assertEqual(projection[key], fixture[key])
        self.assertEqual(groups, fixture['biological_groups'])
        self.assertEqual(binding['sha256'], hashlib.sha256(raw).hexdigest())
        self.assertEqual(binding['bytes'], len(raw))
        self.assertEqual(binding['biological_groups'], 1)
        self.assertEqual(binding['frames'], 1)
        self.assertEqual(projection['fixture_classification'], 'synthetic_fixture')

    def test_rejects_duplicate_top_level_key(self):
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / 'analysis.json'
            source.write_text('{"schema":"a", "schema":"b"}')
            with self.assertRaisesRegex(ValueError, 'duplicate'):
                stream_p4.load(source)

    def test_missing_biological_groups_is_explicitly_rejected(self):
        fixture = synthetic_analysis()
        fixture.pop('biological_groups')
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / 'analysis.json'
            source.write_text(json.dumps(fixture))
            with self.assertRaisesRegex(ValueError, 'missing'):
                stream_p4.load(source)


if __name__ == '__main__':
    unittest.main()
