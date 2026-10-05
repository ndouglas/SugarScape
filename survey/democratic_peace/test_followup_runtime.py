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

    def test_activation_rejects_self_review_mixed_identity_and_overbudget(self):
        self.assertTrue(callable(runtime.verify_activation))
        with self.assertRaises(ValueError): runtime.validate_review({'independent':False})
        with self.assertRaises(ValueError): runtime.validate_runtime_receipt({'status':'approved','forecast_multiplier':1})

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
