"""The War episode's tune: "Poseidon's Horn", an original march in 5/4 in D
minor, after the effect of King Crimson's "The Devil's Triangle" (itself
after Holst's "Mars") — not after either's notes. The ostinato is our own
lopsided figure, long-short-long-long-long (3 + 1 + 2 + 2 + 2 eighths).

A: the long quiet — low strings and timpani, a snare that almost starts —
then trombones, snare and fife join, building. B: after the horn — a drone
and a lone timpani, the march gone. C: the war that never ends — the
ostinato again, grinding against a trombone semitone, unresolved; it ends
on an open fifth.

Sting: the horn. One enormous chord, swelling on every brass patch, choir
and organ over a timpani roll, as the warlord rises; the tune ducks beneath
it (`ducks`), and trombones slide down through it as the march collapses.
The horn has its own tracks, so GarageBand can give it a bigger patch than
FluidR3's General MIDI brass."""

from music import Tune, Voice

STRINGS, TIMPANI, SNARE, TROMBONE, TUBA, FIFE = "strings", "timpani", "snare", "trombone", "tuba", "fife"
BRASS, HORNS, SYNTH, CHOIR, ORGAN = "horn: brass", "horn: french horns", "horn: synth brass", "horn: choir", "horn: organ"
HORN = (BRASS, HORNS, SYNTH, CHOIR, ORGAN)

BAR = "z10"
OSTINATO = "D,3 D, D,2 D,2 A,,2"  # our own: long-short-long-long-long
GRIND = "D,3 D, _E,2 D,2 A,,2"  # the same, with the semitone that won't resolve
TIMP = "D,,2 z4 A,,2 z2"
SNARE_PICKUP = "z8 D,,2"  # a snare that almost starts
SNARE_MARCH = "D,,3 D,, D,,2 D,,2 D,,2"


def _bars(bars):
    return " | ".join(bars) + " |"


def _rest(n=8):
    return _bars([BAR] * n)


TUNE = Tune(
    title="Poseidon's Horn",
    slug="poseidons-horn",
    key="Dm",
    beats_per_bar=5,
    voices=(Voice(STRINGS, 48, 100), Voice(TIMPANI, 47, 110), Voice(SNARE, 0, 90, channel=10),
            Voice(TROMBONE, 57, 100), Voice(TUBA, 58, 100), Voice(FIFE, 72, 85),
            Voice(BRASS, 61, 127), Voice(HORNS, 60, 127), Voice(SYNTH, 62, 110), Voice(CHOIR, 52, 120),
            Voice(ORGAN, 19, 110)),
    sections={
        "A": {  # the quiet, then the build
            STRINGS: _bars([f"!p! {OSTINATO}", OSTINATO, OSTINATO, OSTINATO,
                            f"!mp! {OSTINATO}", OSTINATO, f"!f! {OSTINATO}", OSTINATO]),
            TIMPANI: _bars([f"!p! {TIMP}", TIMP, TIMP, TIMP, f"!mf! {TIMP}", TIMP, f"!ff! {TIMP}", TIMP]),
            SNARE: _bars([BAR, BAR, BAR, f"!pp! {SNARE_PICKUP}", BAR, f"!mp! {SNARE_PICKUP}",
                          f"!f! {SNARE_MARCH}", SNARE_MARCH]),
            TROMBONE: _bars([BAR, BAR, BAR, BAR, "!mp! D,4 F,2 E,2 D,2", "E,4 G,2 F,2 E,2",
                             "!f! F,4 A,2 G,2 F,2", "A,4 _B,2 A,2 G,2"]),
            TUBA: _bars([BAR, BAR, BAR, BAR, "!mp! D,,10", "D,,10", "!f! D,,10", "A,,,10"]),
            FIFE: _bars([BAR] * 6 + ["!mf! d3 d e2 f2 a2", "g3 f e2 d2 A2"]),
            **{v: _rest() for v in HORN},
        },
        "B": {  # after the horn: a drone and a lone drum; the march is gone
            STRINGS: _bars(["!mf! [D,A,]10", "[D,A,]10", "!p! [D,A,]10", "[D,A,]10",
                            "!pp! [D,A,]10", "[D,A,]10", "[D,A,]10", "[D,A,]10"]),
            TIMPANI: _bars(["!mp! D,,2 z8", BAR, "D,,2 z8", BAR, "!p! D,,2 z8", BAR, "D,,2 z8", BAR]),
            SNARE: _rest(),
            TROMBONE: _rest(),
            TUBA: _bars(["!p! D,,10", "D,,10", "D,,10", "D,,10", "!pp! D,,10", "D,,10", "D,,10", "D,,10"]),
            FIFE: _rest(),
            **{v: _rest() for v in HORN},
        },
        "C": {  # the war that never ends: grinding, unresolved
            STRINGS: _bars([f"!mf! {GRIND}", GRIND, GRIND, GRIND, f"!f! {GRIND}", GRIND, GRIND, GRIND]),
            TIMPANI: _bars([f"!mf! {TIMP}"] + [TIMP] * 7),
            SNARE: _bars([f"!mf! {SNARE_MARCH}"] + [SNARE_MARCH] * 7),
            TROMBONE: _bars(["!mf! [D,_E,]10", "[D,_E,]10", "[F,_G,]10", "[F,_G,]10",
                             "!f! [D,_E,]10", "[D,_E,]10", "[A,,_B,,]10", "[A,,_B,,]10"]),
            TUBA: _bars(["!mf! D,,10", "D,,10", "D,,10", "D,,10", "D,,10", "D,,10", "A,,,10", "A,,,10"]),
            FIFE: _rest(),
            **{v: _rest() for v in HORN},
        },
    },
    # The story wants the build to crest as the warlord rises (about 26 s in),
    # the aftermath under the bars, and the endless war last: one form.
    forms=("ABC",),
    ending={STRINGS: "!mf! [D,A,]10", TIMPANI: "D,,10", SNARE: BAR, TROMBONE: "[D,A,]10", TUBA: "D,,10",
            FIFE: BAR, **{v: BAR for v in HORN}},
    bpm=(72, 84, 100),
    stings={
        # Two bars: the chord swells from nothing, re-struck louder and louder,
        # then holds at full strength; beneath, the trombones slide down and
        # the timpani rolls.
        "horn": ({
            BRASS: "!pp! [D,A,DA]2 !p! [D,A,DA]2 !mp! [D,A,DA]2 !mf! [D,A,DA]2 !f! [D,A,DA]2 | !fff! [D,A,DA]10",
            HORNS: "!pp! [A,DF]2 !p! [A,DF]2 !mp! [A,DF]2 !mf! [A,DF]2 !f! [A,DF]2 | !fff! [A,DF]10",
            SYNTH: "!pp! [D,A,D]2 !p! [D,A,D]2 !mp! [D,A,D]2 !mf! [D,A,D]2 !f! [D,A,D]2 | !fff! [D,A,D]10",
            CHOIR: "!pp! [DFA]2 !p! [DFA]2 !mp! [DFA]2 !mf! [DFA]2 !f! [DFA]2 | !fff! [DFAd]10",
            ORGAN: "!mp! [D,,,D,,]10 | !fff! [D,,,D,,A,,]10",
            TIMPANI: "!pp! D,, D,, D,, D,, !mp! D,, D,, !mf! D,, D,, !f! D,, D,, | !fff! D,, D,, D,, D,, D,, D,, D,, D,, D,, D,,",
            TROMBONE: "!f! A,2 _A,2 G,2 _G,2 F,2 | !p! E,2 _E,2 D,6",
        }, 66),
    },
    ducks={"horn": 0.15},
    # The horn swells as Blue 63 begins its rampage: its first kill is at tick
    # 605, about 0.3 s into "warlord".
    cues=(("warlord", "horn", 0.2),),
)
