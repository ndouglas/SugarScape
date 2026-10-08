"""Regression for relative artifact output resolving the adjacent audit receipt."""
import json
import os
from pathlib import Path
import tempfile
import unittest
import render_actual


class RelativePathsTests(unittest.TestCase):
    def test_relative_output_loads_adjacent_audit_after_full_static_scan(self):
        frames = [dict(tick=tick,roles=[],actions=[],choices=[],observations=[],deaths=[],restrictions={})
                  for tick in range(65)]
        fixture = dict(schema='fixture',census={},cells=[],endpoints=[],estimates=[],
                       biological_groups=[dict(id='fixture',members=['sham-ambiguous-seen-off-route-cost0-m0 seed 20001'],frames=frames)],
                       repeated_endpoints=[],effective_aliases=[])
        with tempfile.TemporaryDirectory() as temporary:
            study = Path(temporary)
            artifacts = study / 'actual-artifacts'
            artifacts.mkdir()
            source = study / 'source.json'
            source.write_text(json.dumps(fixture))
            # Deliberately mismatched audit: finding this receipt is the behavior tested.
            (study / 'measurement-data-audit.json').write_text(json.dumps(dict(
                analysis_identity=dict(sha256='0'*64,bytes=0))))
            original = Path.cwd()
            try:
                os.chdir(artifacts)
                try:
                    with self.assertRaisesRegex(ValueError, 'authorized complete-data audit'):
                        render_actual.render_complete(source, Path('figures'))
                except FileNotFoundError as error:
                    self.fail(f'relative output lost adjacent audit receipt: {error}')
            finally:
                os.chdir(original)


if __name__ == '__main__':
    unittest.main()
