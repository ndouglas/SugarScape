import os
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from run import run_native


class NativeBoundaryTests(unittest.TestCase):
    def test_exact_manifest_hash_and_space_containing_paths_reach_executable(self):
        with tempfile.TemporaryDirectory(prefix='polarity fixture ') as directory:
            root=Path(directory)
            manifest=root/'frozen manifest.json'
            manifest.write_text('{"arms":[]}\n')
            executable=root/'fake-native'
            executable.write_text('#!/usr/bin/env python3\nimport sys,json\nprint(json.dumps(sys.argv[1:]))\n')
            executable.chmod(0o755)
            out=root/'raw sessions.jsonl'
            result=run_native(executable,manifest,out,arm='precision.',validate=True)
            self.assertEqual(result.returncode,0)
            argv=json.loads(result.stdout)
            self.assertEqual(argv[argv.index('--manifest-sha256')+1],
                             hashlib.sha256(manifest.read_bytes()).hexdigest())
            self.assertEqual(argv[argv.index('--manifest')+1],str(manifest.resolve()))
            self.assertEqual(argv[argv.index('--out')+1],str(out.resolve()))
            self.assertIn('--validate',argv)

    def test_native_failure_is_returned_for_caller_to_propagate(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)
            manifest=root/'m.json'
            manifest.write_text('{}')
            exe=root/'fail'
            exe.write_text('#!/bin/sh\nexit 2\n')
            exe.chmod(0o755)
            self.assertEqual(run_native(exe,manifest,root/'out').returncode,2)


    @unittest.skipUnless(os.environ.get('POLARITY_BINARY'),'requires built native binary')
    def test_actual_resume_rejects_corrupted_embedded_outcome_without_replacing_raw(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);manifest=root/'manifest.json';out=root/'raw.jsonl'
            manifest.write_text(json.dumps({'schema_version':1,'arms':[{'id':'peace','family':'original',
                'config':{'model':'polarity','predator_share':0,'horizon':3,'periods_per_tick':2},'first_seed':7,'sessions':1}]}))
            binary=os.environ['POLARITY_BINARY']
            self.assertEqual(run_native(binary,manifest,out).returncode,0)
            original=json.loads(out.read_text())
            for change in [{'seed':999},{'config':{}},{'periods':0,'attempted_period':1},
                           {'valid':'true'},{'terminal_category':'one'},{'finish_reason':'hegemony'}]:
                with self.subTest(change=change):
                    row=json.loads(json.dumps(original));row['outcome'].update(change)
                    out.write_text(json.dumps(row)+'\n');before=out.read_bytes()
                    result=run_native(binary,manifest,out)
                    self.assertEqual(result.returncode,2)
                    self.assertEqual(out.read_bytes(),before)

    @unittest.skipUnless(os.environ.get('POLARITY_BINARY'),'requires built native binary')
    def test_native_resolution_is_complete_authority_for_offline_report(self):
        from analysis import report
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);manifest=root/'manifest.json';out=root/'raw.jsonl';resolved=root/'resolved.json'
            m={'schema_version':1,'analysis_draws':99,'arms':[{'id':'peace','family':'original',
                'config':{'model':'polarity','predator_share':0,'horizon':3,'periods_per_tick':2},'first_seed':7,'sessions':1}]}
            manifest.write_text(json.dumps(m));binary=os.environ['POLARITY_BINARY']
            self.assertEqual(run_native(binary,manifest,out,validate=True,resolved_out=resolved).returncode,0)
            self.assertFalse(out.exists())
            self.assertEqual(run_native(binary,manifest,out).returncode,0)
            row=json.loads(out.read_text());authority=json.loads(resolved.read_text())
            clean=report(m,{},[row],manifest.read_bytes(),authority)
            self.assertEqual(clean['arms'][0]['verdict'],'Descriptive')
            row['config']['resource_policy']='floor_zero'
            wrong=report(m,{},[row],manifest.read_bytes(),authority)
            self.assertEqual(wrong['arms'][0]['verdict'],'Unresolved')

if __name__=='__main__':
    unittest.main()
