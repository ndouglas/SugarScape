"""The Tribes episode's tune: "Two Choirs", an original canzona after the
Venetian polychoral style (Gabrieli's cori spezzati at St Mark's). Two
ensembles face each other across the stereo field — Blue, recorders on the
left; Red, oboe and bassoon on the right. A: they answer each other phrase
by phrase; B: the answers shorten to a bar and overlap; C: they play as one.
Sting: both choirs strike a chord as the first Flump changes tribe."""

from music import Tune, Voice

REC1, REC2, OBOE, BASSOON = "recorder", "alto recorder", "oboe", "bassoon"
LEFT, RIGHT = 6, 121

# Two-bar phrases in G: the Blue call (G | C G) and the Red answer (D | … D).
CALL1, CALL1_LO = "G2 B2 d2 B2 | c2 A2 B4", "B,2 D2 G2 D2 | E2 C2 D4"
CALL2, CALL2_LO = "B2 d2 g2 d2 | e2 c2 d4", "G2 B2 d2 B2 | c2 A2 B4"
ANS1, ANS1_LO = "d2 f2 a2 f2 | g2 e2 f4", "D,2 A,,2 D,2 A,,2 | E,2 A,,2 D,4"
ANS2, ANS2_LO = "d2 B2 G2 B2 | A2 F2 G4", "G,,2 D,2 G,2 D,2 | D,2 D,2 G,,4"
REST2 = "z8 | z8"


def _bars(*phrases):
    return " | ".join(phrases) + " |"


TUNE = Tune(
    title="Two Choirs (canzona)",
    slug="two-choirs",
    key="G",
    beats_per_bar=4,
    voices=(Voice(REC1, 74, 105, pan=LEFT), Voice(REC2, 74, 85, pan=LEFT),
            Voice(OBOE, 68, 100, pan=RIGHT), Voice(BASSOON, 70, 95, pan=RIGHT)),
    sections={
        "A": {  # antiphony: call, answer, call, answer
            REC1: _bars(CALL1, REST2, CALL2, REST2),
            REC2: _bars(CALL1_LO, REST2, CALL2_LO, REST2),
            OBOE: _bars(REST2, ANS1, REST2, ANS2),
            BASSOON: _bars(REST2, ANS1_LO, REST2, ANS2_LO),
        },
        "B": {  # the answers come sooner: a bar each, then together
            REC1: "G2 B2 d2 B2 | z8 | B2 d2 g2 d2 | z8 | c2 e2 g2 e2 | z8 | d4 B4 | G8 |",
            REC2: "B,2 D2 G2 D2 | z8 | G2 B2 d2 B2 | z8 | E2 G2 c2 G2 | z8 | B4 G4 | D8 |",
            OBOE: "z8 | d2 f2 a2 f2 | z8 | d2 f2 a2 f2 | z8 | d2 f2 a2 f2 | g4 d4 | B8 |",
            BASSOON: "z8 | D,2 A,,2 D,2 A,,2 | z8 | D,2 A,,2 D,2 A,,2 | z8 | D,2 A,,2 D,2 A,,2 | G,,4 D,4 | G,,8 |",
        },
        "C": {  # tutti: one choir
            REC1: _bars(CALL1, CALL2, CALL1, CALL2),
            REC2: _bars(CALL1_LO, CALL2_LO, CALL1_LO, CALL2_LO),
            OBOE: _bars("G,2 B,2 D2 B,2 | C2 A,2 B,4", "B,2 D2 G2 D2 | E2 C2 D4",
                        "G,2 B,2 D2 B,2 | C2 A,2 B,4", "B,2 D2 G2 D2 | E2 C2 D4"),
            BASSOON: _bars("G,,4 G,,4 | C,4 G,,4", "G,,4 G,,4 | C,4 D,4", "G,,4 G,,4 | C,4 G,,4", "G,,4 G,,4 | C,4 D,4"),
        },
    },
    forms=("ABC", "AABC", "ABBC", "ABCC", "AABBC", "AABCC"),
    ending={REC1: "g8", REC2: "[Bd]8", OBOE: "G8", BASSOON: "G,,8"},
    bpm=(84, 100, 116),
    stings={"convert": ({REC1: "B/c/d4", OBOE: "[DG]6"}, 100)},
    # The Blue Flump turns Red at tick 10, about 4.5 s into "turn".
    cues=(("turn", "convert", 4.4),),
)
