"""The War episode's tune: "Poseidon's Horn", an original march in 5/4 in D
minor, after the effect of King Crimson's "The Devil's Triangle" (itself
after Holst's "Mars") — not after either's notes. The rhythm is our own: a
triplet, a quarter, two eighths, a triplet, a quarter (2 + 2 + 2 + 2 + 2
eighths), played by timpani, snare, strings and brass stabs.

One build across the whole film, under a drone that never stops. A (the
close-ups and the long quiet): timpani and snare out front, strings on the
figure. B (from the warlord on): the march — bass drum, snare, brass stabs on
the triplets, trombones climbing. C (the endless war): fife and tuba join,
louder and louder.

Sting: the horn, on the last title ("Nobody fights an equal…"). One enormous
chord swelling across every brass patch, choir and organ over a timpani and
snare roll and a crash; the march ducks beneath it (`ducks`) and collapses,
the trombones sliding down through it, into the end card. The horn has its
own tracks, so GarageBand can give it a bigger patch than FluidR3's."""

from music import Tune, Voice

DRONE, STRINGS, TIMPANI, SNARE, KICK, CRASH = "drone", "strings", "timpani", "snare", "bass drum", "crash"
TROMBONE, TUBA, FIFE = "trombone", "tuba", "fife"
BRASS, HORNS, SYNTH, CHOIR, ORGAN = "horn: brass", "horn: french horns", "horn: synth brass", "horn: choir", "horn: organ"

BAR = "z10"


def rhythm(note):
    """Our figure on one note: triplet, quarter, two eighths, triplet, quarter."""
    return f"(3{note}{note}{note} {note}2 {note}{note} (3{note}{note}{note} {note}2"


def triplets(note):
    """Only the figure's two triplets: the snare that almost starts."""
    return f"(3{note}{note}{note} z2 z2 (3{note}{note}{note} z2"


# General MIDI drums (channel 10): bass drum C,, (36), snare D,, (38), crash ^C, (49).
SNARE_N, KICK_N, CRASH_N = "D,,", "C,,", "^C,"
KICK_BAR = f"{KICK_N}2 z2 {KICK_N}2 z2 {KICK_N}2"
STAB = "[D,A,D]"


def _bars(bars):
    return " | ".join(bars) + " |"


def _rest(n=8):
    return _bars([BAR] * n)


def _louder(steps, bar):
    """8 bars of `bar`, each marked with its dynamic from `steps` (None keeps it)."""
    return _bars([f"!{s}! {bar}" if s else bar for s in steps])


TUNE = Tune(
    title="Poseidon's Horn",
    slug="poseidons-horn",
    key="Dm",
    beats_per_bar=5,
    voices=(Voice(DRONE, 49, 85), Voice(STRINGS, 48, 100), Voice(TIMPANI, 47, 127),
            Voice(SNARE, 0, 120, channel=10), Voice(KICK, 0, 127, channel=10), Voice(CRASH, 0, 110, channel=10),
            Voice(TROMBONE, 57, 105), Voice(TUBA, 58, 105), Voice(FIFE, 72, 90),
            Voice(BRASS, 61, 127), Voice(HORNS, 60, 127), Voice(SYNTH, 62, 110), Voice(CHOIR, 52, 120),
            Voice(ORGAN, 19, 110)),
    sections={
        "A": {  # the close-ups and the quiet: percussion out front, the figure beneath
            DRONE: _louder(["p"] + [None] * 7, "[D,A,]10"),
            STRINGS: _louder(["p", None, None, None, "mp", None, None, None], rhythm("D,")),
            TIMPANI: _louder(["mp", None, None, None, "mf", None, None, None], rhythm("D,,")),
            SNARE: _bars(["!p! " + triplets(SNARE_N), BAR, triplets(SNARE_N), BAR,
                          "!mp! " + triplets(SNARE_N), triplets(SNARE_N), "!mf! " + rhythm(SNARE_N), rhythm(SNARE_N)]),
            KICK: _rest(), CRASH: _rest(), TROMBONE: _rest(), TUBA: _rest(), FIFE: _rest(),
            BRASS: _rest(), HORNS: _rest(), SYNTH: _rest(), CHOIR: _rest(), ORGAN: _rest(),
        },
        "B": {  # the march: the warlord's
            DRONE: _louder(["mp"] + [None] * 7, "[D,A,]10"),
            STRINGS: _louder(["mf"] + [None] * 7, rhythm("D,")),
            TIMPANI: _louder(["f"] + [None] * 7, rhythm("D,,")),
            SNARE: _louder(["f"] + [None] * 7, rhythm(SNARE_N)),
            KICK: _louder(["f"] + [None] * 7, KICK_BAR),
            CRASH: _bars([f"!mf! {CRASH_N}10"] + [BAR] * 7),
            TROMBONE: _bars(["!mf! D,4 F,2 E,2 D,2", "E,4 G,2 F,2 E,2", "F,4 A,2 G,2 F,2", "A,4 _B,2 A,2 G,2",
                             "!f! D,4 F,2 E,2 D,2", "E,4 G,2 F,2 E,2", "F,4 A,2 G,2 F,2", "A,4 _B,2 A,2 G,2"]),
            TUBA: _rest(), FIFE: _rest(),
            # The horns' percussive side: stabs on the figure's triplets.
            BRASS: _louder(["mf", None, None, None, "f", None, None, None], triplets(STAB)),
            HORNS: _rest(), SYNTH: _rest(), CHOIR: _rest(), ORGAN: _rest(),
        },
        "C": {  # the war that never ends: everything, louder and louder
            DRONE: _louder(["mf", None, None, None, "f", None, None, None], "[D,A,]10"),
            STRINGS: _louder(["f", None, None, None, "ff", None, None, None], rhythm("D,")),
            TIMPANI: _louder(["f", None, None, None, "ff", None, "fff", None], rhythm("D,,")),
            SNARE: _louder(["f", None, None, None, "ff", None, "fff", None], rhythm(SNARE_N)),
            KICK: _louder(["f", None, None, None, "ff", None, "fff", None], KICK_BAR),
            CRASH: _bars([f"!f! {CRASH_N}10", BAR, BAR, BAR, f"!ff! {CRASH_N}10", BAR, f"!fff! {CRASH_N}10", BAR]),
            TROMBONE: _bars(["!f! [D,_E,]10", "[D,_E,]10", "[F,_G,]10", "[F,_G,]10",
                             "!ff! [D,_E,]10", "[D,_E,]10", "[A,,_B,,]10", "[A,,_B,,]10"]),
            TUBA: _louder(["f", None, None, None, "ff", None, None, None], rhythm("D,,")),
            FIFE: _bars(["!f! d3 d e2 f2 a2", "g3 f e2 d2 A2", "d3 d e2 f2 a2", "_b3 a g2 f2 e2",
                         "!ff! d3 d e2 f2 a2", "g3 f e2 d2 A2", "d3 d e2 f2 a2", "_b3 a g2 f2 e2"]),
            BRASS: _louder(["f", None, None, None, "ff", None, "fff", None], rhythm(STAB)),
            HORNS: _louder(["f", None, None, None, "ff", None, "fff", None], triplets("[A,DF]")),
            SYNTH: _rest(), CHOIR: _rest(), ORGAN: _rest(),
        },
    },
    # One form: the build must reach the last title, where the horn comes in.
    forms=("ABC",),
    ending={DRONE: "!mf! [D,A,]10", STRINGS: "[D,A,]10", TIMPANI: "D,,10", SNARE: BAR, KICK: BAR, CRASH: BAR,
            TROMBONE: "[D,A,]10", TUBA: "D,,10", FIFE: BAR, BRASS: BAR, HORNS: BAR, SYNTH: BAR, CHOIR: BAR,
            ORGAN: BAR},
    bpm=(72, 84, 100),
    stings={
        # Two bars: the chord swells from nothing, re-struck louder and louder
        # on the figure's pulse, then holds at full strength over rolls and a
        # crash; beneath, the trombones slide down as the march collapses.
        "horn": ({
            BRASS: "!pp! [D,A,DA]2 !p! [D,A,DA]2 !mp! [D,A,DA]2 !mf! [D,A,DA]2 !f! [D,A,DA]2 | !fff! [D,A,DA]10",
            HORNS: "!pp! [A,DF]2 !p! [A,DF]2 !mp! [A,DF]2 !mf! [A,DF]2 !f! [A,DF]2 | !fff! [A,DF]10",
            SYNTH: "!pp! [D,A,D]2 !p! [D,A,D]2 !mp! [D,A,D]2 !mf! [D,A,D]2 !f! [D,A,D]2 | !fff! [D,A,D]10",
            CHOIR: "!pp! [DFA]2 !p! [DFA]2 !mp! [DFA]2 !mf! [DFA]2 !f! [DFA]2 | !fff! [DFAd]10",
            ORGAN: "!mp! [D,,,D,,]10 | !fff! [D,,,D,,A,,]10",
            TIMPANI: "!p! (3D,,D,,D,, (3D,,D,,D,, !mf! (3D,,D,,D,, (3D,,D,,D,, !f! (3D,,D,,D,, | "
                     "!fff! (3D,,D,,D,, (3D,,D,,D,, (3D,,D,,D,, (3D,,D,,D,, D,,2",
            SNARE: "!p! D,, D,, D,, D,, !mf! D,, D,, D,, D,, !f! D,, D,, | !fff! D,, D,, D,, D,, D,, D,, z4",
            CRASH: f"z10 | !fff! {CRASH_N}10",
            TROMBONE: "!f! A,2 _A,2 G,2 _G,2 F,2 | !mf! E,2 _E,2 D,6",
        }, 66),
    },
    ducks={"horn": 0.2},
    # The horn's swell (its first bar, 4.5 s) rises under the end of
    # "nofront", so its full-strength hold fills the last title, "Nobody
    # fights an equal…".
    cues=(("nofront", "horn", 4.2),),
)
