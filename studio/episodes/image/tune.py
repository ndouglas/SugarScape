"""The Reputation episode's tune, *The Talk of the Town*: an original
Charleston in B♭, 4/4: gossip set to a dance. A clarinet sings each phrase
and a muted trumpet repeats it a bar later, as a reputation gets passed
along, over banjo, tuba and brushed snare, in the Charleston's rhythm (a
hit on one and on the and of two).

One form, section by section with the story: A A (the rules: clarinet and
banjo); B (the paper's run: the full band, the trumpet answering); C (here:
the band drops out a pair at a time until the tuba plays alone); D (let
rules mutate: stop-time hits that fall silent; then the bigger group: the
tune passed along, each echo quieter, until it can't be heard); E (the
title: the band together, ending on a Charleston hit).
"""

from music import Tune, Voice

CLARINET, TRUMPET, BANJO, TUBA, SNARE = "clarinet", "trumpet", "banjo", "tuba", "snare"
BAR = "z8"

# K:Bb makes B and E flat.
TUNE_BARS = ["F2 G A B3 A", "c2 B A G4", "F2 G A B2 d2", "c6 z2", "d2 c B c2 d2", "e2 d c B4", "A2 B c d2 c2",
             "B3 B5"]
CHORDS = ["Bb", "F", "Bb", "F", "Bb", "Eb", "F", "Bb"]
BANJO_HIT = {"Bb": "[B,DF]3 [B,DF]5", "F": "[A,CF]3 [A,CF]5", "Eb": "[B,EG]3 [B,EG]5"}
TUBA_HIT = {"Bb": "B,,3 F,,5", "F": "F,,3 C,5", "Eb": "E,,3 B,,5"}
STOP = {"Bb": "[B,DF]2 z6", "F": "[A,CF]2 z6", "Eb": "[B,EG]2 z6"}
SNARE_BAR = "z2 D,,2 z2 D,,2"  # acoustic snare (key 38) on the backbeat


def _line(bars, mark=None):
    bars = list(bars)
    if mark:
        bars[0] = f"!{mark}! {bars[0]}"
    return " | ".join(bars) + " |"


SILENT = [BAR] * 8
BANJO_BARS = [BANJO_HIT[c] for c in CHORDS]
TUBA_BARS = [TUBA_HIT[c] for c in CHORDS]
ECHO = [BAR] + TUNE_BARS[:7]


def section(mark, clarinet=SILENT, trumpet=SILENT, banjo=SILENT, tuba=SILENT, snare=SILENT):
    return {CLARINET: _line(clarinet, mark), TRUMPET: _line(trumpet, mark), BANJO: _line(banjo, mark),
            TUBA: _line(tuba, mark), SNARE: _line(snare, mark)}


def thinning(bars, keep):
    """`bars` for the first `keep` bars, then rests."""
    return list(bars[:keep]) + [BAR] * (8 - keep)


TUNE = Tune(
    title="The Talk of the Town",
    slug="the-talk-of-the-town",
    key="Bb",
    beats_per_bar=4,
    voices=(Voice(CLARINET, 71, 105, pan=52), Voice(TRUMPET, 59, 95, pan=84), Voice(BANJO, 105, 90, pan=36),
            Voice(TUBA, 58, 100, pan=60), Voice(SNARE, 0, 80, channel=10)),
    sections={
        "A": section("mf", clarinet=TUNE_BARS, banjo=BANJO_BARS),
        "B": section("f", clarinet=TUNE_BARS, trumpet=ECHO, banjo=BANJO_BARS, tuba=TUBA_BARS, snare=[SNARE_BAR] * 8),
        "C": section("f", clarinet=thinning(TUNE_BARS, 2), trumpet=thinning(ECHO, 2), banjo=thinning(BANJO_BARS, 4),
                     tuba=TUBA_BARS, snare=thinning([SNARE_BAR] * 8, 6)),
        # Stop-time for three bars; then the tune passed along, each echo
        # quieter: clarinet, trumpet, banjo, tuba, then nothing.
        "D": section("f",
                     clarinet=[BAR] * 3 + ["!mf! " + TUNE_BARS[0]] + [BAR] * 4,
                     trumpet=[BAR] * 4 + ["!mp! " + TUNE_BARS[0]] + [BAR] * 3,
                     banjo=[STOP[c] for c in CHORDS[:3]] + [BAR, BAR, "!p! " + BANJO_HIT["Bb"], BAR, BAR],
                     tuba=[STOP[c].replace("[B,DF]", "B,,").replace("[A,CF]", "F,,") for c in CHORDS[:3]]
                     + [BAR, BAR, BAR, "!pp! " + TUBA_HIT["Bb"], BAR],
                     snare=["D,,2 z6"] * 3 + [BAR] * 5),
        "E": section("f", clarinet=TUNE_BARS, trumpet=ECHO, banjo=BANJO_BARS, tuba=TUBA_BARS, snare=[SNARE_BAR] * 8),
    },
    forms=("AABCDE",),
    ending={CLARINET: "B2 z6", TRUMPET: "d2 z6", BANJO: "[B,DFB]2 z6", TUBA: "B,,2 z6", SNARE: "D,,2 z6"},
    bpm=(170, 192, 215),
)
