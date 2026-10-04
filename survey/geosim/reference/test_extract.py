"""Behavior checks for the reference-only binary extraction recipe."""
import hashlib
import io
from pathlib import Path
import tempfile
import unittest
import zipfile

from extract import extract


class ExtractionTests(unittest.TestCase):
    def fixture(self, directory, class_member='classes/example/Model.class'):
        nested = io.BytesIO()
        with zipfile.ZipFile(nested, 'w') as model:
            model.writestr(class_member, b'class bytes')
            model.writestr('src/example/Model.java', b'author source')
        archive = directory / 'archive.zip'
        with zipfile.ZipFile(archive, 'w') as outer:
            outer.writestr('lib/runtime.jar', b'jar bytes')
            outer.writestr('source/Framework.java', b'author source')
            outer.writestr('models/geosim2-0.9.0.zip', nested.getvalue())
        return archive, hashlib.sha256(archive.read_bytes()).hexdigest()

    def test_extracts_only_dependencies_and_model_binaries_with_receipt(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            archive, digest = self.fixture(root)
            receipt = extract(archive, root / 'out', digest)
            files = {p.relative_to(root / 'out').as_posix() for p in (root / 'out').rglob('*') if p.is_file()}
            self.assertEqual(files, {'lib/runtime.jar', 'geosim2/classes/example/Model.class', 'extraction-receipt.json'})
            self.assertEqual(receipt['archive_sha256'], digest)
            self.assertEqual(len(receipt['files']), 2)

    def test_wrong_archive_hash_fails_before_writing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            archive, _ = self.fixture(root)
            with self.assertRaisesRegex(ValueError, 'SHA256 mismatch'):
                extract(archive, root / 'out', '0' * 64)
            self.assertFalse((root / 'out').exists())

    def test_unsafe_model_path_fails_before_writing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            archive, digest = self.fixture(root, 'classes/../../../escape.class')
            with self.assertRaisesRegex(ValueError, 'Unsafe archive member'):
                extract(archive, root / 'out', digest)
            self.assertFalse((root / 'out').exists())


if __name__ == '__main__':
    unittest.main()
