import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from survey.democratic_peace import run


class RunTests(unittest.TestCase):
    def dummy(self, root, code=0):
        path = root / 'dummy-native'
        path.write_text('#!/usr/bin/env python3\nimport json,sys\nprint(json.dumps(sys.argv[1:]))\nsys.exit(' + str(code) + ')\n')
        path.chmod(0o755)
        return path

    def test_validation_never_supplies_execution_output_or_constructs_world(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            result = run.run_native(self.dummy(root), root / 'manifest', validate=True,
                resolved_out=root / 'resolved', source_root=root)
            args = json.loads(result.stdout)
            self.assertIn('--validate-only', args)
            self.assertIn('--repo', args)
            self.assertNotIn('--out', args)
            self.assertFalse((root / 'resolved').exists())

    def test_execution_requires_explicit_receipt_and_output(self):
        with self.assertRaises(ValueError):
            run.run_native('/absent', '/absent', resolved_out='/absent')

    def test_nonzero_native_exit_is_not_reported_as_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with self.assertRaises(subprocess.CalledProcessError):
                run.run_native(self.dummy(root, 7), root / 'manifest', validate=True,
                    resolved_out=root / 'resolved', source_root=root)

    def test_failed_build_never_creates_receipt(self):
        with self.assertRaises(ValueError):
            run.make_build_receipt('/absent', '/absent', '/absent', prebuild_inventory_sha256='0' * 64,
                build_command=['cargo'], build_exit_code=1, toolchain={}, lockfile_paths=[])


if __name__ == '__main__':
    unittest.main()
