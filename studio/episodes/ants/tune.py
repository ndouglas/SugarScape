"""The Turning Chain: original Breton-inspired instrumental dance replies.

The overlapping reeds borrow the handoff idea of kan ha diskan, not its vocal
practice. A repeated-note D-Dorian subject rides a plain duple drone/pulse.
"""
from music import Tune, Voice

SUBJECT = ['D2 D E', 'F E D2', 'A2 A G', 'F2 E D']
LIFT = ['G2 A B', 'c2 B A', 'G A F E', 'D3 E']
REST = ['z4'] * 4
PULSE = ['D,2 A,2'] * 8


def line(bars):
    return ' | '.join(bars) + ' |'


def parts(oboe, clarinet, drone=None):
    return dict(oboe=line(oboe), clarinet=line(clarinet),
                drone=line(PULSE[:len(oboe)] if drone is None else drone))


def call(subject):
    return subject + REST


def answer(subject):
    # The next dancer joins for the last beat, then repeats the full phrase.
    return ['z4'] * 3 + ['z2 D E'] + subject


TUNE = Tune(
    title='The Turning Chain', slug='the-turning-chain', key='Ddor',
    beats_per_bar=2, meter='2/4',
    voices=(Voice('oboe', 68, 88, pan=28),
            Voice('clarinet', 71, 88, pan=100),
            Voice('drone', 21, 48, pan=64)),
    sections={
        'A': parts(call(SUBJECT), answer(SUBJECT)),
        'B': parts(SUBJECT + LIFT, REST + REST),
        'C': parts(answer(SUBJECT), call(SUBJECT)),
        'D': parts(call(LIFT), answer(LIFT)),
        'E': parts(call(SUBJECT), answer(SUBJECT)),
        'F': parts(call(SUBJECT), answer(SUBJECT), ['D2 D2'] * 8),
        'G': parts(answer(LIFT), call(LIFT)),
    },
    forms=('AABBCDDEEFG',), bpm=(100, 110, 122),
    ending={'oboe': 'D4', 'clarinet': 'D4', 'drone': '[D,A,]4'},
)
