"""The Credit episode's tune: "Borrowed Time", an original blues in 7/4 in E
(the user's suggestion: the feel of Pink Floyd's "Money", not its riff). The
bar splits 4 + 3; the bassline walks each chord up to its flat fifth, the
blue note, and back down.

A: the bass alone over coin percussion — tambourine, cabasa and a tinkle
bell — the first loans; electric piano joins halfway. B: the band — a light
kit and a tenor sax on the melody — ending on the turnaround. C (for a
longer cut): B again with cowbell. Sting: a "ka-ching" (tinkle bell,
triangle, cowbell) as the first loan's sugar leaves the old Flump."""

from music import Tune, Voice

BASS, RHODES, SAX, BELL = "bass", "electric piano", "tenor sax", "tinkle bell"
TAMB, CABASA, KICK, SNARE, HAT, TRIANGLE, COWBELL = ("tambourine", "cabasa", "bass drum", "snare", "hi-hat",
                                                     "triangle", "cowbell")
PERCUSSION = (TAMB, CABASA, KICK, SNARE, HAT, TRIANGLE, COWBELL)

BAR = "z14"
# The walking figure on each chord: root, up through the fourth to the flat
# fifth (the blue note), and back down.
WALK = {
    "E7": "E,,2 E,, G,, A,,2 _B,, B,, D,2 B,,2 A,,2",
    "A7": "A,,2 A,, C, D,2 _E, E, G,2 E,2 D,2",
    "B7": "B,,2 B,, D, E,2 =F, F, A,2 F,2 E,2",
}
COMP = {"E7": "[E^GBd]", "A7": "[A^ceg]", "B7": "[B^dfa]"}
CHORDS = ("E7", "E7", "A7", "E7", "B7", "A7", "E7", "B7")

# General MIDI drums (channel 10): tambourine ^F, (54), cabasa A (69), bass
# drum C,, (36), snare D,, (38), closed hi-hat ^F,, (42), open triangle a
# (81), cowbell ^G, (56).
COINS = "z ^F, " * 7
SHAKE = "A " * 14
KICK_BAR = "C,,2 z6 C,,2 z4"  # 4 + 3: beats 1 and 5
SNARE_BAR = "z4 D,,2 z4 D,,2 z2"  # beats 3 and 6
HAT_BAR = "^F,, " * 14


def _bars(bars):
    return " | ".join(b.strip() for b in bars) + " |"


def _rest():
    return _bars([BAR] * 8)


def _comp(chord):
    c = COMP[chord]
    return f"z2 {c}2 z2 {c}2 z2 {c}2 z2"


SAX_LINE = ["B2 d2 e2 g e d2 B2 A2", "B2 G2 E4 z4 G A", "c2 e2 g2 a g e2 c2 A2", "B2 d2 e4 d B G2 E2",
            "^d2 f2 a2 f ^d B2 A2", "c2 e2 a4 g e c2 A2", "B2 G2 E2 G B e4 z2", "^d2 B2 ^F2 A2 B4 z2"]
BAND = {
    BASS: _bars([f"!mf! {WALK[CHORDS[0]]}"] + [WALK[c] for c in CHORDS[1:]]),
    RHODES: _bars([f"!mf! {_comp(CHORDS[0])}"] + [_comp(c) for c in CHORDS[1:]]),
    SAX: _bars([f"!f! {SAX_LINE[0]}"] + SAX_LINE[1:]),
    BELL: _rest(),
    TAMB: _bars([f"!mf! {COINS}"] + [COINS] * 7),
    CABASA: _bars([f"!mp! {SHAKE}"] + [SHAKE] * 7),
    KICK: _bars([f"!mf! {KICK_BAR}"] + [KICK_BAR] * 7),
    SNARE: _bars([f"!mf! {SNARE_BAR}"] + [SNARE_BAR] * 7),
    HAT: _bars([f"!mp! {HAT_BAR}"] + [HAT_BAR] * 7),
    TRIANGLE: _rest(),
    COWBELL: _rest(),
}

TUNE = Tune(
    title="Borrowed Time",
    slug="borrowed-time",
    key="Em",
    beats_per_bar=7,
    voices=(Voice(BASS, 33, 120), Voice(RHODES, 4, 90, pan=40), Voice(SAX, 66, 100, pan=85), Voice(BELL, 112, 80),
            *(Voice(p, 0, v, channel=10) for p, v in ((TAMB, 95), (CABASA, 80), (KICK, 110), (SNARE, 95),
                                                        (HAT, 70), (TRIANGLE, 100), (COWBELL, 100)))),
    sections={
        "A": {  # the first loans: the bass alone over coins; the piano joins halfway
            BASS: _bars([f"!mp! {WALK[CHORDS[0]]}"] + [WALK[c] for c in CHORDS[1:4]]
                        + [f"!mf! {WALK[CHORDS[4]]}"] + [WALK[c] for c in CHORDS[5:]]),
            RHODES: _bars([BAR] * 4 + [f"!mp! {_comp(CHORDS[4])}"] + [_comp(c) for c in CHORDS[5:]]),
            SAX: _rest(),
            BELL: _bars(["!mp! " + BAR, "z12 b2", BAR, "z12 e'2", BAR, "z12 b2", BAR, "z12 e'2"]),
            TAMB: _bars([f"!mp! {COINS}"] + [COINS] * 7),
            CABASA: _bars([f"!p! {SHAKE}"] + [SHAKE] * 7),
            KICK: _rest(), SNARE: _rest(), HAT: _rest(), TRIANGLE: _rest(), COWBELL: _rest(),
        },
        "B": BAND,
        "C": {**BAND, COWBELL: _bars(["!mf! ^G,2 z12", "z14", "^G,2 z12", "z14", "^G,2 z12", "z14", "^G,2 z12", "z14"])},
    },
    forms=("AB", "ABC"),
    ending={BASS: "E,,14", RHODES: "[E^GBd]14", SAX: "e14", BELL: "e'14", TAMB: BAR, CABASA: BAR, KICK: "C,,2 z12",
            SNARE: BAR, HAT: BAR, TRIANGLE: "a14", COWBELL: BAR},
    bpm=(96, 110, 126),
    stings={"kaching": ({BELL: "!ff! b' e''3", TRIANGLE: "!ff! a4", COWBELL: "!f! ^G, ^G,3"}, 110)},
    # The first loan's sugar leaves the old Flump as tick 1 is reached,
    # 0.6 + 1 / 0.35 ≈ 3.5 s into "lend".
    cues=(("lend", "kaching", 3.5),),
)
