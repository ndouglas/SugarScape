"""Exercise Python entry points without constructing or advancing any world."""
import hashlib,json,subprocess,sys,tempfile,unittest
from pathlib import Path
from survey.geosim import manifest

ROOT=Path(__file__).resolve().parents[2]
TABLE=ROOT/'docs/superpowers/specs/2026-10-03-geosim-source-table.json'

class CliTests(unittest.TestCase):
    def test_freeze_command_binds_synthetic_source_tree_only(self):
        from survey.geosim.test_provenance import source_tree
        from survey.geosim.records import verify_source_inventory
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_tree(root);m=root/'provisional.json';out=root/'frozen.json'
            data=TABLE.read_bytes();m.write_text(json.dumps(manifest.build_manifest(json.loads(data),data)))
            result=subprocess.run([sys.executable,'-m','survey.geosim.provenance','--manifest',str(m),'--source',str(root/'docs/superpowers/specs/2026-10-03-geosim-source-table.json'),
                                   '--source-root',str(root),'--output',str(out)],cwd=ROOT,capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            frozen=json.loads(out.read_bytes())
            self.assertEqual(frozen['provenance_status'],'frozen')
            self.assertEqual(verify_source_inventory(root,frozen['source_inventory']),frozen['source_inventory_sha256'])
    def test_manifest_command_writes_exact_source_hash_and_provisional_workload(self):
        with tempfile.TemporaryDirectory() as tmp:
            out=Path(tmp)/'manifest.json'
            result=subprocess.run([sys.executable,'-m','survey.geosim.manifest','--source',str(TABLE),'--output',str(out)],cwd=ROOT,capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            self.assertTrue(out.exists())
            m=json.loads(out.read_bytes())
            self.assertEqual(m['source_table_sha256'],hashlib.sha256(TABLE.read_bytes()).hexdigest())
            self.assertEqual(m['provenance_status'],'provisional_unfrozen')
            self.assertEqual(len(manifest.expected_keys(m)),1490)
    def test_analysis_command_rejects_unfrozen_data_before_writing_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);m=root/'manifest.json';m.write_text(json.dumps(manifest.build_manifest(json.loads(TABLE.read_bytes()),TABLE.read_bytes())))
            raw=root/'sessions.jsonl';raw.write_text('');unused=root/'unused.json';unused.write_text('{}');prefix=root/'findings'
            result=subprocess.run([sys.executable,'-m','survey.geosim.analysis','--manifest',str(m),'--source',str(TABLE),'--sessions',str(raw),
                                   '--resolved',str(unused),'--build-receipt',str(unused),'--binary',str(unused),'--source-root',str(ROOT),'--output',str(prefix)],
                                   cwd=ROOT,capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('frozen',result.stderr)
            self.assertFalse(prefix.with_suffix('.json').exists())

if __name__=='__main__':unittest.main()
