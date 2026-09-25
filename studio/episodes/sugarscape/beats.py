"""The pilot, "Sugarscape" (spec Part 3). Board coordinates: one unit per
cell, the 50 × 50 board centered on the origin; the book's hills sit around
cells (35, 15) and (15, 35), i.e. board (10.5, 9.5) and (−9.5, −10.5)."""

from camera import Move
from episode import Beat

HILL = (10.5, 9.5, 2.0)
WIDE_EYE, WIDE_AT = (0, -56, 30), (0, -2, 0)

BEATS = [
    Beat(
        "landscape",
        "Sugar grows in some places… but not others.",
        6.0,
        shot="landscape",
        camera=(Move(0, 6, (10, -4, 4), HILL, WIDE_EYE, WIDE_AT, lens0=35, lens1=40),),
    ),
]
