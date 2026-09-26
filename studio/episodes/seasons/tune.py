"""The Seasons episode's tune: "The Thaw", an original waltz in A minor after
the Russian waltz (Tchaikovsky, Shostakovich). Summer (A) is a violin over
pizzicato strings, the raised seventh (G♯) giving the minor its Russian cast;
winter (B) turns to C major on celesta with sleigh bells — a nod to the Sugar
Plum Fairy, in a world made of sugar. Stings mark each season's entrance."""

from music import Tune, Voice

VIOLIN, CELESTA, PIZZ, BELLS = "violin", "celesta", "pizzicato", "sleigh bells"

_AM, _DM, _E7, _F = "A,,2 [EAc]2 [EAc]2", "D,2 [DFA]2 [DFA]2", "E,2 [^GBd]2 [^GBd]2", "F,,2 [FAc]2 [FAc]2"
_DM_E = "D,2 [DFA]2 [^GBd]2"
_C, _G, _EM, _DM_G = "C,2 [EGc]2 [EGc]2", "G,,2 [GBd]2 [GBd]2", "E,2 [EGB]2 [EGB]2", "D,2 [DFA]2 [GBd]2"
_REST = " | ".join(["z6"] * 8) + " |"


def _bars(*bars):
    return " | ".join(bars) + " |"


TUNE = Tune(
    title="The Thaw (waltz)",
    slug="the-thaw",
    key="Am",
    beats_per_bar=3,
    voices=(Voice(VIOLIN, 40, 105), Voice(CELESTA, 8, 110), Voice(PIZZ, 45, 90), Voice(BELLS, 0, 70, channel=10)),
    sections={
        "A": {  # summer
            VIOLIN: "E2 A2 c2 | e4 dc | d2 f2 a2 | ^g4 fe | e2 a2 c'2 | c'4 ba | f2 d2 ^G2 | A6 |",
            CELESTA: _REST,
            PIZZ: _bars(_AM, _AM, _DM, _E7, _AM, _F, _DM_E, _AM),
            BELLS: _REST,
        },
        "B": {  # winter
            VIOLIN: _REST,
            CELESTA: "e2 g2 c'2 | b4 d'2 | c'2 a2 e2 | g4 e2 | f2 a2 c'2 | e'4 d'c' | d'2 b2 g2 | c'6 |",
            PIZZ: _bars(_C, _G, _AM, _EM, _F, _C, _DM_G, _C),
            # GM percussion: note 83 (b) is the Jingle Bell; on beats two and three.
            BELLS: " | ".join(["z2 b2 b2"] * 8) + " |",
        },
    },
    forms=("ABA", "AABA", "AABABA", "AABAABA", "AABAAABA", "AABAAABAABA"),
    ending={VIOLIN: "A6", CELESTA: "[Ace]6", PIZZ: "[A,,E,A,]6", BELLS: "z6"},
    bpm=(150, 168, 184),
    stings={
        "summer": ({VIOLIN: "E/A/c/e/ a2 e'4", PIZZ: "[A,,EA]4 z4"}, 120),
        "winter": ({BELLS: "b/b/b/b/b/b/b/b/ b4", CELESTA: "e'2 g'2 c''4"}, 120),
    },
    cues=(("seasons", "summer"), ("slower", "winter")),
)
