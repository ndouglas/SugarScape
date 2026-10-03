"""Identical sources, real encounters, and separately labeled measured diagrams."""
import math
import animate
from ants_visual import neighborhood, recorded_event, filmed_frame
from blender import materials
from .parts import CREAM,ball,box,card,text,flump_pose
from .panels import _polyline


def path(name,parent,points,material):
    obj,spline=_polyline(name,parent,points,material)
    spline.points.foreach_set('co',[v for x,y in points for v in (x,y,.006,1)])
    return obj,spline


def selected(d,ctx,frame):
    return filmed_frame(d,ctx.timing,frame)


def ants_stage(beat,d,ctx):
    block=math.ceil(math.sqrt(len(d.placed)))
    cream=materials.knit('butter')
    # Identical geometry and material; source choice is not a walking model.
    for source in (1,2):
        x,y=animate.cell_center((source-1)*(block+2)+block/2,-1,d.width,d.height)
        for k in range(7):
            obj=ball(f'food-{source}-{k}',.40,cream)
            obj.location=(x+(k%3-1)*.55,y+(k//3)*.4,.3+(k==6)*.35)
    anchor=ctx.screen.anchor('source-heading',-.40,.80)
    ink=materials.fading('source-ink',CREAM,1.8)
    text('sources','SOURCE 1                 SOURCE 2',.046,ink,anchor)
    note=ctx.screen.anchor('stage-note',-.40,-.48)
    text('representation','Source choices · positions are illustrative',.037,ink,note)
    expected=ctx.measured['selected'][beat.shot].get('event')
    event=recorded_event(d.frames,expected) if expected else None
    marker=materials.fading('actual-event-glow',CREAM,3.0) if event else cream
    rings={}
    ids=[event['agent']]+([event['partner']] if event['partner'] else []) if event else []
    ids+= [i for i,m in d.frames[0].members.items() if m['independent']]
    for i in ids:
        points=[(.50*math.cos(k*math.tau/24),.50*math.sin(k*math.tau/24)) for k in range(25)]
        obj,spline=path(f'actual-member-{i}',None,points,marker)
        obj.data.bevel_depth=.085 if event else .04
        rings[i]=(obj,spline,points)
    connector=None
    if event and event['partner']:
        connector=path('recorded-recruitment-pair',None,[(0,0),(1,1)],marker)
        connector[0].data.bevel_depth=.075
    def update(frame):
        for i,(obj,spline,points) in rings.items():
            p=flump_pose(ctx,d,i,frame)
            spline.points.foreach_set('co',[v for x,y in points for v in (p.x+x,p.y+y,1.15,1)])
        if connector:
            positions=[flump_pose(ctx,d,i,frame) for i in (event['agent'],event['partner'])]
            connector[1].points.foreach_set('co',[v for p in positions for v in (p.x,p.y,1.10,1)])
    return update


def ants_panel(beat,d,ctx):
    anchor=ctx.screen.anchor('ants-panel',.63,.16)
    card('ants-card',anchor,(0,0,-.01),(.67,.91,.002))
    ink=materials.fading('ants-ink',CREAM,1.8)
    blue=materials.fading('measured-blue',materials.YARN['blue'],1.8)
    coral=materials.fading('exact-coral',materials.YARN['coral'],1.8)
    title=text('ants-title','',.050,ink,anchor,location=(0,.405,.004))
    count=text('actual-counts','',.050,ink,anchor,location=(0,.32,.004))
    clock=text('actual-clock','',.037,ink,anchor,location=(0,.245,.004))
    detail=text('ants-detail','',.038,ink,anchor,location=(0,-.30,.004))
    data=ctx.measured; name=beat.name
    meta=data['selected'][beat.shot]
    def label(obj,body): obj.data.body=body
    def small(name,body,y,size=.037): return text(name,body,size,ink,anchor,location=(0,y,.004))
    def plot(name,values,y,height=.17,mat=blue):
        maximum=max(values) or 1
        for i,value in enumerate(values):
            box(f'{name}-{i}',mat,anchor,location=(-.265+.53*(i+.5)/len(values),y+height*value/maximum/2,.004),scale=(.49/len(values),height*value/maximum,.002))
    def trace(name,points,n,horizon,y,height=.14):
        path(name,anchor,[(-.26+.52*t/horizon,y+height*k/n) for t,k in points],blue)
    if name in ('piles','choices','crowd','flip'):
        trace('selected-film',[(f.period,f.counts[0]) for f in d.frames],len(d.placed),d.frames[-1].period,-.02)
        small('trace-label',f'Filmed example · 0–{d.frames[-1].period} steps',-.07)
        small('trace-axis','Source 1 share · 0–100%',-.115)
        if name=='piles': label(detail,'Real-ant observation reported by\nKirman (1993), pp. 138–139\nNot a new model measurement')
        else: label(detail,'Complete graph · one partner/meeting\n50 meetings per step\nFood and rules stay fixed')
    elif name in ('meet','self'):
        e=recorded_event(d.frames,meta['event'])
        small('event-title','Recorded recruitment' if e['partner'] else 'Recorded spontaneous switch',.13)
        small('event-actor',f"Flump {e['agent']}: source {e['from_source']} → {e['to_source']}",.04)
        small('event-partner',f"Copies actual partner {e['partner']}" if e['partner'] else 'No partner',-.04)
        label(detail,f"Actual meeting {e['tick']} · update {e['update']}\nOne meeting per tick\nRings mark the recorded participants")
    elif name=='average':
        trace('short-example',[(f.period,f.counts[0]) for f in d.frames],100,2000,.02,.10)
        small('short-label','Compatible example · 0–2,000 steps',-.025)
        means=[r['mean'] for r in data['runs']['short'].values()]
        hist=[0]*20
        for m in means: hist[min(int(m*20),19)]+=1
        plot('means',hist,-.18,.10)
        small('means-label','All 1,000 run means · 0–100%',-.215)
        label(detail,'247 / 1,000 means in 40–60%\n2,000 steps each · 50 meetings/step\nOverlapping seeds are paired')
        detail.location.y=-.345
    elif name in ('splits','pull'):
        key='strong' if name=='splits' else 'pull'
        empirical=data['aggregates'][key]['distribution']; exact=data['exact'][key]['distribution']
        maximum=max(max(empirical),max(exact))
        # Shared vertical scale: the curve and histogram are directly comparable.
        for i,value in enumerate(empirical):
            box(f'measured-bin-{i}',blue,anchor,location=(-.265+.53*i/100,-.08+.20*value/maximum/2,.004),scale=(.004,.20*value/maximum,.002))
        path('exact-stationary',anchor,[(-.265+.53*i/100,-.08+.20*v/maximum) for i,v in enumerate(exact)],coral)
        small('distribution-axis','Source 1 agents · 0                 100',-.12)
        small('distribution-key','Blue: measured · coral: exact stationary',-.18,.032)
        ticks=data['protocols'][key]['ticks']
        modes='0, 100' if key=='strong' else '18, 82'
        label(detail,f'20 runs × {ticks:,} steps\nExact modes: {modes}\n'+('Figure IIb settings' if key=='strong' else 'Our majority-pull extension'))
    elif name=='more':
        vals=[data['aggregates'][k]['mean_extreme'] for k in ('strong','large')]
        plot('extreme-occupancy',vals,-.07,.20)
        small('size-labels','N = 100                 N = 1,000',-.115)
        small('occupancy-values',f'{vals[0]:.1%} crowded             {vals[1]:.1%} crowded',-.175,.035)
        label(detail,'Measured: 20 runs per population\n10 million meetings per run\nCrowded: ≤20% or ≥80% at source 1')
    elif name=='neighbors':
        # Only this actual ring neighborhood, never a dense all-graph hairball.
        actor=1;neighbors,_=neighborhood(actor,d.links,d.frames[0].members)
        positions={actor:(-.20,0)}
        positions.update({i:(.13,.14-.028*k) for k,i in enumerate(neighbors)})
        nodes={i:box(f'neighbor-{i}',blue,anchor,location=(*xy,.005),scale=(.017,.017,.002)) for i,xy in positions.items()}
        for i in neighbors:
            path(f'actual-edge-1-{i}',anchor,[positions[actor],positions[i]],ink)
        neighborlabel=small('opposite-count','',-.18)
        label(detail,'AM: counts every opposite neighbor\nP = (0.5 + opposite) / (0.5 + N)\nOne sweep = N sequential updates')
    elif name=='growth':
        small('growth-key','Measured variance ± independent-seed SE',.145,.031)
        small('growth-head','N         Ring k=10           Random p=.1',.075,.034)
        for j,n in enumerate((50,550,1050)):
            r=data['aggregates'][f'ring_{n}'];a=data['aggregates'][f'random_{n}']
            small(f'growth-{n}',f"{n:4}    {r['mean_variance']:.4f}±{r['variance_se']:.4f}    {a['mean_variance']:.4f}±{a['variance_se']:.4f}",.005-j*.075,.032)
        label(detail,'20 runs / size / network\n300,000 sweeps per run\nThree sizes; not the full paper regression')
    elif name=='independent':
        vals=[data['aggregates'][k]['mean_variance'] for k in ('independent_0','independent_0.05')]
        plot('independent-variance',vals,-.06,.19)
        small('independent-label','0%                    5% independent',-.10,.033)
        for j,key in enumerate(('independent_0','independent_0.05')):
            a=data['aggregates'][key]
            small('independent-stat-'+str(j),f"{j*5}%: variance {a['mean_variance']:.5f} ± {a['variance_se']:.5f} SE",-.16-j*.055,.032)
        label(detail,'20 runs × 300,000 sweeps\nSeparately generated graphs · p=.1\n50 actual non-herders: yellow + rings')
        detail.location.y=-.345
    def update(frame):
        f=selected(d,ctx,frame)
        label(title,{'growth':'Growth across three sizes','more':'Equal total meetings','neighbors':'Actual ring neighborhood','independent':'Independent agents'}.get(name,'Two equal food sources'))
        label(count,f"{f.counts[0]:,} at 1  ·  {f.counts[1]:,} at 2")
        label(clock,f"Film: {f.period:,} {meta['time_unit']} · N={len(d.placed):,}\nExample seed {d.seed}")
        if name=='neighbors':
            ids,opposite=neighborhood(actor,d.links,f.members)
            label(neighborlabel,f'Flump 1: {opposite} / {len(ids)} opposite')
            for i,obj in nodes.items(): obj.data.materials[0]=blue if f.members[i]['source']==1 else coral
    return update
