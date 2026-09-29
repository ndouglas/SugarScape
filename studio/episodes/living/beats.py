"""Cooperation, episode 2: "Living neighbors" (Epstein 1998, the demographic
Prisoner's Dilemma).

Board coordinates: one unit per square, square (x, y) at (x − w/2 + 0.5,
h/2 − y − 0.5); the board wraps around. Helpers are blue, cheats red
(colors="strategy"); a Flump's wealth stands in its dump's sugar.

What each shot does (see measurements.md for the 20-seed claims):
closeup — 8 × 8, 14 Flumps, Run 1's rules, seed 45: in cycle 1, helpers 5
  and 14 clone themselves and cheat 4 goes broke. The followed Flumps are
  helper 14 and cheat 4 (focus 13 and 3: founders' ids start at 1).
land — dpd-run-1, seed 8 (typical of 20): the board is full by cycle 34;
  729 helpers to 171 cheats at cycle 500.
soup — dpd-soup, seed 10 (typical): the last helper is gone at cycle 8.
low — dpd-run-4 (R = 1), seed 18 (typical): nobody left at cycle 243.
"""

from camera import Move
from episode import Beat

CLOSE_EYE, CLOSE_AT = (0, -9.2, 10.4), (0, -0.4, 0.2)
WIDE_EYE, WIDE_AT = (0, -30, 30), (0, 1, 0)
TOP_EYE, TOP_AT = (0, -16, 38), (0, 1, 0)
FOCUS = (13, 3)


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("walk", "Now the Flumps walk. Each turn, a Flump steps to an empty square\nand plays each neighbor.", 5.5,
         shot="closeup", closeup=True, ticks_per_second=0.25, lead_in=0.5, params={"colors": "strategy"},
         camera=hold(CLOSE_EYE, CLOSE_AT, lens=34, drift=(0, 0.3, -0.2))),
    Beat("pay", "Two helpers earn 5 each. A cheat takes 6 from a helper.\nTwo cheats lose 5 each.", 6.0,
         shot="closeup", closeup=True, ticks_per_second=0.25, lead_in=0.5, focus=FOCUS, overlays=("labels",),
         params={"colors": "strategy", "label": "wealth"}, camera=hold(CLOSE_EYE, CLOSE_AT, lens=38)),
    Beat("clone", "Reach 11, and a Flump splits in two. Go broke, and it's gone.", 5.5, shot="closeup",
         closeup=True, ticks_per_second=0.5, start_tick=1, focus=FOCUS, overlays=("labels",),
         params={"colors": "strategy", "label": "wealth"}, camera=hold(CLOSE_EYE, CLOSE_AT, lens=38)),
    Beat("start", "Start with 100 Flumps: about half helpers, half cheats.", 4.5, shot="land", ticks_per_second=0.3,
         lead_in=1.0, overlays=("tally",), params={"colors": "strategy"},
         camera=(Move(0, 4.5, (0, -12, 12), (0, 0, 0), WIDE_EYE, WIDE_AT, lens0=35, lens1=35),)),
    Beat("fill", "The land fills up, mostly with helpers: about 730 to 170.", 7.5, shot="land", ticks_per_second=9,
         start_tick=1, overlays=("tally",), params={"colors": "strategy"},
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("book", "Epstein counted 779 helpers. His published rules give about 730.\n"
         "His working paper's rule, one game a turn, comes close\nif the first Flumps start with nothing.", 10.0, shot="land", ticks_per_second=45, start_tick=68,
         overlays=("bars",),
         caption_y=-0.72,
         params={"colors": "strategy", "title": "helpers at cycle 500", "format": "num", "top": 900, "y": 0.26,
                 "rows": [[("Epstein's count", "epstein")], [("published rules", "helpers")],
                          [("working paper, no wealth", "working")]]},
         camera=hold(TOP_EYE, TOP_AT, lens=35, drift=(0, 1, -1))),
    Beat("soup", "Now let any Flump meet any other, anywhere.", 6.0, shot="soup", ticks_per_second=0.6, lead_in=0.8,
         overlays=("tally",), params={"colors": "strategy"},
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("gone", "The last helper is gone within 12 cycles, in 19 of 20 worlds.", 6.0, shot="soup",
         ticks_per_second=1.6, start_tick=3, overlays=("tally",), params={"colors": "strategy"},
         camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 1, -1))),
    Beat("ruin", "Then the cheats ruin each other. By cycle 500, one Flump is left, or none.", 7.0, shot="soup",
         ticks_per_second=70, start_tick=13, overlays=("tally",), params={"colors": "strategy"},
         camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 1, -1))),
    Beat("low", "Now give Flumps 100-cycle lives, and make helping pay less.\n"
         "Epstein saw long booms and busts, and sometimes extinction.\nHere, 19 of 20 worlds die out.", 7.5,
         caption_y=-0.72, shot="low", ticks_per_second=36, overlays=("tally",),
         params={"colors": "strategy"}, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("point", "Mixed with strangers, helpers vanish.\nAmong neighbors, they fill the world.", 6.0, shot="land",
         start_tick=500, caption_y=0.0, title=True, params={"colors": "strategy"},
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -40, 34), WIDE_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Living neighbors — after Epstein, 1998\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
