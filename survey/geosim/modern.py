"""Offline iid diagnostics and separate whole-history parameter uncertainty."""
import math
import numpy as np
from scipy import stats
from . import numerics
from .records import completed_censuses,history_availability

ALTERNATIVES={'exponential':numerics.fit_exp,'lognormal':numerics.fit_lognormal,
              'stretched_exponential':numerics.fit_stretched,'cutoff_pareto':numerics.fit_cutoff}
PARAMETER_METRICS=('alpha_mean','alpha_median','xmin_mean','xmin_median')


def _numeric_error(exc):return f'{type(exc).__name__}: {exc}'


def _final_fit(fit):
    status=fit['status']
    if status in ('converged','boundary_pareto'):
        if not math.isfinite(fit['loglike']) or not np.all(np.isfinite(fit['logpdf'])):
            status='unresolved_nonfinite_final_density'
        for name in ('k','sigma','beta'):
            if name in fit and (not math.isfinite(fit[name]) or fit[name]<=0):
                if not (name=='k' and fit[name]==0 and fit['status']=='boundary_pareto'):
                    status='unresolved_nonpositive_parameter'
        if any(name in fit and not math.isfinite(fit[name]) for name in ('alpha','mu','a','b')):
            status='unresolved_nonfinite_parameter'
    unavailable=[]
    def clean(value,path):
        if isinstance(value,dict):return {k:clean(v,f'{path}.{k}' if path else k) for k,v in value.items()}
        if isinstance(value,(list,tuple)):return [clean(v,f'{path}[{i}]') for i,v in enumerate(value)]
        if isinstance(value,np.ndarray):
            if not np.all(np.isfinite(value)):
                unavailable.append(path);return None
            return value
        if isinstance(value,np.generic):value=value.item()
        if isinstance(value,float) and not math.isfinite(value):
            unavailable.append(path);return None
        return value
    result=clean({**fit,'status':status},'')
    if unavailable:result['unavailable_numeric_fields']=unavailable
    return result


def fit_sizes(sizes,cutoff_search='all_observed',alternatives=True):
    if cutoff_search not in ('all_observed','grid100'):raise ValueError('unknown cutoff search')
    x=np.asarray(sizes,dtype=float)
    base={'n':int(x.size),'cutoff_search':cutoff_search,'minimum_tail':50,'alternatives':{}}
    if x.ndim!=1 or np.any(~np.isfinite(x)) or np.any(x<=0):
        return {**base,'status':'Unresolved','reason':'nonfinite_or_nonpositive_sizes'}
    if len(x)<50:return {**base,'status':'Insufficient_tail','reason':'fewer_than50_positive_completed_sizes'}
    if np.all(x==x[0]):return {**base,'status':'Degenerate_tail','reason':'all_equal_positive_sizes'}
    try:
        with np.errstate(over='raise',invalid='raise',divide='raise'):
            selected=numerics.grid_pareto(x,grid=cutoff_search=='grid100')
            tail=x[x>=selected['xmin']];pure=numerics.pareto_at(tail,selected['xmin'])
        if not math.isfinite(pure['alpha']) or pure['alpha']<=1 or not np.all(np.isfinite(pure['logpdf'])):
            raise numerics.NumericFailure('nonfinite Pareto fit')
    except (numerics.NumericFailure,OverflowError,FloatingPointError,ValueError) as exc:
        return {**base,'status':'Unresolved','reason':_numeric_error(exc)}
    result={**base,**selected,'status':'Available','loglike':pure['loglike'],'iid_limit':'dependent histories are not iid validation'}
    if alternatives:
        y=np.log(tail)-math.log(selected['xmin'])
        for name,fitter in ALTERNATIVES.items():
            try:
                with np.errstate(over='raise',invalid='raise',divide='raise'):
                    f=_final_fit(fitter(y,math.log(selected['xmin'])))
            except (numerics.NumericFailure,OverflowError,FloatingPointError,ValueError) as exc:
                f={'status':'unresolved_numeric_failure','reason':_numeric_error(exc)}
            ratio=numerics.likelihood_ratios(pure,f,nested=name=='cutoff_pareto')
            if name=='cutoff_pareto':ratio.update(calibration='declared_half_chi_square_1',calibration_validity='unvalidated',regular_boundary_information=pure['alpha']>3)
            verdict='Unresolved'
            if ratio.get('status')=='available':
                verdict='Inconclusive'
                if ratio['p']<.1:verdict='Alternative_favored_under_iid_diagnostic' if ratio['R']<0 else 'Pareto_favored_under_iid_diagnostic'
            result['alternatives'][name]={k:v for k,v in f.items() if k!='logpdf'}
            result['alternatives'][name].update(xmin=selected['xmin'],n_tail=selected['n_tail'],ratio=ratio,diagnostic_verdict=verdict)
    return result


def history_modern(row,alternatives=True):
    available=history_availability(row)
    if row.get('outcome') is None:return {'status':'Unresolved','reason':available['status'],'complete':False,'raw_sizes':[],'partial_diagnostic':None}
    census=completed_censuses(row['outcome'],available)
    raw=[w['raw_severity'] for w in census['primary_completed'] if w['raw_severity']>0]
    partial=[w['raw_severity'] for w in census['partial_completed'] if w['raw_severity']>0]
    fit=fit_sizes(raw,alternatives=alternatives) if available['complete'] else {'status':'Unresolved','reason':available['status']}
    return {**fit,'complete':available['complete'],'raw_sizes':raw,
            'partial_diagnostic':fit_sizes(partial,alternatives=alternatives) if partial else None,
            'zero_completed_count':sum(w['raw_severity']==0 for w in census['primary_completed']),
            'excluded':census['excluded'],'censored_count':census['censored_count'],'backlog_count':census['backlog_count']}


def fit_pool(histories,expected,alternatives=True):
    if len(histories)!=expected or any(not h['complete'] for h in histories):
        return {'status':'Unresolved','reason':'incomplete_registered_history_population',
                'registered_history_count':expected,'received_count':len(histories),'complete_history_count':sum(h['complete'] for h in histories)}
    fit=fit_sizes([x for h in histories for x in h['raw_sizes']],cutoff_search='grid100',alternatives=alternatives)
    return {**fit,'registered_history_count':expected,'complete_history_count':expected,
            'individual_eligible_history_count':sum(h['status']=='Available' for h in histories),
            'censored_count':sum(h.get('censored_count',0) for h in histories),'backlog_count':sum(h.get('backlog_count',0) for h in histories)}


def ks_refit_test(fit,sizes,rng,draws=1000):
    if type(draws) is not int or draws<=0:raise ValueError('positive fixed KS replicate count required')
    if fit['status']!='Available':return {'status':'Unresolved','reason':'pooled_fit_unavailable','replicates_attempted':0}
    x=np.asarray(sizes,dtype=float);n=len(x);body=x[x<fit['xmin']];prob=fit['n_tail']/n
    failures=[];Ds=[];cutoffs=[]
    for index in range(draws):
        try:
            nt=int(rng.binomial(n,prob));nb=n-nt
            lower=body[rng.integers(0,len(body),size=nb,dtype=np.int64)] if nb else np.empty(0)
            logtail=math.log(fit['xmin'])+rng.exponential(1/(fit['alpha']-1),nt)
            with np.errstate(over='raise',invalid='raise'):tail=np.exp(logtail)
            simulated=fit_sizes(np.concatenate((lower,tail)),cutoff_search='grid100',alternatives=False)
            if simulated['status']!='Available':raise numerics.NumericFailure(simulated.get('reason',simulated['status']))
            Ds.append(simulated['ks']);cutoffs.append(simulated['xmin'])
        except (ValueError,OverflowError,FloatingPointError,numerics.NumericFailure) as exc:
            failures.append({'index':index,'reason':_numeric_error(exc)})
    out={'replicates_attempted':draws,'successful_replicates':len(Ds),'failed_replicates':len(failures),
         'failures':failures,'refitted_cutoff_search':'grid100','refitted_xmins':cutoffs,
         'iid_limit':'semiparametric iid diagnostic, not dependent-world validation'}
    if failures:return {**out,'status':'Unresolved','reason':'predetermined_refit_failures','p':None,'interval':None}
    b=sum(d>=fit['ks'] for d in Ds)
    lo=0. if b==0 else float(stats.beta.ppf(.025,b,draws-b+1))
    hi=1. if b==draws else float(stats.beta.ppf(.975,b+1,draws-b))
    verdict='Inconclusive' if lo<=.1<=hi else 'rejected_under_iid_diagnostic' if hi<.1 else 'not_rejected_under_iid_diagnostic'
    return {**out,'status':'Available','exceedances':b,'p':(b+1)/(draws+1),'interval':[lo,hi],
            'interval_method':'95%_Clopper_Pearson_b_of_B','verdict':verdict}


def _positive_mean(values):
    scale=float(np.max(values))
    return float(np.mean(values/scale)*scale)


def _positive_midpoints(lower,upper):
    lower=np.asarray(lower);upper=np.asarray(upper);out=np.empty_like(lower,dtype=float)
    safe=lower<=np.finfo(float).max-upper
    out[safe]=(lower[safe]+upper[safe])/2
    out[~safe]=lower[~safe]/2+upper[~safe]/2
    return out


def _positive_median(values):
    ordered=np.sort(values);n=len(values)
    return float(_positive_midpoints(np.array([ordered[(n-1)//2]]),np.array([ordered[n//2]]))[0])


def parameter_bootstrap(histories,rng,expected,draws=100000):
    if type(draws) is not int or draws<=0:raise ValueError('positive parameter bootstrap draw count required')
    eligible=[h['status']=='Available' and math.isfinite(h.get('alpha',math.nan)) and h['alpha']>1 and math.isfinite(h.get('xmin',math.nan)) and h['xmin']>0 for h in histories]
    out={'registered_history_count':expected,'received_count':len(histories),'eligible_history_count':sum(eligible),
         'resamples':draws,'empty_replicates':0,'intervals':{},'estimates':{},'replicates':None,
         'resampling':'whole_registered_history_rows_with_eligibility_masks'}
    if len(histories)!=expected or any(not h['complete'] for h in histories):
        return {**out,'status':'Unresolved','reason':'incomplete_registered_history_population'}
    if not any(eligible):return {**out,'status':'Unresolved','reason':'no_eligible_history_fits','empty_replicates':draws}
    vals=np.array([[h['alpha'],h['xmin']] if ok else [np.nan,np.nan] for h,ok in zip(histories,eligible)])
    observed=vals[np.asarray(eligible)]
    out['estimates']=dict(zip(PARAMETER_METRICS,[_positive_mean(observed[:,0]),_positive_median(observed[:,0]),_positive_mean(observed[:,1]),_positive_median(observed[:,1])]))
    values=[];counts=[]
    for start in range(0,draws,1024):
        idx=rng.integers(0,expected,size=(min(1024,draws-start),expected),dtype=np.int64)
        v=vals[idx];mask=np.isfinite(v[:,:,0]);count=mask.sum(axis=1);nonempty=count>0
        block=np.full((len(v),4),np.nan)
        for mean_column,median_column,j in [(0,1,0),(2,3,1)]:
            finite=np.where(mask,v[:,:,j],0)[nonempty];scale=finite.max(axis=1)
            block[nonempty,mean_column]=(finite/scale[:,None]).sum(axis=1)/count[nonempty]*scale
            ordered=np.sort(v[nonempty,:,j],axis=1);rows=np.arange(len(ordered));c=count[nonempty]
            block[nonempty,median_column]=_positive_midpoints(ordered[rows,(c-1)//2],ordered[rows,c//2])
        values.append(block);counts.extend(count.tolist())
    rep=np.concatenate(values);good=np.all(np.isfinite(rep),axis=1);empty=int((~good).sum())
    intervals={name:np.quantile(rep[good,i],[.025,.975],method='linear').tolist() for i,name in enumerate(PARAMETER_METRICS)} if good.any() else {}
    return {**out,'status':'Unresolved' if empty else 'Available','reason':'empty_eligibility_replicates' if empty else None,
            'empty_replicates':empty,'nonempty_replicates':int(good.sum()),'eligibility_counts':counts,
            'eligibility_frequency':sum(eligible)/expected,'replicates':rep,
            'intervals':intervals if not empty else {},'conditional_nonempty_intervals':intervals if empty else {}}


def parameter_contrast(base,control):
    out={'inference':'descriptive_independent_whole_history','intervals':{},'estimate':{},
         'base_eligible':base['eligible_history_count'],'control_eligible':control['eligible_history_count'],
         'base_registered':base['registered_history_count'],'control_registered':control['registered_history_count']}
    if base['status']!='Available' or control['status']!='Available':
        return {**out,'status':'Unresolved','reason':'incomplete_or_empty_parameter_bootstrap'}
    if base['replicates'].shape!=control['replicates'].shape:raise ValueError('parameter contrast replicate counts mismatch')
    diff=base['replicates']-control['replicates']
    return {**out,'status':'Available','estimate':{k:base['estimates'][k]-control['estimates'][k] for k in PARAMETER_METRICS},
            'intervals':{k:np.quantile(diff[:,i],[.025,.975],method='linear').tolist() for i,k in enumerate(PARAMETER_METRICS)},
            'empty_replicates':0,'resamples':base['resamples']}
