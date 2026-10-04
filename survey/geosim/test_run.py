"""Boundary fixtures invoke only tiny dummy Python processes, never GeoSim periods."""
import hashlib,json,subprocess,tempfile,unittest
from pathlib import Path
from survey.geosim import run
from survey.geosim.methods import canonical_bytes

class RunTests(unittest.TestCase):
    def dummy(self,root,code=0):
        exe=root/'dummy-native';exe.write_text('#!/usr/bin/env python3\nimport json,sys\nprint(json.dumps(sys.argv[1:]))\nsys.exit('+str(code)+')\n');exe.chmod(0o755)
        return exe
    def test_runner_passes_exact_manifest_sha_and_explicit_native_paths(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);m=root/'manifest.json';m.write_bytes(b'{ "fixture": true }\n');exe=self.dummy(root)
            result=run.run_native(exe,m,root/'out.jsonl',validate=True,resolved_out=root/'resolved.json',source_root=root)
            args=json.loads(result.stdout)
            self.assertEqual(args[args.index('--manifest-sha256')+1],hashlib.sha256(m.read_bytes()).hexdigest())
            self.assertIn('--validate',args);self.assertIn('--resolved-out',args)
            self.assertFalse((root/'out.jsonl').exists())
    def test_nonzero_native_exit_is_not_reported_as_success(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);m=root/'manifest.json';m.write_text('{}');exe=self.dummy(root,7)
            with self.assertRaises(subprocess.CalledProcessError):run.run_native(exe,m,root/'out.jsonl',source_root=root)
    def test_receipt_binds_actual_binary_manifest_and_equal_prepost_source(self):
        with tempfile.TemporaryDirectory() as tmp:
            from survey.geosim.test_provenance import source_tree
            from survey.geosim.provenance import source_inventory
            root=Path(tmp);source_tree(root);inv=source_inventory(root)
            digest=hashlib.sha256(canonical_bytes(inv)).hexdigest();m=root/'manifest.json'
            m.write_text(json.dumps({'model':'geosim','provenance_status':'frozen','source_inventory':inv,'source_inventory_sha256':digest}))
            exe=self.dummy(root)
            receipt=run.make_build_receipt(m,exe,root,prebuild_inventory_sha256=digest,build_command=['cargo','build'],build_exit_code=0,
                      toolchain={'target':'fixture','rustc_version':'fixture','cargo_version':'fixture'},lockfile_paths=['Cargo.lock','survey/Cargo.lock'])
            self.assertEqual(receipt['binary_sha256'],hashlib.sha256(exe.read_bytes()).hexdigest())
            self.assertEqual(receipt['manifest_sha256'],hashlib.sha256(m.read_bytes()).hexdigest())
            with self.assertRaisesRegex(ValueError,'locks'):
                run.make_build_receipt(m,exe,root,prebuild_inventory_sha256=digest,build_command=['cargo','build'],build_exit_code=0,
                                      toolchain={'target':'fixture','rustc_version':'fixture','cargo_version':'fixture'},lockfile_paths=[])
            with self.assertRaises(ValueError):run.make_build_receipt(m,exe,root,prebuild_inventory_sha256='0'*64,build_command=['cargo'],build_exit_code=0,toolchain={},lockfile_paths=[])
    def test_failed_build_cannot_produce_integrity_receipt(self):
        with self.assertRaises(ValueError):run.make_build_receipt(Path('absent'),Path('absent'),Path('.'),prebuild_inventory_sha256='0'*64,build_command=['cargo'],build_exit_code=1,toolchain={},lockfile_paths=[])

if __name__=='__main__':unittest.main()
