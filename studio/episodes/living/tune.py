"""The Living neighbors episode's tune, *A Round for Neighbors*: an original
round in G major, 3/4, for flute, clarinet, oboe and bassoon over pizzicato
strings. Each voice enters two bars behind the last with the same tune, as
each clone copies its parent; the tune's four two-bar phrases all sit over
G, so any of them can sound together.

One form, section by section with the story: A A (the close-ups: the flute
alone); B C D (the land fills: clarinet, oboe, then bassoon enter); F (the
soup: the voices crowd in a bar apart, in G minor, and collide); G (they
drop out one by one, until a muted trumpet plays the first phrase alone
and stops); H (fewer rewards: two voices try the round again and thin out);
E (the title: the full round, ending on a held G major chord).
"""

import re

from music import Tune, Voice

FLUTE, CLARINET, OBOE, BASSOON, TRUMPET, PIZZ = "flute", "clarinet", "oboe", "bassoon", "trumpet", "pizzicato"
ROUND = (FLUTE, CLARINET, OBOE, BASSOON)

BAR = "z6"
# The tune: four two-bar phrases, each over G. (K:G makes F sharp.)
TUNE_BARS = ["G2 B2 d2", "d4 B2", "c2 B2 A2", "B4 G2", "d d e d c B", "A2 G2 D2", "B2 A2 F2", "G6"]

NOTE = re.compile(r"([_^=]?)([A-Ga-g])([,']*)")


def minor(bar):
    """The bar in G minor: B and E flatted."""
    return NOTE.sub(lambda m: (m.group(1) or ("_" if m.group(2) in "BbEe" else "")) + m.group(2) + m.group(3), bar)


def octave_down(bar):
    """The bar an octave lower (the bassoon's)."""
    def down(m):
        acc, letter, marks = m.groups()
        if letter.islower():
            return acc + letter.upper() + marks
        return acc + letter + marks + ","
    return NOTE.sub(down, bar)


def line(voice, bars):
    return [octave_down(b) if voice == BASSOON else b for b in bars]


def entering(delay):
    """Eight bars of a voice entering `delay` bars late."""
    return [BAR] * delay + TUNE_BARS[: 8 - delay]


def turned(delay):
    """Eight bars of a voice that entered `delay` bars ago, mid-round."""
    return TUNE_BARS[8 - delay :] + TUNE_BARS[: 8 - delay] if delay else list(TUNE_BARS)


def _bars(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


PIZZ_G = ["G,,2 D,2 G,2"] * 8
SILENT = [BAR] * 8


def section(parts, mark, pizz=PIZZ_G, trumpet=SILENT):
    out = {v: _bars(line(v, parts.get(v, SILENT)), mark) for v in ROUND}
    out[TRUMPET] = _bars(trumpet, mark)
    out[PIZZ] = _bars(pizz, mark)
    return out


FULL = {v: turned(2 * i) for i, v in enumerate(ROUND)}
# The soup: every voice a bar behind the last, in G minor.
CROWD = {v: [minor(b) for b in turned(i)] for i, v in enumerate(ROUND)}
COLLAPSE = {
    FLUTE: [minor(b) for b in TUNE_BARS[:4]] + [BAR] * 4,
    CLARINET: [minor(b) for b in turned(1)[:3]] + [BAR] * 5,
    OBOE: [minor(b) for b in turned(2)[:2]] + [BAR] * 6,
    BASSOON: [minor(b) for b in turned(3)[:1]] + [BAR] * 7,
}

TUNE = Tune(
    title="A Round for Neighbors",
    slug="a-round-for-neighbors",
    key="G",
    beats_per_bar=3,
    voices=(Voice(FLUTE, 73, 100, pan=50), Voice(CLARINET, 71, 92, pan=78), Voice(OBOE, 68, 88, pan=36),
            Voice(BASSOON, 70, 95, pan=64), Voice(TRUMPET, 59, 90), Voice(PIZZ, 45, 80)),
    sections={
        "A": section({FLUTE: TUNE_BARS}, "mp", pizz=["G,,2 z4"] * 8),
        "B": section({FLUTE: TUNE_BARS, CLARINET: entering(2)}, "mp"),
        "C": section({FLUTE: TUNE_BARS, CLARINET: turned(2), OBOE: entering(4)}, "mf"),
        "D": section({FLUTE: TUNE_BARS, CLARINET: turned(2), OBOE: turned(4), BASSOON: entering(6)}, "f"),
        "F": section(CROWD, "f", pizz=["G,,2 D,2 G,2", "_B,,2 D,2 G,2"] * 4),
        "G": section(COLLAPSE, "mf", pizz=["G,,6"] * 2 + SILENT[:6],
                     trumpet=[BAR] * 4 + [minor(b) for b in TUNE_BARS[:2]] + [BAR] * 2),
        "H": section({FLUTE: TUNE_BARS[:6] + [BAR] * 2, CLARINET: entering(2)[:6] + [BAR] * 2}, "p",
                     pizz=["G,,6"] * 8),
        "E": section(FULL, "f"),
    },
    forms=("AABCDFGHE",),
    ending={FLUTE: "d6", CLARINET: "B6", OBOE: "G6", BASSOON: "G,6", TRUMPET: BAR, PIZZ: "G,,6"},
    bpm=(165, 184, 205),
)
