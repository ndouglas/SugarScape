"""Retained-evidence fixtures construct rows; never run a world or benchmark."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import numpy as np
from survey.democratic_peace import followup, records, source, followup_runtime as runtime
from survey.democratic_peace.followup_registration import declaration, source_inventory
from survey.democratic_peace.methods import canonical_bytes

ROOT = Path(__file__).resolve().parents[2]


def write_json(path, value):
    path.write_bytes(canonical_bytes(value)+b'\n')


def rust_toolchain():
    return {'target':'synthetic-fixture-target','rustc_version':'rustc synthetic-fixture',
        'cargo_version':'cargo synthetic-fixture'}


class ConfirmedEvidenceRegressionTests(unittest.TestCase):
    def test_legacy_summary_with_eight_duplicates_and_no_outputs_cannot_attest(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);phases={}
            measurement={'cpu_seconds':.001,'wall_seconds':.002,'peak_rss_bytes':1024,
                'command':['/usr/bin/true'],'exit_code':0,'source_inventory_sha256':'a'*64,
                'binary_sha256':'b'*64,'toolchain':rust_toolchain(),'output_sha256':'c'*64,
                'classification':'unregistered_runtime_only'}
            for phase in followup.PHASES:
                probes=[copy.deepcopy(measurement) for _ in range(8)]
                analysis={kind:copy.deepcopy(measurement) for kind in runtime.ANALYSIS_TYPES}
                writer=copy.deepcopy(measurement)
                phases[phase]={'native_probes':probes,'analysis':analysis,'writer':writer,
                    'forecast':runtime.phase_forecast(probes,analysis,writer)}
            evidence={'schema_version':1,'classification':'unregistered_runtime_preparation_only',
                'study_protocol':followup.PROTOCOL,'schedule_sha256':followup.sha(Path(runtime.__file__).with_name('followup-probe-schedule.json').read_bytes()),
                'source_inventory_sha256':'a'*64,'binary_sha256':'b'*64,'phases':phases,
                'combined_cpu_seconds':sum(p['forecast']['cpu_seconds'] for p in phases.values()),'within_budget':True}
            bp=root/'fixture-summary.json';write_json(bp,evidence)
            bindings=[]
            for phase,values in followup.PHASES.items():
                bindings.append({'phase':phase,'reading':values[0],**dict.fromkeys(records.BINDING_FIELDS,'a'*64),
                    'binary_sha256':'b'*64,'analysis_jobs_sha256':followup.sha(canonical_bytes(followup.analysis_jobs(phase))),
                    'native_keys_sha256':followup.sha(canonical_bytes([(a['id'],a['first_seed']+r) for a in followup.canonical_arms(phase) for r in range(100)]))})
            dp=root/'synthetic-declaration.json';write_json(dp,declaration(bindings,followup.sha(bp.read_bytes())))
            with self.assertRaises(ValueError):runtime.runtime_attestation(dp,bp)

    def test_requested_environment_path_is_preserved_and_preflight_reports_its_prefix(self):
        self.assertTrue(hasattr(runtime,'selected_python'),'selected-runtime preflight is missing')
        import os,sys
        with tempfile.TemporaryDirectory() as tmp:
            env=Path(tmp).resolve()/'venv';(env/'bin').mkdir(parents=True)
            selected=env/'bin/python';os.symlink(sys.executable,selected)
            (env/'pyvenv.cfg').write_text(f"home = {Path(sys.executable).resolve().parent}\ninclude-system-site-packages = true\n")
            identity=runtime.selected_python(selected)
            self.assertEqual(identity['executable'],str(selected.absolute()))
            self.assertEqual((identity['python'],identity['numpy']),('3.13.5','2.4.6'))
            self.assertEqual(identity['prefix'],str(env))
            self.assertNotEqual(identity['executable'],str(selected.resolve()))

    def test_bad_selected_runtime_fails_before_measurement_or_output_creation(self):
        self.assertTrue(hasattr(runtime,'selected_python'),'selected-runtime preflight is missing')
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);out=root/'unused-output'
            result=__import__('subprocess').CompletedProcess([],0,json.dumps({'python':'0.0.0','numpy':'0.0.0','prefix':str(root)}),'')
            with mock.patch.object(runtime.subprocess,'run',return_value=result), mock.patch.object(runtime,'measure_command') as measured:
                with self.assertRaisesRegex(ValueError,'selected numerical runtime'):
                    runtime.benchmark_preparation(binary=root/'missing-binary',source_root=ROOT,
                        source_path=root/'missing-source',build_evidence=root/'missing-build',output_dir=out,python=root/'selected-python')
            measured.assert_not_called()
            self.assertFalse(out.exists())

    def test_nonempty_build_flags_features_are_preserved_by_preparation_receipt(self):
        self.assertTrue(hasattr(runtime,'prepare_build_receipt'),'bound preparation receipt helper is missing')
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);binary=root/'fixture-native';binary.write_bytes(b'synthetic native identity')
            config=root/'captured-cargo.json';write_json(config,{'config_inputs':[], 'environment':{'RUSTFLAGS':'--cfg synthetic_fixture'}})
            inventory=source_inventory(ROOT);inventory_sha=followup.sha(canonical_bytes(inventory))
            proof={'build_command':['cargo','build','--locked','--manifest-path','survey/Cargo.toml','--bin','democratic_peace','--features','synthetic_feature'],
                'build_exit_code':0,'binary_sha256':followup.sha(binary.read_bytes()),'source_inventory_sha256':inventory_sha,
                'toolchain':rust_toolchain(),'effective_build_flags':['--cfg','synthetic_fixture'],
                'features':['synthetic_feature'],'cargo_configuration':{'path':str(config),'bytes':config.stat().st_size,'sha256':followup.sha(config.read_bytes())}}
            table=source.fixture_table();data=canonical_bytes(table)
            value=followup.baseline.build_unregistered_manifest(table,data,configurations=[{'width':15,'height':15,'horizon_periods':1000}],mode='runtime_probe')
            value.update(source_inventory=inventory,source_inventory_sha256=inventory_sha)
            mp=root/'fixture-manifest.json';write_json(mp,value)
            receipt=runtime.prepare_build_receipt(mp,binary,ROOT,proof)
            self.assertEqual((receipt['build_flags'],receipt['features']),(['--cfg','synthetic_fixture'],['synthetic_feature']))
            for key,wrong in [('features',None),('effective_build_flags','wrong'),('toolchain',{}),('cargo_configuration',{'path':'missing'})]:
                bad=copy.deepcopy(proof);bad[key]=wrong
                with self.assertRaises(ValueError):runtime.prepare_build_receipt(mp,binary,ROOT,bad)
            context={'config_inputs':[{'path':str(root/'host-config.toml'),'sha256':'a'*64,
                'parsed':{'build':{'rustflags':['--cfg','synthetic_fixture']}}}],'environment':{}}
            write_json(config,context)
            proof['cargo_configuration'].update(bytes=config.stat().st_size,sha256=followup.sha(config.read_bytes()))
            self.assertEqual(runtime.prepare_build_receipt(mp,binary,ROOT,proof)['build_flags'],['--cfg','synthetic_fixture'])
            bad=copy.deepcopy(proof);bad['effective_build_flags']=[]
            with self.assertRaisesRegex(ValueError,'captured configuration'):
                runtime.prepare_build_receipt(mp,binary,ROOT,bad)

    def test_missing_or_malformed_build_context_fails_before_probes_and_outputs(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);source_path=ROOT/'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json'
            for proof in ({}, {'effective_build_flags':[], 'features':[], 'cargo_configuration':None}):
                bp=root/'bad-build.json';write_json(bp,proof);out=root/'unused-output'
                with mock.patch.object(runtime,'selected_python',return_value={}), mock.patch.object(runtime,'measure_command') as measured:
                    with self.assertRaises(ValueError):
                        runtime.benchmark_preparation(binary=root/'missing-native',source_root=ROOT,source_path=source_path,
                            build_evidence=bp,output_dir=out,python=root/'selected-python')
                measured.assert_not_called();self.assertFalse(out.exists())

    def test_complete_contrast_fixture_matches_producer_resampling_contract(self):
        from survey.democratic_peace import numerics
        from survey.democratic_peace.followup_evidence import validate_synthetic_payload
        fixture=synthetic_payload('contrast_pair','literal_precision')
        produced=numerics.contrast([0.]*50+[1.]*50,[.25]*100,
            np.random.default_rng(7),np.random.default_rng(8),draws=2)
        self.assertEqual(set(fixture['result']),set(produced))
        self.assertEqual(fixture['result']['resampling'],produced['resampling'])
        self.assertEqual(produced['resampling'],'independent_whole_history')
        validate_synthetic_payload(fixture,'contrast_pair','literal_precision')
        for value in (None,'pooled_histories'):
            bad=copy.deepcopy(fixture)
            if value is None:del bad['result']['resampling']
            else:bad['result']['resampling']=value
            with self.subTest(resampling=value), self.assertRaises(ValueError):
                validate_synthetic_payload(bad,'contrast_pair','literal_precision')


def synthetic_payload(kind,phase):
    common={'schema_version':1,'study_protocol':followup.PROTOCOL,'kind':kind,'phase':phase,
        'runtime':{'python':'3.13.5','numpy':'2.4.6'},'analysis_jobs_sha256':followup.sha(canonical_bytes(followup.analysis_jobs(phase)))}
    if kind=='predictive':
        cases=[]
        for name,values in (('varied',[i/99 for i in range(100)]),('ties',[.5]*100),
            ('complete_extinction',[None]*100),('sparse_survivors',[1.,2.]+[None]*98)):
            surviving=sum(v is not None for v in values);defined=0 if surviving==0 else 100000
            result={'status':'Unavailable' if surviving==0 else 'Available','reason':'insufficient_conditional_reference' if surviving==0 else None,
                'requested_draws':100000,'attempted_draws':100000,'defined_draws':defined,'undefined_draws':100000-defined,
                'surviving_precision_histories':surviving,'predictive_interval':None if surviving==0 else ([.5,.5] if name=='ties' else [1.,2.] if name=='sparse_survivors' else [0.,1.]),
                'conditional':surviving!=100,'undefined_fraction':(100000-defined)/100000}
            if surviving:
                result.update(p=1.,maximizer_kind='point',maximizer=.5,tested_point_count=2,tested_open_interval_count=1)
            cases.append({'case':name,'input_values_sha256':followup.sha(canonical_bytes(values)),'job_index':0,'result':result})
        return {**common,'classification':'synthetic_analysis_timing_only','draws':100000,'workloads':cases}
    if kind=='contrast_pair':
        return {**common,'classification':'synthetic_analysis_timing_only','draws_per_job':100000,'analysis_jobs':2,
            'job_indices':[105,106],'input_values_sha256':followup.sha(canonical_bytes([[0.]*50+[1.]*50,[.25]*100])),
            'result':{'estimate':.25,'p':1/100001,'permutation_exceedances':0,'permutations':100000,'bootstrap_draws':100000,
                'interval':[.1,.4],'resampling':'independent_whole_history'}}
    return {**common,'classification':'synthetic_writer_timing_only','histories':[
        {'arm':f'synthetic.{i//100}','seed':i,'status':'completed','complete':True,
            'metrics':dict.fromkeys(records.METRIC_FIELDS,.5),'census':dict.fromkeys(records.COUNTER_FIELDS,0)} for i in range(10800)],
        'targets':[{'id':s['id'],'result':{'defined_draws':100000,'predictive_interval':[.1,.9]},'p':.5,'holm_p':1.} for s in source.slot_roster()]}


class RetainedWorkloadEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(hasattr(runtime,'validate_retained_evidence'),'retained artifact/workload validator is missing')
        self.temporary=tempfile.TemporaryDirectory();self.addCleanup(self.temporary.cleanup)
        self.root=Path(self.temporary.name).resolve()
        # Entirely synthetic fixture seeds; never use the actual probe schedule's keys.
        schedule=copy.deepcopy(runtime.SCHEDULE)
        schedule['seed_bases']={'printed_decreasing':800000001,'prose_increasing':810000001}
        patch=mock.patch.object(runtime,'SCHEDULE',schedule);patch.start();self.addCleanup(patch.stop)
        self.build_fixture()

    def build_fixture(self):
        from survey.democratic_peace.followup_evidence import artifact,bind_measurement
        from survey.democratic_peace.test_records import science
        table_data=(ROOT/'docs/superpowers/specs/2026-10-03-democratic-peace-source-table.json').read_bytes()
        table=records.strict_json(table_data);inventory=source_inventory(ROOT);inventory_sha=followup.sha(canonical_bytes(inventory))
        binary=self.root/'fixture-native';binary.write_bytes(b'NONEXECUTED SYNTHETIC NATIVE IDENTITY');binary_sha=followup.sha(binary.read_bytes())
        config=self.root/'cargo-configuration.json';write_json(config,{'config_inputs':[],'environment':{'RUSTFLAGS':'--cfg synthetic_fixture'}})
        build={'build_command':['cargo','build','--locked','--manifest-path','survey/Cargo.toml','--bin','democratic_peace','--features','synthetic_feature'],
            'build_exit_code':0,'source_inventory_sha256':inventory_sha,'binary_sha256':binary_sha,'toolchain':rust_toolchain(),
            'effective_build_flags':['--cfg','synthetic_fixture'],'features':['synthetic_feature'],
            'cargo_configuration':{'path':str(config),'bytes':config.stat().st_size,'sha256':followup.sha(config.read_bytes())}}
        build_path=self.root/'build-evidence.json';write_json(build_path,build)
        self.evidence={'schema_version':1,'classification':'unregistered_runtime_preparation_only','study_protocol':followup.PROTOCOL,
            'schedule_sha256':followup.sha(Path(runtime.__file__).with_name('followup-probe-schedule.json').read_bytes()),
            'source_inventory_sha256':inventory_sha,'binary_sha256':binary_sha,'source_root':str(ROOT),'binary_path':str(binary),
            'python_runtime':{'executable':str(Path(__import__('sys').executable).absolute()),'python':'3.13.5','numpy':'2.4.6','prefix':str(Path(__import__('sys').prefix).absolute())},
            'build_evidence':artifact(build_path,self.root),'phases':{}}
        def measurement(command):
            return {'cpu_seconds':.001,'wall_seconds':.002,'peak_rss_bytes':1024,'command':command,'exit_code':0,
                'source_inventory_sha256':inventory_sha,'binary_sha256':binary_sha,'toolchain':rust_toolchain(),
                'output_sha256':'a'*64,'classification':'unregistered_runtime_only'}
        base=records.strict_json(Path(followup.__file__).with_name('followup-default-config.json').read_bytes())
        for phase in followup.PHASES:
            full=runtime.probe_manifest(table,table_data,phase);probes=[]
            for i,scheduled in enumerate(full['arms']):
                prefix=self.root/f'{phase}.probe{i}'
                paths={role:Path(str(prefix)+'.'+suffix) for role,suffix in {'manifest':'manifest.json','receipt':'receipt.json',
                    'resolved':'resolved.json','primary':'sessions.jsonl','stdout':'stdout.log','stderr':'stderr.log'}.items()}
                value=followup.baseline.build_unregistered_manifest(table,table_data,configurations=[scheduled['config_overrides']],mode='runtime_probe')
                value['arms'][0]['first_seed']=scheduled['first_seed'];value.update(source_inventory=inventory,source_inventory_sha256=inventory_sha)
                write_json(paths['manifest'],value);receipt=runtime.prepare_build_receipt(paths['manifest'],binary,ROOT,build);write_json(paths['receipt'],receipt)
                arm=value['arms'][0];config={**base,**arm['config_overrides']};payload=canonical_bytes([{'id':arm['id'],'config':config}])
                binding={'manifest_sha256':followup.sha(paths['manifest'].read_bytes()),'binary_sha256':binary_sha,
                    'source_inventory_sha256':inventory_sha,'build_receipt_sha256':followup.sha(paths['receipt'].read_bytes()),'resolved_configs_sha256':followup.sha(payload)}
                resolved={**binding,'schema_version':1,'model':'democratic_peace','arms':records.strict_json(payload),
                    'resolved_configs_json':payload.decode()};write_json(paths['resolved'],resolved)
                outcome=science(config=config,seed=arm['first_seed']);outcome.update(tick=10,last_tick_periods=100)
                if config['initial_democratic_share']==0.:
                    outcome['current_metrics']['clustering_reason']='undefined_initial_density'
                row={**binding,'schema_version':1,'model':'democratic_peace','arm':arm['id'],'arm_index':0,'canonical_index':0,
                    'repeat_index':0,'family':'runtime_probe','execution_mode':'runtime_probe','seed':arm['first_seed'],'config':config,
                    'attempt':{'status':'completed','construction_errors':[],'panic_context':None,'recorder_error':None},'outcome':outcome}
                write_json(paths['primary'],row);paths['stdout'].write_text('recorded 1 attempted histories\n');paths['stderr'].write_text('')
                command=[str(binary),'--manifest',str(paths['manifest']),'--receipt',str(paths['receipt']),'--repo',str(ROOT),
                    '--resolved',str(paths['resolved']),'--out',str(paths['primary'])]
                probes.append(bind_measurement(measurement(command),phase=phase,kind='native_probe',index=i,paths=paths,root=self.root))
            analysis={};writer=None
            for kind in (*runtime.ANALYSIS_TYPES,'writer'):
                prefix=self.root/f'{phase}.{kind}'
                paths={role:Path(str(prefix)+'.'+suffix) for role,suffix in {'primary':'synthetic.json','stdout':'stdout.log','stderr':'stderr.log'}.items()}
                write_json(paths['primary'],synthetic_payload(kind,phase))
                stdout={'classification':'synthetic_analysis_timing_only','cpu_seconds':.001,'wall_seconds':.002,'peak_rss_bytes':1024,
                    'output_sha256':followup.sha(paths['primary'].read_bytes()),'kind':kind,'phase':phase,'runtime':{'python':'3.13.5','numpy':'2.4.6'}}
                write_json(paths['stdout'],stdout);paths['stderr'].write_text('')
                command=[self.evidence['python_runtime']['executable'],'-m','survey.democratic_peace.followup_runtime',
                    '--synthetic-kind',kind,'--phase',phase,'--output',str(paths['primary'])]
                m=bind_measurement(measurement(command),phase=phase,kind=kind,index=None,paths=paths,root=self.root)
                if kind=='writer':writer=m
                else:analysis[kind]=m
            self.evidence['phases'][phase]={'native_probes':probes,'analysis':analysis,'writer':writer,
                'forecast':runtime.phase_forecast(probes,analysis,writer)}
        self.evidence['combined_cpu_seconds']=sum(p['forecast']['cpu_seconds'] for p in self.evidence['phases'].values())
        self.evidence['within_budget']=True

    def test_complete_bound_synthetic_fixture_validates_without_running_workloads(self):
        runtime.validate_retained_evidence(self.evidence,self.root)
        self.assertEqual([m['workload']['probe_index'] for m in self.evidence['phases']['literal_precision']['native_probes']],list(range(8)))

    def test_complete_bound_fixture_attests_then_missing_primary_fails(self):
        bp=self.root/'fixture-summary.json';write_json(bp,self.evidence);bindings=[]
        for phase,values in followup.PHASES.items():
            bindings.append({'phase':phase,'reading':values[0],**dict.fromkeys(records.BINDING_FIELDS,'a'*64),
                'source_inventory_sha256':self.evidence['source_inventory_sha256'],'binary_sha256':self.evidence['binary_sha256'],
                'analysis_jobs_sha256':followup.sha(canonical_bytes(followup.analysis_jobs(phase))),
                'native_keys_sha256':followup.sha(canonical_bytes([(a['id'],a['first_seed']+r) for a in followup.canonical_arms(phase) for r in range(100)]))})
        dp=self.root/'fixture-declaration.json';write_json(dp,declaration(bindings,followup.sha(bp.read_bytes())))
        self.assertEqual(runtime.runtime_attestation(dp,bp)['status'],'approved_complete_phase_forecast')
        path=self.root/self.evidence['phases']['prose_precision']['writer']['artifacts']['primary']['path'];path.unlink()
        with self.assertRaises(ValueError):runtime.runtime_attestation(dp,bp)

    def test_missing_or_changed_primary_artifact_rejected(self):
        first=self.evidence['phases']['literal_precision']['native_probes'][0]
        path=self.root/first['artifacts']['primary']['path'];path.write_text('changed')
        with self.assertRaises(ValueError):runtime.validate_retained_evidence(self.evidence,self.root)
        path.unlink()
        with self.assertRaises(ValueError):runtime.validate_retained_evidence(self.evidence,self.root)

    def test_duplicate_probe_or_unrelated_command_rejected(self):
        duplicate=copy.deepcopy(self.evidence);duplicate['phases']['literal_precision']['native_probes'][1]=copy.deepcopy(duplicate['phases']['literal_precision']['native_probes'][0])
        with self.assertRaises(ValueError):runtime.validate_retained_evidence(duplicate,self.root)
        unrelated=copy.deepcopy(self.evidence);unrelated['phases']['prose_precision']['native_probes'][0]['command']=['/usr/bin/true']
        with self.assertRaises(ValueError):runtime.validate_retained_evidence(unrelated,self.root)

    def test_substituted_phase_coordinate_seed_and_default_rejected_even_with_fresh_hash(self):
        from survey.democratic_peace.followup_evidence import artifact
        for field,value in [('phase','prose_precision'),('coordinate',.85),('seed',800000999),('default',.123)]:
            evidence=copy.deepcopy(self.evidence);m=evidence['phases']['literal_precision']['native_probes'][0]
            if field=='phase':m['workload']['phase']=value
            else:
                path=self.root/m['artifacts']['manifest']['path'];original=path.read_bytes();manifest=records.strict_json(original)
                if field=='coordinate':manifest['arms'][0]['config_overrides']['mobile_share']=value
                elif field=='seed':manifest['arms'][0]['first_seed']=value
                else:manifest['arms'][0]['config_overrides']['tax_rate']=value
                write_json(path,manifest);m['artifacts']['manifest']=artifact(path,self.root)
            with self.assertRaises(ValueError):runtime.validate_retained_evidence(evidence,self.root)
            if field!='phase':path.write_bytes(original)

    def test_wrong_synthetic_kind_draw_count_or_phase_rejected_after_rebinding_bytes(self):
        from survey.democratic_peace.followup_evidence import artifact
        for key,wrong in [('kind','writer'),('draws',99999),('phase','prose_precision')]:
            evidence=copy.deepcopy(self.evidence);m=evidence['phases']['literal_precision']['analysis']['predictive']
            path=self.root/m['artifacts']['primary']['path'];original=path.read_bytes();payload=records.strict_json(original);payload[key]=wrong
            write_json(path,payload);m['artifacts']['primary']=artifact(path,self.root);m['output_sha256']=m['artifacts']['primary']['sha256']
            with self.assertRaises(ValueError):runtime.validate_retained_evidence(evidence,self.root)
            path.write_bytes(original)

    def test_incomplete_writer_and_boolean_predictive_count_rejected(self):
        from survey.democratic_peace.followup_evidence import validate_synthetic_payload
        writer=synthetic_payload('writer','literal_precision');writer['histories'].pop()
        with self.assertRaises(ValueError):validate_synthetic_payload(writer,'writer','literal_precision')
        predictive=synthetic_payload('predictive','literal_precision')
        predictive['workloads'][0]['result']['tested_open_interval_count']=True
        with self.assertRaises(ValueError):validate_synthetic_payload(predictive,'predictive','literal_precision')
