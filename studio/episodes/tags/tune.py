"""The Tags episode's tune, *The Twins' Slip Jig*: an original slip jig in
9/8 (three groups of three eighths a bar), played in unison by twin
instruments: the same notes, doubled exactly. Written in K:C with every
accidental explicit, so a section can be moved up a step by transposing its
notes.

One form, section by section with the story: A A (the rules: the tin
whistle alone over a bodhrán); B (it works, and twins: whistle and fiddle
in unison, in D); C (their narrow tolerance: two mandolins in unison, up a
step, in E); D (the takeovers: accordion and concertina in unison, up
again, in F sharp); F (twins forbidden: the fiddle a beat late and a half
step off, the tune thinning to the bodhrán alone; then, tolerance taken
away, the unison returns, louder); G (the title: everyone in unison in D,
ending on a doubled D).
"""

import re

from music import Tune, Voice

WHISTLE, FIDDLE, MANDOLIN, MANDOLIN2, ACCORDION, CONCERTINA, BODHRAN = (
    "whistle", "fiddle", "mandolin", "mandolin 2", "accordion", "concertina", "bodhran")
UNISON = (WHISTLE, FIDDLE, MANDOLIN, MANDOLIN2, ACCORDION, CONCERTINA)

BAR = "z9"
# The jig in D major, as groups of three eighths (three to a bar), with its
# sharps written out (K:C).
GROUPS = [
    "d2 A", "^F2 A", "B2 A",
    "d2 A", "^F2 A", "E2 ^F",
    "G2 B", "d2 B", "G2 B",
    "A2 ^F", "D2 ^F", "E3",
    "d2 A", "^F2 A", "B2 A",
    "d2 ^f", "e2 d", "^c2 e",
    "d2 A", "B2 G", "A2 ^F",
    "E2 D", "D3", "D3",
]
DRUM = "A,,2 z A,,2 z A,, A,, A,,"  # low tom (key 45) standing in for the bodhrán

NOTE = re.compile(r"([_^=]?)([A-Ga-g])([,']*)(\d*)")
STEP = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}
SPELL = ["=C", "^C", "=D", "^D", "=E", "=F", "^F", "=G", "^G", "=A", "^A", "=B"]


def _pitch(acc, letter, marks):
    p = 60 + STEP[letter.upper()] + (12 if letter.islower() else 0)
    p += {"^": 1, "_": -1}.get(acc, 0)
    return p + 12 * marks.count("'") - 12 * marks.count(",")


def _name(p):
    octave, pc = divmod(p - 60, 12)
    acc_letter = SPELL[pc]
    acc, letter = acc_letter[0], acc_letter[1]
    if octave >= 1:
        return acc + letter.lower() + "'" * (octave - 1)
    return acc + letter + "," * (-octave)


def transpose(text, semitones):
    """ABC notes moved by `semitones`, every accidental written out."""
    def move(m):
        acc, letter, marks, length = m.groups()
        return _name(_pitch(acc, letter, marks) + semitones) + length
    return NOTE.sub(move, text)


def bars(groups):
    return [" ".join(groups[i : i + 3]) for i in range(0, len(groups), 3)]


TUNE_BARS = bars(GROUPS)


def _line(parts, mark=None):
    parts = list(parts)
    if mark:
        parts[0] = f"!{mark}! {parts[0]}"
    return " | ".join(parts) + " |"


SILENT = [BAR] * 8


def section(mark, parts):
    """Eight bars for every voice: `parts` gives some voices their bars; the
    rest rest, and the bodhrán keeps time unless given its own."""
    out = {v: _line(parts.get(v, SILENT), mark) for v in UNISON}
    out[BODHRAN] = _line(parts.get(BODHRAN, [DRUM] * 8), mark)
    return out


def up(steps):
    return [transpose(b, steps) for b in TUNE_BARS]


# Twins forbidden: the fiddle a beat (a group) late and a half step sharp,
# for three bars that thin to the drum alone; then the unison returns.
LATE = ["z3"] + [transpose(g, 1) for g in GROUPS[:8]]
SPLIT_WHISTLE = TUNE_BARS[:2] + [BAR] + TUNE_BARS[3:]
SPLIT_FIDDLE = bars(LATE)[:2] + [BAR] + TUNE_BARS[3:]
SPLIT_DRUM = [DRUM, DRUM, DRUM] + [DRUM] * 5

TUNE = Tune(
    title="The Twins' Slip Jig",
    slug="the-twins-slip-jig",
    key="C",
    beats_per_bar=4.5,
    meter="9/8",
    voices=(Voice(WHISTLE, 78, 100, pan=50), Voice(FIDDLE, 40, 100, pan=78), Voice(MANDOLIN, 25, 95, pan=40),
            Voice(MANDOLIN2, 25, 95, pan=88), Voice(ACCORDION, 21, 95, pan=44), Voice(CONCERTINA, 23, 95, pan=84),
            Voice(BODHRAN, 0, 100, channel=10)),
    sections={
        "A": section("mf", {WHISTLE: TUNE_BARS}),
        "B": section("f", {WHISTLE: TUNE_BARS, FIDDLE: TUNE_BARS}),
        "C": section("f", {MANDOLIN: up(2), MANDOLIN2: up(2)}),
        "D": section("f", {ACCORDION: up(4), CONCERTINA: up(4)}),
        "F": section("mf", {WHISTLE: SPLIT_WHISTLE[:3] + ["!ff! " + SPLIT_WHISTLE[3]] + SPLIT_WHISTLE[4:],
                            FIDDLE: SPLIT_FIDDLE[:3] + ["!ff! " + SPLIT_FIDDLE[3]] + SPLIT_FIDDLE[4:],
                            BODHRAN: SPLIT_DRUM}),
        "G": section("ff", {v: TUNE_BARS for v in UNISON}),
    },
    forms=("AABCDFG",),
    ending={WHISTLE: "d9", FIDDLE: "D9", MANDOLIN: "d9", MANDOLIN2: "D9", ACCORDION: "d9", CONCERTINA: "D9",
            BODHRAN: "A,,9"},
    bpm=(185, 208, 230),
)
