# Emergent polarity source figure extraction

Source-only extraction on 2026-10-02, before any polarity model measurement.
This records manual tracing; a second-reader extraction has not been performed.
No source executable or model output was used.

## Evidence and limitations

The JSON holds 72 sampled source positions: 16 means, 32 original category
positions, 16 PRA positions, and 8 inferred tax positions. Original predator
settings are stated in the source: 0,.05,.1,.2,.4,.6,.8,1. There are no point
markers. Sampling the connecting curve at a stated setting does not add an
observation. Book chapter4 reprises original figures and is not an independent
dataset; chapter5 PRA and tax figures represent changed experiments.

The exact defense/no-alliance/.6 category vector 5/9/5/0/1 comes from original
printed518 and supersedes its raster intervals. All other category targets are
sets of feasible integer vectors, never nearest-integer point estimates.
Twenty replications per tax setting is inherited from the base protocol, not
restated alongside Fig5.8.

Fig5.8 permits nominal candidates 0,.05,.1,.2,.4,.6,.8,1 through visible
endpoints, slope changes and steps. These are visually inferred with retained
abscissa intervals. The source does not certify that this is the complete tax
knot list. Additional knots on straight, overlapping or unchanged boundary
segments are unavailable. A manifest using these candidates must state that
inference; it must not claim exact recovery or interpolate extra observations.

Ordinary tracing ranges are +/-18px in original category figures and +/-14px
in book figures, with +/-3px axis uncertainty. Steep low-share samples use
+/-40px to include horizontal calibration uncertainty. Original11 offense/.05
unipolar tracing uses +/-75px because the boundary rapidly changes direction
and blends into the neighboring texture. Original13 defense/1 unipolar boundary
uses +/-40px because its label overlaps. Tax/.2 bipolar and multipolar boundaries
use +/-45px and +/-24px because adjacent textures blend near the vertex. These
are extraction resolution ranges, not confidence intervals or source sampling
uncertainty. They were selected from the printed images before any model run.
Feasibility is checked after extraction; an empty feasible set would remain
unresolved and would not cause widening toward a desired model result.

All five figure page renders, lossless crops and annotated measurement grids
were inspected with view_image. Book printed118 equations5.2/5.3 and printed119
Fig5.5 were also inspected. The time4 diagram and example print21.4 with
denominator16+40, while the displayed passive-front rule applied to the prior
opposing allocation30 gives30/(16+30)*40=26.0869565. Preserve the discrepancy.

## Reproduction

Images stay outside the tracked repository in `/tmp/polarity-digitization/`.
The JSON records exact page and crop hashes; PNG hashes depend on the rendering
and PNG library versions. Run Poppler at300dpi, then losslessly crop with Pillow.
Printed-page/pdftoppm-page mapping: original518/19,519/20,524/25; book120/135
and126/141. The reading notes' original mapping uses zero-based PDF indexes.

```bash
mkdir -p /tmp/polarity-digitization
pdftoppm -f 19 -l 20 -r 300 -png papers/geopolitics/cederman-1994-isq-emergent-polarity.pdf /tmp/polarity-digitization/figures
pdftoppm -f 25 -l 25 -r 300 -png papers/geopolitics/cederman-1994-isq-emergent-polarity.pdf /tmp/polarity-digitization/figures
pdftoppm -f 135 -l 135 -r 300 -png papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf /tmp/polarity-digitization/pra
pdftoppm -f 141 -l 141 -r 300 -png papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf /tmp/polarity-digitization/tax
pdftoppm -f 133 -l 134 -r 300 -png papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf /tmp/polarity-digitization/equations
```

Lossless crop generator:

```python
from PIL import Image
from pathlib import Path
r=Path('/tmp/polarity-digitization')
boxes={'figures-19.png':(400,480,1400,1450),'figures-20.png':(500,440,1420,2550),'figures-25.png':(500,440,1420,2510),'pra-135.png':(210,1470,1670,2190),'tax-141.png':(600,350,1340,1080)}
for fn,box in boxes.items():
    Image.open(r/fn).crop(box).save(r/('crop-'+fn),dpi=(300,300))
for fn in ['figures-20.png','figures-25.png']:
    im=Image.open(r/('crop-'+fn))
    im.crop((0,0,920,950)).save(r/('top-'+fn),dpi=(300,300))
    im.crop((0,1160,920,2070)).save(r/('bottom-'+fn),dpi=(300,300))
```

The following complete generator was executed as
`python3 /tmp/polarity-digitization/build_source_figures.py`. Paths P and W can
be changed for a different checkout. It regenerates source JSON from the frozen
manual ordinates, propagates axis uncertainty and enumerates all feasible vectors.

```python
import hashlib,json,itertools,math
from pathlib import Path
R=Path('/tmp/polarity-digitization')
W=Path('/Users/nathan/.config/superpowers/worktrees/SugarScape/polarity-scratch/docs/superpowers/specs')
P=Path('/Users/nathan/Projects/ndouglas/SugarScape/papers/geopolitics')
shares=[0,.05,.1,.2,.4,.6,.8,1]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def img(fn,box=None):
 from PIL import Image
 p=R/fn
 return {'scratch_path':str(p),'sha256':sha(p),'dimensions_px':list(Image.open(p).size),'dpi':300,'crop_box_in_page_px':box}
def calibration(ax):
 x0,x1,y0,y20=ax
 return {'x_zero_px':x0,'x_one_px':x1,'y_zero_px':y0,'y_twenty_px':y20,'axis_uncertainty_px':3,'coordinate_origin':'top-left of named image; x right, y down','conversion':'count=20*(y_zero-y)/(y_zero-y_twenty); x=(pixel_x-x_zero)/(x_one-x_zero)'}
def ci(y,e,ax):
 y0,y20=ax[2:]; vals=[20*(z-v)/(z-t) for z in [y0-3,y0+3] for t in [y20-3,y20+3] for v in [y-e,y+e]]
 return [round(max(0,min(vals)),6),round(min(20,max(vals)),6)]
def vectors(intervals):
 ranges=[range(max(0,math.ceil(a)),min(20,math.floor(b))+1) for a,b in intervals]
 return [[v[0],v[1]-v[0],v[2]-v[1],v[3]-v[2],20-v[3]] for v in itertools.product(*ranges) if list(v)==sorted(v)]
sources=[{'id':'cederman1994','paper':'Cederman 1994, Emergent Polarity, ISQ38(4):501-533','local_pdf':'papers/geopolitics/cederman-1994-isq-emergent-polarity.pdf','sha256':sha(P/'cederman-1994-isq-emergent-polarity.pdf')},{'id':'cederman1997','paper':'Cederman1997, Emergent Actors in World Politics','local_pdf':'papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf','sha256':sha(P/'cederman-1997-emergent-actors-in-world-politics.pdf')}]
# Pixel ordinate estimates after direct visual inspection of unannotated 300dpi crops.
# Entries are bottom-up cumulative category boundaries, not category counts.
raw=[
 ('11','defense','top-figures-20.png',[500,440,1420,1390],(66,852,816,33),3,False,'equal',[[816]*4,[620,585,580,552],[583,583,583,423],[542,465,465,350],[580,345,310,310],[620,270,80,75],[620,386,153,75],[696,310,114,112]]),
 ('11','offense','bottom-figures-20.png',[500,1600,1420,2510],(94,878,872,88),2,False,'equal',[[872]*4,[625,328,122,88],[858,598,88,88],[833,675,88,88],[795,675,88,88],[795,714,88,88],[754,636,88,88],[754,754,88,88]]),
 ('13','defense','top-figures-25.png',[500,440,1420,1390],(72,850,807,36),3,True,'equal',[[807]*4,[807]*4,[807]*4,[807]*4,[690,690,690,422],[538,538,538,36],[193,193,154,76],[245,154,76,36]]),
 ('13','offense','bottom-figures-25.png',[500,1600,1420,2510],(76,858,849,75),2,True,'equal',[[849]*4,[694,694,694,578],[578,578,578,346],[500,461,461,193],[387,193,114,75],[426,114,75,75],[230,75,75,75],[270,114,114,75]]),
 ('5.6','defense','crop-pra-135.png',[210,1470,1670,2190],(47,669,644,25),3,False,'pra',[[644]*4,[489,430,430,430],[396,334,334,334],[368,241,241,241],[396,211,149,149],[427,180,87,87],[489,272,56,56],[582,272,25,25]]),
 ('5.6','offense','crop-pra-135.png',[210,1470,1670,2190],(818,1440,644,25),2,False,'pra',[[644]*4,[520,520,25,25],[644,644,25,25],[644,644,25,25],[644,613,56,25],[582,551,25,25],[551,489,87,25],[582,520,87,25]]),
 ('5.8','tax','crop-tax-141.png',[600,350,1340,1080],(68,695,667,38),2,False,'pra',[[667]*4,[667,667,667,38],[667,667,667,38],[415,384,336,38],[510,289,38,38],[667,541,38,38],[667,510,38,38],[667,635,69,38]])]
figures={}
for fid,pid,fn,box,ax,ratio,alliance,allocation,ys in raw:
 f=figures.setdefault(fid,{'id':'figure_'+fid,'source_id':'cederman1994' if fid in ['11','13'] else 'cederman1997','printed_page':{'11':519,'13':524,'5.6':120,'5.8':126}[fid],'pdf_page_one_based':{'11':20,'13':25,'5.6':135,'5.8':141}[fid],'panels':[]})
 panel={'id':pid,'ratio':ratio,'alliances':alliance,'allocation':allocation,'image':img(fn,box),'axis_calibration':calibration(ax),'samples':[]}
 for idx,(x,yy) in enumerate(zip(shares,ys)):
  e=18 if fid in ['11','13'] else 14
  if x in [.05,.1]:e=40 # steep raster gradients amplify horizontal calibration uncertainty
  errors=[e]*4
  if fid=='11' and pid=='offense' and x==.05:errors[0]=75 # fading rapid-turnover texture boundary
  if fid=='5.8' and x==.2:errors[1:3]=[45,24] # adjacent textured bands blend near this vertex
  if fid=='13' and pid=='defense' and x==1:errors[0]=40 # printed bipolar label overlaps boundary
  intervals=[ci(y,err,ax) for y,err in zip(yy,errors)]
  sample={'tax_rate' if fid=='5.8' else 'predator_share':x,'coordinates_px':{'x':round(ax[0]+x*(ax[1]-ax[0]),3),'x_uncertainty':3,'cumulative_boundary_y':yy,'boundary_y_uncertainties':errors},'cumulative_count_intervals':intervals,'admissible_category_count_vectors':vectors(intervals),'extraction_status':'recoverable_interval'}
  if fid=='11' and pid=='defense' and x==.6:
   sample.update(extraction_status='exact_text_override',exact_text_counts=[5,9,5,0,1],admissible_category_count_vectors=[[5,9,5,0,1]],exact_text_source='Cederman1994 printed518, paragraph immediately below Fig10; supersedes image intervals')
  if fid=='5.8':
   sample.update(abscissa_interval=[round(max(0,x-.012),3),round(min(1,x+.012),3)],knot_evidence='Endpoint or visible boundary slope/step change; nominal rounded axis reading, no point marker',knot_status='visually_inferred_nominal_setting')
  if not sample['admissible_category_count_vectors']:sample['extraction_status']='unresolved_no_feasible_integer_vector'
  panel['samples'].append(sample)
 f['panels'].append(panel)
# Fig10 has its own 0..100 ordinate calibration.
ax=(146,934,843,56)
f10={'id':'figure_10','source_id':'cederman1994','printed_page':518,'pdf_page_one_based':19,'image':img('crop-figures-19.png',[400,480,1400,1450]),'axis_calibration':{'x_zero_px':146,'x_one_px':934,'y_zero_px':843,'y_hundred_px':56,'axis_uncertainty_px':3,'conversion':'mean=100*(843-y)/(843-56)'},'panels':[]}
for pid,ratio,ys in [('defense',3,[56,312,420,505,562,789,744,751]),('offense',2,[56,785,815,812,808,808,808,807])]:
 p={'id':pid,'ratio':ratio,'alliances':False,'allocation':'equal','samples':[]}
 for x,y in zip(shares,ys):
  e=14 if x in [.05,.1] else 8
  vals=[100*(z-v)/(z-t) for z in [840,846] for t in [53,59] for v in [y-e,y+e]]
  p['samples'].append({'predator_share':x,'coordinates_px':{'x':round(146+788*x,3),'y':y,'x_uncertainty':3,'y_uncertainty':e},'mean_interval':[round(max(1,min(vals)),6),round(min(100,max(vals)),6)],'extraction_status':'recoverable_interval','mean_quantization':.05})
 f10['panels'].append(p)
data={'schema_version':1,'recovered_on':'2026-10-02','status':'source_only_manual_digitization','categories':['1','2','3-10','11-90','91-100'],'runs_per_setting':20,'sources':sources,'method':{'description':'Visual ordinate sampling at stated original predator settings, 300dpi Poppler page renders and lossless crops. No point markers. Pixel intervals include tracing, scan stair-stepping and axis uncertainty; all feasible integer cumulative vectors enumerated without nearest-integer rounding. No model runs.','ordinary_y_uncertainty_px':{'original':18,'book':14},'steep_low_share_y_uncertainty_px':40,'calibration_uncertainty_px':3,'confidence_intervals':False,'not_sampling_uncertainty':True,'vector_enumeration':'For each four cumulative intervals enumerate every integer cumulative tuple 0<=c1<=c2<=c3<=c4<=20, then differences [c1,c2-c1,c3-c2,c4-c3,20-c4]. Exact text override takes priority.','limitations':['Intervals represent manual tracing resolution, not a statistical confidence level or independent second-reader extraction.','Book chapter4 repeats original results and is not an independent dataset. Chapter5 PRA and tax figures are distinct experiments.','Unmarked Fig5.8 cannot certify the complete experiment tax knot list; visible changes supply inferred nominal settings only. Extra collinear/hidden settings remain unavailable.','Tax sample abscissa intervals are retained; nominal values must be labeled inferred in any manifest.','A missing feasible vector stays unresolved; uncertainty must not be expanded to match reconstruction outcomes.']},'figures':[f10,*figures.values()],'tax_knot_recovery':{'nominal_recoverable_candidates':shares,'complete_source_experiment_knot_list_available':False,'status':'inferred_from_visible_vertices_and_steps','unrecoverable':'Any additional knot on an unchanged/straight boundary segment; exact source abscissae and seeds','do_not_interpolate_extra_observations':True},'equation_inspection':{'source_id':'cederman1997','printed_pages':[118,119],'images':[img('equations-133.png'),img('equations-134.png')],'findings':['Equations5.2/5.3 have previous-period opposing-front commitments in numerator and active-front sum in denominator, passive denominator additionally includes the candidate front.','Fig5.5 time4 prints21.4 and accompanying example uses denominator16+40; applying displayed passive rule to the stated prior opposing front30 gives30/(16+30)*40=26.0869565. Preserve discrepancy.']},'reproduction_notes':'2026-10-02-emergent-polarity-source-extraction.md'}
(W/'2026-10-02-emergent-polarity-source-figures.json').write_text(json.dumps(data,indent=2)+'\n')
print('samples',sum(len(p['samples']) for f in data['figures'] for p in f['panels']))
print('unresolved',[(f['id'],p['id'],s.get('predator_share',s.get('tax_rate'))) for f in data['figures'] for p in f['panels'] for s in p['samples'] if s['extraction_status'].startswith('unresolved')])
```
