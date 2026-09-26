"""The Pollution episode's tune: "Smoke over the Mill", an original chorale in
the English colliery brass-band tradition — Dickens's coketowns, the Brontës'
mill valleys, the Thames at its foulest. A (clean) is a noble E♭ hymn on
cornet over horns, trombones and tuba; B rises through its middle; M (murk)
sinks the hymn's melody into C minor on the trombones, with the tuba low and
a bell tolling. Stings: a tolled bell as the pollution starts, and again at
the closing question."""

from music import Tune, Voice

CORNET, HORN, TROMBONE, TUBA, BELL = "cornet", "horn", "trombone", "tuba", "bell"
_REST = " | ".join(["z8"] * 8) + " |"

TUNE = Tune(
    title="Smoke over the Mill (chorale)",
    slug="smoke-over-the-mill",
    key="Eb",
    beats_per_bar=4,
    voices=(Voice(CORNET, 56, 100), Voice(HORN, 60, 80), Voice(TROMBONE, 57, 90), Voice(TUBA, 58, 95),
            Voice(BELL, 14, 70)),
    sections={
        "A": {  # clean: E♭ | A♭ E♭ | B♭ | E♭ | Cm | A♭ B♭ | E♭ B♭ | E♭
            CORNET: "G4 B4 | c4 B4 | A2 G2 F4 | G8 | G4 E2 G2 | c4 B2 A2 | G4 F4 | E8 |",
            HORN: "E4 E4 | E4 E4 | D4 D4 | E8 | C4 C4 | C4 D4 | E4 D4 | B,8 |",
            TROMBONE: "B,4 B,4 | C4 B,4 | B,4 A,4 | G,8 | G,4 G,4 | A,4 F,4 | B,4 A,4 | G,8 |",
            TUBA: "E,,4 E,,4 | A,,4 E,,4 | B,,4 B,,4 | E,,8 | C,4 C,4 | A,,4 B,,4 | E,,4 B,,4 | E,,8 |",
            BELL: _REST,
        },
        "B": {  # rising: A♭ | E♭ | Fm B♭ | E♭ | A♭ | E♭ Cm | F7 | B♭
            CORNET: "c4 A4 | B4 G4 | A2 F2 B4 | G8 | c4 e4 | d4 c4 | c4 =A4 | B8 |",
            HORN: "E4 E4 | E4 E4 | C4 D4 | E8 | E4 E4 | B,4 C4 | E4 E4 | D8 |",
            TROMBONE: "A,4 C4 | G,4 B,4 | A,4 F,4 | G,8 | A,4 C4 | G,4 G,4 | =A,4 F,4 | F,8 |",
            TUBA: "A,,4 A,,4 | E,,4 E,,4 | F,,4 B,,4 | E,,8 | A,,4 A,,4 | E,,4 C,4 | F,,4 F,,4 | B,,8 |",
            BELL: _REST,
        },
        "M": {  # murk: Cm | Fm Cm | G | Cm | A♭ | Fm G | Cm G | Cm — the hymn sunk low
            CORNET: _REST,
            HORN: "C4 C4 | C4 C4 | =B,4 =B,4 | C8 | C4 C4 | C4 =B,4 | C4 =B,4 | C8 |",
            TROMBONE: "E,4 G,4 | A,4 G,4 | F,2 E,2 D,4 | E,8 | E,4 C,2 E,2 | A,4 G,2 F,2 | E,4 D,4 | C,8 |",
            TUBA: "C,,4 C,,4 | F,,4 C,,4 | G,,4 G,,4 | C,,8 | A,,4 A,,4 | F,,4 G,,4 | C,,4 G,,4 | C,,8 |",
            BELL: "C4 z4 | z8 | C4 z4 | z8 | C4 z4 | z8 | C4 z4 | z8 |",
        },
    },
    forms=("AM", "AMM", "ABMM", "AMMM", "ABMMM"),
    ending={CORNET: "z8", HORN: "C8", TROMBONE: "[C,G,]8", TUBA: "C,,8", BELL: "C8"},
    bpm=(60, 80, 92),
    stings={
        "onset": ({BELL: "C4 z4", TUBA: "C,,8"}, 60),
        "toll": ({BELL: "C4 z4", TROMBONE: "[C,E,G,]8"}, 60),
    },
    cues=(("starts", "onset"), ("question", "toll")),
)
