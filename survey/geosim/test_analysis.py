"""Catch missing registered findings and false complete/equivalent output."""
import json,unittest
from pathlib import Path
from survey.geosim import analysis,manifest,reporting

TABLE=Path(__file__).resolve().parents[2]/'docs/superpowers/specs/2026-10-03-geosim-source-table.json'

class AnalysisTests(unittest.TestCase):
    def fixture(self):
        table=json.loads(TABLE.read_bytes());m=manifest.build_manifest(table,TABLE.read_bytes())
        return m,table
    def test_empty_dataset_exports_all1490_histories_88targets_6contrasts_22pools(self):
        m,t=self.fixture();r=analysis.report(m,t,{},fixture_draws={'source':8,'ks':4,'parameters':8},alternatives=False)
        self.assertEqual((len(r['histories']),len(r['source']['targets']),len(r['source_contrasts']),len(r['modern_pools'])),(1490,88,6,22))
        self.assertEqual(r['received_histories'],0)
        self.assertTrue(all(p['status']=='Unresolved' for p in r['modern_pools'].values()))
        self.assertEqual(len(r['modern_parameter_contrasts']),2)
        self.assertEqual(r['empirical']['clauset_s1']['status'],'Unresolved')
    def test_provisional_manifest_cannot_run_registered_analysis(self):
        m,t=self.fixture()
        with self.assertRaisesRegex(ValueError,'frozen'):analysis.report(m,t,{})
    def test_unregistered_key_is_an_integrity_error_not_missing_data(self):
        m,t=self.fixture()
        with self.assertRaisesRegex(ValueError,'unregistered'):analysis.report(m,t,{('wrong',3):{}},fixture_draws={'source':8,'ks':4,'parameters':8})
    def test_method_change_cannot_silently_reuse_frozen_estimator(self):
        m,t=self.fixture();m['method_contract']['pareto']['minimum_tail']=40
        with self.assertRaisesRegex(ValueError,'method'):analysis.report(m,t,{},fixture_draws={'source':8,'ks':4,'parameters':8})
    def test_serializer_excludes_private_arrays_and_rejects_unlabelled_nonfinite_results(self):
        import numpy as np
        r={'status':'Available','replicates':np.arange(4),'estimate':1.}
        self.assertEqual(json.loads(reporting.serialize_report(r)),{'status':'Available','estimate':1.})
        with self.assertRaises(ValueError):reporting.serialize_report({'status':'Available','estimate':float('inf')})
    def test_markdown_keeps_source_equivalence_and_iid_limits_visible(self):
        m,t=self.fixture();r=analysis.report(m,t,{},fixture_draws={'source':8,'ks':4,'parameters':8},alternatives=False)
        text=reporting.markdown_report(r)
        self.assertIn('Source equivalence',text);self.assertIn('Unresolved',text)
        self.assertIn('iid',text);self.assertIn('1490',text)

class JobBindingTests(unittest.TestCase):
    def test_changed_job_state_cannot_be_reported_as_frozen_replay(self):
        m,t=AnalysisTests().fixture();m['analysis_jobs'][35]['spawn_key']=[999]
        with self.assertRaisesRegex(ValueError,'job'):analysis.report(m,t,{},fixture_draws={'source':8,'ks':4,'parameters':8},alternatives=False)

class MarkdownFindingsTests(unittest.TestCase):
    def test_markdown_prints_judged_modern_estimates_intervals_and_failures(self):
        m,t=AnalysisTests().fixture();r=analysis.report(m,t,{},fixture_draws={'source':8,'ks':4,'parameters':8},alternatives=False)
        r['modern_pools']['precision.base']={'status':'Available','alpha':2.4,'xmin':1200.,'n_tail':81,'ks':.061,
            'ks_test':{'status':'Available','p':.274,'interval':[.246,.302],'verdict':'not_rejected_under_iid_diagnostic'},
            'alternatives':{'lognormal':{'status':'converged','ratio':{'R':-1.4,'p':.041},'diagnostic_verdict':'Alternative_favored_under_iid_diagnostic'},
                            'cutoff_pareto':{'status':'unresolved_near_boundary','reason':'guard contact','ratio':{'status':'unavailable'},'diagnostic_verdict':'Unresolved'}}}
        r['modern_parameters']['precision.base'].update(status='Available',estimates={'alpha_mean':2.35},intervals={'alpha_mean':[2.1,2.6]},eligible_history_count=91)
        r['modern_parameter_contrasts']['base_minus_shock0'].update(status='Available',estimate={'alpha_mean':.32},intervals={'alpha_mean':[.1,.5]})
        text=reporting.markdown_report(r)
        for expected in ('1200','81','0.274','0.246','0.302','-1.4','0.041','guard contact','2.35','2.1','2.6','0.32','0.1','0.5','91'):
            self.assertIn(expected,text)

if __name__=='__main__':unittest.main()
