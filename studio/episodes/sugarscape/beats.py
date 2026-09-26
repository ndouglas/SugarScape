"""The pilot, "Sugarscape" (spec Part 3).

Board coordinates: one unit per cell, the board centered on the origin, cell
(x, y) at (x − w/2 + 0.5, h/2 − y − 0.5). On the 50 × 50 board the book's two
sugar peaks are around cells (38.5, 10.5) and (14.5, 34.5), i.e. board
(14, 14) and (−10, −10); 12 × 12 close-ups put cell (x, y) at (x − 5.5, 5.5 − y).

What each shot does (from its dump; see measurements.md for beats 7–9):
grow — the centre site regrows 0 → 1 → 2 → 3 → 4, a notch a tick.
meet — a Flump lands beside a hill and, at tick 1, hops 3 cells onto it.
hunger — the Flump by the hill eats (4 → 9 sugar); the one on bare felt
  runs out (4 → 1) and starves in tick 2.
look — a Flump that sees 4 hops 4 cells east onto the hill.
differ — the far-sighted, frugal Flump holds steady; the short-sighted,
  hungry one runs down (8 → 5 → 2) and starves in tick 3.
crowd — ii-2-unit, seed 18 (typical of 20 seeds).
wealth — ii-5-wealth, seed 11 (typical of 20 seeds).
"""

from camera import Move
from episode import Beat

HILL = (14, 14, 1.8)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)

BEATS = [
    Beat(
        "grow",
        "This is sugar.",
        5.0,
        shot="grow",
        closeup=True,
        ticks_per_second=1.2,
        lead_in=0.8,
        camera=(Move(0, 5, (0, -3.4, 2.4), (0, 0, 0.9), (0, -2.8, 2.1), (0, 0, 0.9), lens0=55, lens1=60),),
    ),
    Beat(
        "landscape",
        "Sugar grows in some places… but not others.",
        6.0,
        shot="landscape",
        camera=(Move(0, 6, (10, -2, 5), HILL, WIDE_EYE, WIDE_AT, lens0=35, lens1=40),),
    ),
    Beat(
        "meet",
        "This is a Flump.",
        5.0,
        shot="meet",
        closeup=True,
        ticks_per_second=0.3,
        lead_in=0.8,
        camera=(Move(0, 5, (-0.9, -5.0, 2.8), (-0.4, -0.5, 1.3), (-0.2, -4.4, 2.6), (0.0, -0.4, 1.3), lens0=50, lens1=55),),
    ),
    Beat(
        "hunger",
        "Flumps eat sugar. Without it, they die.",
        7.0,
        shot="hunger",
        closeup=True,
        ticks_per_second=0.9,
        lead_in=1.0,
        focus=(0, 1),
        overlays=("belly",),
        camera=(
            Move(0, 3.0, (0, -8.5, 4.4), (0, -0.5, 1.0), (0, -8.2, 4.3), (0, -0.5, 1.0), lens0=33, lens1=34),
            # After the starving Flump poofs, drift to the one that ate.
            Move(3.4, 7, (0, -8.2, 4.3), (0, -0.5, 1.0), (-2.0, -6.2, 3.4), (-2.3, -0.6, 1.3), lens0=34, lens1=40),
        ),
    ),
    Beat(
        "look",
        "A Flump can see a little way…",
        6.0,
        shot="look",
        closeup=True,
        ticks_per_second=0.35,
        lead_in=0.8,
        focus=(0,),
        overlays=("sight",),
        camera=(Move(0, 6, (-1.5, -7.6, 6.0), (-0.6, 0.5, 0.8), (-0.2, -7.0, 5.6), (-0.2, 0.5, 0.8), lens0=36, lens1=38),),
    ),
    Beat(
        "differ",
        "…and every Flump is different.",
        6.0,
        shot="differ",
        closeup=True,
        ticks_per_second=0.8,
        lead_in=1.0,
        focus=(0, 1),
        overlays=("labels",),
        camera=(Move(0, 6, (0, -6.5, 3.0), (0, -0.5, 1.0), (0, -5.8, 2.8), (0, -0.5, 1.0), lens0=45, lens1=48),),
    ),
    Beat(
        "crowd",
        "Now: 400 Flumps.",
        9.0,
        shot="crowd",
        ticks_per_second=10,
        lead_in=1.0,
        camera=(Move(0, 9, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.35),),
    ),
    Beat(
        "gather",
        "Nobody told them to gather there.",
        7.0,
        shot="crowd",
        ticks_per_second=30,
        start_tick=90,
        overlays=("dials",),
        camera=(Move(0, 7, (34, -10, 16), HILL, (30, -4, 13), HILL, lens0=38, lens1=42),),
    ),
    Beat(
        "rich",
        "Same rules for everyone. So why are some rich?",
        10.0,
        shot="wealth",
        ticks_per_second=40,
        lead_in=0.5,
        overlays=("stacks", "histogram"),
        camera=(Move(0, 10, (-2, -44, 28), (9, 0, 0), (-2, -38, 24), (9, 0, 0), orbit=-0.2),),
    ),
    Beat(
        "end",
        "Sugarscape — Epstein & Axtell, 1996\nndouglas.github.io/SugarScape",
        4.0,
        caption_y=0.45,
        camera=(Move(0, 4, (0, -5.5, 1.8), (0, 0, 0.6), (0, -5.2, 1.7), (0, 0, 0.6), lens0=50, lens1=52),),
    ),
]
