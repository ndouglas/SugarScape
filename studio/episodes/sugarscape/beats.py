"""The pilot, "Sugarscape" (spec Part 3, lengthened to ~100 s).

Board coordinates: one unit per cell, the board centered on the origin, cell
(x, y) at (x − w/2 + 0.5, h/2 − y − 0.5). On the 50 × 50 board the book's two
sugar peaks are around cells (38.5, 10.5) and (14.5, 34.5), i.e. board
(14, 14) and (−10, −10); 12 × 12 close-ups put cell (x, y) at (x − 5.5, 5.5 − y).

What each shot does (from its dump; see measurements.md for the crowd beats):
sugar — a few full gumdrops on a small hill.
grow — the centre site regrows 0 → 1 → 2 → 3 → 4, a notch a tick, then stops.
meet — a Flump lands beside a hill and, at tick 1, hops 3 cells onto it.
eat — a Flump hops 2 cells onto the only gumdrop in sight and eats all of it
  (3 → 6 sugar after paying 1).
patch — a lone Flump circles a small hill, (7,6) → (5,6) → (6,6) → (6,5) …,
  coming back to spots as they regrow.
cross — a Flump on bare felt runs down 5 → 4 → 3 → 2 → 1 and starves in tick 5.
look — a Flump that sees 4 hops 4 cells east onto the hill.
differ — the far-sighted, frugal Flump holds steady; the short-sighted,
  hungry one runs down (8 → 5 → 2) and starves in tick 3.
crowd — ii-2-unit, seed 18 (typical of 20 seeds).
wealth — ii-5-wealth, seed 11 (typical of 20 seeds); the rich beat ends at
  tick 500, where its claim is measured.
"""

from camera import Move
from episode import Beat

HILL = (14, 14, 1.8)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)


def hold(eye, at, lens=50, seconds=10, drift=(0, 0.4, -0.15)):
    """A slow push-in: the camera creeps toward its target."""
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("sugar", "This is sugar.", 4.0, shot="sugar", closeup=True,
         camera=hold((0, -3.4, 2.4), (0, 0, 0.9), lens=55)),
    Beat("grow", "It grows back a little each tick… up to a limit.", 5.0, shot="grow", closeup=True,
         ticks_per_second=1.3, lead_in=0.6,
         camera=hold((0, -3.2, 2.3), (0, 0, 0.9), lens=58)),
    Beat("landscape", "Sugar grows in some places… but not others.", 6.0, shot="landscape",
         camera=(Move(0, 6, (10, -2, 5), HILL, WIDE_EYE, WIDE_AT, lens0=35, lens1=40),)),
    Beat("meet", "This is a Flump.", 5.0, shot="meet", closeup=True, ticks_per_second=0.3, lead_in=0.8,
         camera=(Move(0, 5, (-0.9, -5.0, 2.8), (-0.4, -0.5, 1.3), (-0.2, -4.4, 2.6), (0.0, -0.4, 1.3), lens0=50, lens1=55),)),
    Beat("eat", "Flumps eat sugar. All of it, wherever they stand.", 6.0, shot="eat", closeup=True,
         ticks_per_second=0.5, lead_in=0.8, focus=(0,), overlays=("belly",),
         camera=hold((0.5, -5.8, 2.8), (0.5, -0.5, 0.9), lens=42)),
    Beat("regrow", "Eaten sugar takes time to grow back…", 5.0, shot="patch", closeup=True,
         ticks_per_second=1.1, lead_in=0.8, focus=(0,), overlays=("belly",),
         camera=hold((0.5, -6.2, 4.6), (0.5, -0.5, 1.2), lens=40)),
    Beat("keep-moving", "…so a Flump has to keep moving.", 5.0, shot="patch", closeup=True,
         ticks_per_second=1.6, start_tick=5, focus=(0,), overlays=("belly",),
         camera=(Move(0, 5, (0.5, -6.2, 4.6), (0.5, -0.5, 1.2), (0.5, -6.2, 4.6), (0.5, -0.5, 1.2), lens0=40, lens1=44, orbit=0.5),)),
    Beat("cost", "Every tick costs sugar.", 5.0, shot="cross", closeup=True,
         ticks_per_second=0.9, lead_in=0.8, focus=(0,), overlays=("belly",),
         camera=hold((-2.5, -5.2, 2.6), (-2.5, -0.5, 0.8), lens=45)),
    Beat("starve", "Run out, and it's gone.", 4.0, shot="cross", closeup=True,
         ticks_per_second=0.9, start_tick=3, focus=(0,), overlays=("belly",),
         camera=hold((-2.5, -4.9, 2.5), (-2.5, -0.5, 0.8), lens=48)),
    Beat("look", "A Flump can see a little way…", 3.5, shot="look", closeup=True,
         ticks_per_second=0.35, lead_in=0.8, focus=(0,), overlays=("sight",),
         camera=(Move(0, 3.5, (-1.5, -7.6, 6.0), (-0.6, 0.5, 0.8), (-1.0, -7.3, 5.8), (-0.5, 0.5, 0.8), lens0=36, lens1=37),)),
    Beat("choose", "…and heads for the nearest, sweetest spot it can see.", 5.0, shot="look", closeup=True,
         ticks_per_second=0.35, lead_in=-2.0, focus=(0,), overlays=("sight",),
         camera=(Move(0, 5, (-1.0, -7.3, 5.8), (-0.5, 0.5, 0.8), (0.5, -6.6, 5.2), (0.8, 0.5, 0.8), lens0=37, lens1=40),)),
    Beat("differ", "…and every Flump is different.", 6.0, shot="differ", closeup=True,
         ticks_per_second=0.8, lead_in=1.0, focus=(0, 1), overlays=("labels",),
         camera=hold((0, -6.5, 3.0), (0, -0.5, 1.0), lens=45)),
    Beat("crowd", "Now: 400 Flumps.", 9.0, shot="crowd", ticks_per_second=10, lead_in=1.0,
         # Selection happens here, in the die-off (sight 3.48 → 3.73, hunger
         # 2.52 → 1.88 by tick 80 on seed 18).
         overlays=("dials",),
         camera=(Move(0, 9, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.35),)),
    Beat("gather", "Nobody told them to gather there.", 6.0, shot="crowd", ticks_per_second=20, start_tick=80,
         camera=(Move(0, 6, (34, -10, 16), HILL, (30, -4, 13), HILL, lens0=38, lens1=42),)),
    Beat("survivors", "The survivors see farther, and eat less.", 6.0, shot="crowd", ticks_per_second=10,
         start_tick=200, overlays=("dials",),
         camera=(Move(0, 6, (4, -8, 22), HILL, (6, -4, 20), HILL, lens0=40, lens1=44),)),
    Beat("replace", "Now let them grow old… and be replaced.", 7.5, shot="wealth", ticks_per_second=12,
         lead_in=0.5, camera=(Move(0, 7.5, WIDE_EYE, WIDE_AT, (-2, -48, 27), (4, 0, 0), orbit=-0.15),)),
    Beat("rich", "Same rules for everyone. So why are some rich?", 10.0, shot="wealth", ticks_per_second=41,
         start_tick=90, overlays=("stacks", "histogram"),
         camera=(Move(0, 10, (-2, -44, 28), (9, 0, 0), (-2, -38, 24), (9, 0, 0), orbit=-0.2),)),
    Beat("end", "Sugarscape — Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
