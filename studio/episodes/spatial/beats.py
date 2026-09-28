"""Cooperation, episode 1: "Spatial games" (Nowak & May 1992; Huberman &
Glance 1993; Nowak, Bonhoeffer & May 1994).

Board coordinates: one unit per square, square (x, y) at (x − w/2 + 0.5,
h/2 − y − 0.5). Helpers are blue, cheats red; the felt under a Flump that
just switched glows green (turned helper) or yellow (turned cheat).

What each shot does (see measurements.md for the 20-seed claims):
board — a hand-made 7 × 7 board, b = 1.9, fixed edges: a block of cheats in
  the top right and bottom left. In generation 1, 10 helpers turn cheat and
  3 cheats turn helper.
kaleidoscope — nm-3-kaleidoscope: one cheat at the center of 99 × 99; the
  pattern is four-fold symmetric every generation and reaches the edges at
  t = 49 (the same in every seed).
scatter — 99 × 99, 10 % cheats at random, b = 1.9, seed 11 (typical of 20):
  helpers settle near 32 %.
tempt — the same at b = 1.77, seed 8 (typical): about 75 % help.
clock — hg-async-kaleidoscope, one Flump at a time, seed 17 (typical): no
  helper left at t = 97.
below — nbm-continuous on 99 × 99: 50 % cheats, periodic edges, b = 1.71,
  one at a time, seed 8 (typical): about 72 % help.
"""

from camera import Move
from episode import Beat

CLOSE_EYE, CLOSE_AT = (0, -6.8, 9.6), (0, -0.3, 0.3)
TOP_EYE, TOP_AT = (0, -42, 108), (0, 1, 0)
LOW_EYE, LOW_AT = (0, -40, 34), (0, 2, 0)


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("play", "Every Flump plays a game with each of its neighbors.", 5.0, shot="board", closeup=True,
         ticks_per_second=0.01, overlays=("play",), params={"square": (3, 2)},
         camera=hold(CLOSE_EYE, CLOSE_AT, lens=34, drift=(0, 0.3, -0.2))),
    Beat("pay", "Two helpers earn 1 each. A cheat facing a helper earns almost 2,\nand the helper nothing.", 6.0,
         shot="board", closeup=True, ticks_per_second=0.01, overlays=("scores", "play"), params={"square": (3, 2)},
         camera=hold(CLOSE_EYE, CLOSE_AT, lens=36, drift=(0, 0.3, -0.2))),
    Beat("copy", "Then every Flump copies whoever earned the most: itself or a neighbor.", 5.5, shot="board", closeup=True,
         ticks_per_second=0.5, lead_in=1.0, overlays=("scores",), params={"square": (3, 2)},
         camera=hold(CLOSE_EYE, CLOSE_AT, lens=36, drift=(0, 0.2, -0.1))),
    Beat("one", "One cheat, in a world of 9,800 helpers.", 4.5, shot="kaleidoscope", ticks_per_second=0.8,
         lead_in=1.0, camera=(Move(0, 4.5, (0, -7, 11), (0, 0, 0), TOP_EYE, TOP_AT, lens0=35, lens1=35),)),
    Beat("bloom", "Cheating spreads…", 6.0, shot="kaleidoscope", ticks_per_second=8, start_tick=3,
         overlays=("helpers",), camera=hold(TOP_EYE, TOP_AT, lens=35, drift=(0, 1, -1))),
    Beat("never", "…but never wins, and never settles.", 6.0, shot="kaleidoscope", ticks_per_second=35,
         start_tick=51, overlays=("helpers",), camera=hold(TOP_EYE, TOP_AT, lens=36, drift=(0, 1, -1))),
    Beat("third", "Scatter a few cheats anywhere: about a third of the Flumps end up helping.", 6.5, shot="scatter",
         ticks_per_second=25, overlays=("helpers",),
         camera=(Move(0, 6.5, TOP_EYE, TOP_AT, (0, -48, 100), TOP_AT, lens0=35, lens1=37, orbit=0.1),)),
    Beat("why", "On average, helpers earn more than cheats, every generation.", 7.5, shot="scatter",
         ticks_per_second=3, start_tick=162, overlays=("earnings",),
         camera=(Move(0, 7.5, (0, -48, 100), TOP_AT, LOW_EYE, LOW_AT, lens0=37, lens1=45),)),
    Beat("tempt", "Make cheating a little less tempting, and most help.", 6.5, shot="tempt",
         ticks_per_second=30, overlays=("helpers",), camera=hold(TOP_EYE, TOP_AT, lens=35, drift=(0, 1, -1))),
    Beat("clock", "But here, everyone moves at once.\nIn 1993, Huberman and Glance let the Flumps move one at a time.",
         6.5, shot="clock", ticks_per_second=6.3, overlays=("helpers",),
         camera=(Move(0, 6.5, (0, -7, 11), (0, 0, 0), TOP_EYE, TOP_AT, lens0=35, lens1=35),)),
    Beat("lost", "One cheat takes the whole world, in 20 of 20.", 3.8, shot="clock", ticks_per_second=17,
         start_tick=41, overlays=("helpers",), camera=hold(TOP_EYE, TOP_AT, lens=35, drift=(0, 1, -1))),
    Beat("below", "Nowak, Bonhoeffer and May replied in 1994: make cheating\nless tempting, and helpers survive either way.", 7.5, shot="below",
         ticks_per_second=25, overlays=("helpers",), camera=hold(TOP_EYE, TOP_AT, lens=35, drift=(0, 1, -1))),
    Beat("point", "Nobody has to be nice.\nThey just have to be neighbors.", 6.5, shot="below", start_tick=200,
         caption_y=0.0, title=True,
         camera=(Move(0, 6.5, TOP_EYE, TOP_AT, (0, -70, 60), TOP_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Spatial games — after Nowak & May, 1992; Huberman & Glance, 1993;\nNowak, Bonhoeffer & May, 1994\nndouglas.github.io/SugarScape", 5.4, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
