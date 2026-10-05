"""Pure adapters for actual ants events and measured diagrams."""

def color(member):
    return 'butter' if member['independent'] else ('blue' if member['source']==1 else 'coral')


def recorded_event(frames, expected):
    for frame in frames:
        for event in frame.ants_events:
            if event==expected: return event
    raise ValueError('Selected teaching event is absent from the actual shot')


def neighborhood(actor, links, members):
    neighbors=sorted({j if i==actor else i for i,j in links if actor in (i,j)})
    return neighbors,sum(members[i]['source']!=members[actor]['source'] for i in neighbors)


def bins(values, count=20):
    result=[0]*count
    for i,value in enumerate(values): result[min(i*count//(len(values)-1),count-1)]+=value
    return result


def filmed_frame(dump,timing,frame):
    """Hold the recorded state until its actual saved frame is reached."""
    return dump.frames[min(int(timing.tick_at(frame)),dump.ticks)]
