"""The Missing Rung: original A-minor chain, one connecting entrance removed."""
from music import Tune, Voice

F,V,P,B='flute','vibes','pizz','bassoon'
SUBJECT=['E2 A2 B2 c2','d2 c2 B2 A2','G2 A2 B2 c2','B4 E4',
         'E2 A2 c2 e2','d2 c2 B2 A2','G2 B2 A2 E2','A8']
BASS=['A,,2 E,2 A,2 E,2','A,,2 E,2 A,2 E,2','G,,2 D,2 G,2 D,2','E,,2 B,,2 E,2 B,,2',
      'A,,2 E,2 A,2 E,2','F,,2 C,2 F,2 C,2','E,,2 B,,2 E,2 B,,2','A,,4 E,4']
REST='z8'


def line(bars,mark='mp'): return '!'+mark+'! '+' | '.join(bars)+' |'


def section(flute,vibes,pizz,bass=BASS): return {F:line(flute),V:line(vibes),P:line(pizz),B:line(bass)}


def late(n): return [REST]*n+SUBJECT[:8-n]


def gaps(indices): return [bar if i in indices else REST for i,bar in enumerate(SUBJECT)]


TUNE=Tune(title='The Missing Rung',slug='the-missing-rung',key='Am',beats_per_bar=4,
          voices=(Voice(F,73,104,pan=30),Voice(V,11,92,pan=55),Voice(P,45,100,pan=80),Voice(B,70,96,pan=100)),
          sections={
              'A':section(SUBJECT,late(1),late(2)),
              'B':section([SUBJECT[0],REST]*4,[REST]*8,[REST]*8),
              'C':section(gaps({0,3,6}),gaps({1,4,7}),gaps({2,5})),
              'D':section(SUBJECT[:4]+[REST]*4,late(1)[:4]+[REST]*4,late(2)[:4]+[REST]*4),
              'E':section(gaps({0,4}),gaps({2,6}),[REST]*8),
              'F':section(SUBJECT,late(1),late(2)),
              'G':section([SUBJECT[0],REST]*2+SUBJECT[4:], [REST]*4+SUBJECT[4:], [REST]*4+SUBJECT[4:]),
          },forms=('ABCDEFG',),ending={F:'e8',V:'A8',P:'E8',B:'A,,8'},bpm=(100,140,160),
          stings={
              # Original rising cell in eighth notes: each voice's last note
              # leads into the next voice. Two bars keep the cues short.
              'chain': ({F:'E A B c z4 | z8 |', V:'z4 E A B c | z8 |',
                         P:'z8 | E A B c z4 |', B:line(BASS[:2])}, 100),
              'change': ({F:'E A B c z4 | E A B c z4 |', V:'z8 | z8 |',
                          P:'z8 | z8 |', B:line(BASS[:2])}, 160),
              'stalled': ({F:'E A B c z4 | E A B c z4 |', V:'z8 | z8 |',
                           P:'z8 | z8 |', B:line(BASS[:2])}, 120),
              'ceilings': ({F:'E A B c E A B c | E A B c z4 |',
                            V:'z4 E A B c | z8 |', P:'z4 E A B c | z8 |',
                            B:line(BASS[:2])}, 100),
              'sparse': ({F:'E A B c z4 | z8 |', V:'z8 | z4 E A B c |',
                         P:'z8 | z8 |', B:'A,,2 z6 | z8 |'}, 128),
              'middle': ({F:'E A B c E A B c | E A B c E A B c |',
                         V:'z4 E A B c | E A B c E A B c |',
                         P:'z8 | E A B c E A B c |', B:line(BASS[:2])}, 120),
              'dense': ({F:'E A B c z4 | E A B c z4 | z8 |',
                        V:'z8 | z4 E A B c | z8 |',
                        P:'z8 | z8 | E A B c z4 |', B:line(BASS[:3])}, 120),
          },
          cues=tuple((beat,beat,0.) for beat in
                     ('chain','change','stalled','ceilings','sparse','middle','dense')),
          handoffs=tuple(beat for beat in
                 ('chain','change','stalled','ceilings','sparse','middle','dense')))
