import csv,collections,numpy as np,matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from PIL import Image
fig,ax=plt.subplots(1,3,figsize=(15,4.5),layout='constrained');crop=Image.open('cz97-figure4-crop.png');ax[0].imshow(crop);ax[0].axis('off');ax[0].set_title('CZ97 Figure 4: published counts\nN1001 M4 S5; horizon unstated')
hist=collections.defaultdict(lambda:np.zeros(1002))
for r in csv.DictReader(open('inverse-hist.csv')):hist[r['config']][int(r['attendance'])]+=int(r['count'])
for j,variant in enumerate(['rounded','exact'],1):
 for label,color in [('native','black'),('independent-redraw','blue'),('independent-retain','orange')]:
  key='fig4-native-'+variant if label=='native' else 'fig4-independent-'+variant+'-'+label.split('-')[1]
  v=hist[key];b=np.add.reduceat(v,np.arange(0,1002,10));ax[j].plot(np.arange(len(b))*10+5,b/v.sum(),label=label,color=color)
 ax[j].set(xlim=(0,1001),xlabel='Attendance A',ylabel='Probability per 10 attendees',title=f'{variant.title()} N/x−2\n20 seeds; rounds 1001–5000');ax[j].axvline(500.5,c='gray',ls=':');ax[j].legend(fontsize=8)
fig.savefig('figure4-comparison.png',dpi=180)
fig,ax=plt.subplots(1,3,figsize=(15,4.4),layout='constrained');crop=Image.open('arthur-figure1-crop.png');ax[0].imshow(crop);ax[0].axis('off');ax[0].set_title('Arthur 1994 Figure 1\nfirst 100 weeks; original predictor bank unstated')
for j,kind in enumerate(['accuracy','payoff'],1):
 a=np.loadtxt('arthur-native-'+kind+'.csv',delimiter=',')[:,1];b=np.loadtxt('arthur-independent-'+kind+'.csv');ax[j].plot(np.arange(1,101),a[:100],label='native PCG seed1',c='black');ax[j].plot(np.arange(1,101),b[:100],label='independent NumPy seed1',c='blue',alpha=.6);ax[j].axhline(60,c='red',ls=':');ax[j].set(xlim=(1,100),ylim=(0,100),xlabel='Week',ylabel='Attendance',title=f'{kind.title()} scoring; k12\nconstructed 48 predictors; random score ties');ax[j].legend(fontsize=8)
fig.savefig('arthur-figure1-comparison.png',dpi=180)
fig,ax=plt.subplots(1,3,figsize=(15,4),layout='constrained')
crop=Image.open("cz97-figure10-crop.png");ax[0].imshow(crop);ax[0].axis("off");ax[0].set_title("CZ97 Figure 10: pure population\nshown t1000–2000; virtual capitals may differ")
for j,ties in enumerate(['redraw','sticky'],1):
 a=np.loadtxt('monoclonal-'+ties+'.csv');ax[j].plot(np.arange(1001,2001),a[1000:],lw=.5);ax[j].set(ylim=(0,1001),xlabel='Round',ylabel='Attendance',title=f'Pure population: {ties} ties\nN1001 M6 S5; same five tables; seed1')
fig.savefig('monoclonal-comparison.png',dpi=180)
