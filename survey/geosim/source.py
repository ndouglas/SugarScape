"""Source reconstruction statistics on whole histories and fixed families."""
import bisect
import math
import statistics
import numpy as np
from .manifest import TARGETS
from .records import completed_censuses, history_availability

def source_fit(selected_exports):
    """All selected completed exports, including zero; unavailable != zero."""
    if any(not math.isfinite(s) or s < 0 for s in selected_exports):
        raise ValueError('completed exported severity must be finite and nonnegative')
    positive=sorted(s for s in selected_exports if s>0); n=len(positive)
    result={'war_count':len(selected_exports),'positive_count':n,'zero_count':len(selected_exports)-n,
            'slope':None,'r2':None,'log_range':None,'fit_points':[],'fit_status':'unavailable'}
    if not positive:return result
    result['log_range']=math.log10(positive[-1])-math.log10(positive[0])
    for s in sorted(set(positive)):
        count=n-bisect.bisect_right(positive,s); x=math.log10(s)
        if count and x>=2.5:result['fit_points'].append([x,math.log10(count/n)])
    points=result['fit_points']
    if len(points)<3:return result
    xbar=math.fsum(p[0] for p in points)/len(points);ybar=math.fsum(p[1] for p in points)/len(points)
    xx=math.fsum((x-xbar)**2 for x,y in points); yy=math.fsum((y-ybar)**2 for x,y in points)
    xy=math.fsum((x-xbar)*(y-ybar) for x,y in points)
    if xx<=0 or yy<=0:return result
    slope=xy/xx; r2=xy*xy/(xx*yy)
    if not math.isfinite(slope) or not math.isfinite(r2) or not -1e-12<=r2<=1+1e-12:
        raise ValueError('nonfinite or invalid OLS')
    result.update(slope=slope,intercept=ybar-slope*xbar,r2=min(1.,max(0.,r2)),fit_status='available')
    return result

def eight_summaries(vectors):
    if not vectors or any(len(v)!=4 or any(not math.isfinite(x) for x in v) for v in vectors):
        raise ValueError('complete finite joint four-metric vectors required')
    cols=list(zip(*vectors));s,r,g,w=cols
    return dict(zip(TARGETS,[min(s),statistics.median(s),max(s),min(r),statistics.median(r),max(r),statistics.median(g),statistics.median(w)]))

def inclusive_p(sorted_draws,target):
    n=len(sorted_draws)
    if not n:raise ValueError('empty bootstrap')
    left=bisect.bisect_right(sorted_draws,target);right=n-bisect.bisect_left(sorted_draws,target)
    return min(1.,2*(min(left,right)+1)/(n+1))

def maximize_interval_p(sorted_draws,interval):
    lo,hi=interval
    if not math.isfinite(lo) or not math.isfinite(hi) or lo>hi:raise ValueError('invalid interval')
    if not sorted_draws or any(not math.isfinite(x) for x in sorted_draws):raise ValueError('invalid draws')
    if any(a>b for a,b in zip(sorted_draws,sorted_draws[1:])):raise ValueError('draws must be sorted')
    candidates={lo,hi};a=bisect.bisect_left(sorted_draws,lo);b=bisect.bisect_right(sorted_draws,hi)
    if a<b:candidates.add(sorted_draws[max(a,min(b-1,len(sorted_draws)//2))])
    values=[(inclusive_p(sorted_draws,t),t) for t in sorted(candidates)]
    p=max(p for p,t in values)
    return {'p':p,'maximizer':min(t for q,t in values if q==p),'tested':sorted(candidates)}

def fixed_holm(values,expected):
    if len(values)!=expected:raise ValueError('fixed hypothesis family size mismatch')
    ps=[1. if p is None else p for p in values]
    if any(not math.isfinite(p) or not 0<=p<=1 for p in ps):raise ValueError('invalid p-value')
    adjusted=[None]*len(values);running=0.
    for rank,i in enumerate(sorted(range(expected),key=lambda i:(ps[i],i))):
        running=min(1.,max(running,(expected-rank)*ps[i]))
        if values[i] is not None:adjusted[i]=running
    return adjusted

def arm_verdict(adjusted,complete):
    if any(p is not None and p<.05 for p in adjusted):return 'Incompatible'
    if complete and all(p is not None for p in adjusted):return 'Compatible'
    return 'Unresolved'


def contrast_verdict(estimate,adjusted_p):
    if estimate is None or adjusted_p is None:return 'Unresolved'
    if adjusted_p<.05 and estimate>0:return 'Holds'
    if adjusted_p<.05 and estimate<0:return 'Fails'
    return 'Inconclusive'


def _vectors(vectors,columns):
    value=np.asarray(vectors,dtype=float)
    if value.shape!=(100,columns) or not np.all(np.isfinite(value)):
        raise ValueError('one hundred complete finite joint history vectors required')
    return value


def predictive_targets(vectors,rng,draws=100000):
    vectors=_vectors(vectors,4)
    if type(draws) is not int or draws<=0:raise ValueError('positive bootstrap draw count required')
    result=[]
    for start in range(0,draws,1024):
        indices=rng.integers(0,100,size=(min(1024,draws-start),15),dtype=np.int64)
        v=vectors[indices];s=v[:,:,0];r=v[:,:,1]
        result.append(np.column_stack((s.min(axis=1),np.median(s,axis=1),s.max(axis=1),r.min(axis=1),np.median(r,axis=1),r.max(axis=1),np.median(v[:,:,2],axis=1),np.median(v[:,:,3],axis=1))))
    values=np.concatenate(result)
    return {'replicates':values,'targets':{name:{'draws':sorted(values[:,i].tolist()),
            'predictive_interval':np.quantile(values[:,i],[.025,.975],method='linear').tolist()}
            for i,name in enumerate(TARGETS)},'resamples':draws,'source_batch_size':15,'precision_history_count':100}


def independent_contrast(base,control,rng,draws=100000):
    base=_vectors(base,3);control=_vectors(control,3)
    if type(draws) is not int or draws<=0:raise ValueError('positive bootstrap draw count required')
    result=[]
    for start in range(0,draws,1024):
        n=min(1024,draws-start)
        ib=rng.integers(0,100,size=(n,100),dtype=np.int64)
        ic=rng.integers(0,100,size=(n,100),dtype=np.int64)
        result.append(base[ib].mean(axis=1)-control[ic].mean(axis=1))
    values=np.concatenate(result)
    return {'estimate':(base.mean(axis=0)-control.mean(axis=0)).tolist(),
            'replicates':values,'intervals':np.quantile(values,[.025,.975],axis=0,method='linear').T.tolist(),
            'p':[inclusive_p(sorted(values[:,i].tolist()),0.) for i in range(3)],
            'resamples':draws,'resampling':'independent_whole_history'}


def history_source(row):
    availability=history_availability(row)
    if row.get('outcome') is None:return {'availability':availability,'fit':None,'vector':None,'census':None,'partial_diagnostic':None}
    census=completed_censuses(row['outcome'],availability)
    primary=census['source_selected'];partial=census['partial_source_selected']
    fit=source_fit([w['exported_severity'] for w in primary]) if availability['complete'] else None
    diagnostic=source_fit([w['exported_severity'] for w in partial]) if partial else None
    vector=None
    if fit and fit['fit_status']=='available':vector=(fit['slope'],fit['r2'],fit['log_range'],fit['war_count'])
    return {'availability':availability,'fit':fit,'vector':vector,'census':census,'partial_diagnostic':diagnostic}


def complete_vectors(histories,expected):
    if len(histories)!=expected or any(not h['availability']['complete'] or h['vector'] is None for h in histories):return None
    v=np.asarray([h['vector'] for h in histories],dtype=float)
    return v if v.shape==(expected,4) and np.all(np.isfinite(v)) else None


def source_findings(table,histories,jobs,draws=100000):
    from .manifest import job_rng
    targets=[];arms={}
    for row in table['rows']:
        rid=row['id'];original=histories.get(f'original.{rid}',[]);precision=histories.get(f'precision.{rid}',[])
        ov=complete_vectors(original,15);pv=complete_vectors(precision,100)
        original_summary=None if ov is None else eight_summaries(ov.tolist())
        prediction=None if pv is None else predictive_targets(pv,job_rng(jobs[f'predictive.source.{rid}']),draws)
        arms[rid]={'conditional_verdict':None,'source_equivalence':'Unresolved',
                   'original_received':len(original),'precision_received':len(precision),
                   'original_joint_eligible':sum(h['availability']['complete'] and h['vector'] is not None for h in original),
                   'precision_joint_eligible':sum(h['availability']['complete'] and h['vector'] is not None for h in precision),
                   'population_complete':ov is not None and pv is not None,'original_summary':original_summary}
        for name in TARGETS:
            target=row['targets'][name];computed=None
            if prediction is not None:
                d=prediction['targets'][name]
                computed={**maximize_interval_p(d['draws'],target['interval']),
                          'predictive_interval':d['predictive_interval'],'resamples':draws,'source_batch_size':15,
                          'finite_support_limit':'bootstrap extrema are not impossible-model-event claims'}
            targets.append({'id':f'source.{rid}.{name}','row':rid,'target':name,'source_interval':target['interval'],
                            'printed':target['printed'],'definition_status':target['definition_status'],
                            'source_equivalence':'Unresolved','original':None if original_summary is None else original_summary[name],
                            'p':None if computed is None else computed['p'],'result':computed,'holm_p':None,
                            'unavailable_reason':'incomplete_precision_fit_population' if computed is None else None})
    adjusted=fixed_holm([t['p'] for t in targets],88)
    for t,p in zip(targets,adjusted):t['holm_p']=p
    for rid,arm in arms.items():
        arm['conditional_verdict']=arm_verdict([t['holm_p'] for t in targets if t['row']==rid],arm['population_complete'])
    return {'family_size':88,'targets':targets,'arms':arms,'draws':draws}


def contrast_findings(histories,jobs,draws=100000):
    from .manifest import job_rng
    findings=[];base=complete_vectors(histories.get('precision.base',[]),100)
    for cid,control in [('base_minus_shock0','shock0'),('base_minus_context_off','context_off')]:
        cv=complete_vectors(histories.get(f'precision.{control}',[]),100)
        result=None if base is None or cv is None else independent_contrast(base[:,:3],cv[:,:3],job_rng(jobs[f'contrast.{cid}']),draws)
        for i,metric in enumerate(('slope','r2','log_range')):
            findings.append({'id':f'contrast.{cid}.{metric}','metric':metric,
                             'estimate':None if result is None else result['estimate'][i],
                             'interval':None if result is None else result['intervals'][i],
                             'p':None if result is None else result['p'][i],
                             'unavailable_reason':'incomplete_precision_fit_population' if result is None else None,
                             'family_size':6,'resampling':'independent_whole_history','draws':draws})
    adjusted=fixed_holm([f['p'] for f in findings],6)
    for f,p in zip(findings,adjusted):
        f['holm_p']=p;f['verdict']=contrast_verdict(f['estimate'],p)
    return findings
