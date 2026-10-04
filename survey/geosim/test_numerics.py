"""Independent density/KS oracles for independently authored numerical kernels."""
import math,unittest
import numpy as np
from scipy import optimize,stats,special
from survey.geosim import numerics

class NumericTests(unittest.TestCase):
    def test_cutoff_normalizer_matches_independent_special_function_identities(self):
        for k in [.001,.1,1.,10.]:
            self.assertAlmostEqual(math.exp(numerics.cutoff_logj(0,k)),1/k,delta=1e-9/k)
            self.assertAlmostEqual(math.exp(numerics.cutoff_logj(1,k)),math.exp(k)*special.exp1(k),delta=1e-9)
            self.assertAlmostEqual(math.exp(numerics.cutoff_logj(2,k)),1-k*math.exp(k)*special.exp1(k),delta=1e-9)
    def test_tied_ks_checks_both_ecdf_sides_against_independent_scipy(self):
        x=np.array([1.]*20+[2.]*20+[3.]*20+[4.]*20)
        f=numerics.grid_pareto(x,grid=False);tail=x[x>=f['xmin']]
        ref=stats.kstest(tail,lambda z:1-(z/f['xmin'])**(1-f['alpha'])).statistic
        self.assertAlmostEqual(f['ks'],ref,delta=1e-12)
    def test_rescaling_support_leaves_shape_and_ks_unchanged(self):
        x=np.exp(np.linspace(.01,3,150));f=numerics.grid_pareto(x)
        for scale in [1e-100,1e100]:
            g=numerics.grid_pareto(x*scale)
            self.assertAlmostEqual(f['alpha'],g['alpha'],delta=1e-10)
            self.assertAlmostEqual(f['ks'],g['ks'],delta=1e-10)
    def test_truncated_exponential_logdensity_matches_scipy_shifted_distribution(self):
        y=np.linspace(.01,1,100);xmin=3.;f=numerics.fit_exp(y,math.log(xmin));x=xmin*np.exp(y)
        ref=stats.expon.logpdf(x,loc=xmin,scale=xmin/f['k'])
        np.testing.assert_allclose(f['logpdf'],ref,atol=1e-8,rtol=0)
    def test_truncated_lognormal_fit_agrees_with_independent_nelder_mead(self):
        xmin=1.;x=np.exp(stats.truncnorm.ppf((np.arange(100)+.5)/100,0,np.inf,loc=0,scale=.7));y=np.log(x)
        f=numerics.fit_lognormal(y,0.)
        self.assertEqual(f['status'],'converged')
        def objective(z):
            mu,logsigma=z;sigma=math.exp(logsigma)
            return -np.mean(stats.norm.logpdf(y,loc=mu,scale=sigma)-y-stats.norm.logsf(0,loc=mu,scale=sigma))
        ref=optimize.minimize(objective,[0,math.log(.7)],method='Nelder-Mead',options={'xatol':1e-10,'fatol':1e-12,'maxiter':1000})
        self.assertTrue(ref.success)
        self.assertAlmostEqual(f['mu'],ref.x[0],delta=2e-5)
        self.assertAlmostEqual(f['sigma'],math.exp(ref.x[1]),delta=2e-5)
        self.assertAlmostEqual(-f['loglike']/len(y),ref.fun,delta=1e-9)
    def test_stretched_domain_includes_shapes_above_one(self):
        x=(1-np.log1p(-(np.arange(100)+.5)/100))**(1/1.8)
        f=numerics.fit_stretched(np.log(x),0.)
        self.assertEqual(f['status'],'converged');self.assertGreater(f['beta'],1)
        ref=stats.weibull_min.logpdf(x,c=f['beta'],scale=f['k']**(-1/f['beta']))+f['k']
        np.testing.assert_allclose(f['logpdf'],ref,atol=1e-8,rtol=0)
    def test_nonexistent_finite_alternative_mles_are_unresolved(self):
        y=np.r_[np.full(99,.1),20.]
        self.assertEqual(numerics.fit_lognormal(y,0.)['status'],'unattained_pareto_limit')
        self.assertEqual(numerics.fit_stretched(y,0.)['status'],'unattained_pareto_limit')
    def test_exact_cutoff_kkt_boundary_has_equal_fit_p_one_and_qualified_calibration(self):
        y=np.r_[np.full(99,.1),20.];pure=numerics.pareto_at(np.exp(y),1.)
        f=numerics.fit_cutoff(y,0.);self.assertEqual(f['status'],'boundary_pareto')
        r=numerics.likelihood_ratios(pure,f,nested=True)
        self.assertEqual((r['T'],r['p']),(0.,1.))
        self.assertEqual(r['calibration_validity'],'unvalidated')
    def test_unverified_equal_interior_cutoff_cannot_manufacture_boundary_equality(self):
        pure={'alpha':2.5,'logpdf':np.zeros(50)};alt={'status':'converged','logpdf':np.zeros(50)}
        self.assertEqual(numerics.likelihood_ratios(pure,alt,nested=True)['status'],'unresolved_near_boundary')
    def test_zero_variance_nonnested_ratio_is_unresolved(self):
        r=numerics.likelihood_ratios({'logpdf':np.zeros(50)},{'status':'converged','logpdf':np.zeros(50)})
        self.assertEqual(r['status'],'unresolved_zero_variance')

if __name__=='__main__':unittest.main()

class NarrowTailTests(unittest.TestCase):
    def test_narrow_tail_uses_one_fit_and_exact_power_two_rescaling(self):
        from survey.geosim import modern
        from survey.geosim.methods import METHOD_CONTRACT
        tolerance=METHOD_CONTRACT['reference_agreement']
        for base in (1.,1000.,1e100):
            x=base*np.exp(np.linspace(0,1e-12,100))
            f=modern.fit_sizes(x,alternatives=False)
            self.assertEqual(f['status'],'Available')
            tail=x[x>=f['xmin']]
            ref=numerics.pareto_at(tail,f['xmin'])
            self.assertEqual(f['alpha'],ref['alpha'])
            self.assertEqual(f['loglike'],ref['loglike'])
            y=np.array([math.log1p((float(v)-f['xmin'])/f['xmin']) for v in tail])
            lp=math.log(f['alpha']-1)-math.log(f['xmin'])-f['alpha']*y
            self.assertAlmostEqual(f['loglike']/len(tail),float(lp.mean()),delta=tolerance['nll_per_observation_absolute'])
            for exponent in (-300,300):
                g=modern.fit_sizes(np.ldexp(x,exponent),alternatives=False)
                self.assertEqual(g['status'],'Available')
                self.assertAlmostEqual(f['alpha'],g['alpha'],delta=tolerance['rescaling_alpha_ks_absolute'])
                self.assertAlmostEqual(f['ks'],g['ks'],delta=tolerance['rescaling_alpha_ks_absolute'])
                self.assertEqual(g['xmin'],math.ldexp(f['xmin'],exponent))

    def test_density_rejects_unrepresentable_alpha_instead_of_infinite_fit(self):
        x=np.r_[np.ones(49),np.nextafter(1.,2.)]
        # A valid representable narrow fit remains finite; a collapsed tail is unavailable.
        self.assertTrue(math.isfinite(numerics.pareto_at(x,1.)['alpha']))
        with self.assertRaises(numerics.NumericFailure):numerics.pareto_at(np.ones(50),1.)
