"""Exact thresholds, actual links, measured outcomes and participation traces."""
import math
import animate
from thresholds_visual import links, reached, people_counts
from blender import materials
from .panels import _polyline
from .parts import CREAM, box, card, text


def selected(d,ctx,frame):
    return d.frames[min(max(round(ctx.timing.tick_at(frame)),0),d.ticks)]


def thresholds_panel(beat,d,ctx):
    anchor=ctx.screen.anchor('thresholds-panel',.66,.18)
    card('thresholds-card',anchor,(0,0,-.01),(.59,.90,.002))
    ink=materials.fading('thresholds-ink',CREAM,1.8)
    text('participation-title','Participating',.045,ink,anchor,location=(0,.425,.003))
    label=text('participation','',.071,ink,anchor,location=(0,.35,.003))
    note=text('protocol','',.046,ink,anchor,location=(0,.19,.003))
    data=ctx.measured
    name=beat.shot
    protocol=data.get('protocols',{}).get(name)
    if protocol and name in ('city','friends','rescue','sparse','middle','dense'):
        histogram={int(k):v for k,v in protocol['histogram'].items()}
        population=len(d.placed)
        # Twenty bins cover every measured final outcome, including full participation.
        bins=[0]*20
        for count,frequency in histogram.items(): bins[min(count*20//population,19)]+=frequency
        height=max(bins) or 1
        for i,value in enumerate(bins):
            box(f'outcome-bin-{i}',ink,anchor,location=(-.21+i*.022,-.28+.07*value/height,.003),
                scale=(.018,.14*value/height,.002))
        text('histogram-axis','0    final participants    '+str(population),.043,ink,anchor,location=(0,-.31,.003))
        text('measured-runs',f"Our {protocol['runs']:,} measured runs",.046,ink,anchor,location=(0,-.36,.003))
        body=(f"≤1: {protocol['zero_or_one']}/{protocol['runs']}" if name=='city' else
              f"<10: {protocol['below_ten']}/{protocol['runs']}" if name=='friends' else
              f">1: {protocol['above_one']}/{protocol['runs']} · ≤10: {protocol['atmost_ten']}/{protocol['runs']}" if name=='rescue' else
              f"<1%: {protocol['small']}/{protocol['runs']}\n≥90%: {protocol['large']}/{protocol['runs']}")
        text('measured-counts',body,.050,ink,anchor,location=(0,-.415,.003))
    first=500 if name=='ceilings' else 0
    trace=[]
    for f in d.frames[first:]:
        share=sum(a['acting'] for a in f.members.values())/len(d.placed)
        if name=='ceilings': trace.append((-.21+.42*(f.period-500)/100,-.31+.40*(share-.85)/.10))
        else: trace.append((-.21+.42*f.period/max(d.frames[-1].period,1),-.06+.14*share))
    _,spline=_polyline('participation-trace',anchor,trace,ink)
    if name=='ceilings':
        text('trace-axis','Steps 500–600 · axis 85–95%',.047,ink,anchor,location=(0,-.39,.003))
        for y,axis_label in ((-.31,'85%'),(.09,'95%')):
            text('pulse-'+axis_label,axis_label,.036,ink,anchor,location=(-.24,y,.004),align='RIGHT')
    else: text('trace-axis','Participation × step',.043,ink,anchor,location=(0,-.09,.003))
    def update(frame):
        f=selected(d,ctx,frame)
        count=sum(a['acting'] for a in f.members.values())
        label.data.body=f'{count:,} / {len(d.placed):,}\nStep {f.step}'
        if d.config['network']=='random':
            forced=next(a for a in f.members.values() if a['seed'])
            neighbors=forced['neighbors']
            example=min(neighbors,key=lambda i:f.members[i]['degree']) if neighbors else None
            neighbor=f.members[example] if example is not None else None
            detail=(f"Neighbor {example}: {neighbor['sees']}/{neighbor['of']} {'≥' if reached(neighbor) else '<'} 18%" if neighbor else 'Seed has no neighbors')
            note.data.body=f"Mean degree {d.config['degree']:g} · threshold 18%\nOne forced seed · {len(neighbors)} actual neighbors\n{detail}\nRandom order within each step"
        elif name=='ceilings':
            note.data.body='Constructed ceiling extension\n10% leave above 90% · simultaneous'
        elif name in ('friends','rescue'):
            note.data.body=f"Our random friendships · probability 0.25\nFriends count {d.config['friends']['weight']} times"
        else: note.data.body=f'Fixed thresholds · simultaneous\nExample seed {d.seed}'
        now=min(max(round(ctx.timing.tick_at(frame))-first,0),len(trace)-1)
        points=[]
        for k in range(len(trace)):
            x,y=trace[min(k,now)]; points.extend((x,y,.003,1))
        spline.points.foreach_set('co',points)
    return update


def thresholds_markers(beat,d,ctx):
    ink=materials.knit('cream')
    labels=[]
    # Agent numbers teach the rule in close-ups; wide shots use the ruler.
    # Repeating those tiny labels in wide shots puts them behind screen cards.
    instructional = beat.name in ('instigator', 'change') and people_counts(d.config)
    for i in ((1,3) if beat.name=='change' else (1,2,3)) if instructional else ():
        if i not in d.frames[0].members: continue
        a=d.frames[0].agents[i]
        x,y=animate.cell_center(a.x,a.y,d.width,d.height)
        threshold=d.frames[0].members[i]['threshold']
        body=f'{round(threshold*len(d.placed))}' if d.config['network']=='everyone' else '18%'
        labels.append(text(f'threshold-label-{i}',body,.6,ink,None,location=(x,y+.25,1.65)))
    if beat.name=='change':
        a=d.frames[0].agents[2]
        x,y=animate.cell_center(a.x,a.y,d.width,d.height)
        labels.append(text('changed-threshold','1 → 2',.7,materials.knit('butter'),None,location=(x,y,2.1)))
    if people_counts(d.config) and beat.name not in ('instigator','change','stalled'):
        anchor=ctx.screen.anchor('threshold-ruler',-.55,.66)
        card('threshold-ruler-card',anchor,(0,0,-.01),(.75,.13,.002))
        panelink=materials.fading('threshold-ruler-ink',CREAM,1.8)
        text('threshold-ruler-title','Fixed thresholds · people required',.045,panelink,anchor,location=(0,.03,.003))
        for a in d.frames[0].members.values():
            value=a['threshold']*len(d.placed)
            mark=materials.knit('butter') if beat.shot=='perturbed' and a['id']==2 else panelink
            box(f"threshold-ruler-{a['id']}",mark,anchor,location=(-.33+.66*value/99,-.025,.004),scale=(.003,.04,.002))
    for i,a in d.frames[0].members.items():
        if not (a['seed'] or a['threshold']==0 or a['ceiling'] is not None): continue
        member=d.frames[0].agents[i]
        x,y=animate.cell_center(member.x,member.y,d.width,d.height)
        color='butter' if a['seed'] or a['threshold']==0 else 'cream'
        points=[(x+.43*math.cos(k*math.tau/32),y+.43*math.sin(k*math.tau/32)) for k in range(33)]
        obj,spline=_polyline(f'member-ring-{i}',None,points,materials.knit(color))
        obj.data.bevel_depth=.035
        spline.points.foreach_set('co',[v for px,py in points for v in (px,py,.06,1)])
    def update(frame):
        for label in labels: label.rotation_euler=ctx.camera.rotation_euler
    return update


def thresholds_links(beat,d,ctx):
    """Every rendered edge comes from the filmed graph; close-up shows one neighborhood."""
    members=d.frames[0].members
    edges=links(members)
    if beat.name=='network':
        seed=next(i for i,a in members.items() if a['seed'])
        edges=[edge for edge in edges if seed in edge]
    ink=materials.fading('actual-links',materials.YARN['cream'],.45)
    for i,j in edges:
        positions=[]
        for actor in (i,j):
            a=d.frames[0].agents[actor]
            positions.append(animate.cell_center(a.x,a.y,d.width,d.height))
        obj,spline=_polyline(f'actual-link-{i}-{j}',None,positions,ink)
        obj.data.bevel_depth=.015
        spline.points.foreach_set('co',[v for x,y in positions for v in (x,y,.04,1)])
    return lambda frame:None


def thresholds_comparison(beat,d,ctx):
    anchor=ctx.screen.anchor('thresholds-comparison',-.50,.62)
    card('comparison-card',anchor,(0,0,-.01),(.72,.42,.002))
    other=ctx.compare
    ink=materials.fading('comparison-ink',CREAM,1.8)
    label=text('comparison','',.049,ink,anchor,location=(0,.135,.003))
    trace=[(-.2+.4*f.period/max(other.frames[-1].period,1),-.12+.15*sum(a['acting'] for a in f.members.values())/len(other.placed)) for f in other.frames]
    _,spline=_polyline('comparison-trace',anchor,trace,ink)
    text('comparison-clock','Participation × actual step',.042,ink,anchor,location=(0,-.18,.003))
    def update(frame):
        index=min(round((frame-1)/max(beat.frames-1,1)*other.ticks),other.ticks)
        f=other.frames[index]
        count=sum(a['acting'] for a in f.members.values())
        label.data.body=f'Other measured example: {count}/{len(other.placed)}\nseed {other.seed} · step {f.step}'
        spline.points.foreach_set('co',[v for k in range(len(trace)) for v in (*trace[min(k,index)],.003,1)])
    return update


def thresholds_neighborhood(beat,d,ctx):
    """An inset contains precisely the forced seed and every actual neighbor."""
    anchor=ctx.screen.anchor('actual-neighborhood',-.45,.60)
    card('neighborhood-card',anchor,(0,0,-.01),(.9,.39,.002))
    members=d.frames[0].members
    seed=next(i for i,a in members.items() if a['seed'])
    neighbors=members[seed]['neighbors']
    ink=materials.fading('neighborhood-ink',CREAM,1.8)
    text('neighborhood-title',f'Actual seed {seed} · all {len(neighbors)} neighbors',.05,ink,anchor,location=(0,.13,.004))
    positions={seed:(-.25,0)}
    for k,i in enumerate(neighbors): positions[i]=(.20,.08-.16*k/max(len(neighbors)-1,1))
    mats={state:materials.fading(f'neighborhood-{state}',materials.YARN[color],1.8) for state,color in ((0,'blue'),(1,'coral'))}
    nodes={i:box(f'neighborhood-node-{i}',mats[int(members[i]['acting'])],anchor,location=(*xy,.005),scale=(.045,.045,.004)) for i,xy in positions.items()}
    for i in neighbors:
        _,spline=_polyline(f'neighborhood-edge-{i}',anchor,[positions[seed],positions[i]],ink)
        spline.points.foreach_set('co',[v for actor in (seed,i) for v in (*positions[actor],.003,1)])
        text(f'neighborhood-id-{i}',str(i),.037,ink,anchor,location=(.3,positions[i][1],.006))
    text('seed-id','Forced seed',.037,ink,anchor,location=(-.25,-.07,.006))
    text('neighborhood-note','Edges are the recorded graph · blue waits, coral acts',.035,ink,anchor,location=(0,-.135,.004))
    def update(frame):
        f=selected(d,ctx,frame)
        for i,obj in nodes.items(): obj.data.materials[0]=mats[int(f.members[i]['acting'])]
    return update
