"""The Spatial games episode's tune, *Neighbors in Phase*: an original phase
piece after the manner of Steve Reich's *Piano Phase*. A marimba and a
vibraphone play the same twelve-note figure in D Dorian; section by section,
the vibraphone slips ahead by eighth notes, so the same notes keep making
new patterns, as the kaleidoscope does. 6/8, written as bars of six eighths
(M:3/4); the figure spans two bars. A bowed cello holds a drone, and
pizzicato strings join for the chaos.

One form, section by section with the story: A A (the close-ups, in
unison); B C D E (the kaleidoscope and the scattered worlds: the vibraphone
one, two, three, then five eighths ahead, the pizzicato from C); F (the
clock: the vibraphone falls into triplets against the marimba's eighths,
and the drone drops a half step as the cheats take the board); G H (helpers
survive either way: unison again, then softer, ending on the open fifth).
The beats are timed so F begins just after "clock" and G just as "below".
"""

from music import Tune, Voice

MARIMBA, VIBES, CELLO, PIZZ = "marimba", "vibraphone", "cello", "pizzicato"

BAR = "z6"
FIGURE = ["D", "A", "c", "d", "A", "e", "d", "c", "A", "B", "c", "A"]


def _bars(bars):
    return " | ".join(bars) + " |"


def phrase(shift=0):
    """Four statements of the figure (eight bars), starting `shift` eighths in."""
    notes = (FIGURE[shift:] + FIGURE[:shift]) * 4
    return [" ".join(notes[k : k + 3]) + " " + " ".join(notes[k + 3 : k + 6]) for k in range(0, 48, 6)]


def triplets():
    """Eight bars of the figure in triplet eighths: nine notes a bar against
    the marimba's six."""
    notes = FIGURE * 6
    return ["(3" + " ".join(notes[k : k + 3]) + " (3" + " ".join(notes[k + 3 : k + 6]) + " (3"
            + " ".join(notes[k + 6 : k + 9]) for k in range(0, 72, 9)]


def dynamic(bars, mark):
    return [f"!{mark}! {bars[0]}"] + bars[1:]


DRONE = ["D,,6"] * 8
DARK_DRONE = ["^C,,6"] * 8
PIZZ_D = ["D,3 A,,3"] * 8
PIZZ_DARK = ["^C,3 ^G,,3"] * 8


def section(mark, shift=None, pizz=None, drone=DRONE, vibes=None):
    return {
        MARIMBA: _bars(dynamic(phrase(), mark)),
        VIBES: _bars(dynamic(vibes if vibes is not None else phrase(shift), mark)),
        CELLO: _bars(dynamic(drone, mark)),
        PIZZ: _bars(dynamic(pizz, mark)) if pizz else _bars([BAR] * 8),
    }


TUNE = Tune(
    title="Neighbors in Phase",
    slug="neighbors-in-phase",
    key="Ddor",
    beats_per_bar=3,
    voices=(Voice(MARIMBA, 12, 105, pan=44), Voice(VIBES, 11, 95, pan=84), Voice(CELLO, 42, 70),
            Voice(PIZZ, 45, 85)),
    sections={
        "A": section("mp", 0),
        "B": section("mp", 1),
        "C": section("mf", 2, PIZZ_D),
        "D": section("mf", 3, PIZZ_D),
        "E": section("f", 5, PIZZ_D),
        "F": section("f", pizz=PIZZ_DARK, drone=DARK_DRONE, vibes=triplets()),
        "G": section("mf", 0),
        "H": section("p", 0),
    },
    forms=("AABCDEFGH",),
    ending={MARIMBA: "[D,A,D]6", VIBES: "[DA]6", CELLO: "D,,6", PIZZ: BAR},
    bpm=(150, 170, 190),
)
