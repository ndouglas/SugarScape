"""Cooperation, episode 4: "Tags" (Riolo, Cohen & Axelrod 2001; Edmonds &
Hales 2003).

The board is a ring (see ring.py): a Flump stands where its tag names, tags
running clockwise from 0 to 1 with a gap at the top, in that shade's yarn; twins crowd in a pile
outside the ring; a wheel of shades lies just inside it.

What each shot does (see measurements.md for the 20-seed claims):
early — rca-published, seed 6, 8 generations, with gifts: generation 0 is
  100 random shades and tolerances.
long — rca-published, seed 6 (typical of 20), 3,400 generations: a crowd
  by generation 100; at 700–740 84–87 Flumps share tag 0.3523 (lower right,
  at (10.6, −9.2) on the ring), places 0, 1 and 3 with tolerances near 0.01; takeovers
  at generations 987, 1077, 1429, 1559, 1643, 2229, 2456, 2756 and 3330.
strict — rca-strict, seed 2 (typical): giving collapses by generation 50.
clones — eh-clones-only, seed 1 (typical): giving climbs to about 75 % by 75.
"""

from camera import Move
from episode import Beat

RING_EYE, RING_AT = (0, -38, 62), (0, 2.5, 0)
NEAR_EYE, NEAR_AT = (0, -22, 30), (0, 0, 0)
CROWD_EYE, CROWD_AT = (8, -26, 17), (11.5, -11, 0)
GIFT_RATE = {"stat": "donation_rate", "label": "of meetings end in a gift"}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("ring", "No land this time: 100 Flumps, each with a shade, and a tolerance.", 5.5, shot="early",
         ticks_per_second=0.01, camera=(Move(0, 5.5, NEAR_EYE, NEAR_AT, RING_EYE, RING_AT, lens0=35, lens1=35),)),
    Beat("help", "A Flump meets three others, and helps any whose shade\nis within its tolerance of its own.", 6.5,
         shot="early", ticks_per_second=0.2, start_tick=1, lead_in=0.3, overlays=("gifts",),
         camera=hold(RING_EYE, RING_AT, lens=35, drift=(0, 1, -1))),
    Beat("pay", "Helping costs 0.1. Being helped earns 1.", 5.0, shot="early", ticks_per_second=0.2, start_tick=1,
         overlays=("gifts",), camera=hold(RING_EYE, RING_AT, lens=36, drift=(0, 1, -1))),
    Beat("copy", "Then each faces a random other,\nand the higher score has the child.", 6.0, shot="early",
         ticks_per_second=0.4, start_tick=2, lead_in=0.3, camera=hold(RING_EYE, RING_AT, lens=35, drift=(0, 1, -1))),
    Beat("works", "It works: about 74% of meetings end in a gift, as the paper says.", 8.0, shot="long",
         ticks_per_second=60, overlays=("rate",), params=GIFT_RATE,
         camera=hold(RING_EYE, RING_AT, lens=35, drift=(0, 1, -1))),
    Beat("twins", "The paper saw it too: most Flumps share one exact shade.", 7.0, shot="long",
         ticks_per_second=3, start_tick=700, overlays=("rate",),
         params={"stat": "cluster_share", "label": "stand in the biggest crowd"},
         camera=hold(RING_EYE, RING_AT, lens=35, drift=(0, 1, -1))),
    Beat("narrow", "Their tolerance is tiny. They help almost no one but their twins.", 6.5, shot="long",
         ticks_per_second=1, start_tick=721, overlays=("tolerance",), params={"places": [0, 1, 3]},
         camera=(Move(0, 6.5, RING_EYE, RING_AT, CROWD_EYE, CROWD_AT, lens0=35, lens1=42),)),
    Beat("cycle", "Every few hundred generations, a new crowd takes over:\nalways right next door.", 8.0, shot="long",
         ticks_per_second=130, start_tick=1300, camera=(Move(0, 8, CROWD_EYE, CROWD_AT, RING_EYE, RING_AT,
                                                             lens0=42, lens1=35),)),
    Beat("strict", "Stop twins from having to help each other,\nand giving collapses to 1.4%.", 7.0, shot="strict",
         ticks_per_second=20, overlays=("rate",), params=GIFT_RATE,
         camera=hold(RING_EYE, RING_AT, lens=35, drift=(0, 1, -1))),
    Beat("clones", "Take tolerance away entirely, and they give even more: 75%.", 8.0, shot="clones",
         ticks_per_second=25, overlays=("bars",),
         params={"title": "meetings that end in a gift", "format": "pct",
                 "rows": [[("coin-flip ties", "literal")], [("coin-flip ties, no tolerance", "clones")]]},
         camera=hold(RING_EYE, RING_AT, lens=35, drift=(0, 1, -1))),
    Beat("point", "It wasn't tolerance.\nIt was twins.", 6.0, shot="long", start_tick=3400, caption_y=0.0, title=True,
         camera=(Move(0, 6, RING_EYE, RING_AT, (0, -40, 56), RING_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Tags — after Riolo, Cohen & Axelrod, 2001; Edmonds & Hales, 2003\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
