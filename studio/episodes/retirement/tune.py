"""The Later Step: an original D-major, 3/4 polska-inspired dance.

The approved subject returns in AABACBA. B varies its middle turn; C
uses longer notes. Accompaniment carries the subject's closing rest so
habits pass between strains without a collective silence.
"""

from music import Tune, Voice

SUBJECT = ['D2 FA dA', 'B2 AF ED', 'F2 Ad fd', 'e2 cA G2',
           'F A d A F D', 'G B e B G E', 'F2 E2 C2', 'D4 z2']
VARIATION = SUBJECT[:2] + ['A2 df af', 'g2 ec A2',
                         'd f a f d A', 'e g b g e B'] + SUBJECT[6:]
LONG_NOTES = ['D4 A2', 'B4 A2', 'F2 A2 d2', 'e4 G2',
              'F2 A2 d2', 'G2 B2 e2', 'F2 E2 C2', 'D4 z2']
CHORDS = ['[DFA]2 z4', '[GBd]2 z4', '[DFA]2 z4', '[Ace]2 z4',
          '[DFA]2 z4', '[GBd]2 z4', '[Ace]2 z4', '[DFA]6']
BASS = ['D,2 A,2 D2', 'G,2 D2 G,2', 'D,2 A,2 D2', 'A,,2 E,2 A,2',
        'D,2 A,2 D2', 'G,2 D2 G,2', 'A,,2 E,2 A,2', 'D,6']


def section(lead):
    return {voice: ' | '.join(bars) + ' |'
            for voice, bars in (('fiddle', lead), ('accordion', CHORDS), ('guitar', BASS))}


TUNE = Tune(
    title='The Later Step', slug='the-later-step', key='D', beats_per_bar=3,
    voices=(Voice('fiddle', 40, 88), Voice('accordion', 21, 45), Voice('guitar', 24, 58)),
    sections={'A': section(SUBJECT), 'B': section(VARIATION), 'C': section(LONG_NOTES)},
    forms=('AABACBA',),
    ending={'fiddle': 'D6', 'accordion': '[DFA]6', 'guitar': 'D,6'},
    bpm=(100, 108, 110),
)
