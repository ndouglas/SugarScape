"""The "Variations on a theme" episode's tune, *Variations on a Theme by
Schelling*: an original theme in D, 4/4, on a music box (Schelling's), then a
variation for each paper, over strings.

One form, section by section with the story: A (the theme, plain, on the
music box); B (Pancs & Vriend: two clarinets a third apart that drift into
unison); C (Gauvin: a held chord that never moves, then the theme split
between a high and a low register, then the two interleaved); D (Singh: the
theme on a lone fiddle, then the whole band in scattered fragments); E
(Zhang: clarinet and fiddle trading the theme bar by bar); F (the misses: the
theme with a wrong last note, left unresolved); G (the title: the theme whole,
everyone, ending on D).
"""

from music import Tune, Voice

BOX, CLARINET, CLARINET2, FIDDLE, STRINGS, PIZZ = "musicbox", "clarinet", "clarinet2", "fiddle", "strings", "pizzicato"
REST = "z8"

# K:D: F and C sharp. Eight eighths a bar.
THEME = ["F2 A2 d2 A2", "B2 G2 E4", "A2 F2 D2 F2", "E6 z2", "F2 A2 d2 f2", "e2 d2 B2 c2", "d2 A2 F2 E2", "D6 z2"]
# A third below, meeting the theme in unison from bar 5.
THIRD = ["D2 F2 B2 F2", "G2 E2 C4", "F2 D2 B,2 D2", "C6 z2"] + THEME[4:]
LOW = ["F,2 A,2 D2 A,2", "B,2 G,2 E,4", "A,2 F,2 D,2 F,2", "E,6 z2",
       "F,2 A,2 D2 F2", "E2 D2 B,2 C2", "D2 A,2 F,2 E,2", "D,6 z2"]
HIGH = ["f2 a2 d'2 a2", "b2 g2 e4", "a2 f2 d2 f2", "e6 z2", "f2 a2 d'2 f'2", "e'2 d'2 b2 c'2", "d'2 a2 f2 e2",
        "d6 z2"]
CHORDS = ["[DFA]8", "[GBd]8", "[DFA]8", "[EAc]8", "[DFA]8", "[GBe]8", "[DFA]4 [EAc]4", "[DFA]8"]
ROOTS = ["D,,4 A,,4", "G,,4 D,4", "D,,4 A,,4", "A,,4 E,4", "D,,4 A,,4", "G,,4 D,4", "D,,4 A,,4", "D,,4 z4"]


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


def section(mark, box=None, clarinet=None, clarinet2=None, fiddle=None, strings=None, pizz=None):
    parts = {BOX: box, CLARINET: clarinet, CLARINET2: clarinet2, FIDDLE: fiddle, STRINGS: strings, PIZZ: pizz}
    return {name: _line(bars or [REST] * 8, mark) for name, bars in parts.items()}


def alternate(a, b):
    """Bars from `a` on odd bars and from `b` on even ones (the other rests)."""
    return [x if k % 2 == 0 else REST for k, x in enumerate(a)], [REST if k % 2 == 0 else x for k, x in enumerate(b)]


def fragments(bars, keep):
    """Only the bars in `keep`, the rest silent: the theme in pieces."""
    return [bar if k in keep else REST for k, bar in enumerate(bars)]


FROZEN = ["[DFA]8"] * 3 + [REST] * 5
SPLIT_HIGH = [REST] * 3 + HIGH[3:6] + ["f2 z2 d'2 z2", "z2 a2 z2 f2"]
SPLIT_LOW = [REST] * 3 + LOW[3:6] + ["z2 A,2 z2 D2", "F,2 z2 D,2 z2"]
TRADE_CLARINET, TRADE_FIDDLE = alternate(THEME, THEME)
WRONG = THEME[:7] + ["^G6 z2"]

TUNE = Tune(
    title="Variations on a Theme by Schelling",
    slug="variations-on-a-theme-by-schelling",
    key="D",
    beats_per_bar=4,
    voices=(Voice(BOX, 10, 115, pan=64), Voice(CLARINET, 71, 110, pan=40), Voice(CLARINET2, 71, 100, pan=88),
            Voice(FIDDLE, 40, 110, pan=76), Voice(STRINGS, 48, 70, pan=64), Voice(PIZZ, 45, 90, pan=64)),
    sections={
        "A": section("mp", box=THEME, strings=CHORDS, pizz=ROOTS),
        "B": section("mp", clarinet=THEME, clarinet2=THIRD, strings=CHORDS, pizz=ROOTS),
        "C": section("mf", box=SPLIT_HIGH, clarinet=SPLIT_LOW, strings=FROZEN),
        "D": section("mf", box=fragments(HIGH, {5}), clarinet=fragments(THEME, {4, 7}),
                     clarinet2=fragments(LOW, {6}), fiddle=THEME[:4] + [REST] * 4, pizz=fragments(ROOTS, {5, 7})),
        "E": section("mf", clarinet=TRADE_CLARINET, fiddle=TRADE_FIDDLE, strings=CHORDS, pizz=ROOTS),
        "F": section("mp", box=WRONG, strings=CHORDS[:7] + [REST]),
        "G": section("mf", box=THEME, clarinet=THEME, clarinet2=THIRD, fiddle=HIGH, strings=CHORDS, pizz=ROOTS),
    },
    forms=("ABCDEFG",),
    ending={BOX: "D8", CLARINET: "D8", CLARINET2: "F8", FIDDLE: "d8", STRINGS: "[DFA]8", PIZZ: "D,,8"},
    bpm=(120, 140, 160),
)
