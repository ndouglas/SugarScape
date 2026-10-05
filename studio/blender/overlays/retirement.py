"""Actual recorded Agents on a felt dance floor; brief evidence above the stage."""
import math
import textwrap
import bpy
import animate
from retirement_dance import invitation_gaze, join_amount, attention_angle
from retirement_visual import (filmed_frame, color, teaching_members, nearby_ids,
                               CastTimeline, _smooth, smooth_visibility, music_seconds, group_method, policy_legend, decision_summary, birth_visibility, contact_attainment)
from blender import materials, flump
from blender.retirement_cast import build_character, pose_character
from .parts import CREAM, box, text
from .panels import _polyline


def ink(name, color_name='cream'):
    return materials.fading('retirement-'+name+'-'+color_name,
                            CREAM if color_name == 'cream' else materials.YARN[color_name], 1.5)


def label(anchor, name, body, y, size=.038, x=0):
    return text('retirement-'+name, body, size, ink('ink'), anchor, location=(x,y,.013))


def stage():
    bpy.ops.mesh.primitive_plane_add(size=1)
    felt = bpy.context.active_object
    felt.name = 'retirement-felt-stage'
    felt.scale = (200, 200, 1)
    cloth = materials.felt().copy()
    cloth.name = 'retirement-blue-felt'
    cloth.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = (.035,.075,.12,1)
    felt.data.materials.append(cloth)


def actual_line(name, points):
    obj, spline = _polyline(name, None, [(0,0)]*len(points), ink('link','butter'))
    obj.data.bevel_depth = .009
    spline.points.foreach_set('co', [v for x,y in points for v in (x,y,.012,1)])
    return obj, spline


def _population(beat, d, ctx, key, offset=0, half=False):
    """One polygon for every slot, degenerate only when that slot has a rig."""
    name = beat.name
    ids = nearby_ids(name, d, ctx.measured, filmed_frame(d,ctx.timing,1))
    event = ctx.measured['selected']['teaching']['decision']
    teaching = name in ('ages', 'decision')
    first = teaching_members(event) if teaching else filmed_frame(d, ctx.timing, 1).members
    rigs = {}
    positions = {}
    if teaching:
        positions[event['id']] = (-1.65,0,0)
        counted = [m for m in event['neighbors'] if m['counted']]
        for j,m in enumerate(counted): positions[m['id']] = (.05+j*1.65,.25,0)
        young = [m for m in event['neighbors'] if not m['counted']]
        for j,m in enumerate(young): positions[m['id']] = (-2.7+j*.54,2.4,0)
    elif name == 'renewal': positions[ids[0]] = (0,0,0)
    elif name == 'habits':
        positions = {i:(-1.65+j*1.65,0,0) for j,i in enumerate(ids)}
    else:
        columns = 4 if half else 5
        spacing = .73 if half else .88
        positions = {i:((j%columns-(columns-1)/2)*spacing+offset,
                        (j//columns)*.86,0) for j,i in enumerate(ids)}
    for j,i in enumerate(ids):
        m = first[i]
        rig = build_character('retirement-'+key+'-agent-'+str(i), m, hero=i==event['id'])
        rig.origin = positions[i]
        scale = 1.5 if name == 'renewal' else 1.28 if name in ('ages','habits','decision') else .82
        if teaching and i not in [event['id'],7428,7457]: scale=.40
        if half: scale=.70
        rig.root.scale = (scale,)*3
        rig.root['run'] = key
        rigs[i] = rig
    n = len(d.frames[0].members)
    mesh = bpy.data.meshes.new('retirement-population-'+key)
    mesh.from_pydata([(0,0,0)]*(n*8), [], [tuple(range(i*8,(i+1)*8)) for i in range(n)])
    names = ('cream','teal','butter','coral','blue','slate')
    for c in names: mesh.materials.append(materials.matte('retirement-crowd-'+c,materials.YARN[c]))
    obj = bpy.data.objects.new('retirement-population-mesh'+('' if key==beat.shot else '-'+key), mesh)
    bpy.context.scene.collection.objects.link(obj)
    obj['agents'] = n; obj['run'] = key; obj['focus_slots'] = ids
    obj['source_slots'] = sorted(d.frames[0].members)
    template = [(-1,0),(-.8,.6),(-.55,1),(.55,1),(.8,.6),(1,0),(.45,-.12),(-.45,-.12)]
    links = []
    if teaching:
        for m in event['neighbors']:
            edge, spline = actual_line('retirement-actual-edge-'+str(m['id']),[positions[event['id']][:2],positions[m['id']][:2]])
            edge['from_slot']=event['id'];edge['to_slot']=m['id'];edge['run']=key
    elif name in ('groups','contact'):
        holder=ids[0]
        for i in ids[1:]:
            edge,spline=actual_line('retirement-group-edge-'+key+'-'+str(i),[positions[holder][:2],positions[i][:2]])
            edge['from_slot']=holder;edge['to_slot']=i;edge['run']=key
            links.append((edge,spline,i))
    pool=list(rigs.values())
    locations=list(positions.values())
    dynamic=name not in ('ages','decision','habits','renewal')
    timeline = None if teaching else CastTimeline(d,ctx.timing,beat.frames,ids,
        lambda f: nearby_ids(name,d,ctx.measured,f),
        eligible=name not in ('habits','renewal'), groups=name in ('groups','contact'),
        policy=name in ('policy','thresholds'))
    base_scales=[rig.root.scale.x for rig in pool]
    # Each recorded edge owns its fade material; opacity never leaks to another run.
    link_tracks=[[] for _ in links]
    if timeline is not None and links:
        for index,selected in timeline.rows.items():
            source=d.frames[index]
            boundary=ctx.timing.frame(index-.5)
            for seat,track in enumerate(link_tracks,1):
                pair=(selected[0],selected[seat])
                valid=pair[1] in source.members[pair[0]]['network']
                track.append((boundary,pair if valid else None))
        for edge,_,_ in links:
            edge.data.materials[0]=edge.data.materials[0].copy()
    def update(frame):
        f = filmed_frame(d,ctx.timing,frame)
        seconds = music_seconds(name,frame)
        if dynamic:
            selected=timeline.ids(frame)
            rigs.clear();rigs.update(zip(selected,pool))
            positions.clear();positions.update(zip(selected,locations))
            obj['focus_slots']=selected
        members = teaching_members(event, name=='decision' and frame>beat.frames*.55) if teaching else f.members
        obj['period'] = f.period
        obj['snapshot'] = 'period-end; focus uses activation-local event' if teaching else 'period-end'
        # Source slots remain stable; birth, age, retirement and kind always update.
        coords=[];indices=[]
        for j,(i,m) in enumerate(sorted(f.members.items())):
            if i in rigs:
                coords.extend([0.] * 24)
            else:
                x=(j%100-49.5)*(.026 if half else .055)+offset
                y=3.6+(j//100)*.031
                moving=.012*math.sin(seconds*math.tau*104/180+i) if m['retired'] else 0
                for dx,dz in template: coords.extend((x+dx*.019,y,.025+dz*.037+moving))
            indices.append(names.index(color(m, name in ('groups','contact'))))
        mesh.vertices.foreach_set('co',coords)
        mesh.polygons.foreach_set('material_index',indices);mesh.update()
        for seat,(i,rig) in enumerate(rigs.items()):
            m=members[i]
            rig.origin=positions[i]
            elapsed = (frame - 1) / 30
            onset = (math.floor(beat.frames * .55)) / 30
            gaze = response = 0.
            amount = None
            if teaching:
                # Only actual counted neighbors form the host's attention sequence.
                targets = [friend['id'] for friend in event['neighbors']
                           if friend['counted'] and friend['retired'] and friend['id'] != i]
                if name == 'decision' and i == event['id']:
                    targets += [friend['id'] for friend in event['neighbors']
                                if friend['counted'] and not friend['retired']]
                    amount = join_amount(elapsed, onset)
                if i == event['id'] or not m['retired']:
                    angles = [attention_angle(positions[i], positions[target])
                              for target in targets]
                    gaze = invitation_gaze(elapsed, angles,
                        onset if name == 'decision' and i == event['id'] else None)
                    response = .025 * math.sin(math.pi * min(1., elapsed / 2.4))
                    if name == 'decision' and i == event['id']:
                        response *= 1 - amount
            if not teaching:
                amount=timeline.amount(frame,seat)
                gaze=timeline.gaze(frame,seat,locations)
            pose_character(rig,m,seconds,dance_amount=amount,gaze=gaze,response=response,
                           age_amount=1. if teaching else timeline.age_amount(frame,seat))
            if not teaching:
                visibility=timeline.visibility(frame,seat)
                rig.root.scale=(base_scales[seat]*visibility,)*3
                rig.root['identity_visibility']=visibility
                rig.root['dance_amount']=amount
                rig.root['age_amount']=timeline.age_amount(frame,seat)
            if name in ('groups','contact'): flump.recolor(rig,color(m,True))
            rig.root['retired']=m['retired'];rig.root['period']=event['tick'] if teaching else f.period
            rig.root['snapshot']='activation-local' if teaching else 'period-end'
            rig.root['kind']=m.get('kind','unobserved')
            rig.root.rotation_euler.z=0
            rig.eyes.scale=(1,1,animate.blink(i,frame+int(music_seconds(name,1)*30)))
        for seat,((edge,spline,_),i) in enumerate(zip(links,list(rigs)[1:]),1):
            holder=next(iter(rigs))
            edge['from_slot']=holder;edge['to_slot']=i
            valid=i in f.members[holder]['network']
            edge.hide_render = not valid
            changes=[boundary for (boundary,pair),(old_boundary,old_pair) in
                     zip(link_tracks[seat-1][1:],link_tracks[seat-1]) if pair!=old_pair]
            opacity=min(timeline.visibility(frame,0),timeline.visibility(frame,seat),
                        smooth_visibility(frame,changes)) if valid else 0.
            edge.data.materials[0].node_tree.nodes['Mix'].inputs[0].default_value=opacity
            spline.points.foreach_set('co',[v for x,y in (positions[holder][:2],positions[i][:2]) for v in (x,y,.012,1)])
    return update


def comparison_data(ctx):
    """Optional second run, accessed only by the two comparison beats."""
    return ctx.compare


def retirement_population(beat,d,ctx):
    stage()
    compare = beat.name in ('denominator','contact')
    updaters=[_population(beat,d,ctx,beat.shot,-2 if compare else 0,compare)]
    if compare:
        updaters.append(_population(beat,comparison_data(ctx),ctx,beat.compare,2,True))
    return lambda frame:[update(frame) for update in updaters]


def retirement_panel(beat,d,ctx):
    """Short stage titles/readouts. Evidence charts appear only for age questions."""
    anchor=ctx.screen.anchor('retirement-readout',0,0)
    title=label(anchor,'method-title','',.465,.050)
    main=label(anchor,'main-measure','',.390,.060)
    detail=label(anchor,'method-detail','',.292,.038)
    foot=label(anchor,'method-footer','',-.267,.045)
    name=beat.name; key=beat.shot
    selected=ctx.measured['selected'][key]
    case=ctx.measured['cases'].get('policy_revised' if key=='policy_censored' else key)
    event=ctx.measured['selected']['teaching']['decision']
    bars=[];ratebars=[];plot_objects=[]
    if name in ('observable','meaning'):
        # A brief native rolling-window plot, with separate exposure denominator.
        for title_name,body,x in [('age-events-title','Retirement events · rolling 10 periods',-.46),
                                  ('age-rates-title','Events / actual working opportunities',.46)]:
            plot_objects.append(label(anchor,title_name,body,.390,.038,x))
        for i in range(81):
            bars.append(box('retirement-age-events-'+str(i),ink('events','butter'),anchor,scale=(.006,.001,.002)))
            ratebars.append(box('retirement-age-rates-'+str(i),ink('rates','teal'),anchor,scale=(.006,.001,.002)))
        for x in (-.46,.46):
            plot_objects.append(label(anchor,'plot-age-axis-'+str(x),'Age 20                     65                100',.174,.038,x))
        plot_objects.append(label(anchor,'rate-scale','Rate scale 0–1 · blank means no exposure',.347,.038,.46))
    for graphic in plot_objects+bars+ratebars+[title,detail]:
        graphic.data.materials[0]=graphic.data.materials[0].copy()
    def update(frame):
        f=filmed_frame(d,ctx.timing,frame);p=f.retirement
        title.data.body='';main.data.body='';detail.data.body='';foot.data.body=''
        if name=='ages':
            title.data.body='A friend is already dancing'
            foot.data.body='Close-up: actual decision · distant crowd: period end'
        elif name=='renewal':
            m=f.members[ctx.measured['selected']['teaching']['renewal']['id']]
            title.data.body='Back to the beginning · period 0' if f.period==0 else f'Native period {f.period} · the same slot, a new life'
            main.data.body=f"Age {m['age']} · born {m['born']}"
            detail.data.body=f"{len(f.members):,} Flumps · initial ages {min(a['age'] for a in d.frames[0].members.values())}–{max(a['age'] for a in d.frames[0].members.values())}"
            foot.data.body=f"Actual slot {m['id']} · population remains 8,100"
        elif name=='habits':
            title.data.body='Three habits · actual eligible Flumps'
            main.data.body='Rational              Random              Imitator'
            detail.data.body=f"{100*d.config['rational']:g}%                        {100*d.config['random']:g}%                         {100*(1-d.config['rational']-d.config['random']):g}%"
            foot.data.body=f"Eligibility {p['eligibility']} · random chance {d.config['p']:g} per eligible period · imitation threshold {d.config['threshold']:g}"
        elif name=='decision':
            title.data.body=f"One actual decision · age {event['age']}"
            main.data.body=decision_summary(event).split(' → ')[0]
            detail.data.body=f"{event['retired_counted']} retired friend · {event['counted']} eligible friends · threshold {event['threshold_units']/event['threshold_scale']:g}"
            foot.data.body=('He joins the dance' if frame>beat.frames*.55 else 'He watches his friends')+f" · {len(event['neighbors'])-event['counted']} younger friends excluded\nClose-up: actual decision · distant crowd: period end"
        elif name in ('quick','slow'):
            title.data.body=f"Rational {100*d.config['rational']:g}% · random {100*d.config['random']:g}% · threshold {d.config['threshold']:g}"
            main.data.body=f"{100*p['retired']:.1f}% of eligible Flumps retired"
            detail.data.body=f'Actual eligible representatives · native period {f.period}'
            s=case['first95']; mean=s['conditional_mean']
            foot.data.body=f"First 95% eligible-retired proxy: {s['attained']}/{s['total']} attained · {s['censored']} censored at {s['horizon']} · conditional mean {mean:.1f}"
        elif name=='denominator':
            title.data.body='Who counts as a friend?'
            main.data.body='Eligible friends                         All friends'
            q=filmed_frame(comparison_data(ctx),ctx.timing,frame).retirement
            detail.data.body=f"{100*p['retired']:.1f}% eligible retired                         {100*q['retired']:.1f}% eligible retired\nActual eligible representatives · native period {f.period}"
            a=ctx.measured['cases']['all']['first95'];e=case['first95']
            foot.data.body=f"First 95%: eligible {e['attained']}/{e['total']} · all {a['attained']}/{a['total']} · all censored {a['censored']} at {a['horizon']}"
        elif name in ('observable','meaning'):
            row=selected['retained_trace'][f.period]
            title.data.body='At what age?                         How many?'
            main.data.body=f"Mode {row['mode']}                         {100*row['share']:.1f}%"
            detail.data.body=f"Retirement-event age                  Eligible retired / eligible · native {f.period}"
            foot.data.body='A common event age can coexist with a small retired minority'
            entry,exit=.48*beat.frames,.94*beat.frames
            # Clear the original text before chart ink appears; restore it only
            # after chart ink leaves. Main readout moves into its safe position first.
            header_clear=_smooth((frame-(entry-12))/6)*(1-_smooth((frame-(exit+6))/6))
            shift=_smooth((frame-(entry-6))/6)*(1-_smooth((frame-exit)/6))
            opacity=_smooth((frame-entry)/12)*(1-_smooth((frame-(exit-12))/12))
            visible=opacity>0
            for obj in (title,detail):
                obj.data.materials[0].node_tree.nodes['Mix'].inputs[0].default_value=1-header_clear
            main.location.y=.390+.075*shift
            for obj in plot_objects+bars+ratebars:
                obj.hide_render=not visible
                obj.data.materials[0].node_tree.nodes['Mix'].inputs[0].default_value=opacity
            events=row['events_by_age'];exposure=row['exposure_by_age'];high=max(max(events),1)
            for i,(bar,ratebar) in enumerate(zip(bars,ratebars)):
                h=.12*events[i]/high
                bar.location=(-.80+.68*i/80,.205+h/2,.011);bar.scale.y=max(h,.00001)
                h=.12*events[i]/exposure[i] if exposure[i] else 0
                ratebar.location=(.12+.68*i/80,.205+h/2,.011);ratebar.scale.y=max(h,.00001)
            if visible: foot.data.body=f"{row['rolling_events']} native rolling events · mode counts events; share counts living eligible Flumps"
        elif name in ('groups','contact'):
            title.data.body='Two communities · actual directed contacts'
            main.data.body='Blue A · coral B' if name=='groups' else 'Contact .20                         Contact .05'
            detail.data.body=f"Blue A: rational 0% · coral B: rational {100*d.config['rational']:g}%\nBoth random {100*d.config['random']:g}% · actual eligible holder + contacts · period {f.period}"
            a=case['group_a95'];b=case['group_b95']
            if name=='contact':
                ref=ctx.measured['cases']['groups05']
                foot.data.body=f"First 95% conditional medians .05→.20: A {ref['group_a95']['conditional_median']:g}→{a['conditional_median']:g} · B {ref['group_b95']['conditional_median']:g}→{b['conditional_median']:g}\n"+contact_attainment(ctx.measured['cases'])
            else: foot.data.body=f"First 95%: A {a['attained']}/{a['total']}, B {b['attained']}/{b['total']} attained · censored {a['censored']}/{b['censored']} · horizon {a['horizon']}"
        elif name=='policy':
            title.data.body='Eligibility changes; each Flump still decides'
            old=ctx.measured['cases']['policy_original']['config']['eligibility']
            new=d.config['policy']['to']
            main.data.body=f'{old} → {new}'
            detail.data.body=f"Actual eligible representatives · native period {f.period} · eligibility {p['eligibility']}"
            at=d.retirement['retirement_policy_at'];switched=d.retirement['policy_switched_at']
            foot.data.body=f"Fixed {at}-year warm-up · switch {switched} · mandatory {d.config['mandatory']}\nNew-age decisions {switched+1}–{d.periods[-1]['tick']}"
        elif name=='thresholds':
            title.data.body='Revised thresholds · this recorded example is censored'
            original=ctx.measured['cases']['policy_original']['policy_new95']
            revised=ctx.measured['cases']['policy_revised']['policy_new95']
            main.data.body=f"{original['attained']} / {original['total']}                  {revised['attained']} / {revised['total']}"
            detail.data.body=policy_legend(ctx.measured['cases']['policy_original']['config'],ctx.measured['cases']['policy_revised']['config']).replace('Blue: ', '').replace('Coral: ', '').replace('threshold','threshold ').replace('\n','                   ')
            foot.data.body=f"New-age first 95% · {revised['censored']} revised runs censored at {revised['horizon']}\nMeans conditional on attainment · native period {f.period}"
        foot.data.body='\n'.join(textwrap.fill(part,76) for part in foot.data.body.splitlines())
        detail.data.body='\n'.join(textwrap.fill(part,85) for part in detail.data.body.splitlines())
    return update


def retirement_closing(beat,d,ctx):
    stage()
    selected=ctx.measured['selected']['teaching']['decision']['id']
    member=d.frames[beat.start_tick].members[selected]
    rig=build_character('host',member,hero=True)
    rig.root['run']='teaching';rig.root['retired']=member['retired'];rig.root['period']=d.frames[beat.start_tick].period
    # Quiet title-credit pose: the recorded retiree deliberately rests for this farewell.
    pose_character(rig,member,music_seconds('end',1),dancing=False)
    return lambda frame:setattr(rig.eyes,'scale',(1,1,animate.closing_blink(frame,beat.frames)))
