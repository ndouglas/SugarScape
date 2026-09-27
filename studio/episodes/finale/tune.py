"""The finale's tune: "The Walk Home", a new piece on the pilot's gånglåt
theme ("Flumps' Walk", in D). W and X: the pilot's A theme passed two bars
at a time from episode to episode's instruments — Seasons' violin,
Pollution's brass, Inheritance's cello, Tribes' recorder; then Markets'
piano, Credit's tenor sax, Contagion's accordion — over War's timpani, the
pilot's guitar and its fiddle drone. B: the pilot's B theme, thinner, for
the waves and the collision. L: the theme on a lone accordion over the drone,
for the ledger and the title, ending on the pilot's opening figure brought
home to D."""

import re

from music import Tune, Voice

VIOLIN, BRASS, CELLO, RECORDER = "violin", "brass", "cello", "recorder"
PIANO, SAX, ACCORDION, TIMPANI = "piano", "tenor sax", "accordion", "timpani"
GUITAR, DRONE = "guitar", "fiddle drone"
MELODISTS = (VIOLIN, BRASS, CELLO, RECORDER, PIANO, SAX, ACCORDION)

BAR = "z8"
# The pilot's themes, bar by bar.
THEME_A = ["A2 FA d2 AF", "G2 FE D2 FA", "B2 GB d2 BG", "A2 ce a2 gf",
           "e2 dc d2 AF", "B2 dB f2 dB", "e2 dc B2 A2", "d2 f2 d4"]
THEME_B = ["B2 dg b2 ag", "f2 ed f2 a2", "e2 ce a2 ge", "f2 df a4",
           "g2 bg e2 dB", "A2 FA d2 fd", "e2 fe dc BA", "d2 A2 D4"]
# The pilot's guitar oom-pah, and its chords under each theme.
OOMPAH = {"D": "D,2 [DFA]2 A,,2 [DFA]2", "G": "G,,2 [GBd]2 D,2 [GBd]2", "A": "A,,2 [Ace]2 E,2 [Ace]2",
          "Bm": "B,,2 [Bdf]2 F,2 [Bdf]2", "EmA": "E,2 [EGB]2 A,,2 [Ace]2"}
CHORDS_A = ("D", "D", "G", "A", "D", "Bm", "EmA", "D")
CHORDS_B = ("G", "D", "A", "D", "G", "D", "EmA", "D")


def octave_down(bar):
    """The bar an octave lower: lowercase notes to uppercase, uppercase ones
    down with a comma."""
    def lower(m):
        acc, note, marks = m.group(1), m.group(2), m.group(3)
        if note.islower():
            return acc + note.upper() + marks.replace("'", "", 1)
        return acc + note + marks + ","
    return re.sub(r"([_^=]?)([A-Ga-g])([,']*)", lower, bar)


def _bars(bars):
    return " | ".join(bars) + " |"


def _relay(order, dynamic):
    """The A theme, two bars to each instrument in `order`; the rest rest."""
    parts = {v: [BAR] * 8 for v in MELODISTS}
    for i, voice in enumerate(order):
        for k in (2 * i, 2 * i + 1):
            bar = octave_down(THEME_A[k]) if voice == CELLO else THEME_A[k]
            parts[voice][k] = (f"!{dynamic}! {bar}" if k == 2 * i else bar)
    return {v: _bars(b) for v, b in parts.items()}


def _guitar(chords, dynamic):
    return _bars([f"!{dynamic}! {OOMPAH[chords[0]]}"] + [OOMPAH[c] for c in chords[1:]])


DRONE_BARS = _bars(["[DA]8"] * 8)
TIMP = "D,,2 z2 A,,,2 z2"

TUNE = Tune(
    title="The Walk Home",
    slug="the-walk-home",
    key="D",
    beats_per_bar=4,
    voices=(Voice(VIOLIN, 40, 100, pan=30), Voice(BRASS, 61, 95), Voice(CELLO, 42, 100, pan=95),
            Voice(RECORDER, 74, 100, pan=40), Voice(PIANO, 3, 95), Voice(SAX, 66, 95, pan=85),
            Voice(ACCORDION, 21, 110), Voice(TIMPANI, 47, 100), Voice(GUITAR, 24, 80), Voice(DRONE, 110, 55)),
    sections={
        "W": {**_relay((VIOLIN, BRASS, CELLO, RECORDER), "mf"), TIMPANI: _bars([BAR] * 8),
              GUITAR: _guitar(CHORDS_A, "mf"), DRONE: DRONE_BARS},
        "X": {**_relay((PIANO, SAX, ACCORDION, VIOLIN), "f"), TIMPANI: _bars([f"!mf! {TIMP}"] + [TIMP] * 7),
              GUITAR: _guitar(CHORDS_A, "f"), DRONE: DRONE_BARS},
        "B": {**{v: _bars([BAR] * 8) for v in MELODISTS},
              VIOLIN: _bars([f"!mp! {THEME_B[0]}"] + THEME_B[1:]),
              CELLO: _bars(["!p! " + octave_down(THEME_B[0])] + [octave_down(b) for b in THEME_B[1:]]),
              TIMPANI: _bars([BAR] * 8), GUITAR: _guitar(CHORDS_B, "mp"), DRONE: DRONE_BARS},
        "L": {**{v: _bars([BAR] * 8) for v in MELODISTS},
              ACCORDION: _bars([f"!p! {THEME_A[0]}"] + THEME_A[1:]),
              TIMPANI: _bars([BAR] * 8), GUITAR: _bars([BAR] * 8), DRONE: _bars(["!p! [DA]8"] + ["[DA]8"] * 7)},
    },
    # One form: the relay under the opening and the charts, the thinner B
    # under the waves and the collision, the lone accordion under the ledger.
    forms=("WXBL",),
    # The pilot's opening figure, brought home to D.
    ending={ACCORDION: "!pp! A2 FA d4", DRONE: "[DA]8", **{v: BAR for v in MELODISTS if v != ACCORDION},
            TIMPANI: BAR, GUITAR: BAR},
    bpm=(92, 106, 120),
)
