"""Real temporary Git repositories exercise replay and rejection before mutation."""
import hashlib,json,subprocess,sys,tempfile,unittest
from pathlib import Path
REPLAY=Path(__file__).with_name('replay.py')
def digest(data):return hashlib.sha256(data).hexdigest()
def spec(data,mode='100644'):return {'sha256':digest(data),'mode':mode}
class ReplayTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.target=self.root/'target';self.assets=self.root/'packet'
        self.target.mkdir();self.assets.mkdir();self.git('init','-q')
        self.git('config','user.name','Replay Fixture');self.git('config','user.email','fixture@example.invalid')
        (self.target/'README.md').write_bytes(b'base\n');(self.target/'docs').mkdir();(self.target/'docs/source.md').write_bytes(b'protected\n')
        self.git('add','.');self.git('commit','-qm','base');base=self.git('rev-parse','HEAD').strip()
        checkpoints=[{'README.md':spec(b'base\n')}];patches=[]
        for i,data in enumerate((b'tests\n',b'implementation\n')):
            (self.target/'README.md').write_bytes(data)
            patch=self.git('diff','--binary','--full-index').encode();name=f'{i+1}.patch';(self.assets/name).write_bytes(patch)
            after={'README.md':spec(data)}
            patches.append({'name':name,'sha256':digest(patch),'changes':{'README.md':{'before':checkpoints[-1]['README.md'],'after':after['README.md']}}})
            checkpoints.append(after);self.git('add','.');self.git('commit','-qm',name)
        self.git('checkout','-q','--detach',base)
        self.manifest={'version':1,'complete':True,'base':base,'product_roots':['web/'],'product_files':['README.md'],'excluded_prefixes':[],'excluded_names':['IMPLEMENTATION_PLAN.md'],'metadata_prefixes':[],'metadata_files':[],'protected':{'docs/source.md':spec(b'protected\n')},'checkpoints':checkpoints,'patches':patches}
        self.save()
    def git(self,*args):return subprocess.check_output(['git',*args],cwd=self.target,text=True,stderr=subprocess.PIPE)
    def save(self):(self.assets/'manifest.json').write_text(json.dumps(self.manifest))
    def run_replay(self,*args):return subprocess.run([sys.executable,str(REPLAY),'--manifest',str(self.assets/'manifest.json'),'--target',str(self.target),*args],text=True,capture_output=True)
    def rejected(self,fragment):
        result=self.run_replay();self.assertNotEqual(result.returncode,0,result.stdout)
        self.assertIn(fragment,result.stderr)
        self.assertEqual((self.target/'README.md').read_bytes(),b'base\n','Rejected packet must not apply first patch')
    def test_replays_test_checkpoint_then_resumes_exact_implementation(self):
        first=self.run_replay('--through','1');self.assertEqual(first.returncode,0,first.stderr)
        self.assertEqual((self.target/'README.md').read_bytes(),b'tests\n')
        final=self.run_replay();self.assertEqual(final.returncode,0,final.stderr)
        self.assertEqual((self.target/'README.md').read_bytes(),b'implementation\n')
        verify=self.run_replay('--verify-only');self.assertEqual(verify.returncode,0,verify.stderr)
    def test_late_patch_hash_tampering_fails_before_first_patch(self):
        with (self.assets/'2.patch').open('ab') as f:f.write(b'\nchanged\n')
        self.rejected('patch SHA256')
    def test_wrong_preimage_is_not_overwritten(self):
        (self.target/'README.md').write_bytes(b'local work\n');result=self.run_replay()
        self.assertNotEqual(result.returncode,0);self.assertIn('checkpoint',result.stderr)
        self.assertEqual((self.target/'README.md').read_bytes(),b'local work\n')
    def test_executable_bit_is_a_preimage_contract(self):
        (self.target/'README.md').chmod(0o755);self.rejected('checkpoint')
    def test_protected_reference_bytes_cannot_change(self):
        (self.target/'docs/source.md').write_bytes(b'altered source\n');self.rejected('protected')
    def test_undeclared_product_path_is_rejected(self):
        (self.target/'web').mkdir();(self.target/'web/extra.ts').write_text('extra')
        self.rejected('checkpoint')
    def test_manifest_traversal_is_rejected_before_write(self):
        self.manifest['protected']['../outside']=spec(b'x');self.save();self.rejected('Unsafe path')
    def test_symlink_ancestor_is_rejected_before_write(self):
        outside=self.root/'outside';outside.mkdir();(outside/'source.md').write_bytes(b'protected\n')
        (self.target/'web').symlink_to(outside,target_is_directory=True)
        self.manifest['protected']['web/source.md']=spec(b'protected\n');self.save()
        self.rejected('symlink');self.assertEqual((outside/'source.md').read_bytes(),b'protected\n')
    def test_patch_cannot_modify_an_undeclared_contained_path(self):
        self.git('checkout','-q','--detach',self.manifest['base'])
        (self.target/'README.md').write_bytes(b'tests\n');(self.target/'docs/source.md').write_bytes(b'corrupt\n')
        patch=self.git('diff','--binary','--full-index').encode();(self.assets/'1.patch').write_bytes(patch)
        self.manifest['patches'][0]['sha256']=digest(patch);self.save()
        (self.target/'README.md').write_bytes(b'base\n');(self.target/'docs/source.md').write_bytes(b'protected\n')
        self.rejected('declared changes')
    def test_symlink_mode_in_patch_is_rejected_before_write(self):
        p=self.assets/'1.patch';data=p.read_bytes()+b'\nnew file mode 120000\n';p.write_bytes(data)
        self.manifest['patches'][0]['sha256']=digest(data);self.save();self.rejected('Unsupported patch mode')
    def test_incomplete_packet_requires_explicit_prefix(self):
        self.manifest['complete']=False;self.save();self.rejected('incomplete')
        result=self.run_replay('--through','1');self.assertEqual(result.returncode,0,result.stderr)
    def test_only_named_nonproduct_cache_symlinks_are_allowed(self):
        outside=self.root/'cache';outside.mkdir()
        (self.target/'papers').symlink_to(outside,target_is_directory=True)
        self.manifest['metadata_files']=['papers'];self.save()
        result=self.run_replay();self.assertEqual(result.returncode,0,result.stderr)
    def test_unnamed_cache_symlink_is_rejected_before_write(self):
        outside=self.root/'cache';outside.mkdir()
        (self.target/'other-cache').symlink_to(outside,target_is_directory=True)
        self.rejected('symlink')
    def test_changed_final_file_mode_is_rejected_by_verify_only(self):
        result=self.run_replay();self.assertEqual(result.returncode,0,result.stderr)
        (self.target/'README.md').chmod(0o755)
        result=self.run_replay('--verify-only');self.assertNotEqual(result.returncode,0)
        self.assertIn('checkpoint',result.stderr)
    def test_patch_asset_traversal_is_rejected_before_write(self):
        self.manifest['patches'][0]['name']='../escape.patch';self.save();self.rejected('Unsafe path')
    def test_verify_only_checks_identical_bytes_on_an_independent_history(self):
        self.git('checkout','--orphan','independent')
        (self.target/'README.md').write_bytes(b'implementation\n')
        self.git('add','.');self.git('commit','-qm','independent equivalent tree')
        result=self.run_replay('--verify-only');self.assertEqual(result.returncode,0,result.stderr)
    def test_apply_requires_source_base_ancestry_even_on_an_equivalent_tree(self):
        self.git('checkout','--orphan','independent')
        self.git('add','.');self.git('commit','-qm','independent base bytes')
        result=self.run_replay();self.assertNotEqual(result.returncode,0)
        self.assertEqual((self.target/'README.md').read_bytes(),b'base\n')
    def test_verify_only_requires_the_requested_final_checkpoint(self):
        result=self.run_replay('--verify-only');self.assertNotEqual(result.returncode,0)
        self.assertEqual((self.target/'README.md').read_bytes(),b'base\n')
if __name__=='__main__':unittest.main()
