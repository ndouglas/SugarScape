"""The "Neighbors like me" episode's tune, *Hocket for Two Colors*: an original
piece in D minor, 4/4, in which an oboe (Red, Schelling's stars) and a clarinet
(Blue, his zeros) share one melody note by note — a hocket — interleaved like
the starting row, over a bassoon's roots.

One form, section by section with the story: A (the row: the hocket plain, a
note each); B (the squeeze: the notes gather into pairs, then fours); C (the
checkerboard: a bar each, then back and forth); D (sorted: the oboe alone high
with the melody, the clarinet low and long, apart); E (the title: the hocket
returns, both colors in turn, ending in unison on D).
"""

import re

from music import Tune, Voice

OBOE, CLARINET, BASSOON, PIZZ = "oboe", "clarinet", "bassoon", "pizzicato"
BAR = "z8"
TOKEN = re.compile(r"[_^=]?[A-Ga-gz][,']*\d*")

# K:Dm makes B flat.
MELODY = ["D2 F A d2 A F", "G2 B d g2 d B", "A2 c e a2 e c", "d2 A F D4",
          "F2 A c f2 c A", "E2 G B e2 B G", "F2 E D ^C2 E A,", "D6 z2"]
ROOTS = ["D,,", "G,,", "A,,", "D,,", "F,,", "E,,", "A,,", "D,,"]
LOW = ["D,8", "G,8", "A,8", "D,8", "F,8", "E,8", "A,8", "D,8"]


def _length(token):
    n = re.search(r"(\d*)$", token).group(1)
    return int(n or 1)


def hocket(bars, run, first=0):
    """The melody split between two voices: `run` notes to one, then `run` to
    the other (rests counting as notes); the other voice rests meanwhile.
    Returns (oboe bars, clarinet bars)."""
    a, b, k = [], [], first
    for bar in bars:
        ta, tb = [], []
        for token in TOKEN.findall(bar):
            rest = f"z{_length(token)}" if _length(token) > 1 else "z"
            if (k // run) % 2 == 0:
                ta.append(token), tb.append(rest)
            else:
                ta.append(rest), tb.append(token)
            k += 1
        a.append(" ".join(ta))
        b.append(" ".join(tb))
    return a, b


def up(bar):
    """The bar an octave higher."""
    def one(m):
        t = m.group(0)
        if t[0] == "z":
            return t
        acc, rest = (t[0], t[1:]) if t[0] in "_^=" else ("", t)
        note, marks = rest[0], rest[1:]
        if "," in marks:
            return acc + note + marks.replace(",", "", 1)
        if note.isupper():
            return acc + note.lower() + marks
        return acc + note + "'" + marks
    return TOKEN.sub(one, bar)


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


def section(mark, oboe, clarinet, bassoon, pizz=None):
    return {OBOE: _line(oboe, mark), CLARINET: _line(clarinet, mark), BASSOON: _line(bassoon, mark),
            PIZZ: _line(pizz or [BAR] * 8, mark)}


BASS = [f"{r}4 {r}4" for r in ROOTS]
PULSE = [f"{r[:-1]}2 z2 {r[:-1]}2 z2" for r in ROOTS]
one = hocket(MELODY, 1)
pairs = hocket(MELODY[:4], 2)
fours = hocket(MELODY[4:], 4)
bars = hocket(MELODY, 8)

TUNE = Tune(
    title="Hocket for Two Colors",
    slug="hocket-for-two-colors",
    key="Dm",
    beats_per_bar=4,
    voices=(Voice(OBOE, 68, 122, pan=44), Voice(CLARINET, 71, 122, pan=84), Voice(BASSOON, 70, 105, pan=64),
            Voice(PIZZ, 45, 95, pan=64)),
    sections={
        "A": section("mp", one[0], one[1], BASS),
        "B": section("mf", pairs[0] + fours[0], pairs[1] + fours[1], BASS, PULSE),
        "C": section("mf", bars[0], bars[1], BASS, PULSE),
        "D": section("mf", [up(b) for b in MELODY], LOW, BASS, PULSE),
        "E": section("mp", one[0][:7] + ["D6 z2"], one[1][:7] + ["D6 z2"], BASS),
    },
    forms=("ABCDE",),
    ending={OBOE: "D8", CLARINET: "D8", BASSOON: "D,,8", PIZZ: "D,2 z6"},
    bpm=(108, 124, 140),
)
