"""The "One culture or many" episode's tune, *Round of Villages*: an original
round in F, 4/4, for three recorders (villages), over strings and a pizzicato
bass, with a fiddle for drift.

One form, section by section with the story: A (the villages: one recorder,
the tune plain); B (the rule: the round, each recorder entering two bars
after the last); C (converging: the recorders fall into unison, first two,
then all three); D (the regions: three recorders on three different lines
that never merge); E (more traits, more features, more neighbors, bigger
maps: the tune in two-bar pieces passed around); F (shattering, then drift:
scattered fragments, until the fiddle leads everyone into unison); G (the
title: the round once more, ending on three notes held apart).
"""

from music import Tune, Voice

REC1, REC2, REC3, FIDDLE, STRINGS, PIZZ = "recorder1", "recorder2", "recorder3", "fiddle", "strings", "pizzicato"
REST = "z8"

# K:F: B flat. Eight eighths a bar.
TUNE_BARS = ["F2 A2 c2 A2", "B2 d2 c4", "A2 c2 f2 c2", "d2 B2 c4", "c2 A2 F2 A2", "G2 B2 A2 G2", "F2 A2 G2 E2",
             "F6 z2"]
COUNTER = ["A2 c2 f2 c2", "d2 f2 e4", "c2 f2 a2 f2", "f2 d2 e4", "e2 c2 A2 c2", "B2 d2 c2 B2", "A2 c2 B2 G2",
           "A6 z2"]
DRONE = ["F,8", "B,,8", "F,8", "C,8", "F,8", "C,8", "F,4 C,4", "F,8"]
LOW = ["F,2 A,2 C2 A,2", "B,2 D2 C4", "A,2 C2 F2 C2", "D2 B,2 C4", "C2 A,2 F,2 A,2", "G,2 B,2 A,2 G,2",
       "F,2 A,2 G,2 E,2", "F,6 z2"]
CHORDS = ["[FAc]8", "[Bdf]8", "[FAc]8", "[Bdf]4 [EGc]4", "[FAc]8", "[CEG]8", "[FAc]4 [CEG]4", "[FAc]8"]
ROOTS = ["F,,4 C,4", "B,,,4 F,,4", "F,,4 C,4", "B,,,4 C,4", "F,,4 C,4", "C,4 G,,4", "F,,4 C,4", "F,,4 z4"]


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


def section(mark, rec1=None, rec2=None, rec3=None, fiddle=None, strings=None, pizz=None):
    parts = {REC1: rec1, REC2: rec2, REC3: rec3, FIDDLE: fiddle, STRINGS: strings, PIZZ: pizz}
    return {name: _line(bars or [REST] * 8, mark) for name, bars in parts.items()}


def late(bars, n):
    """The tune entering `n` bars late (a round's next voice)."""
    return [REST] * n + bars[: 8 - n]


def pieces(bars, keep):
    """Only the bars in `keep`, the rest silent."""
    return [bar if k in keep else REST for k, bar in enumerate(bars)]


TUNE = Tune(
    title="Round of Villages",
    slug="round-of-villages",
    key="F",
    beats_per_bar=4,
    voices=(Voice(REC1, 74, 112, pan=40), Voice(REC2, 74, 105, pan=64), Voice(REC3, 74, 100, pan=88),
            Voice(FIDDLE, 40, 105, pan=76), Voice(STRINGS, 48, 70, pan=64), Voice(PIZZ, 45, 90, pan=64)),
    sections={
        "A": section("mp", rec1=TUNE_BARS, strings=CHORDS),
        "B": section("mp", rec1=TUNE_BARS, rec2=late(TUNE_BARS, 2), rec3=late(TUNE_BARS, 4), pizz=ROOTS),
        "C": section("mf", rec1=TUNE_BARS, rec2=late(TUNE_BARS, 2)[:4] + TUNE_BARS[4:], rec3=late(TUNE_BARS, 6)[:6]
                     + TUNE_BARS[6:], strings=CHORDS, pizz=ROOTS),
        "D": section("mf", rec1=TUNE_BARS, rec2=COUNTER, rec3=DRONE, pizz=ROOTS),
        "E": section("mf", rec1=pieces(TUNE_BARS, {0, 1, 6, 7}), rec2=pieces(COUNTER, {2, 3}),
                     rec3=pieces(LOW, {4, 5}), strings=CHORDS, pizz=ROOTS),
        "F": section("mf", rec1=pieces(TUNE_BARS, {1, 6, 7}), rec2=pieces(COUNTER, {0, 3, 7}),
                     rec3=pieces(TUNE_BARS, {2, 6, 7}), fiddle=pieces(TUNE_BARS, {4, 5, 6, 7}), pizz=ROOTS),
        "G": section("mf", rec1=TUNE_BARS, rec2=late(TUNE_BARS, 2), rec3=late(TUNE_BARS, 4), strings=CHORDS,
                     pizz=ROOTS),
    },
    forms=("ABCDEFG",),
    ending={REC1: "F8", REC2: "A8", REC3: "c8", FIDDLE: "z8", STRINGS: "[FAc]8", PIZZ: "F,,8"},
    bpm=(130, 150, 170),
)
