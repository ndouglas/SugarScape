"""The "Listening to the like-minded" episode's tune, *Converging Lines*: an
original piece in E minor, 4/4, for a string quartet (two violins, viola and
cello), each voice an opinion that moves toward the others it can hear.

One form, section by section with the story: A (the crowd and the rule: the
subject entering in each voice in turn, scattered over the range); B (few:
short fragments that never meet); C (many: the voices close in until all
four hold one E); D (the middle: the voices pair off in two registers, then
three); E (the edges and leaning: the whole texture climbs); F (the extremes,
then neighbors: the outer voices pushed to the top and bottom, then one line
with stray notes); G (the title: the subject in every voice once more,
ending on an open fifth).
"""

import re

from music import Tune, Voice

VN1, VN2, VLA, VC = "violin1", "violin2", "viola", "cello"
REST = "z8"
TOKEN = re.compile(r"[_^=]?[A-Ga-gz][,']*\d*")

# K:Em: F sharp. Eight eighths a bar.
SUBJECT = ["E2 G2 B2 e2", "d2 B2 c4", "B2 A2 G2 ^D2", "E6 z2", "G2 B2 e2 g2", "f2 e2 d4", "c2 B2 A2 ^D2", "E6 z2"]


def _shift(bar, octaves):
    """The bar `octaves` up (negative: down), in ABC's marks."""
    def one(m):
        t = m.group(0)
        if t[0] == "z":
            return t
        acc, rest = (t[0], t[1:]) if t[0] in "_^=" else ("", t)
        note, marks = rest[0], "".join(c for c in rest[1:] if c in ",'")
        length = rest[1 + len(marks):]
        # Height: 0 for C–B, 1 for c–b, +1 per ', −1 per ,.
        h = (1 if note.islower() else 0) + marks.count("'") - marks.count(",") + octaves
        if h >= 1:
            return acc + note.lower() + "'" * (h - 1) + length
        return acc + note.upper() + "," * (-h) + length
    return TOKEN.sub(one, bar)


def up(bars, n=1):
    return [_shift(b, n) for b in bars]


def down(bars, n=1):
    return [_shift(b, -n) for b in bars]


def late(bars, n):
    """The subject entering `n` bars late."""
    return [REST] * n + bars[: 8 - n]


def pieces(bars, keep):
    return [bar if k in keep else REST for k, bar in enumerate(bars)]


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


def section(mark, vn1, vn2, vla, vc):
    return {VN1: _line(vn1, mark), VN2: _line(vn2, mark), VLA: _line(vla, mark), VC: _line(vc, mark)}


HOLD = ["E8"] * 4

TUNE = Tune(
    title="Converging Lines",
    slug="converging-lines",
    key="Em",
    beats_per_bar=4,
    voices=(Voice(VN1, 40, 110, pan=36), Voice(VN2, 40, 104, pan=54), Voice(VLA, 41, 104, pan=74),
            Voice(VC, 42, 110, pan=92)),
    sections={
        "A": section("mp", up(SUBJECT), late(SUBJECT, 2), late(SUBJECT, 4), late(down(SUBJECT), 6)),
        "B": section("mp", pieces(up(SUBJECT), {0, 3, 6}), pieces(SUBJECT, {1, 4, 7}), pieces(SUBJECT, {2, 5}),
                     pieces(down(SUBJECT), {0, 4})),
        "C": section("mf", up(SUBJECT)[:4] + up(HOLD), SUBJECT[:4] + up(HOLD), SUBJECT[:4] + HOLD,
                     down(SUBJECT)[:4] + HOLD),
        "D": section("mf", up(SUBJECT), up(SUBJECT), down(SUBJECT)[:4] + SUBJECT[4:], down(SUBJECT)),
        "E": section("mf", up(SUBJECT), SUBJECT[:4] + up(SUBJECT)[4:], SUBJECT, down(SUBJECT)[:4] + SUBJECT[4:]),
        "F": section("f", up(SUBJECT, 2)[:4] + up(SUBJECT)[4:], SUBJECT, pieces(SUBJECT, {1, 5, 6}),
                     down(SUBJECT)[:4] + SUBJECT[4:]),
        "G": section("mf", up(SUBJECT), late(SUBJECT, 2), late(SUBJECT, 4), late(down(SUBJECT), 6)),
    },
    forms=("ABCDEFG",),
    ending={VN1: "b8", VN2: "e8", VLA: "B8", VC: "E,8"},
    bpm=(130, 150, 170),
)
