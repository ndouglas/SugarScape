"""Episode 7, "War" (Animations III-9 and III-11; rule C).

Board coordinates: one unit per cell, cell (x, y) at (x − w/2 + 0.5, h/2 − y − 0.5).
Flumps wear their tribe's color: Blue starts on the south-west hill, Red on
the north-east.

What each shot does (from its dump; see measurements.md for the world beats):
fight — an 8 × 8 flat board: a Blue with 40 sugar at (3, 3), a Red with 3 at
  (3, 5). In tick 1 the Blue takes the Red's site and its 3 sugar.
wary — the same Blue with a poorer Red at (3, 4), in reach, and a richer Red
  (60) at (3, 6), which would see the Blue on the poor Red's site. Nobody
  moves and nobody attacks in the first ticks; without the richer Red, the
  Blue kills the poor Red in tick 1 (checked).
conquest — iii-9-combat, seed 14 (the war starts at tick 618, the median
  is 591): a first kill at tick 95, then quiet. Blue Flump 63 makes its first
  kill at tick 605 and its tenth by 620; its sugar goes from 625 at tick 600
  to 53,842 at 700; Red has 59 Flumps at 700, 17 at 800, none after 1009.
  It makes 117 of the 124 kills.
standoff — seed 7: 11 kills in 2000 ticks; both tribes keep their hills.
endless — iii-11-combat-fixed, seed 1: both tribes held at 200; 4,564 kills
  in 500 ticks, spread over every quarter of the board (21–30 % each).
"""

from camera import Move
from episode import Beat

FIGHT_EYE, FIGHT_AT = (-0.5, -6.5, 4.2), (-0.5, -0.5, 0.8)
WARY_EYE, WARY_AT = (-0.5, -8.0, 5.0), (-0.5, -1.0, 0.8)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)
SIDE_EYE, SIDE_AT = (0, -60, 34), (-8, 0, 0)
TRIBE = {"colors": "tribe", "belly_full": 40}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("fight", "Now Flumps can fight. A Flump may attack an enemy poorer than itself…", 5.0, shot="fight",
         closeup=True, ticks_per_second=0.12, lead_in=0.4, focus=(0, 1), overlays=("belly",), params=TRIBE,
         camera=hold(FIGHT_EYE, FIGHT_AT, lens=38)),
    Beat("take", "…and takes everything it had.", 5.0, shot="fight", closeup=True, ticks_per_second=0.35,
         focus=(0, 1), overlays=("belly", "loot"), params=TRIBE, camera=hold(FIGHT_EYE, FIGHT_AT, lens=42)),
    Beat("wary", "But never if a richer enemy could see it afterward.", 5.5, shot="wary", closeup=True,
         ticks_per_second=0.3, lead_in=0.3, focus=(0, 1, 2), overlays=("belly",), params=TRIBE,
         camera=hold(WARY_EYE, WARY_AT, lens=36)),
    Beat("hills", "Two tribes of 200, each on its own hill.", 5.0, shot="conquest", ticks_per_second=4,
         lead_in=0.6, overlays=("kills",), params=TRIBE,
         camera=(Move(0, 5, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.2),)),
    Beat("quiet", "For hundreds of ticks, almost nothing happens.", 7.0, shot="conquest", ticks_per_second=82,
         start_tick=20, overlays=("kills",), params=TRIBE, camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 3, -1.5))),
    Beat("warlord", "Then one Flump gets rich enough that no one can stop it.", 9.0, shot="conquest",
         ticks_per_second=30, start_tick=596, overlays=("warlord", "loot", "kills"), params=TRIBE,
         # It starts in the middle and ranges over the north-east quarter
         # (board x 8–20, y 4–17): the camera follows it there and holds.
         camera=(Move(0, 2.5, (0, -40, 24), (0, 0, 0), (13, -11, 22), (13, 13, 0), lens0=36, lens1=34),
                 Move(2.5, 9, (13, -11, 22), (13, 13, 0), (13, -9, 21), (13, 13, 0), lens0=34, lens1=36))),
    Beat("fallen", "In 14 of 20 worlds, one tribe is wiped out, or nearly.", 7.0, shot="conquest", start_tick=2000,
         overlays=("bars",),
         params={**TRIBE, "title": "in 20 worlds",
                 "rows": [[("one tribe holds 90% or more", "conquest_seeds"), ("the other is gone", "wiped_seeds")]]},
         camera=hold(SIDE_EYE, SIDE_AT, lens=36)),
    Beat("one", "Nearly all the killing is done by a single Flump.", 7.0, shot="conquest", start_tick=2000,
         overlays=("bars",),
         params={**TRIBE, "title": "who does the killing (worlds with a conquest)",
                 "rows": [[("the top killer", "top_share"), ("everyone else", "others_share")]]},
         camera=hold(SIDE_EYE, SIDE_AT, lens=37)),
    Beat("spoils", "The winner ends up about 25 times richer than the richest Flump in a world without war.", 7.5,
         shot="conquest", start_tick=2000, overlays=("bars",),
         params={**TRIBE, "title": "the richest Flump's sugar at tick 2000", "format": "num", "top": 120000,
                 "rows": [[("with war", "richest_war"), ("without", "richest_peace")]]},
         camera=hold(SIDE_EYE, SIDE_AT, lens=38)),
    Beat("standoff", "In 5 of 20, no warlord rises: a few skirmishes, then the tribes keep their hills.", 7.0,
         shot="standoff", ticks_per_second=280, lead_in=0.3, overlays=("kills",), params=TRIBE,
         camera=(Move(0, 7, WIDE_EYE, WIDE_AT, (4, -52, 30), WIDE_AT, orbit=0.2),)),
    Beat("endless", "Now cap the loot, and replace every Flump that dies.", 6.5, shot="endless", ticks_per_second=20,
         lead_in=0.4, overlays=("kills",), params=TRIBE, camera=hold(WIDE_EYE, WIDE_AT, lens=36)),
    Beat("nofront", "The book shows a battle front. Under its own rules there isn't one:\n"
         "the killing is everywhere, and most of the dead are newcomers.", 9.0, shot="endless",
         ticks_per_second=44, start_tick=100, overlays=("killmap", "kills"), params=TRIBE,
         camera=(Move(0, 9, WIDE_EYE, WIDE_AT, (0, -45, 42), (0, 0, 0), orbit=0.15),)),
    Beat("question", "Nobody fights an equal.\nWar waits until someone can't lose.", 6.0, shot="conquest",
         start_tick=2000, caption_y=0.0, title=True, params=TRIBE,
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -70, 40), WIDE_AT, orbit=0.1),)),
    Beat("end", "War — after Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
