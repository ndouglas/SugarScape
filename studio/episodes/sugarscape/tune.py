"""The pilot's tune: "Flumps' Walk", an original gånglåt (a Swedish walking
tune) in D major — accordion melody, a fiddle's open D and A as a drone (the
nyckelharpa's hum), and nylon guitar oom-pah."""

from music import Tune, Voice

_D, _G = "D,2 [DFA]2 A,,2 [DFA]2", "G,,2 [GBd]2 D,2 [GBd]2"
_A, _BM = "A,,2 [Ace]2 E,2 [Ace]2", "B,,2 [Bdf]2 F,2 [Bdf]2"
_EM_A = "E,2 [EGB]2 A,,2 [Ace]2"
_DRONE = " | ".join(["[DA]8"] * 8) + " |"

TUNE = Tune(
    title="Flumps' Walk (gånglåt)",
    slug="flumps-walk",
    key="D",
    beats_per_bar=4,
    voices=(Voice("accordion", 21, 110), Voice("fiddle drone", 110, 55), Voice("guitar", 24, 85)),
    sections={
        "A": {
            "accordion": "A2 FA d2 AF | G2 FE D2 FA | B2 GB d2 BG | A2 ce a2 gf | e2 dc d2 AF | B2 dB f2 dB | e2 dc B2 A2 | d2 f2 d4 |",
            "fiddle drone": _DRONE,
            "guitar": " | ".join([_D, _D, _G, _A, _D, _BM, _EM_A, _D]) + " |",
        },
        "B": {
            "accordion": "B2 dg b2 ag | f2 ed f2 a2 | e2 ce a2 ge | f2 df a4 | g2 bg e2 dB | A2 FA d2 fd | e2 fe dc BA | d2 A2 D4 |",
            "fiddle drone": _DRONE,
            "guitar": " | ".join([_G, _D, _A, _D, _G, _D, _EM_A, _D]) + " |",
        },
    },
    forms=("AA", "ABA", "ABBA", "AABA", "AABBA", "AABBAA", "AABBAABA", "AABBAABBA"),
    ending={"accordion": "d8", "fiddle drone": "[DA]8", "guitar": "[D,A,D]8"},
    bpm=(92, 104, 116),
)
