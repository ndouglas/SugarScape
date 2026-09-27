"""The Markets episode's tune: "The Spice Rag", an original slow rag in F
("not fast", after Joplin's markings), for honky-tonk piano, banjo, tuba
and clarinet. The tuba's oom-pah walks between two notes, as the Flumps
walk between two hills; the banjo answers on the off-beats.
A: the piano's syncopated strain in F. B: the clarinet takes over in D minor,
for trade making the Flumps less equal. C: the trio in B♭, the market
settling. Stings: a bell as the price settles near one; a falling clarinet
as the Gini rises."""

from music import Tune, Voice

PIANO, CLARINET, BANJO, TUBA, BELL = "piano", "clarinet", "banjo", "tuba", "bell"

# Oom (tuba, beats 1 and 3, root then fifth) and pah (banjo, beats 2 and 4).
OOM = {"F": "F,,2 z2 C,2 z2", "C7": "C,2 z2 G,,2 z2", "Bb": "_B,,2 z2 F,,2 z2", "Dm": "D,2 z2 A,,2 z2",
       "A7": "A,,2 z2 E,2 z2", "Gm": "G,,2 z2 D,2 z2", "F7": "F,,2 z2 C,2 z2", "Eb": "_E,2 z2 _B,,2 z2"}
PAH = {"F": "z2 [FAc]2 z2 [FAc]2", "C7": "z2 [EG_B]2 z2 [EG_B]2", "Bb": "z2 [DF_B]2 z2 [DF_B]2",
       "Dm": "z2 [DFA]2 z2 [DFA]2", "A7": "z2 [^CEG]2 z2 [^CEG]2", "Gm": "z2 [G_Bd]2 z2 [G_Bd]2",
       "F7": "z2 [A,C_E]2 z2 [A,C_E]2", "Eb": "z2 [_EG_B]2 z2 [_EG_B]2"}
REST = "z8"


def _bars(bars):
    return " | ".join(bars) + " |"


def _accompany(chords):
    return {TUBA: _bars([OOM[c] for c in chords]), BANJO: _bars([PAH[c] for c in chords])}


A_CHORDS = ("F", "F", "C7", "F", "F", "Bb", "C7", "F")
B_CHORDS = ("Dm", "Dm", "A7", "Dm", "Gm", "Dm", "A7", "Dm")
C_CHORDS = ("Bb", "Bb", "F7", "Bb", "Eb", "Bb", "F7", "Bb")

TUNE = Tune(
    title="The Spice Rag",
    slug="the-spice-rag",
    key="F",
    beats_per_bar=4,
    voices=(Voice(PIANO, 3, 105), Voice(CLARINET, 71, 95, pan=40), Voice(BANJO, 105, 80, pan=90),
            Voice(TUBA, 58, 100), Voice(BELL, 14, 90)),
    sections={
        "A": {  # the piano's strain: a short note, a long one across the beat, a short one
            PIANO: _bars(["z c A F c d2 c", "f e f g a2 f2", "g e c G _B2 A2", "A c f a2 f c2",
                          "z c A F c d2 c", "d _B d f d2 _B2", "c _B G E G2 _B2", "A2 c2 F4"]),
            CLARINET: _bars([REST] * 8),
            BELL: _bars([REST] * 8),
            **_accompany(A_CHORDS),
        },
        "B": {  # the clarinet in D minor; the piano steps back
            PIANO: _bars([REST] * 8),
            CLARINET: _bars(["A2 d2 f2 e d", "^c d e f d4", "e2 ^c2 A2 G E", "F A d f d4",
                             "g2 _b2 d'2 _b g", "f e d A d4", "^c e g e ^c2 A2", "d2 A2 D4"]),
            BELL: _bars([REST] * 8),
            **_accompany(B_CHORDS),
        },
        "C": {  # the trio in B♭: the piano leads, the clarinet holds beneath
            PIANO: _bars(["d2 f2 _b2 f d", "c d f d c4", "c2 A2 F2 _E2", "D F _B d f4",
                          "_e2 g2 _b2 g _e", "d c _B F _B4", "A c _e c A2 F2", "_B2 d2 _B4"]),
            CLARINET: _bars(["D8", "D8", "C8", "D8", "_E8", "D8", "C8", "D8"]),
            BELL: _bars([REST] * 8),
            **_accompany(C_CHORDS),
        },
    },
    forms=("ABC", "AABC", "ABAC", "ABCC", "AABBC", "ABBAC"),
    ending={PIANO: "[_B,DF_B]8", CLARINET: "D8", BANJO: "[DF_B]8", TUBA: "_B,,8", BELL: "_b8"},
    bpm=(84, 96, 112),
    stings={
        "settle": ({BELL: "c'4 f'4"}, 96),
        "unequal": ({CLARINET: "a2 f2 d2 A2 | D8"}, 96),
    },
    # The averaged price has settled on one early in "agree" (the ticks
    # before 200 still swing); the Gini bars are up a second into "unequal".
    cues=(("agree", "settle", 1.0), ("unequal", "unequal", 1.0)),
)
