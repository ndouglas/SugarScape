import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path('../figure-prep').resolve()))
import plot_p4
from test_plot_p4 import synthetic_analysis
from stream_p4 import load


class BindingTests(unittest.TestCase):
    def test_projection_render_binds_full_original_and_distinct_projection_bytes(self):
        source = json.dumps(synthetic_analysis(), indent=2).encode()
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'source.json'
            path.write_bytes(source)
            projection, binding = load(path)
            projected = json.dumps(projection).encode()
            try:
                result = plot_p4.export(projected, Path(temporary) / 'figures',
                                        synthetic=True, source_binding=binding)
            except TypeError as error:
                self.fail(f'full-source projection binding export missing: {error}')
            chart = json.loads((Path(temporary) / 'figures/chart-inputs.json').read_text())
            self.assertEqual(chart['analysis_sha256'], hashlib.sha256(source).hexdigest())
            self.assertEqual(chart['presentation_projection']['projection_sha256'], hashlib.sha256(projected).hexdigest())
            self.assertEqual(result['source_analysis_identity']['bytes'], len(source))


if __name__ == '__main__':
    unittest.main()
