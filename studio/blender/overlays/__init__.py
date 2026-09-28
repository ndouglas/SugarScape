"""On-screen text and data displays. Each overlay builder takes (beat, dump,
ctx) and returns a per-frame updater; `ctx` carries the camera's `Screen`,
the beat's timing, the agents' tracks, the board's corners and the close-up
rigs. Screen-space things hang from `Screen` anchors, whose units are half
the frame's width, so they keep their size on screen as the lens changes."""

from .caption import caption_scene
from .followers import belly, bequests, labels, rings_hungry, rings_migrants, rings_shuttlers, sight, stacks, trades, traits, loot, warlord, killmap, loanlines, infections
from .panels import alike, bars, census, tally, counter, dials, hills, histogram, kills, ladder, ledger, popchart, prices, sick, season_card, wealth
from .lattice import earnings, helpers, kinds, legend, play, scores
from .parts import Screen

__all__ = ["BUILDERS", "SCREEN", "Screen", "caption_scene"]


# The overlays drawn in screen space (on `Screen` anchors).
SCREEN = {"season-card", "counter", "hills", "alike", "survival", "bars", "dials", "histogram", "wealth",
          "census", "prices", "kills", "ladder", "sick", "popchart", "ledger", "helpers", "earnings", "tally", "kinds", "legend"}


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
    "census": census,
    "prices": prices,
    "rings-shuttlers": rings_shuttlers,
    "trades": trades,
    "loot": loot,
    "warlord": warlord,
    "killmap": killmap,
    "kills": kills,
    "loanlines": loanlines,
    "ladder": ladder,
    "infections": infections,
    "sick": sick,
    "popchart": popchart,
    "ledger": ledger,
    "helpers": helpers,
    "earnings": earnings,
    "scores": scores,
    "play": play,
    "tally": tally,
    "kinds": kinds,
    "legend": legend,
}
