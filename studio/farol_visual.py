"""One atomic clock for simultaneous recorded decisions and their readouts.

Positions are stable illustrative slots, not a model of walking or observation.
Every agent changes side on the same half-tick; no intermediate attendance is
invented. A small collective bounce marks that simultaneous transition.
"""
import math
import animate


def filmed_frame(d, timing, frame):
    return d.frames[min(int(math.floor(timing.tick_at(frame)+.5)),d.ticks)]


def pose(d, agent_id, timing, frame):
    tick=timing.tick_at(frame)
    f=filmed_frame(d,timing,frame)
    a=f.agents[agent_id]
    x,y=animate.cell_center(a.x,a.y,d.width,d.height)
    phase=tick-math.floor(tick)
    z=.18*math.sin(math.pi*phase)**2 if tick<d.ticks else 0
    return animate.Pose(x,y,z+.045,1,1,1,0,True,6,0)


def selected_strategy(member):
    index=member['selected']
    return None if index is None else member['strategies'][index]


def forecast_advice(forecast):
    return forecast<60


def memory_bits(bits, memory):
    return '—' if bits is None else format(bits, f'0{memory}b')[-memory:]


def lookup(member, bits):
    chosen=selected_strategy(member)
    return (memory_bits(bits,member['memory']),
            '—' if chosen is None or chosen['attend'] is None else ('A' if chosen['attend'] else 'B'),
            None if chosen is None else chosen['score'])


def outcome(attendance, population, capacity, game):
    if game=='el_farol':
        return 'CROWDED' if attendance>=capacity else 'QUIET'
    if attendance*2==population:
        return 'Equal sides'
    return 'A wins' if attendance*2<population else 'B wins'


def rebin(counts,bins=26):
    """Adjacent integer attendance values, including both endpoints."""
    out=[0]*bins
    for i,count in enumerate(counts):
        out[min(i*bins//len(counts),bins-1)]+=count
    return out


def distribution(counts,bins=26):
    total=sum(counts)
    return [c/total if total else 0 for c in rebin(counts,bins)]
