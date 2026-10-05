"""The Certain Few: persistent outer figures and an absorbing inner voice."""
from music import Tune, Voice
B, C, M, V = 'bassoon', 'clarinet', 'marimba', 'cello'
BASS = ['D,2 A, z2 A,', 'D,3 z3', 'D,2 A, z2 A,', 'C,3 A,,3', 'D,2 A, D2 A,', 'F,3 E,3', 'D,2 A, z2 A,', 'D,3 A,,3']
INNER = ['A B A G F E', 'F2 E D2 E', 'G A B A G F', 'E3 C3', 'F E D E F G', 'A2 G F2 E', 'D E F E D C', 'D3 z3']
HIGH = ['z6', "a2 d' z2 d'", 'a3 z3', "g2 c' z2 c'", "a2 d' a2 d'", "c'3 a3", "a2 d' z2 d'", "a3 d'3"]
CELLAR = ['D,3 A,,3', 'D,3 A,,3', 'B,,3 F,3', 'C,3 G,,3', 'D,3 A,,3', 'B,,3 C,3', 'D,3 A,,3', 'D,6']

def section(inner, high=HIGH, bass=BASS, cellar=CELLAR):
    return {voice: ' | '.join(bars) + ' |' for voice, bars in zip((B, C, M, V), (bass, inner, high, cellar))}

TUNE = Tune(title='The Certain Few', slug='the-certain-few', key='Dm', beats_per_bar=3, meter='6/8',
    voices=(Voice(B, 70, 98, pan=25), Voice(C, 71, 106, pan=62), Voice(M, 12, 92, pan=100), Voice(V, 42, 95, pan=48)),
    sections={
        'A': section(INNER),
        'B': section(INNER),
        'C': section(['D2 A z2 A', "a2 d' z2 d'"] * 2 + ['A2 d z2 d', 'A3 z3', 'A2 d z2 d', 'A3 d3']),
        'D': section(['A3 G3', 'F3 E3'] * 4, high=['a3 z3'] * 8, cellar=['D,3 A,,3'] * 8),
        'E': section(['D2 z A2 z', 'F2 E D2 z', 'A z G F2 E', 'E3 z3'] * 2),
        'F': section(['A2 d z2 d', 'A3 d3', 'A2 d A2 d', 'A3 d3'] + INNER[:4],
                     high=["a2 d' z2 d'"] + HIGH[1:], bass=BASS),
        'G': section(INNER[:6] + ['D3 A3', 'D6'], high=HIGH[:6] + ['a3 d3', 'a6']),
    }, forms=('ABCDEFG',), ending={B:'D,6', C:'A6', M:'d6', V:'D,6'}, bpm=(100, 132, 170))
