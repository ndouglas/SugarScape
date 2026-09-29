"""The Ethnocentrism episode's tune, *Cradle Song for Four Colors*: an
original lullaby in F major, 6/8 (written as bars of six eighths, M:3/4),
rocking like a cradle at about 69 dotted quarters a minute. Four
instruments, one per color: music box, harp, clarinet and cello, each
keeping to its own part, as each color helps its own.

One form, section by section with the story: A (the close-ups: the music
box alone); B (the land fills: harp arpeggios and the cello's roots join);
C (favoritism and family: the clarinet answers the tune's phrase ends); D
(the scattering: fragments on the wrong instruments in the wrong registers,
bare fifths, then the music box alone, its notes lengthening as a box winds
down, stopping short of home); E (color-blind: the clarinet alone, the tune
without its answers); G (the usual cost and the title: all four, the
lullaby whole, ending on a soft F major chord in the harp).
"""

from music import Tune, Voice

BOX, HARP, CLARINET, CELLO = "music box", "harp", "clarinet", "cello"
BAR = "z6"

# The tune (K:F makes B flat).
TUNE_BARS = ["c2 A F2 A", "G2 F E2 C", "D2 F A2 d", "c3 A3", "c2 A F2 A", "G2 A B2 G", "A2 F G2 E", "F6"]
CHORDS = ["F", "C", "Dm", "F", "F", "Gm", "C", "F"]
ARP = {"F": "F,C F A F C", "C": "C,G, C E C G,", "Dm": "D,A, D F D A,", "Gm": "G,D G B G D"}
ROOT = {"F": "F,,6", "C": "C,6", "Dm": "D,6", "Gm": "G,,6"}
# The clarinet answers each phrase end, an octave's reach below the box.
ANSWER = ["z3 A3", "z3 G3", "z3 F3", "z3 E3", "z3 A3", "z3 B3", "z3 c3", "A6"]


def _bars(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


SILENT = [BAR] * 8
HARP_BARS = [ARP[c] for c in CHORDS]
CELLO_BARS = [ROOT[c] for c in CHORDS]


def section(mark, box=SILENT, harp=SILENT, clarinet=SILENT, cello=SILENT):
    return {BOX: _bars(box, mark), HARP: _bars(harp, mark), CLARINET: _bars(clarinet, mark),
            CELLO: _bars(cello, mark)}


TUNE = Tune(
    title="Cradle Song for Four Colors",
    slug="cradle-song-for-four-colors",
    key="F",
    beats_per_bar=3,
    voices=(Voice(BOX, 10, 127, pan=64), Voice(HARP, 46, 100, pan=40), Voice(CLARINET, 71, 95, pan=88),
            Voice(CELLO, 42, 90, pan=56)),
    sections={
        "A": section("mf", box=TUNE_BARS),
        "B": section("mf", box=TUNE_BARS, harp=HARP_BARS, cello=CELLO_BARS),
        "C": section("f", box=TUNE_BARS, harp=HARP_BARS, clarinet=ANSWER, cello=CELLO_BARS),
        # The scattering: each phrase on the wrong instrument, in the wrong
        # register, with rests between; bare fifths; then the box alone,
        # winding down, stopping on C, short of home.
        "D": section("mf",
                     box=[BAR, BAR, BAR, BAR, "c2 A F2 A", "G3 F3", "E6", "C6"],
                     harp=["[F,C]6", BAR, "[D,A,]6", BAR, BAR, BAR, BAR, BAR],
                     clarinet=[BAR, "g2 f e2 c", BAR, "c'3 a3", BAR, BAR, BAR, BAR],
                     cello=["c2 A F2 A", BAR, "D,,2 F,, A,,2 D,", BAR, BAR, BAR, BAR, BAR]),
        # Color-blind: the clarinet alone, unanswered.
        "E": section("mf", clarinet=TUNE_BARS),
        # The usual cost and the title: all four, whole.
        "G": section("mf", box=TUNE_BARS, harp=HARP_BARS, clarinet=ANSWER, cello=CELLO_BARS),
    },
    forms=("ABCDEG",),
    ending={BOX: "F6", HARP: "[F,A,CF]6", CLARINET: "A6", CELLO: "F,,6"},
    bpm=(92, 110, 118),
)
