"""Independent authored offline density kernels, adopted from verified preparation.

Reference-only author R/MATLAB code is not imported or copied. All methods,
starts, guards and tolerances are frozen by methods.METHOD_CONTRACT.
"""
import math
import warnings
import numpy as np
from scipy import integrate,optimize,special,stats

class NumericFailure(Exception): pass

def checked_quad(fun, a, b):
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter('always',integrate.IntegrationWarning)
        value,error=integrate.quad(fun,a,b,epsabs=1e-11,epsrel=1e-10,limit=250)
    if caught or not math.isfinite(value) or value<=0 or not math.isfinite(error) or error>1e-8*value:
        raise NumericFailure(f'quadrature {value=} {error=} {caught=}')
    return value,error

def cutoff_logj(alpha,k,with_mean=False):
    """J=integral exp((1-alpha)*y-k*expm1(y)) dy, y>=0."""
    if k==0:
        if alpha<=1: raise NumericFailure('improper zero-cutoff density')
        return (-math.log(alpha-1),1/(alpha-1)) if with_mean else -math.log(alpha-1)
    a=alpha-1
    mode=math.log(-a/k) if a<0 and k < -a else 0.
    def h(y):
        if y>700: return -math.inf
        return -a*y-k*math.expm1(y)
    hm=h(mode)
    scale=math.sqrt(-a) if mode else max(1.,a+k)
    def integral(moment):
        def right(u):
            y=mode+u/scale; z=h(y)-hm
            return (math.exp(z)*((scale*y)**moment)) if z>-745 else 0.
        r,err=checked_quad(right,0,math.inf)
        if mode:
            def left(u):
                y=mode-u/scale;z=h(y)-hm
                return math.exp(z)*((scale*y)**moment) if z>-745 else 0.
            l,le=checked_quad(left,0,mode*scale);r+=l;err+=le
        return r
    j=integral(0); result=hm+math.log(j)-math.log(scale)
    return (result,integral(1)/j/scale) if with_mean else result

def lognormal_logj(a,b,with_moments=False):
    if b<=0: raise NumericFailure('lognormal curvature must be positive')
    mode=max(0.,-a/(2*b)); hm=-a*mode-b*mode*mode
    scale=max(1.,a) if mode==0 else max(1.,math.sqrt(2*b))
    def integ(j):
        def right(u):
            t=mode+u/scale;z=-a*t-b*t*t-hm
            return math.exp(z)*(scale*t)**j if z>-745 else 0.
        value,_=checked_quad(right,0,math.inf)
        if mode:
            def left(u):
                t=mode-u/scale;z=-a*t-b*t*t-hm
                return math.exp(z)*(scale*t)**j if z>-745 else 0.
            leftv,_=checked_quad(left,0,mode*scale);value+=leftv
        return value
    j=integ(0); logj=hm+math.log(j)-math.log(scale)
    return (logj,integ(1)/j/scale,integ(2)/j/(scale*scale)) if with_moments else logj

def centered_log_ratios(x,xmin):
    """Accurate near xmin; avoid overflow for far-separated finite supports."""
    x=np.asarray(x,dtype=float)
    if x.ndim!=1 or not math.isfinite(xmin) or xmin<=0 or np.any(~np.isfinite(x)) or np.any(x<xmin):
        raise NumericFailure('invalid Pareto support')
    close=x/2<=xmin;y=np.empty_like(x)
    y[close]=np.log1p((x[close]-xmin)/xmin)
    y[~close]=np.log(x[~close])-math.log(xmin)
    return y


def pareto_at(x,xmin):
    x=np.sort(np.asarray(x,dtype=float));y=centered_log_ratios(x,xmin);s=float(np.mean(y))
    if len(x)<50 or s<=0 or not math.isfinite(s):raise NumericFailure('insufficient or degenerate Pareto')
    alpha=1+1/s
    if not math.isfinite(alpha) or alpha<=1:raise NumericFailure('unrepresentable Pareto alpha')
    lp=math.log(alpha-1)-math.log(xmin)-alpha*y
    if not np.all(np.isfinite(lp)):raise NumericFailure('unrepresentable Pareto density')
    return {'alpha':alpha,'xmin':float(xmin),'n_tail':len(x),'loglike':float(lp.sum()),'logpdf':lp,'log_ratios':y}

def fit_exp(y,logxmin):
    excess=np.expm1(y);m=float(np.mean(excess))
    if not math.isfinite(m) or m<=0: raise NumericFailure('degenerate exponential')
    k=1/m;lp=math.log(k)-logxmin-k*excess
    return {'status':'converged','k':k,'loglike':float(lp.sum()),'logpdf':lp}

def fit_lognormal(y,logxmin):
    s=float(np.mean(y));t=y/s;m2=float(np.mean(t*t))
    if m2>=2: return {'status':'unattained_pareto_limit','m2_scaled':m2}
    def fun(theta):
        a,b=theta
        try:lj,e1,e2=lognormal_logj(a,b,True)
        except NumericFailure:return 1e100,np.zeros(2)
        return a+b*m2+lj,np.array([1-e1,m2-e2])
    attempts=[]
    bounds=[(-100.,100.),(1e-8,1e4)]
    for start in [(-1.,.5),(0.,.5),(1.,.1),(1.,.01)]:
        r=optimize.minimize(fun,start,method='L-BFGS-B',jac=True,bounds=bounds,options={'ftol':1e-13,'gtol':1e-8,'maxiter':1000,'maxls':50})
        grad=float(np.max(np.abs(fun(r.x)[1])))
        attempts.append({'x':r.x.tolist(),'success':bool(r.success),'message':str(r.message),'objective':float(r.fun),'gradient':grad,'nit':int(r.nit),'nfev':int(r.nfev)})
    good=[r for r in attempts if r['success'] and r['gradient']<1e-6 and r['objective']<1e90]
    if not good:return {'status':'unconverged','attempts':attempts}
    best=min(good,key=lambda r:r['objective']);a,b=best['x']
    if a<=-99.999 or a>=99.999 or b<=1.01e-8 or b>=9999:return {'status':'bound_hit','attempts':attempts}
    sig=s/math.sqrt(2*b);mu=logxmin-a*s/(2*b);lj=lognormal_logj(a,b)
    lp=-logxmin-y-math.log(s)-a*t-b*t*t-lj
    return {'status':'converged','a':a,'b':b,'mu':mu,'sigma':sig,'loglike':float(lp.sum()),'logpdf':lp,'attempts':attempts}

def fit_stretched(y,logxmin):
    s=float(np.mean(y));m2=float(np.mean(y*y));boundary=s-m2/(2*s)
    def stats_beta(beta):
        v=beta*y; vmax=float(v.max())
        scaled=np.exp(v-vmax);den=float(np.sum(scaled))-len(y)*math.exp(-vmax)
        if den<=0:raise NumericFailure('unresolved expm1 sum')
        logden=vmax+math.log(den)
        if vmax < .01:
            # v*exp(v)-expm1(v), evaluated without its small-v cancellation.
            numer=float(np.sum(v*v*(.5+v*(1/3+v*(1/8+v*(1/30+v*(1/144+v/840)))))))
            score=s-numer/(beta*float(np.sum(np.expm1(v))))
        else:
            numer=float(np.sum((v-1)*scaled))+len(y)*math.exp(-vmax)
            score=s-numer/(beta*den)
        return logden,score
    # Endpoint score is exact in the beta->0 limit. A nonpositive value puts supremum at beta=0.
    if boundary<=0:return {'status':'unattained_pareto_limit','boundary_score':boundary}
    lo=1e-8;hi=1.
    # Tiny beta score evaluated from its analytic limit avoids cancellation.
    def score(beta):
        return stats_beta(beta)[1] if beta>=lo else boundary
    while score(hi)>0 and hi<100:hi=min(100.,hi*2)
    if score(lo)<=0 or score(hi)>=0:return {'status':'bound_hit','bracket':[lo,hi]}
    beta,r=optimize.brentq(score,lo,hi,xtol=1e-10,rtol=1e-10,maxiter=200,full_output=True,disp=False)
    if not r.converged:return {'status':'unconverged'}
    logden,_=stats_beta(beta);logk=math.log(len(y))-logden;k=math.exp(logk)
    if k<=0 or not math.isfinite(k):return {'status':'unresolved_rate_underflow','iterations':r.iterations}
    lp=math.log(beta)+logk-logxmin+(beta-1)*y-(np.exp(logk+beta*y)-k)
    return {'status':'converged','beta':beta,'k':k,'loglike':float(lp.sum()),'logpdf':lp,'iterations':r.iterations,'score':score(beta)}

def fit_cutoff(y,logxmin):
    mean_y=float(np.mean(y));mean_excess=float(np.mean(np.expm1(y)));ap=1+1/mean_y
    pure_lp=math.log(ap-1)-logxmin-ap*y;pure_nll=-float(np.mean(pure_lp))
    # Convex natural-parameter likelihood: exact KKT certificate at k=0.
    if ap>2 and mean_excess>=1/(ap-2):
        return {'status':'boundary_pareto','alpha':ap,'k':0.,'loglike':float(pure_lp.sum()),'logpdf':pure_lp,'boundary_score':mean_excess-1/(ap-2),'certificate':'convex_likelihood_KKT'}
    def fun(theta):
        alpha,logk=theta;k=math.exp(logk)
        try:
            lj,ey=cutoff_logj(alpha,k,True);ez=math.exp(cutoff_logj(alpha-1,k)-lj)
        except (NumericFailure,OverflowError):return 1e100,np.zeros(2)
        f=logxmin+alpha*mean_y+k*mean_excess+lj
        g=np.array([mean_y-ey,k*(mean_excess-(ez-1))])
        return f,g
    bounds=[(-100.,100.),(-32.,32.)];attempts=[]
    for alpha in [min(99.,ap),1.,0.]:
        for factor in [1.,.1,.01]:
            start=(alpha,float(np.clip(math.log(factor/mean_excess),-31.,31.)))
            r=optimize.minimize(fun,start,method='L-BFGS-B',jac=True,bounds=bounds,options={'ftol':1e-13,'gtol':1e-8,'maxiter':1000,'maxls':50})
            attempts.append({'x':r.x.tolist(),'success':bool(r.success),'message':str(r.message),'objective':float(r.fun),'gradient':float(np.max(np.abs(fun(r.x)[1]))),'nit':int(r.nit),'nfev':int(r.nfev)})
    good=[r for r in attempts if r['success'] and r['gradient']<1e-6 and r['objective']<1e90]
    if not good:return {'status':'unconverged','attempts':attempts}
    best=min(good,key=lambda r:r['objective']);alpha,logk=best['x'];gain=pure_nll-best['objective']
    if gain<=1e-10:
        return {'status':'unresolved_near_boundary','attempts':attempts,'per_observation_gain':gain}
    if any(abs(v-b[0])<1e-5 or abs(v-b[1])<1e-5 for v,b in zip(best['x'],bounds)):
        return {'status':'bound_hit','attempts':attempts,'per_observation_gain':gain}
    k=math.exp(logk);lp=-logxmin-alpha*y-k*np.expm1(y)-cutoff_logj(alpha,k)
    return {'status':'converged','alpha':alpha,'k':k,'loglike':float(lp.sum()),'logpdf':lp,'attempts':attempts,'per_observation_gain':gain}

def grid_pareto(x,grid=True):
    x=np.sort(np.asarray(x,dtype=float))
    values,first,counts=np.unique(x,return_index=True,return_counts=True);nt=len(x)-first
    eligible=np.flatnonzero(nt>=50)
    if grid and len(eligible)>100:
        eligible=eligible[np.array([j*(len(eligible)-1)//99 for j in range(100)])]
    best=None
    # One fitted distribution supplies both ECDF-side KS and final likelihood.
    # Degenerate supports are skipped; unsupported numerical fits fail closed.
    for ci in eligible:
        start=int(first[ci]);n=int(nt[ci]);xmin=float(values[ci])
        if x[-1]==xmin:continue
        pure=pareto_at(x[start:],xmin)
        tailvals=pure['log_ratios'][first[ci:]-start]
        cdf=-np.expm1(-(pure['alpha']-1)*tailvals)
        right=np.cumsum(counts[ci:])/n;left=right-counts[ci:]/n
        D=float(max(np.max(np.abs(cdf-left)),np.max(np.abs(right-cdf))))
        item={**pure,'ks':D,'candidate_count':len(eligible)}
        if best is None or D<best['ks']:best=item
    if best is None:raise NumericFailure('no nondegenerate eligible cutoff')
    return best

def likelihood_ratios(pure,alt,nested=False):
    if alt['status'] not in ('converged','boundary_pareto'):
        return {'status':'unresolved','fit_status':alt['status']}
    diff=np.asarray(pure['logpdf'])-np.asarray(alt['logpdf'])
    if diff.ndim!=1 or not len(diff) or not np.all(np.isfinite(diff)):
        return {'status':'unresolved_nonfinite_likelihood'}
    R=float(diff.sum())
    if not math.isfinite(R):return {'status':'unresolved_nonfinite_likelihood'}
    if nested:
        calibration={'calibration':'declared_half_chi_square_1','calibration_validity':'unvalidated',
                     'regular_boundary_information':pure['alpha']>3,
                     'calibration_limit':'alpha<=3 lacks finite second moment; alpha>3 alone is not validation'}
        if alt['status']=='boundary_pareto' and alt.get('certificate')=='convex_likelihood_KKT':
            return {'status':'available','R':0.,'T':0.,'p':1.,**calibration}
        T=-2*R
        if T<=0:return {'status':'unresolved_near_boundary','R':R,**calibration}
        return {'status':'available','R':R,'T':T,'p':float(.5*stats.chi2.sf(T,1)),**calibration}
    variance=float(np.mean((diff-np.mean(diff))**2))
    if variance<=1e-24 or not math.isfinite(variance):return {'status':'unresolved_zero_variance','R':R}
    z=R/math.sqrt(len(diff)*variance)
    return {'status':'available','R':R,'normalized_R':z,'p':float(special.erfc(abs(z)/math.sqrt(2))),'variance_divisor':'n'}
