"""Following the Crowd, episode 2: "The tipping point" (Schelling 1971,
pp. 167–186; 1969).

The board (see area.py): a fenced area in the middle, Red's queue on the
left and Blue's on the right, each ordered by tolerance, the most tolerant
nearest the gate. Flumps walk in and out as the area changes; a warm ring
marks an insider about to leave. Schelling's plane rides top right: Red
inside across, Blue inside up, each color's tolerance curve, the path.

What each shot does (exact schedules, so each is one run; see
measurements.md): intro — Fig. 18, 4 steps. fig18 — from 25 and 25, Blue
leaves; all Red (100) by step 76. fig19 — from 50 and 50 to 80 and 80 by
step 31. entry — Fig. 19 from 100 Red and 28 Blue to 80 and 80 by step 53.
fig20 — from 60 and 40, all Red by step 42. cap — Red limited to 40, from 10
and 10 to 40 and 40 by step 31. lesstol — the least tolerant two-thirds of
Red intolerant: from 40 and 40 to 33 and 42 by step 8. allless — every Red
with a third the tolerance: from 40 and 40, all Red by step 61.
"""

from camera import Move
from episode import Beat

WIDE_EYE, WIDE_AT = (0, -22, 17), (0, -1.0, 0)
LOW_EYE, LOW_AT = (0, -12, 6), (0, 1, 0)
AREA = ("fence",)
PLAY = ("fence", "rings-unhappy", "tipplane")
TIP = {"colors": "strategy"}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


def wide(lens=26):
    return hold(WIDE_EYE, WIDE_AT, lens=lens, drift=(0, 0.8, -0.6))


BEATS = [
    Beat("area", "Schelling's second model: one neighborhood everyone prefers.\nYou're in it, or you're not.", 6.0,
         shot="intro", ticks_per_second=0.01, overlays=AREA, params=TIP,
         camera=(Move(0, 6, LOW_EYE, LOW_AT, WIDE_EYE, WIDE_AT, lens0=30, lens1=30),)),
    Beat("limit", "Each Flump has a limit: how many of the other color,\nfor each of its own, it will live with.", 6.5,
         shot="intro", ticks_per_second=0.01, overlays=AREA, params=TIP, camera=wide()),
    Beat("rule", "The least tolerant leave first.\nThe most tolerant come in first.", 6.5, shot="intro",
         ticks_per_second=0.55, overlays=AREA + ("rings-unhappy",), params=TIP, camera=wide()),
    Beat("fig18", "100 Red, 50 Blue, most able to live with some of the other color.\n"
         "Start mixed, and one color leaves entirely.", 8.0, shot="fig18", ticks_per_second=9.7, overlays=PLAY,
         params=TIP, camera=wide()),
    Beat("plane", "Schelling drew every mix both colors would stay in.\nIn this one, no mix can last.", 7.0,
         shot="fig18", start_tick=78, overlays=("fence", "tipplane"), params=TIP, camera=wide(28)),
    Beat("fig19", "Make them more tolerant, and a mix holds:\n80 and 80, from any start with enough of each.", 7.5,
         shot="fig19", ticks_per_second=4.2, overlays=PLAY, params=TIP, camera=wide()),
    Beat("entry", "Start all Red, and it takes 28 Blue\narriving together to get in.", 7.0, shot="entry",
         ticks_per_second=7.6, overlays=PLAY, params=TIP, camera=wide()),
    Beat("fig20", "Make one color twice as many,\nand the mix is lost.", 6.5, shot="fig20", ticks_per_second=6.7,
         overlays=PLAY, params=TIP, camera=wide()),
    Beat("cap", "Let in only the 40 most tolerant Red,\nand it holds at 40 and 40.", 6.5, shot="cap",
         ticks_per_second=4.8, overlays=PLAY, params=TIP, camera=wide()),
    Beat("lesstol", "Make the least tolerant even less tolerant,\nand it holds too.", 6.5, shot="lesstol",
         ticks_per_second=1.5, overlays=PLAY, params=TIP, camera=wide()),
    Beat("allless", "Make every Red less tolerant, and it doesn't.", 5.5, shot="allless", ticks_per_second=11.2,
         overlays=PLAY, params=TIP, camera=wide()),
    Beat("reproduces", "Every result Schelling reported here, the Flumps reproduce.", 6.0, shot="fig19",
         start_tick=32, overlays=("fence", "tipplane"), params=TIP, camera=wide(28)),
    Beat("point", "Whether a neighborhood stays mixed depends not only\non how tolerant people are, but on who, "
         "and how many.", 6.5, shot="fig19", start_tick=32, caption_y=0.0, title=True, overlays=AREA, params=TIP,
         camera=(Move(0, 6.5, WIDE_EYE, WIDE_AT, (0, -28, 22), WIDE_AT, lens0=26, lens1=26, orbit=0.1),)),
    Beat("end", "The tipping point — after Schelling, 1969, 1971\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
