"""An internally consistent digest cannot excuse omitted scientific sources."""
import hashlib,json,tempfile,unittest
from pathlib import Path
from survey.geosim import provenance,records,manifest

TABLE=Path(__file__).resolve().parents[2]/'docs/superpowers/specs/2026-10-03-geosim-source-table.json'

def source_tree(root):
    for name in provenance.REQUIRED_FILES:
        path=root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'fixture source\n')
    for name in ['crates/sugarscape-core/src/lib.rs','crates/sugarscape-core/src/geosim/mod.rs','survey/geosim/provenance.py']:
        path=root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'abc')
    (root/provenance.TABLE).write_bytes(TABLE.read_bytes())

class ProvenanceTests(unittest.TestCase):
    def test_inventory_is_sorted_and_binds_actual_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_tree(root);inventory=provenance.source_inventory(root)
            self.assertEqual([e['path'] for e in inventory],sorted(e['path'] for e in inventory))
            entry=next(e for e in inventory if e['path']=='crates/sugarscape-core/src/lib.rs')
            self.assertEqual(entry,{'path':'crates/sugarscape-core/src/lib.rs','bytes':3,'sha256':hashlib.sha256(b'abc').hexdigest()})
            self.assertTrue(records.verify_source_inventory(root,inventory))
    def test_empty_truncated_and_unsorted_inventory_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_tree(root);inventory=provenance.source_inventory(root)
            for forged in ([],inventory[1:],list(reversed(inventory))):
                with self.assertRaises(ValueError):records.verify_source_inventory(root,forged)
    def test_process_outputs_are_excluded_but_new_source_is_required(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_tree(root);before=provenance.source_inventory(root)
            for name in ['survey/geosim/IMPLEMENTATION_PLAN.md','survey/geosim/reference/raw/x.py','survey/out/manifest.json','docs/superpowers/plans/packet/replay.py',
                         'docs/superpowers/specs/2026-10-03-geosim-findings.md','docs/superpowers/specs/2026-10-03-geosim-provenance.json']:
                path=root/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'ignored')
            self.assertEqual(provenance.source_inventory(root),before)
            (root/'survey/geosim/new_estimator.py').write_bytes(b'new')
            with self.assertRaisesRegex(ValueError,'omits'):records.verify_source_inventory(root,before)
    def test_changed_normative_audit_fails_prior_binding(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_tree(root);inventory=provenance.source_inventory(root)
            (root/'docs/superpowers/specs/2026-10-03-geosim-artifact-audit.md').write_bytes(b'changed')
            with self.assertRaisesRegex(ValueError,'SHA256'):records.verify_source_inventory(root,inventory)
    def test_missing_required_dependency_or_symlink_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_tree(root);path=root/'survey/geosim/requirements.txt';path.unlink()
            with self.assertRaisesRegex(ValueError,'missing'):provenance.source_inventory(root)
            path.symlink_to(root/'Cargo.toml')
            with self.assertRaisesRegex(ValueError,'symlink'):provenance.source_inventory(root)
    def test_freeze_checks_exact_provisional_population_and_source_payload(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_tree(root);data=TABLE.read_bytes();m=manifest.build_manifest(json.loads(data),data)
            frozen=provenance.freeze_manifest(m,root,data)
            self.assertEqual(frozen['provenance_status'],'frozen')
            self.assertEqual(m['provenance_status'],'provisional_unfrozen')
            self.assertEqual(records.verify_source_inventory(root,frozen['source_inventory']),frozen['source_inventory_sha256'])
            m['arms'][0]['config_overrides']['shock']=3
            with self.assertRaisesRegex(ValueError,'provisional'):provenance.freeze_manifest(m,root,data)

if __name__=='__main__':unittest.main()
