from pathlib import Path
import csv,json,math
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from PIL import Image
HERE=Path(__file__).resolve().parent; BASE=HERE.parent; N=BASE/'native'
plt.rcParams.update({'font.size':10,'axes.spines.top':False,'axes.spines.right':False})
ROWS={(r['case'],r['metric']):r for r in csv.DictReader((N/'summary.csv').open())}
CROPS={'64':('ae-fig-09.png',(285,365,735,645)),'65':('ae-fig-10.png',(285,120,730,400)),'66':('ae-fig-11.png',(190,510,825,840)),'67':('ae-fig-12.png',(230,480,790,745)),'69':('ae-fig-14.png',(260,810,765,1110)),'610':('ae-p15-fig610.png',(250,170,940,510)),'611':('ae-p16-fig611.png',(235,365,945,750))}
# Source figure crops are retained verbatim; original full PDF pages are not distributed.
for key in CROPS: assert (HERE/f'source-{key}.png').is_file()
def frame(key,title):
 f,axs=plt.subplots(1,2,figsize=(14,5.5),gridspec_kw={'width_ratios':[1.1,1]});axs[0].imshow(Image.open(HERE/f'source-{key}.png'));axs[0].axis('off');axs[0].set_title('Original AE Figure '+key[0]+'-'+key[1:]+' (axes / labels unchanged)');axs[1].set_title(title);return f,axs[1]
def save(f,name,note):
 f.text(.51,.035,note,ha='center',fontsize=9);f.tight_layout(rect=(0,.09,1,1));f.savefig(HERE/(name+'.png'),dpi=170);plt.close(f)
def stat(case,metric):
 r=ROWS[case,metric];assert int(r['n'])==50,(case,metric,r['n']);assert int(r['attained'])+int(r['censored'])==50
 return r,float(r['conditional_mean'] or 'nan'),float(r['conditional_sample_sd'] or 'nan')
def error(ax,cases,x,metric,label,color):
 values=[stat(c,metric) for c in cases];y=np.array([v[1] for v in values]);sd=np.array([v[2] for v in values]);ax.errorbar(x,y,yerr=sd,fmt='o-',capsize=3,label=label,color=color)
 for xx,(r,yy,ss) in zip(x,values):ax.annotate(r['attained']+'/50',(xx,yy),xytext=(0,8),textcoords='offset points',ha='center',fontsize=8,color=color)
 ax.set_ylabel('95% eligible-retired proxy time (periods)');ax.grid(alpha=.2)
def trace(case):return list(csv.DictReader((N/(case+'-1001-trace.csv')).open()))
for key,rs,limit in [('64',[.15,.2],25),('65',[.05],500)]:
 f,ax=frame(key,'Native: frozen seed 1001 (no trajectory selection)')
 for r in rs:
  c='trajectory_'+str(r);data=trace(c);ax.plot([int(t['tick']) for t in data],[float(t['retired']) for t in data],label=f'R={r:.0%}, seed1001')
  row,mean,sd=stat(c,'first95');ax.axvspan(max(0,mean-sd),mean+sd,alpha=.12,label=f'50-run crossing mean ± SD: {mean:.1f} ± {sd:.1f}')
 ax.set(xlim=(0,limit),ylim=(0,1.05),xlabel='Period',ylabel='Retired / eligible');ax.legend(fontsize=8);ax.grid(alpha=.2)
 save(f,'comparison-'+key,'Native C100, random5%, Slot / ByCohort / Literal, τ=.5, S10–25, extent5; horizon500.\nBand is variability of first95 crossing times, NOT a time-series ensemble envelope.')
f,ax=frame('66','Native selected R; 95% proxy, horizon2000')
error(ax,['critical_Slot_'+str(x) for x in [0,.02,.05,.1]],[0,2,5,10],'first95','Slot; random5%','#1565a8');ax.set(xlabel='Rational population (%)');ax.legend();save(f,'comparison-66','Conditional mean ± sample SD; labels attained/50, remaining runs right-censored at2000.\nOriginal age65 norm stopping rule unpublished; 0/2% native points have no matched source5%-random points.')
f,ax=frame('67','Native source grid: R10%, τ mean .5')
xs=[.05,.1,.2,.3,.4,.5];error(ax,['spread-source_'+str(x) for x in xs],np.array(xs)/math.sqrt(3),'first95','Uniform threshold; source positive grid','#1565a8');ax.set(xlabel='Threshold standard deviation');ax.legend(fontsize=8);save(f,'comparison-67','50 fresh runs/point; conditional mean ± sample SD; labels attained/50; horizon600.\nPositive σ≈.0289,.0577,.1155,.1732,.2309,.2887; zero extension excluded. 95% proxy ≠ author norm.')
f,ax=frame('69','Native source branch: R5%, extent6–10')
error(ax,['extent_'+str(x) for x in range(6,11)],range(6,11),'first95','R5%, random5%','#1565a8');ax.set(xlabel='Maximum cohort extent');ax.legend();save(f,'comparison-69','Conditional mean ± sample SD, attained/50 labels; horizon600. Source R5% branch only6–10.\nNative Slot / ByCohort / Literal; original stopping rule unspecified.')
f,ax=frame('611','Native source groups: B rational10%, A0%')
cs=[.05,.08,.1,.12,.15,.2];cases=['groups-source_'+str(x) for x in cs]
error(ax,cases,cs,'group_a','A: no rationals','#ae5029');error(ax,cases,cs,'group_b','B: 10% rational','#1565a8');ax.set(xlabel='Cross-group coupling');ax.legend(fontsize=8);save(f,'comparison-611','Config rational=.10 suppresses A rationals: expected global5%, B10%; Slot, C100 total, horizon600.\nConditional mean ± SD; labels attained/50. Source rational group ALSO slows as coupling rises.')
for setup in ['ae','gss']:
 f,ax=frame('610',f'Native policy: {setup.upper()}, automatic proxy warmup')
 xs=[.01,.02,.04,.05];cases=[f'policy_{setup}_{x}_auto' for x in xs]
 error(ax,cases,np.array(xs)*100,'proxy_new','95% proxy after actual switch','#1565a8');error(ax,cases,np.array(xs)*100,'sustained62_post','Rolling mode62 ×10 diagnostic','#ae5029');ax.set_ylabel('Periods after actual eligibility switch');ax.plot([1,2,3,4],[70,40,30,20],'k:x',label='AE source means (visual estimates)');ax.set(xlabel='Rational population (%)');ax.legend(fontsize=8)
 save(f,'comparison-policy-'+setup,'AE τ=.5 vs GSS U[.5,1] plotted separately; random5%, mandatory70, 100 post-switch periods.\nMeans conditional on attainment; sample SD; labels attained/50. Neither proxy nor mode diagnostic is author stopping rule.')
# Exact event ages and native mode on representative policy traces, including actual switch.
f,axs=plt.subplots(2,2,figsize=(13,7),sharex='col')
for col,setup in enumerate(['ae','gss']):
 c=f'policy_{setup}_0.05_auto';rows=trace(c);seeds=list(csv.DictReader((N/'policy-seeds.csv').open()));seed=next(x for x in seeds if x['case']==c and x['seed']=='1001');switch=int(seed['switch']);t=np.array([int(x['tick'])-switch for x in rows]);mode=[float(x['modal_age']) for x in rows];fr=[]
 for x in rows:
  ages={int(a):int(b) for a,b in (v.split(':') for v in x['age_events'].split(';') if v)};assert sum(ages.values())==int(x['new_events']);fr.append(ages.get(int(x['eligibility']),0)/int(x['new_events']) if int(x['new_events']) else np.nan)
 axs[0,col].plot(t,mode);axs[0,col].set_title(setup.upper()+': R5%, seed1001');axs[0,col].set_ylabel('Native rolling event mode (age)');axs[1,col].plot(t,fr);axs[1,col].set_ylabel('New events at eligibility age / all events');axs[1,col].set_xlabel('Period relative to actual policy switch');axs[1,col].set_ylim(0,1.05)
 for ax in axs[:,col]:ax.axvline(0,color='k',ls=':');ax.set_xlim(-30,100);ax.grid(alpha=.2)
f.suptitle('Policy age diagnostics: mode and earliest retirement event fraction are distinct');save(f,'policy-age-diagnostics','Exact age histogram totals verified against new_events; all seeds frozen, seed1001 chosen in protocol.\nEvent count mode is NOT retirement hazard or populationwide norm; zero-event fractions are missing.')
print('Generated comparisons and verified histogram / summary count consistency.')
