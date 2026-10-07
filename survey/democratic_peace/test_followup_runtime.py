import copy
import unittest
from survey.democratic_peace import followup, source
try:
    from survey.democratic_peace import followup_runtime as runtime
except ImportError:
    runtime = None


class FollowupRuntimeTests(unittest.TestCase):
    def setUp(self):
        self.assertIsNotNone(runtime, 'prospective runtime gates are not implemented')

    def measurement(self):
        return {'cpu_seconds': 1., 'wall_seconds': 1.5, 'peak_rss_bytes': 100000,
            'command': ['synthetic'], 'exit_code': 0, 'source_inventory_sha256': 'a'*64,
            'binary_sha256': 'b'*64, 'toolchain': {'python':'3.13.5','numpy':'2.4.6'},
            'output_sha256': 'c'*64, 'classification': 'unregistered_runtime_only'}

    def test_probe_roster_is_predeclared_and_cannot_enter_registered_phase(self):
        table=source.fixture_table();data=followup.canonical_bytes(table)
        left=runtime.probe_manifest(table,data,'literal_precision')
        right=runtime.probe_manifest(table,data,'prose_precision')
        self.assertEqual((left['arms'][0]['first_seed'], right['arms'][0]['first_seed']), (420000001,430000001))
        self.assertEqual(len(left['arms']),8)
        self.assertEqual(left['execution_mode'],'runtime_probe')
        self.assertFalse(left['precision_registered'])
        self.assertEqual(right['arms'][0]['config_overrides']['probability_direction'],'prose_increasing')

    def test_combined_forecast_counts_native_analysis_writer_and_double_factor(self):
        probes=[self.measurement() for _ in range(8)]
        jobs={kind:self.measurement() for kind in runtime.ANALYSIS_TYPES}
        forecast=runtime.phase_forecast(probes,jobs,self.measurement())
        expected=2*(10800+105+12+1)
        self.assertEqual(forecast['cpu_seconds'],expected)
        self.assertEqual(forecast['forecast_multiplier'],2)
        self.assertTrue(forecast['within_budget'])

    def test_failed_missing_or_unbounded_timing_never_authorizes_subset(self):
        probes=[self.measurement() for _ in range(8)];jobs={k:self.measurement() for k in runtime.ANALYSIS_TYPES}
        with self.assertRaises(ValueError): runtime.phase_forecast(probes[:1],jobs,self.measurement())
        bad=copy.deepcopy(probes);bad[0]['exit_code']=1
        with self.assertRaises(ValueError): runtime.phase_forecast(bad,jobs,self.measurement())
        bad=copy.deepcopy(probes);bad[0]['peak_rss_bytes']=2**31+1
        self.assertFalse(runtime.phase_forecast(bad,jobs,self.measurement())['within_budget'])
        bad=copy.deepcopy(probes);bad[0]['cpu_seconds']=100.
        self.assertFalse(runtime.phase_forecast(bad,jobs,self.measurement())['within_budget'])

class ActivationChainTests(unittest.TestCase):
    """Complete synthetic evidence and a committed fixture repo; no native execution."""

    def setUp(self):
        from pathlib import Path
        import subprocess
        import tempfile
        from survey.democratic_peace import records
        from survey.democratic_peace.followup_registration import declaration, source_inventory
        from survey.democratic_peace.test_followup_evidence import RetainedWorkloadEvidenceTests, ROOT, write_json

        fixture=RetainedWorkloadEvidenceTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.write_json=write_json
        self.temporary=tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root=Path(self.temporary.name)
        inventory=source_inventory(ROOT)
        for entry in inventory:
            target=self.root/entry['path']
            target.parent.mkdir(parents=True,exist_ok=True)
            target.write_bytes((ROOT/entry['path']).read_bytes())
        self.values={};bindings=[]
        table=source.fixture_table();data=followup.canonical_bytes(table)
        for phase in followup.PHASES:
            value=followup.build_manifest(table,data,phase,followup.EXPECTED_HISTORICAL)
            value.update(source_inventory=inventory,
                source_inventory_sha256=fixture.evidence['source_inventory_sha256'],provenance_status='frozen')
            self.values[phase]=value
            bindings.append({'phase':phase,'reading':value['reading'],
                **dict.fromkeys(records.BINDING_FIELDS,'a'*64),
                'source_inventory_sha256':fixture.evidence['source_inventory_sha256'],
                'binary_sha256':fixture.evidence['binary_sha256'],
                'analysis_jobs_sha256':followup.sha(followup.canonical_bytes(value['analysis_jobs'])),
                'native_keys_sha256':followup.sha(followup.canonical_bytes(followup.expected_keys(value)))})
        benchmark=fixture.root/'fixture-summary.json'
        write_json(benchmark,fixture.evidence)
        self.declaration=declaration(bindings,followup.sha(benchmark.read_bytes()))
        self.declaration_path=self.root/'fixture-declaration.json'
        write_json(self.declaration_path,self.declaration)
        self.receipt=runtime.runtime_attestation(self.declaration_path,benchmark)
        self.runtime_path=fixture.root/'fixture-runtime.json'
        write_json(self.runtime_path,self.receipt)
        self.review={'schema_version':1,'study_protocol':followup.PROTOCOL,
            'decision':'accepted_prospective_followup','independent':True,'reviewer':'synthetic external reviewer',
            'registered_histories':0,'declaration_sha256':followup.sha(self.declaration_path.read_bytes()),
            'runtime_receipt_sha256':followup.sha(self.runtime_path.read_bytes())}
        self.review_path=fixture.root/'fixture-review.json'
        write_json(self.review_path,self.review)
        for args in (['init','--quiet'],['add','.'],
            ['-c','user.name=Synthetic Fixture','-c','user.email=fixture@example.invalid',
             '-c','commit.gpgsign=false','commit','--quiet','-m','commit synthetic activation authorities']):
            subprocess.run(['git',*args],cwd=self.root,check=True,capture_output=True)

    def activate(self,phase='literal_precision',binding=None):
        selected=next(p for p in self.declaration['phases'] if p['phase']==phase)
        return runtime.verify_activation(self.values[phase],selected if binding is None else binding,self.root,
            declaration_path=self.declaration_path,review_path=self.review_path,runtime_path=self.runtime_path)

    def test_complete_attested_and_independently_reviewed_chain_activates_both_phases(self):
        for phase in followup.PHASES:
            with self.subTest(phase=phase):
                self.assertEqual(self.activate(phase),self.declaration)

    def test_mixed_external_receipt_hashes_fail_at_chain_binding(self):
        for authority,field in [('review','declaration_sha256'),('review','runtime_receipt_sha256'),
            ('runtime','declaration_sha256'),('runtime','benchmark_evidence_sha256')]:
            value=copy.deepcopy(self.review if authority=='review' else self.receipt)
            value[field]='f'*64
            path=self.review_path if authority=='review' else self.runtime_path
            self.write_json(path,value)
            with self.subTest(authority=authority,field=field), self.assertRaisesRegex(ValueError,'activation external receipt chain mismatch'):
                self.activate()
            self.write_json(path,self.review if authority=='review' else self.receipt)

    def test_each_mixed_phase_binding_fails_at_phase_identity_check(self):
        from survey.democratic_peace import records
        for field in records.BINDING_FIELDS:
            binding=copy.deepcopy(self.declaration['phases'][0]);binding[field]='f'*64
            with self.subTest(field=field), self.assertRaisesRegex(ValueError,'activation phase/source/binary/config binding mismatch'):
                self.activate(binding=binding)

    def test_complete_nonindependent_review_is_rejected(self):
        review=copy.deepcopy(self.review);review['independent']=False
        self.write_json(self.review_path,review)
        with self.assertRaisesRegex(ValueError,'independent premeasurement review required'):
            self.activate()

    def test_complete_overbudget_receipt_is_rejected_even_with_rebound_review(self):
        receipt=copy.deepcopy(self.receipt)
        forecast=receipt['phases']['literal_precision']
        forecast['generation_cpu_seconds']=30000.
        forecast['cpu_seconds']=2*sum(forecast[k] for k in (
            'generation_cpu_seconds','analysis_cpu_seconds','analysis_writer_cpu_seconds'))
        forecast['cpu_hours']=forecast['cpu_seconds']/3600
        forecast['within_budget']=False
        receipt['combined_cpu_seconds']=sum(f['cpu_seconds'] for f in receipt['phases'].values())
        self.write_json(self.runtime_path,receipt)
        review=copy.deepcopy(self.review)
        review['runtime_receipt_sha256']=followup.sha(self.runtime_path.read_bytes())
        self.write_json(self.review_path,review)
        with self.assertRaisesRegex(ValueError,'complete phase forecast failed scheduling gate'):
            self.activate()


class BindingDriftTests(unittest.TestCase):
    def test_wellformed_wrong_archive_identity_is_rejected(self):
        from survey.democratic_peace.test_followup import HISTORICAL
        table=source.fixture_table();wrong=copy.deepcopy(HISTORICAL);wrong['raw_sha256']='a'*64
        with self.assertRaisesRegex(ValueError,'wrong historical'):
            followup.build_manifest(table,followup.canonical_bytes(table),'literal_precision',wrong)

    def test_consistent_resolved_payload_cannot_hide_default_drift(self):
        from pathlib import Path
        from survey.democratic_peace import records
        from survey.democratic_peace.test_followup import HISTORICAL
        table=source.fixture_table();value=followup.build_manifest(table,followup.canonical_bytes(table),'literal_precision',HISTORICAL)
        base=records.strict_json(Path(followup.__file__).with_name('followup-default-config.json').read_bytes())
        arms=[{'id':a['id'],'config':{**base,**a['config_overrides']}} for a in value['arms']]
        def export():
            payload=followup.canonical_bytes(arms)
            return {'schema_version':2,'study_protocol':followup.PROTOCOL,'phase':'literal_precision',
                'arms':arms,'resolved_configs_json':payload.decode(),'resolved_configs_sha256':followup.sha(payload)}
        followup.validate_resolved(value,export())
        arms[0]['config']['tax_rate']=.123
        with self.assertRaisesRegex(ValueError,'defaults/reading/config drift'):followup.validate_resolved(value,export())

class CheckpointAndOutputTests(unittest.TestCase):
    def test_between_history_chunk_preserves_all_keys_and_pending_count(self):
        from survey.democratic_peace.followup_run import batch_keys
        from survey.democratic_peace.test_followup import HISTORICAL
        table=source.fixture_table();value=followup.build_manifest(table,followup.canonical_bytes(table),'literal_precision',HISTORICAL)
        first=batch_keys(value,{},2)
        self.assertEqual((len(first['keys']),first['pending_after'],first['status']),(2,10798,'pending'))
        second=batch_keys(value,set(first['keys']),2)
        self.assertEqual(second['keys'][0][1],390000003)
        self.assertEqual(second['pending_after'],10796)
        with self.assertRaises(ValueError):batch_keys(value,{('wrong',0)},2)

    def test_existing_normative_alias_cannot_be_written_by_new_tools(self):
        import tempfile
        from pathlib import Path
        from survey.democratic_peace.followup_registration import new_outputs
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'methods.json';p.write_text('immutable')
            with self.assertRaises(ValueError):new_outputs([], [p])
            with self.assertRaises(ValueError):new_outputs([p], [p])
            self.assertEqual(p.read_text(),'immutable')
