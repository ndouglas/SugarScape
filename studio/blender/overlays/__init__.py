"""On-screen text and data displays. Each overlay builder takes (beat, dump,
ctx) and returns a per-frame updater; `ctx` carries the camera's `Screen`,
the beat's timing, the agents' tracks, the board's corners and the close-up
rigs. Screen-space things hang from `Screen` anchors, whose units are half
the frame's width, so they keep their size on screen as the lens changes."""

from .caption import caption_scene
from .followers import belly, bequests, labels, rings_hungry, rings_migrants, sight, stacks, traits
from .panels import alike, bars, counter, dials, hills, histogram, season_card, wealth
from .parts import Screen

__all__ = ["BUILDERS", "SCREEN", "Screen", "caption_scene"]


# The overlays drawn in screen space (on `Screen` anchors).
SCREEN = {"season-card", "counter", "hills", "alike", "survival", "bars", "dials", "histogram", "wealth"}


BUILDERS = {
    "season-card": season_card,
    "rings-migrants": rings_migrants,
    "rings-hungry": rings_hungry,
    "counter": counter,
    "survival": bars,
    "bars": bars,
    "hills": hills,
    "bequests": bequests,
    "traits": traits,
    "alike": alike,
    "wealth": wealth,
    "belly": belly,
    "sight": sight,
    "labels": labels,
    "dials": dials,
    "stacks": stacks,
    "histogram": histogram,
}
