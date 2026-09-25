"""The pilot, "Sugarscape" (spec Part 3). Board coordinates: one unit per
cell, the 50 × 50 board centered on the origin; the book's hills sit around
cells (35, 15) and (15, 35), i.e. board (10.5, 9.5) and (−9.5, −10.5)."""

from camera import Move
from episode import Beat

HILL = (10.5, 9.5, 2.0)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)

BEATS = [
    Beat(
        "crowd",
        "Now: 400 Flumps.",
        9.0,
        shot="crowd",
        ticks_per_second=8,
        lead_in=1.0,
        overlays=("dials",),
        camera=(Move(0, 9, (0, -38, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.35),),
    ),
    Beat(
        "meet",
        "This is a Flump.",
        5.0,
        shot="meet",
        closeup=True,
        ticks_per_second=0.3,
        lead_in=0.8,
        focus=(0,),
        overlays=("belly", "labels", "sight"),
        camera=(Move(0, 5, (-0.9, -6.0, 3.2), (-0.3, -0.5, 1.5), (-0.2, -5.4, 3.0), (0.0, -0.4, 1.5), lens0=45, lens1=48),),
    ),
    Beat(
        "landscape",
        "Sugar grows in some places… but not others.",
        6.0,
        shot="landscape",
        camera=(Move(0, 6, (10, -4, 4), HILL, WIDE_EYE, WIDE_AT, lens0=35, lens1=40),),
    ),
]
