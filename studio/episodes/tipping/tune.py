"""The "Tipping point" episode's tune, *A Waltz That Tips*: an original waltz in
G, 3/4, for two partners, a muted trumpet (Red) and a flute (Blue), over
strings and a pizzicato bass.

One form, section by section with the story: A, B (the area, the limit, the
rule: both partners); C (Fig. 18: the flute drops out bar by bar); D (the
plane: the trumpet alone, quietly); E (Fig. 19: both again, together); F
(the entry: the flute's fragments, then its full entry with a chord); G (Fig.
20: the flute crowded out again); H (the cap and the paradox: both return);
I (the title: the partners in canon, ending together).
"""

from music import Tune, Voice

TRUMPET, FLUTE, STRINGS, PIZZ = "trumpet", "flute", "strings", "pizzicato"
REST = "z6"

# K:G: F sharp. Six eighths a bar.
MELODY = ["D2 G2 B2", "d4 B2", "c2 A2 F2", "G4 D2", "E2 G2 c2", "B4 G2", "A2 F2 D2", "G6"]
PARTNER = ["B2 d2 g2", "g4 d2", "e2 c2 A2", "B4 G2", "c2 e2 g2", "d4 B2", "c2 A2 F2", "B6"]
CHORDS = ["G", "G", "D", "G", "C", "G", "D", "G"]
OOM = {"G": "G,2 [GBd]2 [GBd]2", "D": "D,2 [FAd]2 [FAd]2", "C": "C,2 [EGc]2 [EGc]2"}
ROOT = {"G": "G,,2 z4", "D": "D,,2 z4", "C": "C,,2 z4"}
STRINGS_BARS = [OOM[c] for c in CHORDS]
BASS = [ROOT[c] for c in CHORDS]


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


def section(mark, trumpet, flute, strings=STRINGS_BARS, pizz=BASS):
    return {TRUMPET: _line(trumpet, mark), FLUTE: _line(flute, mark), STRINGS: _line(strings, mark),
            PIZZ: _line(pizz, mark)}


def thinning(bars):
    """The partner's bars dropping out: whole, then half a bar, then a note, then gone."""
    out = []
    for k, b in enumerate(bars):
        if k < 2:
            out.append(b)
        elif k < 4:
            out.append(b.split()[0] + " z4" if b.split()[0].endswith("2") else "z6")
        else:
            out.append(REST)
    return out


FRAGMENTS = ["B2 z4", REST, "e2 z4", REST, "c2 z4", "[Bdg]6", "c2 A2 F2", "B6"]
CANON = [REST] + MELODY[:7]  # the partner a bar behind

TUNE = Tune(
    title="A Waltz That Tips",
    slug="a-waltz-that-tips",
    key="G",
    beats_per_bar=3,
    voices=(Voice(TRUMPET, 59, 115, pan=44), Voice(FLUTE, 73, 115, pan=84), Voice(STRINGS, 48, 80, pan=64),
            Voice(PIZZ, 45, 100, pan=64)),
    sections={
        "A": section("mp", MELODY, PARTNER),
        "B": section("mp", MELODY, PARTNER),
        "C": section("mp", MELODY, thinning(PARTNER)),
        "D": section("p", MELODY, [REST] * 8, [REST] * 8),
        "E": section("mf", MELODY, PARTNER),
        "F": section("mf", MELODY, FRAGMENTS),
        "G": section("mf", MELODY, thinning(PARTNER)),
        "H": section("mf", MELODY, PARTNER),
        "I": section("mf", MELODY, CANON),
    },
    forms=("ABCDEFGHI",),
    ending={TRUMPET: "G6", FLUTE: "B6", STRINGS: "[GBd]6", PIZZ: "G,,6"},
    bpm=(130, 150, 175),
)
