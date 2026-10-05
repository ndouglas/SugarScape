"""The Empty Chair: original continuous schottische, five eight-bar strains."""
from music import Tune, Voice

SUBJECT=['G2 B2 d2 B2','A2 c2 e2 c2','B2 d2 g2 d2','c2 A2 F2 D2',
         'G B d B G B d B','c e g e c e g e','A2 F2 D2 F2','G4 z4']
CHORDS=['[GBd]4 z4','[Ace]4 z4','[GBd]4 z4','[FAc]4 z4',
        '[GBd]4 z4','[ceg]4 z4','[FAc]4 z4','[GBd]4 z4']
BASS=['G,,2 D,2 G,2 D,2','A,,2 E,2 A,2 E,2','G,,2 D,2 G,2 D,2','D,,2 A,,2 D,2 A,,2',
      'G,,2 D,2 G,2 D,2','C,2 G,2 C2 G,2','D,,2 A,,2 D,2 A,,2','G,,4 D,4']


def phrase(bars):
    return ' | '.join(bars)+' |'


def strain(lead,chords=CHORDS,bass=BASS):
    return dict(clarinet=phrase(lead),accordion=phrase(chords),guitar=phrase(bass))


TUNE=Tune(title='The Empty Chair',slug='the-empty-chair',key='G',beats_per_bar=4,
          voices=(Voice('clarinet',71,88),Voice('accordion',21,48),Voice('guitar',24,65)),
          sections={
              'A':strain(SUBJECT),
              # Recognizable answer: neighboring tones, same harmonic rhythm.
              'B':strain(['B2 d2 B2 G2','c2 e2 c2 A2','d2 g2 d2 B2','A2 c2 A2 F2',
                          'G B d B d B G B','e g e c e c A c','F2 A2 D2 F2','G4 B2 d2']),
              # Sparse clarinet leaves space, with accompaniment uninterrupted.
              'C':strain(['G2 z2 d2 z2','A2 z2 e2 z2','B2 z2 d2 z2','A2 z2 D2 z2',
                          'G2 B2 z4','c2 e2 z4','A2 F2 D2 z2','G4 z4']),
              # Even step pattern over the same bass; only one melody at a time.
              'D':strain(['G2 B2 G2 B2','A2 c2 A2 c2','B2 d2 B2 d2','A2 F2 A2 F2',
                          'G2 B2 G2 B2','c2 e2 c2 e2','A2 F2 D2 F2','G2 B2 d2 B2']),
              'E':strain(SUBJECT[:4]+['G2 z2 B2 z2','c2 z2 e2 z2','A2 F2 D2 z2','G4 D4']),
          },forms=('ABCDE',),ending={'clarinet':'G4 D4','accordion':'[GBd]4 [DFA]4','guitar':'G,,4 D,,4'},
          bpm=(100,104,108))
