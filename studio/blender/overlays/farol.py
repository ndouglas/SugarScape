"""Reserved Farol diagrams: actual choices and separately labeled ensembles."""
import math
from farol_visual import filmed_frame, selected_strategy, memory_bits, outcome, distribution
from blender import materials
from .parts import CREAM,box,card,text
from .panels import _polyline


def path(name,parent,points,material):
    obj,spline=_polyline(name,parent,points,material)
    spline.points.foreach_set('co',[v for x,y in points for v in (x,y,.006,1)])
    return obj,spline


def farol_stage(beat,d,ctx):
    minority=d.config['game']=='minority'
    ink=materials.fading('farol-stage-ink',CREAM,1.8)
    anchor=ctx.screen.anchor('farol-heading',-.42,.83)
    text('farol-places','B                                        A' if minority else 'HOME                                  BAR',.047,ink,anchor)
    subtitle=ctx.screen.anchor('farol-rule',-.42,.70)
    text('farol-capacity','101 Flumps · smaller side wins' if minority else '100 Flumps · crowded at 60',.033,ink,subtitle)
    note=ctx.screen.anchor('farol-stage-note',-.42,-.48)
    text('farol-positions','Actual choices · illustrative positions',.032,ink,note)
    # Two equal felt areas under the real agent slots; no invented agents.
    block=math.ceil(math.sqrt(len(d.placed)))
    for side,color in ((0,'blue'),(1,'coral')):
        obj=box(f'farol-felt-{side}',materials.knit(color),None,
                location=(side*(block+2)+block/2-d.width/2,0,.025),
                scale=(block+.8,d.height+.8,.04))
    return lambda frame: None


def farol_panel(beat,d,ctx):
    anchor=ctx.screen.anchor('farol-panel',.63,.17)
    card('farol-card',anchor,(0,0,-.01),(.66,.90,.002))
    ink=materials.fading('farol-ink',CREAM,1.8)
    blue=materials.fading('farol-blue',materials.YARN['blue'],1.8)
    coral=materials.fading('farol-coral',materials.YARN['coral'],1.8)
    def label(name,body,y,size=.031):
        return text(name,body,size,ink,anchor,location=(0,y,.007))
    minority=d.config['game']=='minority'
    label('farol-title','MINORITY GAME' if minority else 'EL FAROL',.405,.045)
    counts=label('actual-counts','',.345,.044)
    clock=label('actual-clock','',.292,.028)
    result=label('actual-outcome','',.245,.029)
    name=beat.name; meta=ctx.measured['selected'][beat.shot]
    case='m6' if beat.shot=='teaching' else beat.shot
    measured=ctx.measured['cases'][case]
    teaching=name in ('forecasts','decide','memory')
    lookup_rows=[]
    if teaching:
        if minority:
            label('lookup-caption','Flump 1 · actual held strategies',.172)
            bits=label('memory-bits','',.112,.039)
            label('bits-order','Oldest → newest · A=1, B=0',.063,.027)
            label('lookup-columns','Held       score before       advice',.008,.030)
            for i in range(2): lookup_rows.append(label(f'held-{i}','',-.048-i*.065,.038))
            decision=label('selected-advice','',-.181,.031)
            label('lookup-note','Past winners → this week’s advice',-.267,.031)
            label('variant-note','N=101 · S=2 · M=6\nOrdinary unbiased binary variant',-.369,.027)
        else:
            label('forecast-bank','Our reconstruction: 48 forecast types\n12 held per Flump · decay 0.9',.167,.028)
            history=label('forecast-history','',.098,.025)
            label('forecast-columns','Flump 1: forecast / error score before',.063,.026)
            for i in range(6): lookup_rows.append(label(f'held-pair-{i}','',.016-i*.046,.027))
            decision=label('selected-advice','',-.267,.030)
            strategy_name=label('actual-forecast-name','',-.334,.024)
            label('reconstruction-note','Lower error wins · random ties · at 60, stay',-.405,.024)
    else:
        compare=beat.compare
        keys=[case]+([compare] if compare else [])
        if name in ('short','middle','long'): keys=['m2','m6','m12']
        # Same vertical and horizontal scales in every comparison.
        max_probability=max(max(distribution(ctx.measured['histograms'][k])) for k in keys)
        if name in ('mean','coin','advice'): max_probability=.42
        if name in ('short','middle','long'): max_probability=.65
        traces=[]
        trace_keys=[case]+([compare] if compare else [])
        for key,mat in zip(trace_keys,(blue,coral)):
            selected=ctx.measured['selected'][key]
            trace=selected['retained_trace']
            # Retained corpus starts after burn-in; label its true clock.
            times=trace.get('rounds')
            if times is None:
                start=selected['first_decision_round']
                times=range(start,start+len(trace['attendance']))
            pts=[(t,n) for t,n in zip(times,trace['attendance']) if meta['start_round']<=t<=meta['end_round']]
            population=ctx.measured['cases'][key]['config']['agents']
            points=[(-.268+.536*(t-meta['start_round'])/(meta['end_round']-meta['start_round']),.037+.12*n/population) for t,n in pts]
            obj,spline=path('recorded-trace-'+key,anchor,points,mat)
            traces.append((spline,points,[t for t,n in pts]))
        label('trace-axis',f"One run / line · rounds {meta['start_round']}–{meta['end_round']}",.190,.026)
        label('trace-scale',f"Attendance: 0–{len(d.placed)} · same scale",.001,.026)
        for index,key in enumerate(keys):
            values=distribution(ctx.measured['histograms'][key])
            mat=blue if key==case else coral
            bottom=-.108-index*.09
            for i,value in enumerate(values):
                height=.061*value/max_probability
                box(f'ensemble-{key}-{i}',mat,anchor,
                    location=(-.25+.50*(i+.5)/len(values),bottom+height/2,.004),
                    scale=(.45/len(values),max(height,.00001),.002))
            label('hist-label-'+key,f"{key.upper()} · mean {ctx.measured['cases'][key]['mean']:.1f}",bottom-.020,.024)
        bottom=-.108-(len(keys)-1)*.09
        label('ensemble-scope',f"{measured['seeds']} runs · {measured['samples']:,} rounds\nAttendance 0–{len(d.placed)} · frequencies",bottom-.067,.025)
        detail={'bar':'Initial state · no decision yet',
                'react':'Recorded example · no fixed cycle claimed',
                'coin':'Known p=0.6 · supplied, not learned',
                'advice':'Blue: advice · coral: accuracy',
                'shared':f"All / none: {measured['all_or_none']:,}/{measured['samples']:,}\nMeasured rounds 401–2,000 · 48 held each",
                'minority':'N=101 · S=2 · M=6\nOrdinary unbiased binary variant',
                'long':f"N=101 · S=2 · M=12\nVariance/N {measured['variance_per_agent']:.3f} · chance .250",
                'short':'N=101 · S=2 · M=2',
                'middle':'N=101 · S=2 · M=6',
                'mean':f"Ensemble variance {measured['variance']:.1f}"}.get(name,'')
        # Three distributions need the lower line as a compact parameter label.
        label('farol-detail',detail,-.405 if len(keys)>2 else -.350,.025)
    def update(frame):
        f=filmed_frame(d,ctx.timing,frame)
        counts.data.body=f"{f.counts[0]} {'B' if minority else 'home'}  /  {f.counts[1]} {'A' if minority else 'bar'}"
        clock.data.body=f'Round {f.period:,} · one recorded run · seed {d.seed}'
        result.data.body='Before first decision' if f.period==0 else outcome(f.counts[1],len(d.placed),f.farol['capacity'],d.config['game'])
        if teaching:
            member=f.members[1]; chosen=selected_strategy(member)
            if minority:
                bits.data.body=memory_bits(f.farol['history_bits'],member['memory'])
                for i,row in enumerate(lookup_rows):
                    s=member['strategies'][i]
                    side='—' if s['attend'] is None else ('A' if s['attend'] else 'B')
                    row.data.body=f"{'▶' if member['selected']==i else ' '} {i+1}               {s['score']:g}                  {side}"
                decision.data.body=f"Selected {member['selected']+1} → {'A' if member['went'] else 'B'}" if chosen else 'No decision yet'
            else:
                history.data.body='Past six: '+', '.join(map(str,f.farol['attendance_history'][-6:]))
                strategy_name.data.body=chosen['label'] if chosen else 'No selected forecast yet'
                for i,row in enumerate(lookup_rows):
                    entries=[]
                    for j in (2*i,2*i+1):
                        s=member['strategies'][j]
                        entries.append(f"{'▶' if member['selected']==j else ''}{j+1}: {s['forecast']} / {s['score']:.1f}")
                    row.data.body='    |    '.join(entries)
                decision.data.body=(f"Selected #{member['selected']+1}: {chosen['forecast']} → {'GO' if member['went'] else 'STAY'}" if chosen else 'No selected forecast yet')
        else:
            for spline,points,times in traces:
                # Reveal only observations at or before the actual crowd clock.
                visible=max([i for i,t in enumerate(times) if t<=f.period],default=0)
                spline.points.foreach_set('co',[v for i in range(len(points)) for v in (*points[min(i,visible)],.006,1)])
    return update
