"""Cooperation, episode 7: "Friends and strangers" (Cohen, Riolo & Axelrod
2001).

The board is a 16 × 16 grid of Flumps colored by p, how readily each helps
after being helped (see grid.py): red wary, green friendly. On the torus
each stands on its own square, so its partners are next door; otherwise the
grid is just the agents in order. Yarn lines show six followed Flumps'
partners this period.

What each shot does (see measurements.md for the 20-seed claims; each seed
typical of 20 for its structure's payoff):
intro — cra-2dk, seed 6, 3 periods.
strangers — cra-rwr, seed 20: mean payoff near 1.1 throughout.
neighbors — cra-2dk, seed 6: 2.46 by period 100, about 2.55 after.
friends — cra-frn, seed 15: 1.44 at 100, then about 2.5.
tenth — cra-ffr-01, seed 12: about 2.4 from period 100.
third — cra-ffr-03, seed 9: low to 300, high by 400, dipping at times.
half — cra-ffr-05, seed 5: about 2.2 to period 250, collapsed by 300.
"""

from camera import Move
from episode import Beat

WIDE_EYE, WIDE_AT = (0, -24, 26), (0, 0.5, 0)
LOW_EYE, LOW_AT = (0, -15, 11), (0, 1, 0)
FOLLOW = {"follow": [17, 60, 119, 150, 200, 238]}
PLAY = ("ties", "payoff")


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("habits", "256 Flumps, each with three habits: how often it helps first,\n"
         "after being helped, and after being cheated.", 7.5, shot="intro", ticks_per_second=0.01,
         camera=(Move(0, 7.5, LOW_EYE, LOW_AT, WIDE_EYE, WIDE_AT, lens0=35, lens1=35),)),
    Beat("games", "Each period, each plays four rounds with four partners.", 5.5, shot="intro", ticks_per_second=0.3,
         start_tick=1, overlays=("ties",), params=FOLLOW, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("copy", "Then each copies its best-scoring partner, if it did better.", 5.5, shot="intro",
         ticks_per_second=0.5, start_tick=1, overlays=("ties",), params=FOLLOW,
         camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 1, -1))),
    Beat("strangers", "Meet strangers every period, and cooperation almost never takes hold.", 7.0, shot="strangers",
         ticks_per_second=40, overlays=PLAY, params=FOLLOW, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("neighbors", "Keep the same four neighbors,\nand it takes hold in every world, and stays.", 7.0,
         shot="neighbors", ticks_per_second=55, overlays=PLAY, params=FOLLOW,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("friends", "Keep the same partners, scattered anywhere: nearly as good.\nNo geography needed.", 7.5,
         shot="friends", ticks_per_second=52, overlays=PLAY, params=FOLLOW,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("tenth", "Swap a tenth of them each period, and it mostly holds.", 5.5, shot="tenth", ticks_per_second=70,
         overlays=PLAY, params=FOLLOW, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("third", "Swap 30%, and it holds only about a third of the time.", 6.5, shot="third", ticks_per_second=220,
         overlays=PLAY, params=FOLLOW, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("half", "Half, and it collapses.", 6.0, shot="half", ticks_per_second=95, overlays=PLAY, params=FOLLOW,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("paper", "That's what Cohen, Riolo and Axelrod found in 2001, row by row.", 8.0, shot="neighbors",
         start_tick=400, overlays=("bars",),
         params={"title": "average payoff, of 3", "format": "num", "digits": 2, "top": 3, "y": 0.2,
                 "rows": [[("strangers, the paper", "paper_rwr"), ("strangers, here", "pay_rwr")],
                          [("neighbors, the paper", "paper_2dk"), ("neighbors, here", "pay_2dk")],
                          [("same partners, the paper", "paper_frn"), ("same partners, here", "pay_frn")],
                          [("half swapped, the paper", "paper_ffr5"), ("half swapped, here", "pay_ffr5")]]},
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("point", "Cooperation doesn't need neighbors.\nIt needs the same faces.", 6.0, shot="friends", start_tick=400,
         caption_y=0.0, title=True,
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -32, 34), WIDE_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Friends and strangers — after Cohen, Riolo & Axelrod, 2001\nndouglas.github.io/SugarScape", 5.0,
         caption_y=0.45, camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
