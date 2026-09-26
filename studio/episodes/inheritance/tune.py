"""The Inheritance episode's tune: "The Ground Beneath", an original passacaglia
in D minor. The cello's ground — a descending lament, D C B♭ A … — repeats
unchanged under every variation, as each generation inherits what came
before and builds on it: A is plain harpsichord chords, B flows in eighths
while the strings enter, C gives the strings a counter-melody. Stings: a
harpsichord flourish at the first birth, a string chord as the first
inheritance passes."""

from music import Tune, Voice

HARPSICHORD, STRINGS, CELLO = "harpsichord", "strings", "cello"
GROUND = "D,4 E,2 | C,4 D,2 | B,,4 C,2 | A,,6 | G,,4 A,,2 | B,,4 ^C,2 | A,,6 | D,6 |"
_REST = " | ".join(["z6"] * 8) + " |"

TUNE = Tune(
    title="The Ground Beneath (passacaglia)",
    slug="the-ground-beneath",
    key="Dm",
    beats_per_bar=3,
    voices=(Voice(HARPSICHORD, 6, 100), Voice(STRINGS, 48, 85), Voice(CELLO, 42, 105)),
    sections={
        "A": {
            HARPSICHORD: "F2 A2 d2 | E2 G2 c2 | D2 F2 B2 | ^C2 E2 A2 | B,2 D2 G2 | D2 G2 E2 | ^C2 E2 A2 | D6 |",
            STRINGS: _REST,
            CELLO: GROUND,
        },
        "B": {
            HARPSICHORD: "F A d A F A | E G c G E G | D F B F D F | ^C E A E ^C E | B, D G D B, D | D G B G E G | ^C E A G F E | D6 |",
            STRINGS: "a4 f2 | g4 e2 | f4 d2 | e6 | d4 B2 | B4 ^c2 | e6 | d6 |",
            CELLO: GROUND,
        },
        "C": {
            HARPSICHORD: "F2 A2 d2 | E2 G2 c2 | D2 F2 B2 | ^C2 E2 A2 | B,2 D2 G2 | D2 G2 E2 | ^C2 E2 A2 | D6 |",
            STRINGS: "d'4 c'2 | c'4 B2 | B4 A2 | A6 | G4 F2 | F4 E2 | E6 | F6 |",
            CELLO: GROUND,
        },
    },
    forms=("ABC", "AABC", "ABCC", "ABBCC", "AABBCC"),
    ending={HARPSICHORD: "[DFA]6", STRINGS: "d6", CELLO: "D,6"},
    bpm=(72, 88, 104),
    stings={
        "birth": ({HARPSICHORD: "D/F/A/d/ f4"}, 90),
        "legacy": ({STRINGS: "[DFAd]6", HARPSICHORD: "d/f/a/d'/ z4"}, 90),
    },
    # The first child arrives 2.5 s into "pair" (tick 16); the parents poof
    # about 2.5 s into "inherit" (tick 32).
    cues=(("pair", "birth", 2.3), ("inherit", "legacy", 2.4)),
)
