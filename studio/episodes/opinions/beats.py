"""Following the Crowd, episode 5: "Listening to the like-minded" (Hegselmann
& Krause 2002; Lorenz 2006).

A felt ruler: opinion 0 to 1 across 200 columns, a row per Flump ordered by
where it started, so the start reads as a diagonal; each period every Flump
hops to its new opinion's column, and camps stand in lines. Colors by start,
red at 0 to magenta at 1, as in their figures. The panel top right is their
own picture: opinion against period, a line per agent for a sample of 120.

What each shot does, each seed typical of 20 (see measurements.md):
middle — hk-polarisation (625, ε 0.15), seed 8: two camps (55 % and 45 %),
  still from period 9.
three — the same, seed 1: three camps (36 %, 35 % and 29 % in the middle),
  still from period 7.
few — hk-plurality (ε 0.01), seed 8: 37 opinions, still from period 9.
many — hk-consensus (ε 0.25), seed 8: one opinion, still from period 52.
edges — hk-regular-50 (50 evenly spaced, ε 0.2), seed 1: the ends move in
  first; two camps from period 8.
lean — confidence 0.2 to the right, 0.02 to the left, seed 10: one camp at
  0.94 by period 12.
extremes — 50 evenly spaced, ε 0.4, bias m = 1, seed 1: camps at both ends.
neighbors — a 25 × 25 torus, four neighbors, ε 0.2, seed 16, a frame every
  25 periods: 76 % in one camp and 81 stranded minorities (period 2,785).
small — 50 agents, ε 0.22, seed 1: two camps (66 % and 34 %).
"""

from camera import Move
from episode import Beat

S = {"colors": "start"}
HK = ("diagram",)
# Each shot's board depth (rows; see dump._opinions): its front edge sits at
# y = −depth / 2, where the blocks start.
DEPTH = {"middle": 115, "three": 75, "few": 12, "many": 209, "edges": 9, "lean": 209, "extremes": 8, "neighbors": 159, "small": 11}


# The 50-Flump shots: half as wide (dump.SMALL_CROWD).
SMALL_CROWDS = {"edges", "extremes", "small"}


def view(shot, back=1.0):
    """Eye and target looking at the shot's front edge, the board left of the
    panel; `back` pulls the eye farther off."""
    front = -DEPTH[shot] / 2
    back *= 1.35
    ahead, x = 30, 16
    if shot in SMALL_CROWDS:
        back *= 0.75  # the 50-Flump shots: half as wide (dump.SMALL_CROWD), both ends in view
        ahead, x = 8, 8
    return (x, front - 45 * back, 80 * back), (x, front + ahead * back, 0)


def hold(shot, lens=32, seconds=10, drift=(0, 2, -2)):
    eye, at = view(shot)
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("crowd", "625 Flumps, each with an opinion between 0 and 1.", 6.0, shot="middle", ticks_per_second=0.01,
         caption_y=0.0, title=True, params=S,
         camera=(Move(0, 6, (22, -110, 25), (22, -50, 0), *view("middle"), lens0=30, lens1=32),)),
    Beat("rule", "Each listens only to those close enough,\nand moves to their average.", 7.0, shot="middle",
         ticks_per_second=0.6, overlays=HK, params=S, camera=hold("middle")),
    Beat("few", "Listen to very few, and dozens of opinions survive.", 6.0, shot="few", ticks_per_second=2.2,
         overlays=HK, params=S, camera=hold("few")),
    Beat("many", "Listen widely, and everyone agrees.", 6.0, shot="many", ticks_per_second=9.5, overlays=HK,
         params=S, camera=hold("many")),
    Beat("middle", "In between, Hegselmann and Krause's run ends in two camps.", 6.5, shot="middle",
         ticks_per_second=2.0, overlays=HK, params=S, camera=hold("middle")),
    Beat("three", "Here that happens about a third of the time.\nMore often, a third camp holds the middle.", 7.0,
         shot="three", ticks_per_second=1.6, overlays=HK, params=S, camera=hold("three")),
    Beat("edges", "Camps form at the edges first: those at the ends\nhear only one side, and move in.", 7.5,
         shot="edges", ticks_per_second=1.4, overlays=HK, params=S, camera=hold("edges")),
    Beat("lean", "Listen more to one side, and the whole crowd drifts there.", 6.5, shot="lean",
         ticks_per_second=2.3, overlays=HK, params=S, camera=hold("lean")),
    Beat("extremes", "Let each lean toward its own side,\nand the camps are pushed to the ends.", 6.5, shot="extremes",
         ticks_per_second=8.5, overlays=HK, params=S, camera=hold("extremes")),
    Beat("neighbors", "Hear only your neighbors on a map, and two camps become rare:\none crowd, with stranded minorities.",
         7.5, shot="neighbors", ticks_per_second=16.0, overlays=HK, params=S, camera=hold("neighbors")),
    Beat("size", "And numbers matter: a big crowd agrees\nwhere a small one splits.", 6.0, shot="small",
         ticks_per_second=1.8, overlays=HK, params=S, camera=hold("small")),
    Beat("reproduces", "Nearly all they reported, the Flumps reproduce.", 5.5, shot="three", start_tick=10,
         overlays=HK, params=S, camera=hold("three")),
    Beat("point", "Who you'll listen to decides whether a crowd agrees,\nsplits, or shatters.", 6.5, shot="three",
         start_tick=10, caption_y=0.0, title=True, params=S,
         camera=(Move(0, 6.5, *view("three"), *view("three", 1.3), lens0=32, lens1=32, orbit=0.1),)),
    Beat("end", "Listening to the like-minded — after Hegselmann & Krause, 2002\nndouglas.github.io/SugarScape", 5.0,
         caption_y=0.45, camera=(Move(0, 5, (0, -5.5, 1.8), (0, 0, 0.6), (0, -5.2, 1.7), (0, 0, 0.6), lens0=50, lens1=53),)),
]
