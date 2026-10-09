"""Render all registered native contrasts; never recompute scientific summaries."""
from pathlib import Path
import argparse,hashlib,json,platform,sys,xml.etree.ElementTree as ET
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.text import Text
from PIL import Image
from bind_rows import bind_rows,FAMILIES,METRICS
TITLES={'quota_attained':('Quota attained by tick 64','Difference in attainment proportion; positive = more attainment'), 'restricted_completion_ticks':('Restricted completion time (64-tick cap)','Difference in restricted ticks; negative = earlier attainment'), 'gross_gathered':('Gross food gathered through tick 64','Difference in gross food units; positive = more gathered'), 'living_ticks':('Living ticks through tick 64','Difference in living ticks; positive = longer capped life')}
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--data',type=Path,required=True);parser.add_argument('--out',type=Path,required=True);args=parser.parse_args()
    analysis_bytes=(args.data/'analysis.json').read_bytes();analysis=json.loads(analysis_bytes)
    chart_bytes=(args.data/'native-chart-inputs.json').read_bytes();chart=json.loads(chart_bytes)
    groups=bind_rows(chart,analysis,hashlib.sha256(analysis_bytes).hexdigest())
    args.out.mkdir(exist_ok=True)
    if list(args.out.glob('*.png')) or list(args.out.glob('*.svg')):raise ValueError('create-only plots: output already populated')
    plt.rcParams.update({'font.family':'DejaVu Sans','font.size':11,'svg.fonttype':'none','svg.hashsalt':'minds-bt-e67b8d2','axes.spines.top':False,'axes.spines.right':False})
    limits={m:max(1,max(abs(v) for (f,metric),rs in groups.items() if metric==m for r in rs for v in (r['native']['summary']['ci95'] or [r['native']['summary']['mean']]))) for m in METRICS}
    bindings=[];inventory=[];layout=[]
    for (family,metric),rows in groups.items():
        fig,ax=plt.subplots(figsize=(15,10.5),dpi=140)
        fig.subplots_adjust(left=.335,right=.705,bottom=.20,top=.80)
        fig.text(.045,.945,f'{family.replace("_"," ").title()} · Guarded Tree − {FAMILIES[family]}',fontsize=20,weight='bold')
        fig.text(.045,.898,TITLES[metric][0],fontsize=17)
        fig.text(.045,.86,'All 16 registered strata • 8 base / 8 reflected • all 40 paired labels per row',fontsize=11)
        ax.axvline(0,color='#5c6068',linewidth=1,zorder=1)
        ax.axhspan(7.5,15.7,color='#f0f4f7',zorder=0)
        ax.axhline(7.5,color='#bac4cf',linewidth=.8)
        labels=[]
        for y,r in enumerate(rows):
            native=r['native'];s=native['summary'];mean=s['mean'];ci=s['ci95'];color='#176e8a' if r['orientation']=='base' else '#7d4c91'
            if ci is not None:ax.errorbar(mean,y,xerr=[[mean-ci[0]],[ci[1]-mean]],fmt='o',color=color,capsize=4,markersize=6,linewidth=1.5,zorder=3)
            else:ax.plot(mean,y,'o',color=color)
            labels.append(f'{r["orientation"].title()} · {r["scenario"].replace("-"," ")} · quota {r["quota"]}')
            interval='unavailable' if ci is None else f'[{ci[0]:g}, {ci[1]:g}]'
            ax.text(1.035,y,f'{mean:g}  {interval}',transform=ax.get_yaxis_transform(),va='center',fontsize=10)
            ax.text(1.39,y,f'{native["denominator"]} | {s["positive"]}/{s["zero"]}/{s["negative"]}',transform=ax.get_yaxis_transform(),va='center',fontsize=10)
            bindings.append({'figure':f'{family}-{metric}','row':y,**r})
        ax.set_yticks(range(16),labels,fontsize=11);ax.tick_params(axis='y',length=0,pad=12)
        ax.set_ylim(15.7,-.7);ax.set_xlim(-limits[metric]*1.15,limits[metric]*1.15)
        ax.grid(axis='x',alpha=.18);ax.set_xlabel(TITLES[metric][1],labelpad=12,fontsize=10)
        ax.text(1.035,1.05,'Mean [native 95% CI]',transform=ax.transAxes,fontsize=10,weight='bold')
        ax.text(1.39,1.05,'n | + / 0 / −',transform=ax.transAxes,fontsize=10,weight='bold')
        fig.text(.045,.105,'Paired descriptive native intervals; zero-width intervals are retained. Seed labels are not independent animals.',fontsize=11)
        foot='Unattained tasks receive restricted time 64; this is not observed completion. Depleted quota 40: unrestricted time unavailable (died before quota).'
        if metric!='restricted_completion_ticks':foot='Depleted quota 40 is a floor: available food 28 < 40. Unrestricted completion remains unavailable; no successful-only subset.'
        fig.text(.045,.075,foot,fontsize=10)
        fig.text(.045,.045,'Frozen native e67b8d2 • fixed supplied routines • chart values copied exactly from native analysis • no pooled family estimates',fontsize=10,color='#555555')
        fig.canvas.draw();renderer=fig.canvas.get_renderer();bad=[]
        for text in fig.findobj(Text):
            if text.get_visible() and text.get_text():
                box=text.get_window_extent(renderer)
                if box.x0 < -1 or box.y0 < -1 or box.x1 > fig.bbox.width+1 or box.y1>fig.bbox.height+1:bad.append(text.get_text())
        if bad:raise ValueError(f'clipped text: {family}/{metric}: {bad}')
        layout.append({'figure':f'{family}-{metric}','visible_text_inside_canvas':True,'rows':16,'base':8,'reflected':8})
        for ext in ['png','svg']:
            path=args.out/f'{family}-{metric}.{ext}'
            metadata={'Title':f'{family}: Guarded Tree minus {FAMILIES[family]}; {TITLES[metric][0]}','Description':'All native registered paired rows; n=40 each; descriptive intervals retained.'}
            if ext=='svg':metadata['Date']=None
            fig.savefig(path,metadata=metadata)
            entry={'path':path.name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':path.stat().st_size,'mode':format(path.stat().st_mode,'o'),'family':family,'metric':metric,'rows':16}
            if ext=='png':entry['pixel_dimensions']=list(Image.open(path).size)
            else:
                svg=ET.parse(path).getroot();entry['svg_dimensions']={k:svg.attrib[k] for k in ['width','height','viewBox']}
            inventory.append(entry)
        plt.close(fig)
    for name,data in [('plot-row-bindings.json',bindings),('plot-inventory.json',inventory),('layout-checks.json',layout)]:
        with (args.out/name).open('x') as f:json.dump(data,f,indent=2);f.write('\n')
    environment={'python':sys.version,'executable':sys.executable,'platform':platform.platform(),'matplotlib':matplotlib.__version__,'matplotlib_module':matplotlib.__file__,'backend':matplotlib.get_backend(),'font':'DejaVu Sans','analysis_sha256':hashlib.sha256(analysis_bytes).hexdigest(),'chart_inputs_sha256':hashlib.sha256(chart_bytes).hexdigest(),'scientific_recomputation':False}
    with (args.out/'plot-environment.json').open('x') as f:json.dump(environment,f,indent=2);f.write('\n')
    print('Rendered 16 PNG + 16 SVG; 256 exact native row bindings; 16 canvas checks passed')
if __name__=='__main__':main()
