"""The Norms episode's tune, *A Fugue for Keeping Order*: an original fugue in
C minor for string quartet, a rule every voice must follow and must enforce
in the others. 2/4, written as bars of four eighths; the subject is two
bars, the answer the subject a fifth up (a real answer). Every accidental is
written out (K:C), so entries can be moved by transposing their notes.

One form, section by section with the story: A A (the rules: the subject
alone on the viola, with its countersubject); B (the norms game: each voice
starts the subject but nobody answers, single lines fraying); C C (the
metanorms game: a proper exposition, each voice answering the last); D (a
million generations: the same exposition with the entries slipping out of
step, until they stop); E (the sentence: the cello alone, the subject slowed
to half speed); F (read Axelrod's way: stretto, the entries a bar apart,
holding); G (milder metapunishment: fraying again); H (the title: all four on
the subject together, ending on a C major chord, a Picardy third).
"""

import re

from music import Tune, Voice

VIOLIN1, VIOLIN2, VIOLA, CELLO = "violin I", "violin II", "viola", "cello"
VOICES = (VIOLIN1, VIOLIN2, VIOLA, CELLO)
BAR = "z4"

SUBJECT = ["G, C _E G", "F _E D C"]
COUNTER = ["_E2 D2", "C B, C2"]

NOTE = re.compile(r"([_^=]?)([A-Ga-g])([,']*)(\d*)")
STEP = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}
SPELL = ["=C", "_D", "=D", "_E", "=E", "=F", "^F", "=G", "_A", "=A", "_B", "=B"]


def _pitch(acc, letter, marks):
    p = 60 + STEP[letter.upper()] + (12 if letter.islower() else 0)
    p += {"^": 1, "_": -1}.get(acc, 0)
    return p + 12 * marks.count("'") - 12 * marks.count(",")


def _name(p):
    octave, pc = divmod(p - 60, 12)
    acc, letter = SPELL[pc][0], SPELL[pc][1]
    if octave >= 1:
        return acc + letter.lower() + "'" * (octave - 1)
    return acc + letter + "," * (-octave)


def moved(bars, semitones):
    """Bars moved by `semitones`, every accidental written out."""
    def move(m):
        acc, letter, marks, length = m.groups()
        return _name(_pitch(acc, letter, marks) + semitones) + length
    return [NOTE.sub(move, b) for b in bars]


def halved(bars):
    """Bars at half speed: every note twice as long (twice as many bars)."""
    out = []
    for b in bars:
        doubled = NOTE.sub(lambda m: m.group(1) + m.group(2) + m.group(3) + str(int(m.group(4) or 1) * 2), b)
        notes = doubled.split()
        half, total = [], 0
        for n in notes:
            half.append(n)
            total += int(re.sub(r"[^0-9]", "", n) or 1)
            if total == 4:
                out.append(" ".join(half))
                half, total = [], 0
    return out


def late(bars):
    """Bars one eighth late: a rest first, the last eighth lost."""
    flat = []
    for b in bars:
        for n in b.split():
            length = int(re.sub(r"[^0-9]", "", NOTE.sub(lambda m: m.group(4) or "1", n)) or 1)
            flat.extend([n.rstrip("0123456789")] + ["-"] * (length - 1))
    shifted = ["z"] + flat[:-1]
    out, cur = [], []
    for i, n in enumerate(shifted):
        cur.append(n)
        if len(cur) == 4:
            out.append(_join(cur))
            cur = []
    return out


def _join(eighths):
    """Four eighths (a note, "-" continuing the note before, or "z") as a bar."""
    out, i = [], 0
    while i < len(eighths):
        n, length = eighths[i], 1
        while i + length < len(eighths) and eighths[i + length] == "-":
            length += 1
        if n == "-":
            n = "z"
        out.append(n + (str(length) if length > 1 else ""))
        i += length
    return " ".join(out)


# Registers: the viola states the subject; violin II answers a fifth up; violin
# I takes the subject an octave up; the cello answers an octave and a fourth
# below.
PART = {VIOLA: 0, VIOLIN2: 7, VIOLIN1: 12, CELLO: -17}


def subject(voice):
    return moved(SUBJECT, PART[voice])


def counter(voice):
    return moved(COUNTER, PART[voice])


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


SILENT = [BAR] * 8


def section(mark, parts):
    return {v: _line(parts.get(v, SILENT), mark) for v in VOICES}


def exposition():
    """Each voice enters two bars after the last and then keeps the
    countersubject going."""
    parts = {}
    for i, v in enumerate((VIOLA, VIOLIN2, VIOLIN1, CELLO)):
        body = subject(v) + counter(v) * 3
        parts[v] = [BAR] * (2 * i) + body[: 8 - 2 * i]
    return parts


def fraying():
    """Each voice starts the subject's first bar, and nobody answers."""
    parts = {}
    for i, v in enumerate((VIOLA, VIOLIN2, VIOLIN1, CELLO)):
        parts[v] = [BAR] * (2 * i) + [subject(v)[0]] + [BAR] * (7 - 2 * i)
    return parts


def slipping():
    """The exposition, each later entry an eighth later still, then silence."""
    parts = exposition()
    parts[VIOLIN2] = parts[VIOLIN2][:2] + late(parts[VIOLIN2][2:6]) + [BAR] * 2
    parts[VIOLIN1] = parts[VIOLIN1][:4] + late(late(parts[VIOLIN1][4:7])) + [BAR]
    parts[CELLO] = parts[CELLO][:6] + [BAR, BAR]
    parts[VIOLA] = parts[VIOLA][:6] + [BAR, BAR]
    return parts


def stretto():
    """The entries a bar apart, overlapping, and holding."""
    parts = {}
    for i, v in enumerate((VIOLA, VIOLIN2, VIOLIN1, CELLO)):
        body = (subject(v) + counter(v)) * 2
        parts[v] = [BAR] * i + body[: 8 - i]
    return parts


TUNE = Tune(
    title="A Fugue for Keeping Order",
    slug="a-fugue-for-keeping-order",
    key="C",
    beats_per_bar=2,
    voices=(Voice(VIOLIN1, 40, 100, pan=34), Voice(VIOLIN2, 40, 92, pan=54), Voice(VIOLA, 41, 100, pan=74),
            Voice(CELLO, 42, 105, pan=94)),
    sections={
        "A": section("mf", {VIOLA: (subject(VIOLA) + counter(VIOLA)) * 2}),
        "B": section("mp", fraying()),
        "C": section("f", exposition()),
        "D": section("mf", slipping()),
        "E": section("mp", {CELLO: halved(subject(CELLO)) + halved(counter(CELLO))}),
        "F": section("f", stretto()),
        "G": section("mp", fraying()),
        "H": section("f", {v: (subject(v) + counter(v)) * 2 for v in VOICES}),
    },
    forms=("AABCCDEFGH",),
    ending={VIOLIN1: "[=c=e]4", VIOLIN2: "[=G=c]4", VIOLA: "[=E=G]4", CELLO: "[=C,,=C,]4"},
    bpm=(115, 128, 140),
)
