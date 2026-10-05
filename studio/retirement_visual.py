"""Actual members and native clocks for the retirement film.

Positions are compact illustrations of current age cohorts, never social distance.
Each slot appears once, including the thousands of renewed youngest members.
"""
import math
from collections import Counter


def filmed_frame(d,timing,frame):
    index=min(max(int(math.floor(timing.tick_at(frame)+.5)),0),d.ticks)
    return d.frames[index]


def cohort_layout(members):
    counts=Counter(a['age']for a in members.values())
    columns=100
    bands={age:math.ceil(n/columns)for age,n in counts.items()}
    rows=sum(bands.values())+max(0,len(bands)-1)
    positions={}; row=0
    for age in sorted(counts):
        ids=sorted(i for i,a in members.items()if a['age']==age)
        for k,i in enumerate(ids):
            positions[i]=((k%columns+.5)/columns,(row+k//columns+.5)/max(rows,1))
        row+=bands[age]+1
    return positions,dict(sorted(counts.items()))


def color(member,groups=False):
    if groups: return 'blue'if member['group']==0 else'coral'
    if member['retired']:return 'cream'
    return {'rational':'butter','random':'coral','imitator':'teal'}.get(member.get('kind'), 'slate')


def identity(member):
    return f"slot {member['id']} · born {member['born']}"


def decision_summary(d):
    n=d['counted']; r=d['retired_counted']; tau=d['threshold_units']/d['threshold_scale']
    if not n:return f'0 counted → keep working (threshold {tau:.2f})'
    return f"{r} / {n} = {r/n:.2f}  {'≥'if r*d['threshold_scale']>=d['threshold_units']*n else'<'}  {tau:.2f} → {'retire'if d['retired_after']else'keep working'}"


def minority_period(selection):
    trace={p['tick']:p for p in selection['retained_trace']}
    for tick in selection['frame_periods']:
        p=trace[tick]
        if p['mode']==65 and p['rolling_events']>0 and p['share']<=.25:return tick
    raise ValueError('no recorded population frame supports the minority example')


def rebin(values,bins=16):
    out=[0]*bins
    for i,v in enumerate(values):out[min(i*bins//len(values),bins-1)]+=v
    return out


def timing_histogram(rows,field,horizon,bins=16):
    counts=[0]*bins;censored=0
    for row in rows:
        value=row[field]
        if value is None:censored+=1
        else:
            if not 0<=value<=horizon:raise ValueError('timing exceeds declared horizon')
            counts[min(int(value*bins/(horizon+1)),bins-1)]+=1
    return counts,censored


def policy_color(case):
    return 'blue'if case=='policy_original'else'coral'


def _number(value):
    return f'{value:g}'.replace('0.','.',1)


def policy_legend(original,revised):
    half=math.sqrt(3)*revised['spread']
    low=max(0,revised['threshold']-half);high=min(1,revised['threshold']+half)
    return f"Blue: original threshold{_number(original['threshold'])}\nCoral: revised U[{_number(low)},{_number(high)}]"


def group_method(config):
    return f"A rational0% · B rational{100*config['rational']:g}%\nBoth random{100*config['random']:g}% · coupling{_number(config['groups']['coupling'])}"


def trace_legend(beat,coupling=None,reference=None):
    if beat=='denominator':return 'Blue: eligible · coral: all'
    return f"Blue:A{coupling:.2f} · coral:B{coupling:.2f}\nTeal:A{reference:.2f} · lilac:B{reference:.2f}".replace('0.','.')


MUSIC_STARTS = dict(zip(
    ('ages', 'renewal', 'habits', 'decision', 'quick', 'slow', 'denominator',
     'observable', 'groups', 'contact', 'policy', 'thresholds', 'meaning', 'end'),
    (0, 7, 13, 21, 29, 36, 43, 50, 57, 64, 71, 77, 85, 92)))


def music_seconds(name, frame):
    return MUSIC_STARTS[name] + (frame - 1) / 30


def teaching_members(event, after=False):
    """Only the host outcome changes; neighbor kind is genuinely unobserved."""
    members = {m['id']: dict(m) for m in event['neighbors']}
    members[event['id']] = {k: event[k] for k in ('id', 'born', 'age')}
    members[event['id']].update(kind='imitator', retired=event['retired_after' if after else 'retired_before'])
    return members


def nearby_ids(name, d, measured, current=None):
    """Actual age-selected representatives; teaching/habit identities stay fixed."""
    from retirement_dance import eligible_member
    if name in ('ages', 'decision', 'end'):
        event = measured['selected']['teaching']['decision']
        return [event['id']] + [m['id'] for m in event['neighbors']]
    if name == 'renewal':
        return [measured['selected']['teaching']['renewal']['id']]
    if name == 'habits':
        f = d.frames[1]
        return [eligible_member(f.members, kind, f.retirement['eligibility'])['id']
                for kind in ('rational', 'random', 'imitator')]
    # Select across actual eligible ages, not the first (youngest) slots.
    index = 1 if name in ('observable', 'meaning') else 0
    if name == 'policy': index = 34  # native102, first retained new-age decisions
    if name == 'thresholds': index = 34
    f = current if current is not None else d.frames[index]
    eligible = sorted((i for i, m in f.members.items() if m['age'] >= f.retirement['eligibility']),
                      key=lambda i:(f.members[i]['age'],i))
    if name == 'policy':
        newly = [i for i in eligible if 62 <= f.members[i]['age'] < 65 and not f.members[i]['retired']]
        return (newly + [i for i in eligible if i not in newly])[:12]
    if name in ('groups', 'contact'):
        holder = next(f.members[i] for i in eligible
                      if any(f.members[j]['group'] != f.members[i]['group'] for j in f.members[i]['network']))
        ids = list(dict.fromkeys([holder['id']] + holder['network'][:14]))
        ids += [i for i in eligible if i not in ids][:15-len(ids)]
        return ids
    return [eligible[round(j*(len(eligible)-1)/14)] for j in range(15)]


def birth_visibility(d, timing, frame, slot, window=12):
    """Gentle visual disappearance/arrival at the unchanged sampled birth switch."""
    visibility=1.
    for index in range(1,len(d.frames)):
        if d.frames[index-1].members[slot]['born'] != d.frames[index].members[slot]['born']:
            distance=abs(frame-timing.frame(index-.5))/window
            t=min(distance,1.)
            visibility=min(visibility,t*t*(3-2*t))
    return visibility


def contact_attainment(cases):
    """Compress identical context only after checking all group/treatment rows."""
    rows=[cases[k][g] for k in ('groups05','groups20') for g in ('group_a95','group_b95')]
    values=[(r['attained'],r['total'],r['censored'],r['horizon']) for r in rows]
    if len(set(values))==1:
        a,n,c,h=values[0]
        return f'All groups / contacts: {a}/{n} attained · {c} censored · horizon {h}'
    return '\n'.join(f"{k} {g}: {r['attained']}/{r['total']} attained · {r['censored']} censored at {r['horizon']}"
                     for k in ('groups05','groups20') for g,r in ((g,cases[k][g]) for g in ('group_a95','group_b95')))


def _smooth(value):
    value = max(0., min(1., value))
    return value * value * (3 - 2 * value)


def smooth_visibility(frame, boundaries, window=12):
    """Leave as the old identity, arrive as the new one; zero at the switch."""
    visibility=1.
    for index,boundary in enumerate(boundaries):
        wing=window
        if index: wing=min(wing,.35*(boundary-boundaries[index-1]))
        if index+1<len(boundaries): wing=min(wing,.35*(boundaries[index+1]-boundary))
        visibility=min(visibility,_smooth(abs(frame-boundary)/max(wing,1e-9)))
    return visibility


def eased_target(events, frame, window=30):
    """Pure target easing, including changes before the previous turn finishes."""
    value = events[0][1] if events else 0.
    start = value
    previous_frame = events[0][0] if events else 1
    for boundary, target in events[1:]:
        if frame < boundary:
            break
        value = start + (value-start)*_smooth((boundary-previous_frame)/window)
        start, value, previous_frame = value, target, boundary
    return start + (value-start)*_smooth((frame-previous_frame)/window)


class CastTimeline:
    """Stable recorded seats and presentation clocks computed once per scene.

    All assignments and onset clocks are sampled-source decisions. Evaluation
    is independent of callback order. Only focused seats are scheduled, never
    all population slots for all film frames.
    """
    def __init__(self, d, timing, last_frame, initial, candidates, eligible=True, groups=False, policy=False):
        self.d, self.timing = d, timing
        self.start = min(int(math.floor(timing.tick_at(1)+.5)),d.ticks)
        self.end = min(int(math.floor(timing.tick_at(last_frame)+.5)),d.ticks)
        self.rows, self.onsets, self.age_onsets = {}, {}, {}
        self.boundaries = [[] for _ in initial]
        self.groups = groups
        previous = None
        onset = [None]*len(initial)
        age_onset = [None]*len(initial)
        ids = list(initial)
        for index in range(self.start,self.end+1):
            f = d.frames[index]
            boundary = timing.frame(index-.5)
            old_ids=list(ids)
            proposed = list(dict.fromkeys(candidates(f)))
            def suitable(slot, seat):
                m=f.members[slot]
                if groups and seat == 0:
                    return (m['age'] >= f.retirement['eligibility'] and
                            any(f.members[j]['group'] != m['group'] for j in m['network']))
                if groups:
                    contacts=list(dict.fromkeys(i for i in f.members[ids[0]]['network'] if i!=ids[0]))[:len(ids)-1]
                    if seat <= len(contacts): return slot in contacts
                    if slot in contacts or slot == ids[0]: return False
                return not eligible or m['age'] >= f.retirement['eligibility']
            # Reserve retained seats before replacements so no survivor is stolen.
            kept=[]
            for seat,slot in enumerate(ids):
                same = previous is None or previous.members[slot]['born'] == f.members[slot]['born']
                kept.append(slot if same and suitable(slot,seat) else None)
            if groups and kept[0] is None:
                ids[0]=proposed[0]
                kept[0]=ids[0]
                for seat in range(1,len(ids)):
                    if kept[seat] is not None and (kept[seat] == ids[0] or not suitable(kept[seat],seat)): kept[seat]=None
            if groups:
                holder=kept[0]
                contacts=list(dict.fromkeys(i for i in f.members[holder]['network'] if i!=holder))[:len(ids)-1]
                fillers=sorted(i for i,m in f.members.items() if m['age']>=f.retirement['eligibility'])
                proposed=list(dict.fromkeys([holder]+contacts+fillers))
            if policy and previous is not None and previous.retirement['eligibility'] != f.retirement['eligibility']:
                newly=[i for i in proposed if f.retirement['eligibility'] <= f.members[i]['age'] < previous.retirement['eligibility']]
                for seat in range(min(len(newly),len(kept))): kept[seat]=None
                proposed=newly+[i for i in proposed if i not in newly]
            reserved={slot for slot in kept if slot is not None}
            for seat,slot in enumerate(kept):
                if slot is None:
                    slot=next(i for i in proposed if i not in reserved and suitable(i,seat))
                    kept[seat]=slot;reserved.add(slot)
            for seat,slot in enumerate(kept):
                m=f.members[slot]
                old=previous.members[old_ids[seat]] if previous is not None else None
                changed=old is not None and (slot != old['id'] or m['born'] != old['born'])
                if changed:
                    self.boundaries[seat].append(boundary)
                    onset[seat]=None
                    age_onset[seat]=None
                elif old is not None and m['retired'] and not old['retired']:
                    onset[seat]=boundary
                if not changed and old is not None and old['age'] < 65 <= m['age']:
                    age_onset[seat]=boundary
                if m['age'] < 65: age_onset[seat]=None
                if not m['retired']: onset[seat]=None
            ids=kept
            self.rows[index]=tuple(ids)
            self.onsets[index]=tuple(onset)
            self.age_onsets[index]=tuple(age_onset)
            previous=f
        self._gaze_cache={}

    def index(self, frame):
        return min(max(int(math.floor(self.timing.tick_at(frame)+.5)),self.start),self.end)

    def ids(self, frame):
        return list(self.rows[self.index(frame)])

    def visibility(self, frame, seat):
        return smooth_visibility(frame,self.boundaries[seat])

    def amount(self, frame, seat):
        from retirement_dance import join_amount
        index=self.index(frame)
        if not self.d.frames[index].members[self.rows[index][seat]]['retired']: return 0.
        onset=self.onsets[index][seat]
        return 1. if onset is None else join_amount((frame-1)/30,(onset-1)/30)

    def age_amount(self, frame, seat):
        index=self.index(frame)
        if self.d.frames[index].members[self.rows[index][seat]]['age'] < 65: return 0.
        onset=self.age_onsets[index][seat]
        return 1. if onset is None else _smooth((frame-onset)/30)

    def links(self, frame):
        index=self.index(frame);ids=self.rows[index]
        return [(ids[0],i) for i in ids[1:] if i in self.d.frames[index].members[ids[0]]['network']]

    def gaze(self, frame, seat, positions):
        from retirement_dance import attention_angle
        key=tuple(positions)
        if key not in self._gaze_cache:
            tracks=[[(1,0.)] for _ in positions]
            for index,ids in self.rows.items():
                members=self.d.frames[index].members
                for s,i in enumerate(ids):
                    dancers=[j for j,k in enumerate(ids) if j!=s and members[k]['retired']]
                    target=0.
                    if not members[i]['retired'] and dancers:
                        other=min(dancers,key=lambda j:abs(positions[j][0]-positions[s][0]))
                        target=attention_angle(positions[s],positions[other])
                    boundary=1 if index==self.start else self.timing.frame(index-.5)
                    if target != tracks[s][-1][1]: tracks[s].append((boundary,target))
            self._gaze_cache[key]=tracks
        return eased_target(self._gaze_cache[key][seat],frame)
