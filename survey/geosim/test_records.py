"""Catch aborted-period completions entering fit data and forged bindings."""
import copy,hashlib,json,math,tempfile,unittest
from pathlib import Path
from survey.geosim import records

class RecordsTests(unittest.TestCase):
    def outcome(self,valid=False):
        war={'id':0,'parents':[],'start_period':501,'end_period':521,'last_active_period':501,
             'active_periods':1,'elapsed_periods':21,'raw_severity':10.,'exported_severity':10.,
             'participants':[{'state':{'capital_cell':0,'sovereignty_generation':0},'last_fighting_period':501}],
             'end_cause':'shadow_expired_or_participants_retired','java_saturated':False,'java_subunit_zero':False,'fighting_periods':[501]}
        return {'config':{'width':2,'height':2,'initialization_periods':500,'observation_periods':10000,
                          'count_boundary':'after_initialization','completed_export':'all_completed','severity_export':'raw_damage'},
                'seed':3,'rng_mode':'portable_pcg64_mcg','periods':10500 if valid else 521,
                'attempted_period':10500 if valid else 522,'counting_start':501,'valid':valid,'state_available':True,
                'finish_reason':'horizon' if valid else 'invalid','invalid_reason':None if valid else 'nonfinite cumulative science ledger',
                'completed_wars':[war],'censored_wars':[],'legacy_visible_wars':[copy.deepcopy(war)],'exporter_backlog':[],
                'merges':[],'retired_states':[],'sovereign_count':0,'states':[],'cells':[{'id':i,'owner':{'capital_cell':0,'sovereignty_generation':0},'last_threshold':3.,'next_generation':1} for i in range(4)],'ledger':{**dict.fromkeys(('attacks','fighting_front_periods','mutual_front_periods','conquests','collapses','disconnections','stale_claims','locked_claims','double_successes','path_collisions','shocks'),0),**dict.fromkeys(('damage','measured_damage','capacity_increase','capacity_decrease','clipping','retirement_capacity','reemergence_capacity','recurrence_residual'),0.)},'fronts':[],'resource_updates':[],
                'partial_period_fights':[]}
    def test_attempted_period_completion_is_excluded_even_from_partial_diagnostics(self):
        out=self.outcome();bad=copy.deepcopy(out['completed_wars'][0]);bad.update(id=1,end_period=522,elapsed_periods=22)
        out['completed_wars'].append(bad);out['legacy_visible_wars'].append(copy.deepcopy(bad));before=copy.deepcopy(out)
        c=records.completed_censuses(out)
        self.assertEqual([w['id'] for w in c['partial_completed']],[0])
        self.assertEqual(c['excluded'][0]['reasons'],['invalid_partial_period_completion'])
        self.assertEqual(c['primary_completed'],[]);self.assertEqual(out,before)
    def test_backlog_is_completed_biology_but_never_legacy_source_census(self):
        out=self.outcome(True);out['config']['completed_export']='one_per_period'
        out['legacy_visible_wars']=[];out['exporter_backlog']=copy.deepcopy(out['completed_wars'])
        c=records.completed_censuses(out)
        self.assertEqual((len(c['primary_completed']),len(c['source_selected'])),(1,0))
    def test_generation_distinct_participants_are_not_merged_by_capital(self):
        out=self.outcome();out['completed_wars'][0]['participants'].append({'state':{'capital_cell':0,'sovereignty_generation':1},'last_fighting_period':501})
        out['legacy_visible_wars']=copy.deepcopy(out['completed_wars'])
        self.assertEqual(len(records.completed_censuses(out)['partial_completed']),1)
    def test_invalid_raw_severity_is_preserved_with_exclusion_reason(self):
        out=self.outcome();out['completed_wars'][0]['raw_severity']=-1
        out['legacy_visible_wars']=[]
        c=records.completed_censuses(out)
        self.assertIn('invalid_raw_severity',c['excluded'][0]['reasons'])
    def test_valid_horizon_census_with_future_completion_is_integrity_error(self):
        out=self.outcome(True);out['completed_wars'][0]['end_period']=10501
        with self.assertRaises(ValueError):records.completed_censuses(out)
    def test_strict_json_rejects_duplicate_fields_and_nonfinite_numbers(self):
        for text in ['{"seed":1,"seed":2}','{"x":NaN}']:
            with self.assertRaises(ValueError):records.strict_json(text)
    def test_native_config_payload_uses_exact_native_float_bytes(self):
        payload='[{"id":"a","config":{"x":1e-6}}]'
        export={'resolved_configs_json':payload,'resolved_configs_sha256':hashlib.sha256(payload.encode()).hexdigest(),
                'arms':[{'id':'a','config':{'x':.000001}}]}
        self.assertEqual(records.resolved_payload(export),[{'id':'a','config':{'x':.000001}}])
        export['arms'][0]['config']['x']=.000002
        with self.assertRaises(ValueError):records.resolved_payload(export)
    def test_source_inventory_rejects_traversal_symlink_and_missing_bytes(self):
        with tempfile.TemporaryDirectory() as t:
            from survey.geosim.test_provenance import source_tree
            from survey.geosim.provenance import source_inventory
            root=Path(t);source_tree(root);valid=source_inventory(root)
            self.assertTrue(records.verify_source_inventory(root,valid))
            for path in ['../file','/file']:
                with self.assertRaises(ValueError):records.verify_source_inventory(root,[{**valid[0],'path':path}])
            (root/'escape').symlink_to('/etc/passwd')
            with self.assertRaises(ValueError):records.verify_source_inventory(root,[{**valid[0],'path':'escape'}])


class EnvelopeTests(unittest.TestCase):
    def fixture(self):
        out=RecordsTests().outcome(True)
        expected={'id':'original.base','index':0,'sessions':15,'first_seed':370000001,'config':out['config']}
        binding={k:'a'*64 for k in ('manifest_sha256','binary_sha256','source_inventory_sha256','build_receipt_sha256','resolved_configs_sha256')}
        out['seed']=370000001
        row={'schema_version':1,'model':'geosim','arm':'original.base','seed':370000001,'arm_index':0,'repeat_index':0,
             'config':out['config'],**binding,'attempt':{'status':'completed','construction_errors':[],'panic_context':None,'recorder_error':None},'outcome':out}
        return row,expected,binding
    def test_unknown_model_or_fields_cannot_be_resumed_as_geosim(self):
        row,arm,binding=self.fixture()
        self.assertEqual(records.validate_record(row,arm,binding),('original.base',370000001))
        for key,value in [('model','polarity'),('unexpected',True)]:
            bad={**row,key:value}
            with self.assertRaises(ValueError):records.validate_record(bad,arm,binding)
    def test_changed_provenance_and_unregistered_seed_are_rejected(self):
        row,arm,binding=self.fixture()
        for key,value in [('binary_sha256','b'*64),('seed',370000999),('repeat_index',4)]:
            with self.assertRaises(ValueError):records.validate_record({**row,key:value},arm,binding)
    def test_missing_outcome_needs_explicit_incomplete_error(self):
        row,arm,binding=self.fixture();row['outcome']=None
        row['attempt']={'status':'incomplete','construction_errors':[],'panic_context':None,'recorder_error':'native run omitted Outcome'}
        self.assertEqual(records.history_availability(row)['status'],'incomplete')
        self.assertEqual(records.validate_record(row,arm,binding),('original.base',370000001))
        row['attempt']['recorder_error']=None
        with self.assertRaises(ValueError):records.validate_record(row,arm,binding)
    def test_required_unavailable_context_rejects_blank_and_wrong_types(self):
        for status in ('construction_panic','implementation_panic','incomplete'):
            for context in (None,17,'',' \t\n','\u0085\u001c'):
                with self.subTest(status=status,context=context):
                    row,arm,binding=self.fixture()
                    if status!='implementation_panic':row['outcome']=None
                    row['attempt']={'status':status,'construction_errors':[],
                        'panic_context':None if status=='incomplete' else context,
                        'recorder_error':context if status=='incomplete' else None}
                    with self.assertRaises(ValueError):records.validate_record(row,arm,binding)
    def test_construction_error_members_require_exact_string_fields(self):
        for errors in ([17],[None],[[]],[{}],[{'field':'config'}],[{'message':'failure'}],
                [{'field':17,'message':'failure'}],[{'field':'config','message':False}],
                [{'field':'config','message':'failure','unexpected':True}]):
            with self.subTest(errors=errors):
                row,arm,binding=self.fixture();row['outcome']=None
                row['attempt']={'status':'construction_error','construction_errors':errors,
                    'panic_context':None,'recorder_error':None}
                with self.assertRaises(ValueError):records.validate_record(row,arm,binding)
    def test_valid_unavailable_diagnostics_are_retained(self):
        for attempt in (
                {'status':'construction_error','construction_errors':[{'field':'config','message':'failure'}],'panic_context':None,'recorder_error':None},
                {'status':'construction_panic','construction_errors':[],'panic_context':'constructor failure','recorder_error':None},
                {'status':'incomplete','construction_errors':[],'panic_context':None,'recorder_error':'missing Outcome'}):
            with self.subTest(status=attempt['status']):
                row,arm,binding=self.fixture();row['outcome']=None;row['attempt']=attempt
                self.assertEqual(records.validate_record(row,arm,binding),('original.base',370000001))
    def test_postfinish_panic_preserves_valid_outcome_as_partial_only(self):
        from survey.geosim import source,modern
        row,arm,binding=self.fixture();row['attempt'].update(status='implementation_panic',panic_context='after native advance')
        before=copy.deepcopy(row['outcome'])
        self.assertFalse(records.history_availability(row)['complete'])
        self.assertEqual(records.validate_record(row,arm,binding),('original.base',370000001))
        s=source.history_source(row);m=modern.history_modern(row,alternatives=False)
        self.assertIsNone(s['vector']);self.assertEqual(m['raw_sizes'],[])
        self.assertEqual(len(s['census']['partial_completed']),1)
        self.assertEqual(row['outcome'],before)
        row['attempt']['panic_context']=None
        with self.assertRaisesRegex(ValueError,'panic_context'):records.history_availability(row)
    def test_build_receipt_rejects_unknown_fields_and_changed_manifest(self):
        receipt={'schema_version':1,'model':'geosim','manifest_sha256':'a'*64,'source_inventory_sha256':'b'*64,
                 'prebuild_inventory_sha256':'b'*64,'postbuild_inventory_sha256':'b'*64,'binary_sha256':'c'*64,
                 'build_command':['cargo','build','--release'],'cwd':'/fixture','target':'aarch64-apple-darwin',
                 'rustc_version':'fixture','cargo_version':'fixture','lockfile_hashes':{'Cargo.lock':'d'*64,'survey/Cargo.lock':'e'*64},'features':[],'build_flags':[]}
        self.assertTrue(records.validate_build_receipt(receipt,'a'*64,'c'*64,'b'*64))
        with self.assertRaises(ValueError):records.validate_build_receipt({**receipt,'unknown':1},'a'*64,'c'*64,'b'*64)
        with self.assertRaises(ValueError):records.validate_build_receipt(receipt,'d'*64,'c'*64,'b'*64)
        with self.assertRaisesRegex(ValueError,'locks'):records.validate_build_receipt({**receipt,'lockfile_hashes':{}},'a'*64,'c'*64,'b'*64)


class ReadSessionsTests(unittest.TestCase):
    def reader_fixture(self):
        row,arm,binding=EnvelopeTests().fixture()
        payload=json.dumps([{'id':arm['id'],'config':arm['config']}],separators=(',',':'))
        binding['resolved_configs_sha256']=hashlib.sha256(payload.encode()).hexdigest()
        row['resolved_configs_sha256']=binding['resolved_configs_sha256']
        resolved={**binding,'arms':[{'id':arm['id'],'config':arm['config']}],
                  'resolved_configs_json':payload}
        return row,{'arms':[arm]},resolved,binding
    def test_duplicate_registered_keys_fail_without_silently_overwriting(self):
        row,m,r,b=self.reader_fixture()
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'raw.jsonl';p.write_text(json.dumps(row)+'\n'+json.dumps(row)+'\n')
            with self.assertRaisesRegex(ValueError,'duplicate'):records.read_sessions(p,m,r,b)
    def test_mixed_manifest_and_truncated_last_record_fail(self):
        row,m,r,b=self.reader_fixture()
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'raw.jsonl';p.write_text(json.dumps(row)+'\n{')
            with self.assertRaisesRegex(ValueError,'raw line 2'):records.read_sessions(p,m,r,b)
            row['manifest_sha256']='b'*64;p.write_text(json.dumps(row)+'\n')
            with self.assertRaisesRegex(ValueError,'provenance'):records.read_sessions(p,m,r,b)
    def test_valid_raw_key_is_returned_without_mutating_census(self):
        row,m,r,b=self.reader_fixture()
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'raw.jsonl';p.write_text(json.dumps(row)+'\n')
            data=records.read_sessions(p,m,r,b)
            self.assertEqual(data[('original.base',370000001)],row)

class JsonOverflowTests(unittest.TestCase):
    def test_finite_token_exponent_overflow_is_rejected_at_every_depth(self):
        for text in ['{"x":1e999}','{"x":-1e999}','[1,{"x":[1e999]}]','{"x":{"y":[-1e999]}}']:
            with self.subTest(text=text):
                with self.assertRaisesRegex(ValueError,'nonfinite'):records.strict_json(text)
    def test_null_unavailable_numbers_survive_overflow_validation(self):
        self.assertEqual(records.strict_json('{"x":[null,1e-6]}'),{'x':[None,.000001]})

class CensusConservationTests(unittest.TestCase):
    def test_same_id_cannot_be_completed_and_censored(self):
        out=RecordsTests().outcome(True);w=copy.deepcopy(out['completed_wars'][0]);w.update(end_period=None,end_cause=None)
        out['censored_wars']=[w]
        with self.assertRaisesRegex(ValueError,'censored'):records.completed_censuses(out)
    def test_positive_completed_war_cannot_disappear_from_collector(self):
        out=RecordsTests().outcome(True);out['legacy_visible_wars']=[]
        with self.assertRaisesRegex(ValueError,'collector'):records.completed_censuses(out)
    def test_all_completed_cannot_retain_positive_backlog(self):
        out=RecordsTests().outcome(True);out['exporter_backlog']=out['legacy_visible_wars'];out['legacy_visible_wars']=[]
        with self.assertRaisesRegex(ValueError,'backlog'):records.completed_censuses(out)
    def test_integer_subunit_zero_still_belongs_to_positive_raw_collector(self):
        out=RecordsTests().outcome(True);out['config']['severity_export']='java_int100'
        out['completed_wars'][0].update(raw_severity=.001,exported_severity=0.,java_subunit_zero=True)
        out['legacy_visible_wars']=copy.deepcopy(out['completed_wars'])
        self.assertEqual(len(records.completed_censuses(out)['primary_completed']),1)
        out['legacy_visible_wars']=[]
        with self.assertRaisesRegex(ValueError,'collector'):records.completed_censuses(out)
    def test_raw_zero_is_completed_but_not_queued(self):
        out=RecordsTests().outcome(True);out['completed_wars'][0].update(raw_severity=0.,exported_severity=0.)
        out['legacy_visible_wars']=[]
        self.assertEqual(len(records.completed_censuses(out)['primary_completed']),1)
        out['legacy_visible_wars']=copy.deepcopy(out['completed_wars'])
        with self.assertRaisesRegex(ValueError,'collector'):records.completed_censuses(out)
    def test_censored_war_has_no_completion_and_unique_generation_identities(self):
        out=RecordsTests().outcome(True);w=copy.deepcopy(out['completed_wars'][0]);w.update(id=1,end_period=None,end_cause='horizon_censored',elapsed_periods=10000)
        out['censored_wars']=[w]
        self.assertEqual(records.completed_censuses(out)['censored_count'],1)
        w['participants'].append(copy.deepcopy(w['participants'][0]))
        with self.assertRaisesRegex(ValueError,'censored'):records.completed_censuses(out)

class CoreCensoredShapeTests(unittest.TestCase):
    def test_actual_core_horizon_censored_cause_is_retained_and_not_completed(self):
        out=RecordsTests().outcome(True);w=copy.deepcopy(out['completed_wars'][0])
        w.update(id=1,end_period=None,end_cause='horizon_censored',elapsed_periods=10000)
        out['censored_wars']=[w]
        c=records.completed_censuses(out)
        self.assertEqual(c['censored_count'],1)
        self.assertEqual([war['id'] for war in c['primary_completed']],[0])
        self.assertEqual(out['censored_wars'][0]['end_cause'],'horizon_censored')

if __name__=='__main__':unittest.main()

class StrictNestedTests(unittest.TestCase):
    def test_interrupted_append_with_complete_json_but_no_newline_is_rejected(self):
        row,m,r,b=ReadSessionsTests().reader_fixture()
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'raw.jsonl';p.write_text(json.dumps(row))
            with self.assertRaisesRegex(ValueError,'raw line 1.*newline'):
                records.read_sessions(p,m,r,b)

    def test_unknown_or_missing_nested_fields_and_wrong_types_are_rejected(self):
        row,arm,binding=EnvelopeTests().fixture()
        sid={'capital_cell':0,'sovereignty_generation':0}
        out=row['outcome']
        out['states']=[{'id':sid,'capacity':1.,'threshold':3.,'alert':False,'campaign':None,
            'previous_damage':0.,'newly_independent':False,'extracted_yield':1.,'recurrence_residual':0.}]
        out['sovereign_count']=1
        out['fronts']=[{'states':[sid,{**sid,'capital_cell':1}],'previous':[False,False],'actions':[False,False],
            'old_commitments':[0.,0.],'commitments':[0.,0.],'path':None,'initiator':None,'last_damage':[0.,0.],
            'last_victory_probabilities':[None,None]}]
        out['resource_updates']=[{'state':sid,'period':1,'old_capacity':1.,'extracted_yield':1.,'applied_damage':0.,
            'target_capacity':1.,'new_capacity':1.,'clipping':0.,'residual':0.,'reset':False}]
        out['merges']=[{'period':1,'survivor':0,'absorbed':1}]
        self.assertEqual(records.validate_record(row,arm,binding),('original.base',370000001))
        paths=[('completed_wars',0),('completed_wars',0,'participants',0),
               ('completed_wars',0,'participants',0,'state'),('states',0),('states',0,'id'),
               ('cells',0),('ledger',),('fronts',0),('resource_updates',0),('merges',0)]
        for path in paths:
            for mode in ('extra','missing','type'):
                with self.subTest(path=path,mode=mode):
                    bad=copy.deepcopy(row);obj=bad['outcome']
                    for key in path:obj=obj[key]
                    if mode=='extra':obj['unexpected']='corrupt'
                    elif mode=='missing':del obj[next(iter(obj))]
                    else:obj[next(iter(obj))]=True
                    bad['outcome']['legacy_visible_wars']=copy.deepcopy(bad['outcome']['completed_wars'])
                    with self.assertRaises(ValueError):records.validate_record(bad,arm,binding)

    def test_nested_availability_clocks_and_array_shapes_fail_closed(self):
        mutations=[('rng_mode','other'),('counting_start',True),('sovereign_count',1),('cells',[]),
                   ('partial_period_fights',[[{'capital_cell':0,'sovereignty_generation':0}]]),
                   ('ledger',{'damage':0.})]
        for key,value in mutations:
            with self.subTest(key=key):
                row,arm,b=EnvelopeTests().fixture();row['outcome'][key]=value
                with self.assertRaises(ValueError):records.validate_record(row,arm,b)
        for field,value in [('java_saturated',1),('parents',[True]),('raw_severity',None),('last_active_period',True)]:
            with self.subTest(field=field):
                row,arm,b=EnvelopeTests().fixture();row['outcome']['completed_wars'][0][field]=value
                row['outcome']['legacy_visible_wars']=copy.deepcopy(row['outcome']['completed_wars'])
                with self.assertRaises(ValueError):records.validate_record(row,arm,b)

class InvalidNestedTests(unittest.TestCase):
    def test_invalid_outcome_still_rejects_corrupt_war_clocks_and_participants(self):
        for field,value in [('fighting_periods',[501,501]),('active_periods',2),('elapsed_periods',1),
                            ('participants',[{'state':{'capital_cell':0,'sovereignty_generation':0},'last_fighting_period':600}])]:
            with self.subTest(field=field):
                row,arm,b=EnvelopeTests().fixture();out=row['outcome']
                row['attempt']['status']='invalid';out.update(valid=False,periods=521,attempted_period=522,finish_reason='invalid',invalid_reason='fixture')
                out['completed_wars'][0][field]=value;out['legacy_visible_wars']=copy.deepcopy(out['completed_wars'])
                with self.assertRaises(ValueError):records.validate_record(row,arm,b)

    def test_invalid_required_float_nulls_preserved_but_valid_nulls_rejected(self):
        row,arm,b=EnvelopeTests().fixture();out=row['outcome'];out['ledger']['damage']=None
        with self.assertRaises(ValueError):records.validate_record(row,arm,b)
        row['attempt']['status']='invalid';out.update(valid=False,periods=521,attempted_period=522,finish_reason='invalid',invalid_reason='fixture')
        self.assertEqual(records.validate_record(row,arm,b),('original.base',370000001))
        self.assertIsNone(out['ledger']['damage'])

class NativeBoundaryParityTests(unittest.TestCase):
    def test_bare_carriage_return_is_not_native_complete_line(self):
        row,m,r,b=ReadSessionsTests().reader_fixture()
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'raw.jsonl';p.write_bytes(json.dumps(row).encode()+b'\r')
            with self.assertRaisesRegex(ValueError,'newline'):records.read_sessions(p,m,r,b)

    def test_integer_config_cannot_be_replaced_by_equal_float(self):
        row,arm,b=EnvelopeTests().fixture();row=copy.deepcopy(row)
        row['config']['width']=2.;row['outcome']['config']['width']=2.
        with self.assertRaises(ValueError):records.validate_record(row,arm,b)
