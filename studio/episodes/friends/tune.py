"""The Friends and strangers episode's tune, *Swing Your Partner*: an
original reel in G, 4/4, as for a square dance, where who you dance with is
the whole game: fiddle, banjo, guitar, upright bass and claves standing in
for the spoons.

One form, section by section with the story: A A (the rules: fiddle and
guitar, the tune plain); B (strangers: the band loses the beat, each player
later than the last); C (neighbors and friends: the reel in time, the full
band); D (a tenth swapped: in time, then slipping out of time every other
bar); E (30 % and half: slipping, then falling apart to fragments); F (the
paper: steady, the fiddle leading); G (the title: the full band, ending on a
"shave and a haircut").
"""

import re

from music import Tune, Voice

FIDDLE, BANJO, GUITAR, BASS, CLAVES = "fiddle", "banjo", "guitar", "bass", "claves"
BAR = "z8"

# K:G makes F sharp.
TUNE_BARS = ["G2 B d g d B d", "c A F A d A F A", "G B d B g B d B", "A B c A B G G2",
             "d g g f g a b g", "a f d f a f d f", "g f e d c B A F", "G2 B2 G4"]
CHORDS = ["G", "D", "G", "D", "G", "D", "C", "G"]
STRUM = {"G": "G,2 [GBd]2 D,2 [GBd]2", "D": "A,2 [FAd]2 D,2 [FAd]2", "C": "C2 [EGc]2 G,2 [EGc]2"}
ROLL = {"G": "G B d g d B G B", "D": "F A d f d A F A", "C": "E G c e c G E G"}
ROOT = {"G": "G,,4 D,,4", "D": "D,,4 A,,4", "C": "C,,4 G,,4"}
CLICK = "z ^d z ^d z ^d z ^d"  # claves (key 75) on the off-beats
NOTE = re.compile(r"([_^=]?)([A-Ga-g])([,']*)(\d*)")


def _eighths(bars):
    """Bars as a flat list of eighths: a note, "-" continuing it, or "z"."""
    flat = []
    for b in bars:
        for token in re.findall(r"\[[^\]]+\]\d*|[_^=]?[A-Ga-gz][,']*\d*", b):
            length = int(re.search(r"(\d*)$", token).group(1) or 1)
            flat += [token[: len(token) - len(str(length))] if re.search(r"\d$", token) else token] + ["-"] * (length - 1)
    return flat


def _bars_of(flat):
    out = []
    for k in range(0, len(flat), 8):
        chunk, notes, i = flat[k : k + 8], [], 0
        while i < len(chunk):
            n, length = chunk[i], 1
            while i + length < len(chunk) and chunk[i + length] == "-":
                length += 1
            notes.append(("z" if n == "-" else n) + (str(length) if length > 1 else ""))
            i += length
        out.append(" ".join(notes))
    return out


def late(bars, eighths):
    """Bars shifted `eighths` late (rests first, the end lost): out of time."""
    flat = _eighths(bars)
    return _bars_of(["z"] * eighths + flat[: len(flat) - eighths])


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


SILENT = [BAR] * 8
STRUMS = [STRUM[c] for c in CHORDS]
ROLLS = [ROLL[c] for c in CHORDS]
ROOTS = [ROOT[c] for c in CHORDS]
CLICKS = [CLICK] * 8


def section(mark, fiddle=SILENT, banjo=SILENT, guitar=SILENT, bass=SILENT, claves=SILENT):
    return {FIDDLE: _line(fiddle, mark), BANJO: _line(banjo, mark), GUITAR: _line(guitar, mark),
            BASS: _line(bass, mark), CLAVES: _line(claves, mark)}


def alternating(bars, shifted):
    """Every other bar out of time."""
    return [s if k % 2 else b for k, (b, s) in enumerate(zip(bars, shifted))]


TUNE = Tune(
    title="Swing Your Partner",
    slug="swing-your-partner",
    key="G",
    beats_per_bar=4,
    voices=(Voice(FIDDLE, 40, 105, pan=64), Voice(BANJO, 105, 90, pan=40), Voice(GUITAR, 25, 85, pan=88),
            Voice(BASS, 32, 100, pan=64), Voice(CLAVES, 0, 70, channel=10)),
    sections={
        "A": section("mf", fiddle=TUNE_BARS, guitar=STRUMS),
        "B": section("mf", fiddle=TUNE_BARS, banjo=late(ROLLS, 1), guitar=late(STRUMS, 3), bass=late(ROOTS, 2),
                     claves=late(CLICKS, 1)),
        "C": section("f", fiddle=TUNE_BARS, banjo=ROLLS, guitar=STRUMS, bass=ROOTS, claves=CLICKS),
        "D": section("f", fiddle=TUNE_BARS, banjo=ROLLS[:4] + alternating(ROLLS[4:], late(ROLLS, 2)[4:]),
                     guitar=STRUMS[:4] + alternating(STRUMS[4:], late(STRUMS, 3)[4:]), bass=ROOTS, claves=CLICKS),
        "E": section("mf", fiddle=TUNE_BARS[:4] + [TUNE_BARS[4], BAR, "d2 z6", BAR],
                     banjo=alternating(ROLLS[:4], late(ROLLS, 2)[:4]) + [late(ROLLS, 3)[4], BAR, BAR, BAR],
                     guitar=alternating(STRUMS[:4], late(STRUMS, 3)[:4]) + [BAR] * 4,
                     bass=ROOTS[:4] + [late(ROOTS, 1)[4], BAR, BAR, BAR], claves=CLICKS[:4] + [BAR] * 4),
        "F": section("mf", fiddle=TUNE_BARS, guitar=STRUMS, bass=ROOTS),
        "G": section("f", fiddle=TUNE_BARS, banjo=ROLLS, guitar=STRUMS, bass=ROOTS, claves=CLICKS),
    },
    forms=("AABCDEFG",),
    ending={FIDDLE: "G2 D D E2 D2", BANJO: "G2 D D E2 D2", GUITAR: "[GBd]2 z6", BASS: "G,,2 z6", CLAVES: "^d2 z6"},
    bpm=(190, 216, 240),
)
