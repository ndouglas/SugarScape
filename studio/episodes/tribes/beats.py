"""Episode 5, "Tribes" (Animations III-6/III-7).

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).
The book's two tribes: Blue when a Flump's eleven tags hold more zeros than
ones, else Red.

What each shot does (from its dump; see measurements.md for the world beats):
rub — a Blue Flump among three Reds on a small, sugar-rich board: its tags
  flip at ticks 2 and 7 (01001000100 → …), and at tick 10 it turns Red
  (01011001101). The influence runs both ways: a Red briefly turns Blue too.
world — iii-6-culture, seed 7 (typical of 20): 50/50 and mixed at the start
  (neighbors alike 56%); Blue holds 95% and both hills by tick 3000.
divided — seed 9: at tick 3000 the north-east hill is all Red, the
  south-west 99% Blue; the world splits 52/48.
calm — seed 7 with culture off, for the population comparison.
"""

from camera import Move
from episode import Beat

CLOSE_EYE, CLOSE_AT = (0.5, -6.4, 4.0), (0.5, -0.5, 1.2)
# The whole 8 × 8 board, so the family stays in frame as it wanders (the board wraps).
BOARD_EYE, BOARD_AT = (0, -9.5, 8.0), (0, -0.3, 0.4)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)
TRIBE = {"colors": "tribe"}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("culture", "Every Flump carries a little culture: eleven traits.", 5.0, shot="rub", closeup=True,
         ticks_per_second=0.3, lead_in=0.6, focus=(0,), overlays=("traits",), params=TRIBE,
         camera=hold(CLOSE_EYE, CLOSE_AT, lens=40)),
    Beat("rule", "More zeros: Blue. More ones: Red.", 4.5, shot="rub", closeup=True, ticks_per_second=0.15,
         start_tick=1, focus=(0,), overlays=("traits",), params=TRIBE, camera=hold(CLOSE_EYE, CLOSE_AT, lens=42)),
    Beat("rub", "Neighbors rub off on each other: one trait flips to match.", 5.0, shot="rub", closeup=True,
         ticks_per_second=0.8, start_tick=1, focus=(0,), overlays=("traits",), params=TRIBE,
         camera=hold(BOARD_EYE, BOARD_AT, lens=34)),
    Beat("turn", "Flip enough, and a Flump changes tribe.", 6.0, shot="rub", closeup=True, ticks_per_second=1.0,
         start_tick=5, focus=(0,), overlays=("traits",), params=TRIBE, camera=hold(BOARD_EYE, BOARD_AT, lens=36)),
    Beat("mixed", "400 Flumps, half Blue, half Red, all mixed up.", 5.0, shot="world", ticks_per_second=4,
         lead_in=0.6, overlays=("alike",), params=TRIBE,
         camera=(Move(0, 5, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.2),)),
    Beat("converge", "In most worlds, every neighborhood ends up one color…", 8.0, shot="world",
         ticks_per_second=120, start_tick=20, overlays=("alike",), params=TRIBE,
         camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 3, -1.5))),
    Beat("hills", "…and each hill becomes one tribe.", 8.0, shot="world", ticks_per_second=250, start_tick=980,
         overlays=("alike",), params=TRIBE, camera=(Move(0, 8, WIDE_EYE, WIDE_AT, (4, -52, 30), WIDE_AT, orbit=0.25),)),
    Beat("united", "In this world, Blue took everything.", 5.0, shot="world", start_tick=3000,
         overlays=("alike",), params=TRIBE, camera=hold(WIDE_EYE, WIDE_AT, lens=36)),
    Beat("divided", "In this one, each hill kept its own.", 6.0, shot="divided", start_tick=3000,
         overlays=("alike",), params=TRIBE, camera=hold(WIDE_EYE, WIDE_AT, lens=36)),
    Beat("half", "The book expects one tribe to win. In our runs, that happens half the time.", 8.0, shot="divided",
         start_tick=3000, overlays=("bars",),
         params={**TRIBE, "title": "in 20 worlds",
                 "rows": [[("one tribe takes the world", "one_tribe_seeds"), ("the hills split", "split_seeds")],
                          [("each hill one tribe", "hills_one_tribe")]]},
         camera=(Move(0, 8, (0, -60, 34), (-8, 0, 0), (-2, -58, 33), (-8, 0, 0)),)),
    Beat("nothing", "Being Red or Blue changes nothing else. Just as many Flumps live either way.", 7.0,
         shot="divided", start_tick=3000, overlays=("bars",),
         params={**TRIBE, "title": "Flumps alive at tick 3000", "format": "num", "top": 250,
                 "rows": [[("with tribes", "pop"), ("without", "pop_calm")]]},
         camera=(Move(0, 7, (0, -60, 34), (-8, 0, 0), (-2, -58, 33), (-8, 0, 0)),)),
    Beat("question", "Nobody chose a side.\nTheir neighbors chose for them.", 6.0, shot="divided", start_tick=3000,
         caption_y=0.0, title=True, params=TRIBE,
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "Tribes — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
