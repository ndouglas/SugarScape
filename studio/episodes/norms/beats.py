"""Cooperation, episode 6: "Norms" (Axelrod 1986; Galán & Izquierdo 2005).

The board is the plane of temperaments (see plane.py): boldness from left to
right, vengefulness from front to back, each Flump on its square; Galán &
Izquierdo's "established" squares tinted green (back left), "collapsed" red
(front right).

What each shot does, all seed 2 (see measurements.md for the 20-seed claims;
seed 2 is one of the 12 whose norm collapses under Galán & Izquierdo's tie
reading but holds under Axelrod's):
close, metaclose — the norms and metanorms games, 3 generations, with events.
normsrun — the norms game to generation 1,000: cheating falls at first, then
  the norm collapses by 500.
holds — metanorms to generation 100: established by 25, and held.
long — metanorms to 1,000,000 under Galán & Izquierdo's tie reading, every
  2,500th generation: established until about 290,000, then collapsed.
axel — the same under Axelrod's: established throughout.
mild — milder metapunishment to 10,000, every 25th: collapsed by 2,500.
"""

from camera import Move
from episode import Beat

WIDE_EYE, WIDE_AT = (0, -46, 40), (0, -1.5, 0)
LOW_EYE, LOW_AT = (0, -28, 20), (0, 2, 0)
TRACK = ("generation", "mean-traits")


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("traits", "Each Flump has a boldness, and a vengefulness.", 5.0, shot="close", ticks_per_second=0.01,
         camera=(Move(0, 5, LOW_EYE, LOW_AT, WIDE_EYE, WIDE_AT, lens0=35, lens1=35),)),
    Beat("cheat", "A bold Flump cheats when it thinks nobody is looking:\nit gains 3, and everyone else loses 1.", 6.5,
         shot="close", ticks_per_second=0.13, start_tick=1, overlays=("events",),
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("punish", "Anyone who sees it may punish it, if vengeful enough:\n−9 to the cheat, −2 to the punisher.", 6.5,
         shot="close", ticks_per_second=0.13, start_tick=2, overlays=("events",),
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("norms", "Axelrod, 1986: on their own, punishments fade.\nThe cheats take over, in 20 of 20 worlds.", 7.0,
         shot="normsrun", ticks_per_second=150, overlays=TRACK,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("meta", "So he added one rule: punish anyone who looks away.", 6.0, shot="metaclose",
         ticks_per_second=0.17, start_tick=1, overlays=("events",),
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("holds", "The norm holds, in 17 of 20 worlds, as he found.", 7.0, shot="holds", ticks_per_second=15,
         overlays=TRACK, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("long", "In 2005, Galán and Izquierdo ran it a million generations.\nThe norm collapsed, in 18 of 20.", 8.0,
         shot="long", ticks_per_second=52, overlays=TRACK, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("sentence", "But when every Flump earns the same, who has the children?\n"
         "Axelrod: each has one. They: each has two, and half are removed.", 9.0, shot="long", start_tick=400,
         overlays=("generation",), camera=(Move(0, 9, WIDE_EYE, WIDE_AT, LOW_EYE, LOW_AT, lens0=35, lens1=35),)),
    Beat("axelrod", "Read his way, the norm still holds after a million generations,\nin 14 of 20.", 8.0, shot="axel",
         ticks_per_second=52, overlays=TRACK, camera=(Move(0, 8, LOW_EYE, LOW_AT, WIDE_EYE, WIDE_AT, lens0=35, lens1=35),)),
    Beat("mild", "Either way, make metapunishment milder, and the norm collapses.", 7.0, shot="mild",
         ticks_per_second=60, overlays=TRACK, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("point", "Both were right.\nOne sentence decides the long run.", 6.0, shot="axel", start_tick=400,
         caption_y=0.0, title=True,
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -54, 48), WIDE_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Norms — after Axelrod, 1986; Galán & Izquierdo, 2005\nndouglas.github.io/SugarScape", 5.0,
         caption_y=0.45, camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
