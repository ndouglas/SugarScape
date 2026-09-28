"""Cooperation, episode 3: "Ethnocentrism" (Hammond & Axelrod 2006; Jansson
2013).

Board coordinates: one unit per square, square (x, y) at (x − w/2 + 0.5,
h/2 − y − 0.5); the land wraps around. A Flump's yarn is its color (its tag:
coral, teal, lilac or blue, colors="tag"); the felt under it, its kind:
amber helps only its own color, cream helps everyone, charcoal helps no one,
magenta helps only other colors.

What each shot does (see measurements.md for the 20-seed claims):
closeup — 8 × 8, every square filled at random (seed 3), 4 periods.
land — ha-standard, seed 15 (typical of 20), to period 1,500: help-own-only
  70 % by period 800, 84 % at 1,500.
scatter — ha-standard, seed 9 (typical of 20 for the scattering), with each
  child placed anywhere from period 1,000: help-no-one 7 % at 1,000, 47 % at
  1,200 and 88 % at 1,500; the land fills to about 2,250.
blind — ha-cost-2-blind, seed 2 (typical), 2,000 periods.
"""

from camera import Move
from episode import Beat

CLOSE_EYE, CLOSE_AT = (0, -9.2, 10.4), (0, -0.4, 0.2)
WIDE_EYE, WIDE_AT = (0, -46, 44), (0, 2, 0)
LOW_EYE, LOW_AT = (0, -34, 26), (0, 4, 0)
TAG = {"colors": "tag"}


def hold(eye, at, lens=40, seconds=10, drift=(0, 0.4, -0.15)):
    end = tuple(e + d for e, d in zip(eye, drift))
    return (Move(0, seconds, eye, at, end, at, lens0=lens, lens1=lens + 3),)


BEATS = [
    Beat("colors", "Each Flump wears one of four colors, and passes it on to its children.", 5.0, shot="closeup",
         closeup=True, ticks_per_second=0.01, params=TAG, camera=hold(CLOSE_EYE, CLOSE_AT, lens=34)),
    Beat("rules", "Each carries two rules: help my own color? Help the others?", 5.5, shot="closeup", closeup=True,
         ticks_per_second=0.01, overlays=("legend",), params=TAG, camera=hold(CLOSE_EYE, CLOSE_AT, lens=36)),
    Beat("kinds", "Help only your own, help everyone, help no one, or help only others.", 6.0, shot="closeup",
         closeup=True, ticks_per_second=0.01, overlays=("legend",), params=TAG,
         camera=hold(CLOSE_EYE, CLOSE_AT, lens=37)),
    Beat("cost", "Helping a neighbor costs a little of your chance to have a child,\n"
         "and gives the neighbor three times as much.", 6.5, shot="closeup", closeup=True, ticks_per_second=0.35,
         lead_in=1.0, params=TAG, camera=hold(CLOSE_EYE, CLOSE_AT, lens=36, drift=(0, 0.3, -0.2))),
    Beat("arrive", "The land starts empty. Newcomers arrive one at a time;\nchildren are born next door.", 6.0,
         shot="land", ticks_per_second=25, overlays=("kinds",), params=TAG,
         camera=(Move(0, 6, (0, -20, 22), (0, 0, 0), WIDE_EYE, WIDE_AT, lens0=35, lens1=35),)),
    Beat("favor", "Favoritism wins: about 3 in 4 Flumps help only their own color,\nin all 20 worlds.", 7.5,
         shot="land", ticks_per_second=180, start_tick=150, overlays=("kinds",), params=TAG,
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("family", "Why? Their neighbors are family:\nover 8 in 10 of all helps go to relatives.", 7.0, shot="land",
         ticks_per_second=3, start_tick=1480, overlays=("bars",),
         params={**TAG, "title": "family", "format": "pct",
                 "rows": [[("helps that go to relatives", "kin_help")], [("neighbors who are relatives", "relatives")]]},
         camera=(Move(0, 7, WIDE_EYE, WIDE_AT, LOW_EYE, LOW_AT, lens0=35, lens1=42),)),
    Beat("anywhere", "Now put each child anywhere on the land.", 5.5, shot="scatter", ticks_per_second=6,
         start_tick=990, overlays=("kinds",), params=TAG, camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("dark", "Favoritism collapses. Nine in ten Flumps help no one.", 7.5, shot="scatter", ticks_per_second=70,
         start_tick=1023, overlays=("kinds",), params=TAG, camera=hold(WIDE_EYE, WIDE_AT, lens=36, drift=(0, 1, -1))),
    Beat("blind", "The paper says color-blind Flumps, paying double to help,\nhelp 14% of the time. Here: 42%.", 8.0,
         shot="blind", ticks_per_second=13, start_tick=1896, overlays=("bars",),
         params={**TAG, "title": "how often color-blind Flumps help", "format": "pct",
                 "rows": [[("the paper", "paper_blind")], [("here", "blind")]]},
         camera=hold(WIDE_EYE, WIDE_AT, lens=35, drift=(0, 1, -1))),
    Beat("point", "Favoritism didn't make them helpful.\nFamily did.", 6.0, shot="land", start_tick=1500, caption_y=0.0,
         title=True, params=TAG,
         camera=(Move(0, 6, WIDE_EYE, WIDE_AT, (0, -60, 50), WIDE_AT, lens0=35, lens1=35, orbit=0.1),)),
    Beat("end", "Ethnocentrism — after Hammond & Axelrod, 2006\nndouglas.github.io/SugarScape", 5.0, caption_y=0.45,
         camera=hold((0, -5.5, 1.8), (0, 0, 0.6), lens=50, drift=(0, 0.3, -0.1))),
]
