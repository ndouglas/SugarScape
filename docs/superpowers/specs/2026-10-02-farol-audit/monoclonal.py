import numpy as np,csv
f=csv.writer(open('monoclonal-results.csv','w'));f.writerow(['ties','seed','mean','rms_center','variance','central_share'])
for seed in range(1,21):
 for sticky in [False,True]:
  rng=np.random.default_rng(seed);tables=rng.integers(0,2,(5,64));scores=np.zeros((1001,5));active=np.zeros(1001,dtype=int);h=int(rng.integers(64));trace=[]
  for t in range(2000):
   top=scores.max(axis=1);ties=scores==top[:,None]
   if sticky:change=scores[np.arange(1001),active]<top;active[change]=(rng.random((change.sum(),5))*ties[change]+ties[change]).argmax(axis=1)
   else:active=(rng.random((1001,5))*ties+ties).argmax(axis=1)
   a=int(tables[active,h].sum());win=a<=500;scores+=(tables[:,h]==win);h=((h<<1)|int(win))&63;trace.append(a)
  a=np.array(trace[1000:]);f.writerow(['sticky'if sticky else'redraw',seed,a.mean(),np.sqrt(np.mean((a-500.5)**2)),a.var(),np.mean(np.abs(a-500.5)<=50.05)])
  if seed==1:np.savetxt('monoclonal-'+('sticky'if sticky else'redraw')+'.csv',trace,fmt='%d')
