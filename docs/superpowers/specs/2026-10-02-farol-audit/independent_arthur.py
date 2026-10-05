import numpy as np,csv
from pathlib import Path
out=csv.writer(open('arthur-independent-results.csv','w'));out.writerow(['scoring','seed','mean','variance','rms60','lag1'])
for seed in range(1,21):
 for payoff in [False,True]:
  rng=np.random.default_rng(seed);h=list(rng.integers(0,101,12));hold=np.array([rng.choice(48,12,replace=False) for _ in range(100)]);scores=np.zeros(48);trace=[]
  for t in range(2000):
   pred=[]
   pred.extend(h[-k] for k in range(1,13));pred.extend(100-h[-k] for k in range(1,9));pred.extend(np.floor(np.mean(h[-k:])+0.5) for k in range(2,13))
   for k in range(3,13):
    x=np.arange(k);y=np.array(h[-k:]);slope=np.sum((x-x.mean())*(y-y.mean()))/np.sum((x-x.mean())**2);pred.append(np.floor(y.mean()+slope*(k-x.mean())+0.5))
   pred.extend(100-np.floor(np.mean(h[-k:])+0.5) for k in range(2,9));pred=np.clip(pred,0,100)
   ratings=scores[hold];best=ratings.max(axis=1) if payoff else ratings.min(axis=1);tie=ratings==best[:,None];choices=(rng.random((100,12))*tie+tie).argmax(axis=1)
   a=int(np.sum(pred[hold[np.arange(100),choices]]<60));trace.append(a)
   if payoff:scores+=(pred>=60)==(a>=60)
   else:scores=0.9*scores+0.1*np.abs(pred-a)
   h.append(a);h=h[-12:]
  a=np.array(trace[400:]);mean=a.mean();var=a.var();lag=np.sum((a[:-1]-mean)*(a[1:]-mean))/np.sum((a-mean)**2);out.writerow(['payoff' if payoff else 'accuracy',seed,mean,var,np.sqrt(np.mean((a-60)**2)),lag])
  if seed==1:np.savetxt('arthur-independent-'+('payoff'if payoff else'accuracy')+'.csv',trace,fmt='%d')
 print('Arthur independent seed',seed,flush=True)
