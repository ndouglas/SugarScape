"""Premeasurement numerical and resampling contract; never fitted from outcomes."""
from copy import deepcopy
import hashlib
import json

METHOD_CONTRACT = {
    'revision':'geosim-offline-v1',
    'runtime':{'python':'3.13.5','numpy':'2.4.6','scipy':'1.17.1'},
    'source':{'cutoff_log10':2.5,'ccdf':'strict_unique_positive',
              'minimum_fit_points':3,'range':'full_positive_selected_census',
              'count':'all_selected_completed_including_zero','ols_roundoff_tolerance':1e-12,
              'prediction_history_count':15,'precision_history_count':100,
              'draws':100000,'family_size':88,'holm_reject_below':.05,
              'p':'inclusive_two_sided_plus_one_maximize_closed_source_interval'},
    'contrasts':{'ids':['base_minus_shock0','base_minus_context_off'],
                 'metrics':['slope','r2','log_range'],'draws':100000,
                 'resampling':'independent_whole_history','family_size':6,'holm_reject_below':.05},
    'rng':{'root':2026100301,'jobs':72,'generator':'PCG64',
           'child':'SeedSequence(root,spawn_key=(fixed_job_index,))',
           'batch_size':1024,'contrast_order':'base_block_then_control_block',
           'ks_order':'binomial_tail_count,body_indices,tail_exponential'},
    'interval':{'level':.95,'quantile_method':'linear'},
    'pareto':{'minimum_tail':50,'individual_search':'all_observed','pool_search':'grid100',
              'grid_indices':'j*(M-1)//99 for j=0..99 when M>100',
              'ks':'max_left_and_right_ecdf_at_unique_ties','cutoff_ties':'smallest_xmin_exact'},
    'ks_test':{'draws':1000,'level':.1,'exceedance':'Dstar>=Dobs',
               'p':'(b+1)/(B+1)','interval':'95%_Clopper_Pearson_b_of_B',
               'failure':'any_predetermined_failed_replicate_unresolved_no_replacements',
               'calibration':'iid_semiparametric_generated_refitted_grid100'},
    'quadrature':{'epsabs':1e-11,'epsrel':1e-10,'limit':250,'max_relative_error':1e-8,
                  'normalizer':'scaled_positive_dimensionless_integral'},
    'optimizer':{'method':'L-BFGS-B','ftol':1e-13,'gtol':1e-8,'maxiter':1000,
                  'maxls':50,'accept_gradient_inf_below':1e-6,'guard_contact':'unresolved'},
    'lognormal':{'coordinates':'mean_scaled_natural_a_b',
                 'starts':[[-1.,.5],[0.,.5],[1.,.1],[1.,.01]],
                 'bounds':[[-100.,100.],[1e-8,1e4]],'nonexistence':'scaled_second_moment>=2'},
    'stretched':{'beta_bounds':[1e-8,100.],'brent_xtol':1e-10,'brent_rtol':1e-10,
                 'maxiter':200,'domain':'beta>0_including_beta>1','nonexistence':'endpoint_score<=0'},
    'cutoff':{'bounds':[[-100.,100.],[-32.,32.]],
              'starts':'alpha=[min(99,pareto_alpha),1,0] x k=[1,.1,.01]/mean_excess; clip_logk[-31,31]',
              'boundary':'exact_convex_KKT_alpha>2_and_mean_excess>=1/(alpha-2)',
              'near_boundary_gain_per_observation':1e-10,'near_boundary':'unresolved'},
    'ratios':{'level':.1,'nonnested':'CSN_normalized_two_sided','variance_divisor':'n',
              'variance_floor':1e-24,'nested':'declared_half_chi_square_1',
              'exact_equal_p':1.,'calibration_validity':'unvalidated',
              'qualification':'alpha<=3_lacks_finite_boundary_information; alpha>3_not_validation'},
    'parameter_bootstrap':{'draws':100000,'metrics':['alpha_mean','alpha_median','xmin_mean','xmin_median'],
                           'sampling':'all_registered_history_rows_with_joint_eligibility_masks',
                           'positive_mean':'max_scaled_mean','positive_median':'arithmetic_midpoint_with_overflow_safe_branch',
                           'empty_or_incomplete':'primary_unresolved',
                           'conditional_eligible_interval':'descriptive_only',
                           'rng':'independent_fixed_per_arm_parameter_child',
                           'contrasts':['precision.base-minus-precision.shock0','precision.base-minus-precision.context_off'],
                           'contrast_method':'same_index_independent_arm_replicate_array_differences',
                           'contrast_inference':'descriptive_linear95_interval_no_p_family_or_verdict'},
    'availability':{'successful_completion':'end_period<=Outcome.periods',
                    'invalid_history':'partial_diagnostics_only_no_primary_pool_or_bootstrap',
                    'incomplete_pool':'unresolved_reserved_ks_slot',
                    'zero_eligible_parameter_replicate':'unresolved_no_replacements'},
    'reference_agreement':{'parameters_absolute':2e-5,'log_density_absolute':1e-8,
                            'ks_absolute':1e-12,'relative_normalization_error':1e-9,'nll_per_observation_absolute':1e-9,'rescaling_alpha_ks_absolute':1e-10},
}

def canonical_bytes(value):
    return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode('utf-8')

def method_contract():
    return deepcopy(METHOD_CONTRACT)

def contract_sha256():
    return hashlib.sha256(canonical_bytes(METHOD_CONTRACT)).hexdigest()
