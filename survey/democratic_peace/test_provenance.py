import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from survey.democratic_peace import provenance, records, manifest, source
from survey.democratic_peace.methods import canonical_bytes


def source_tree(root):
    for directory in ('crates/sugarscape-core/tests', 'survey/tests'):
        (root / directory).mkdir(parents=True, exist_ok=True)
    for name in provenance.REQUIRED_FILES:
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b'fixture source\n')
    for name in ['crates/sugarscape-core/src/lib.rs', 'survey/democratic_peace/provenance.py']:
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b'abc')


class ProvenanceTests(unittest.TestCase):
    def test_omitted_sources_and_changed_bytes_fail_even_with_consistent_digest(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source_tree(root)
            inventory = provenance.source_inventory(root)
            self.assertEqual(records.verify_source_inventory(root, inventory), hashlib.sha256(canonical_bytes(inventory)).hexdigest())
            with self.assertRaises(ValueError):
                records.verify_source_inventory(root, inventory[1:])
            (root / 'survey/democratic_peace/provenance.py').write_bytes(b'changed')
            with self.assertRaises(ValueError):
                records.verify_source_inventory(root, inventory)

    def test_registered_freeze_rejects_synthetic_or_pending_source_audit(self):
        table = source.fixture_table()
        data = json.dumps(table).encode()
        value = manifest.build_manifest(table, data)
        with self.assertRaisesRegex(ValueError, 'source audit'):
            provenance.freeze_manifest(value, Path('.'), data)

    def test_symlinks_are_not_scientific_source_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            source_tree(root)
            path = root / 'survey/democratic_peace/methods.json'
            path.unlink()
            path.symlink_to(root / 'Cargo.toml')
            with self.assertRaisesRegex(ValueError, 'symlink'):
                provenance.source_inventory(root)


if __name__ == '__main__':
    unittest.main()

class SourceReviewTests(unittest.TestCase):
    def test_actual_review_binds_slots_and_rejects_changed_envelope(self):
        root = Path(__file__).resolve().parents[2]
        table = records.strict_json((root / provenance.TABLE).read_bytes())
        provenance.verify_source_review(root, table)
        table['slots'][0]['interval'][1] += .0001
        with self.assertRaisesRegex(ValueError, 'slot digest'):
            provenance.verify_source_review(root, table)
