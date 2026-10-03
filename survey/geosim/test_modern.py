"""Catch survivor pooling, missing fit refits and false optimizer/calibration success."""
import copy,unittest
import numpy as np
from survey.geosim import modern
from survey.geosim import test_records as record_fixtures

class ModernTests(unittest.TestCase):
    def test_tail_sufficiency_and_equal_value_degeneracy_are_explicit(self):
        self.assertEqual(modern.fit_sizes(np.arange(1,50),alternatives=False)['status'],'Insufficient_tail')
        self.assertEqual(modern.fit_sizes(np.ones(50),alternatives=False)['status'],'Degenerate_tail')
    def test_nonfinite_fit_data_is_unresolved_without_nonfinite_json(self):
        f=modern.fit_sizes([float('inf')]*50,alternatives=False)
        self.assertEqual(f['status'],'Unresolved')
        self.assertIn('nonfinite',f['reason'])
    def test_four_alternatives_share_one_selected_support_and_preserve_failure(self):
        x=np.exp(np.linspace(.01,1,80));f=modern.fit_sizes(x)
        self.assertEqual(f['status'],'Available')
        self.assertEqual(set(f['alternatives']),{'exponential','lognormal','stretched_exponential','cutoff_pareto'})
        self.assertTrue(all(a['xmin']==f['xmin'] and a['n_tail']==f['n_tail'] for a in f['alternatives'].values()))
        self.assertEqual(f['alternatives']['cutoff_pareto']['ratio']['calibration_validity'],'unvalidated')
    def test_invalid_history_positive_completions_never_become_primary_modern_data(self):
        row,_,_=record_fixtures.EnvelopeTests().fixture();out=row['outcome'];row['attempt']['status']='invalid'
        out.update(valid=False,periods=521,attempted_period=522,finish_reason='invalid',invalid_reason='fixture')
        f=modern.history_modern(row,alternatives=False)
        self.assertEqual(f['status'],'Unresolved')
        self.assertEqual(f['raw_sizes'],[])
        self.assertEqual(f['partial_diagnostic']['n'],1)
    def test_artifact_modern_data_uses_completed_raw_backlog_once(self):
        row,_,_=record_fixtures.EnvelopeTests().fixture();out=row['outcome'];out['config']['completed_export']='one_per_period'
        out['legacy_visible_wars']=[];out['exporter_backlog']=copy.deepcopy(out['completed_wars'])
        f=modern.history_modern(row,alternatives=False)
        self.assertEqual(f['raw_sizes'],[10.])
        self.assertEqual(f['backlog_count'],1)
    def test_incomplete_arm_keeps_pool_and_ks_unresolved(self):
        h=[{'complete':True,'raw_sizes':[1.,2.,3.]}]*14
        p=modern.fit_pool(h,expected=15,alternatives=False)
        self.assertEqual(p['status'],'Unresolved')
        self.assertEqual(p['reason'],'incomplete_registered_history_population')
    def test_semiparametric_draws_refit_every_cutoff_and_replay_seed(self):
        x=np.exp(np.linspace(.01,2,100));f=modern.fit_sizes(x,cutoff_search='grid100',alternatives=False)
        a=modern.ks_refit_test(f,x,np.random.default_rng(13),draws=8)
        b=modern.ks_refit_test(f,x,np.random.default_rng(13),draws=8)
        self.assertEqual(a,b)
        self.assertEqual(a['replicates_attempted'],8)
        self.assertEqual(a['refitted_cutoff_search'],'grid100')
        if a['status']!='Unresolved':self.assertEqual(a['p'],(a['exceedances']+1)/9)
    def test_any_predetermined_failed_refit_is_unresolved_without_replacements(self):
        x=np.geomspace(1e300,1.7e308,50)
        fit=modern.fit_sizes(x,cutoff_search='grid100',alternatives=False)
        a=modern.ks_refit_test(fit,x,np.random.default_rng(13),draws=8)
        self.assertEqual(a['status'],'Unresolved')
        self.assertEqual(a['replicates_attempted'],8)
        self.assertGreater(a['failed_replicates'],0)
        self.assertEqual(a['failed_replicates']+a['successful_replicates'],8)
    def test_parameter_bootstrap_keeps_empty_eligibility_draws_visible(self):
        h=[{'complete':True,'status':'Available','alpha':2.,'xmin':1.}]+[{'complete':True,'status':'Insufficient_tail'}]*14
        p=modern.parameter_bootstrap(h,np.random.default_rng(13),expected=15,draws=32)
        self.assertEqual(p['status'],'Unresolved')
        self.assertGreater(p['empty_replicates'],0)
        self.assertEqual(p['registered_history_count'],15)
        self.assertEqual(p['eligible_history_count'],1)
    def test_parameter_contrasts_reuse_independent_arm_replicate_arrays(self):
        a=modern.parameter_bootstrap([{'complete':True,'status':'Available','alpha':3.,'xmin':10.}]*15,np.random.default_rng(13),expected=15,draws=16)
        b=modern.parameter_bootstrap([{'complete':True,'status':'Available','alpha':2.,'xmin':4.}]*15,np.random.default_rng(14),expected=15,draws=16)
        c=modern.parameter_contrast(a,b)
        self.assertEqual(c['intervals'],{'alpha_mean':[1.,1.],'alpha_median':[1.,1.],'xmin_mean':[6.,6.],'xmin_median':[6.,6.]})
        self.assertNotIn('p',c)
        self.assertEqual(c['inference'],'descriptive_independent_whole_history')

class LargeParameterTests(unittest.TestCase):
    def test_large_finite_parameter_means_and_even_medians_do_not_overflow(self):
        histories=[{'complete':True,'status':'Available','alpha':2.,'xmin':1e308}]*100
        p=modern.parameter_bootstrap(histories,np.random.default_rng(13),expected=100,draws=8)
        self.assertEqual(p['status'],'Available')
        self.assertAlmostEqual(p['estimates']['xmin_mean']/1e308,1.)
        self.assertEqual(p['estimates']['xmin_median'],1e308)
        self.assertEqual(p['empty_replicates'],0)

class FinalDiagnosticsTests(unittest.TestCase):
    def test_nonfinite_failed_density_has_null_telemetry_and_explicit_reason(self):
        fit={'status':'converged','loglike':float('inf'),'logpdf':np.full(50,float('inf')),'k':1.}
        result=modern._final_fit(fit)
        self.assertEqual(result['status'],'unresolved_nonfinite_final_density')
        self.assertIsNone(result['loglike'])
        self.assertIn('loglike',result['unavailable_numeric_fields'])

if __name__=='__main__':unittest.main()
